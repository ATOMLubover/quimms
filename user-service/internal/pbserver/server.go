package pbserver

import (
	"context"
	"fmt"
	"log/slog"
	"math/rand/v2"
	"net"

	"user-service/internal/config"
	"user-service/internal/model/dto"
	"user-service/internal/repo"
	"user-service/internal/service"
	"user-service/internal/state"
	"user-service/pb"

	"github.com/bwmarrin/snowflake"
	"google.golang.org/grpc"
)

type serverImpl struct {
	pb.UnimplementedUserServiceServer
	state *state.AppState
}

func NewServer(cfg *config.AppConfig) (pb.UserServiceServer, error) {
	db, err := repo.InitDB()

	if err != nil {
		return nil, err
	}

	// TODO: Use a fixed node number in production to avoid ID collisions
	node, err := snowflake.NewNode(rand.Int64() % 1024)

	if err != nil {
		return nil, err
	}

	return &serverImpl{
		state: &state.AppState{
			Cfg:      cfg,
			DB:       db,
			SnowNode: node,
		},
	}, nil
}

func (s *serverImpl) RegisterUser(
	ctx context.Context,
	req *pb.RegisterUserRequest,
) (*pb.RegisterUserResponse, error) {
	user := dto.RegisterUserDTO{
		Nickname: req.GetNickname(),
		Password: req.GetPassword(),
	}

	newID, err := service.RegisterUser(s.state.DB, s.state.SnowNode, user)

	if err != nil {
		return nil, err
	}

	return &pb.RegisterUserResponse{
		UserId: newID,
	}, nil
}

func (s *serverImpl) LoginUser(
	ctx context.Context,
	req *pb.LoginUserRequest,
) (*pb.LoginUserResponse, error) {
	user := dto.LoginUserDTO{
		Nickname: req.GetNickname(),
		Password: req.GetPassword(),
	}

	token, err := service.LoginUser(s.state.DB, user)

	if err != nil {
		return nil, err
	}

	return &pb.LoginUserResponse{
		Token: token,
	}, nil
}

func (s *serverImpl) GetUserInfo(
	ctx context.Context,
	req *pb.GetUserInfoRequest,
) (*pb.GetUserInfoResponse, error) {
	info, err := service.GetUserInfo(s.state.DB, req.GetUserId())

	if err != nil {
		return nil, err
	}

	return &pb.GetUserInfoResponse{
		UserId:    info.ID,
		Nickname:  info.Nickname,
		CreatedAt: info.CreatedAt,
	}, nil
}

func Run() error {
	cfg, err := config.LoadConfig()

	if err != nil {
		return err
	}

	srv, err := NewServer(cfg)

	if err != nil {
		return err
	}

	lis, err := net.Listen("tcp", fmt.Sprintf("%s:%s", cfg.Host, cfg.Port))

	if err != nil {
		return err
	}

	grpcSrv := grpc.NewServer()

	pb.RegisterUserServiceServer(grpcSrv, srv)

	slog.Info("gRPC server is running", "address", lis.Addr().String())

	return grpcSrv.Serve(lis)
}
