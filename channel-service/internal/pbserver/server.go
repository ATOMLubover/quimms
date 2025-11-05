package pbserver

import (
	"channel-service/internal/config"
	"channel-service/internal/registry"
	"channel-service/internal/repo"
	"channel-service/internal/service"
	"channel-service/internal/state"
	"channel-service/pb"
	"context"
	"fmt"
	"log/slog"
	"net"
	"strconv"

	"github.com/bwmarrin/snowflake"
	"google.golang.org/grpc"
)

type serverImpl struct {
	pb.UnimplementedChannelServiceServer
	state *state.AppState
}

func newServer(cfg *config.AppConfig) (pb.ChannelServiceServer, error) {
	db, err := repo.InitDB()

	if err != nil {
		return nil, err
	}

	serviceIDNum, err := strconv.Atoi(cfg.ServiceID)

	if err != nil {
		return nil, err
	}

	node, err := snowflake.NewNode(int64(serviceIDNum))

	if err != nil {
		return nil, err
	}

	err = registry.RunRegistryClient(
		cfg.ConsulsAddr,
		cfg.ServiceID, cfg.ServiceName,
		cfg.Host, int(cfg.Port),
	)

	return &serverImpl{
		state: &state.AppState{
			Cfg:   cfg,
			DB:    db,
			IDGen: node,
		},
	}, nil
}

func (s *serverImpl) CreateChannel(
	ctx context.Context,
	req *pb.CreateChannelRequest,
) (*pb.CreateChannelResponse, error) {
	newID, err := service.CreateChannel(s.state.DB, s.state.IDGen, req.GetName())

	if err != nil {
		return nil, err
	}

	return &pb.CreateChannelResponse{
		Id:   newID,
		Name: req.GetName(),
	}, nil
}

func (s *serverImpl) ListChannelDetails(
	ctx context.Context,
	req *pb.ListChannelDetailRequest,
) (
	*pb.ListChannelDetailResponse, error,
) {
	channelIDs, err := service.GetChannelIDsByUserID(s.state.DB, req.GetUserId())

	if err != nil {
		return nil, err
	}

	channelVOs, err := service.GetChannelDetailsByIDs(s.state.DB, channelIDs)

	if err != nil {
		return nil, err
	}

	var channels []*pb.ChannelDetail

	for _, channelVO := range channelVOs {
		var pbChannelVOs []*pb.ChannelMember

		for _, member := range channelVO.Members {
			pbChannelVOs = append(pbChannelVOs, &pb.ChannelMember{
				UserId:   member.UserID,
				JoinedAt: int64(member.JoinedAt),
			})
		}

		channels = append(channels, &pb.ChannelDetail{
			Id:      channelVO.ID,
			Name:    channelVO.Name,
			Members: pbChannelVOs,
		})
	}

	return &pb.ListChannelDetailResponse{
		Channels: channels,
	}, nil
}

func (s *serverImpl) JoinChannel(
	ctx context.Context, req *pb.JoinChannelRequest,
) (
	*pb.JoinChannelResponse, error,
) {
	err := service.JoinChannel(s.state.DB, s.state.IDGen, req.GetChannelId(), req.GetUserId())

	if err != nil {
		return nil, err
	}

	return &pb.JoinChannelResponse{
		ChannelId: req.GetChannelId(),
		UserId:    req.GetUserId(),
	}, nil
}

func RunServer() error {
	cfg, err := config.LoadConfig()

	if err != nil {
		return err
	}

	srv, err := newServer(cfg)

	if err != nil {
		return err
	}

	lis, err := net.Listen("tcp", fmt.Sprintf("%s:%d", cfg.Host, cfg.Port))

	if err != nil {
		return err
	}

	grpcSrv := grpc.NewServer()

	pb.RegisterChannelServiceServer(grpcSrv, srv)

	slog.Info("gRPC server is running", "address", lis.Addr().String())

	return grpcSrv.Serve(lis)
}
