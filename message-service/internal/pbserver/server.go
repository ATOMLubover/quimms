package pbserver

import (
	"context"
	"fmt"
	"log/slog"
	"math/rand/v2"
	"message-service/internal/cache"
	"message-service/internal/config"
	"message-service/internal/model/dto"
	"message-service/internal/mq"
	"message-service/internal/repo"
	"message-service/internal/service"
	"message-service/internal/state"
	"message-service/pb"
	"net"

	"github.com/bwmarrin/snowflake"
	"google.golang.org/grpc"
)

type serverImpl struct {
	pb.UnimplementedMessageServiceServer
	state *state.AppState
}

func newServer(cfg *config.AppConfig) (pb.MessageServiceServer, error) {
	db, err := repo.InitDB()

	if err != nil {
		return nil, err
	}

	node, err := snowflake.NewNode(rand.Int64()%1024 + 200)

	if err != nil {
		return nil, err
	}

	mqCli, err := mq.NewClient(cfg.MQURL)

	if err != nil {
		return nil, err
	}

	cacheCli, err := cache.NewClient(cfg.RedisURL)

	return &serverImpl{
		state: &state.AppState{
			Cfg:      cfg,
			DB:       db,
			IDGen:    node,
			MQCli:    mqCli,
			CacheCli: cacheCli,
		},
	}, nil
}

func (s *serverImpl) CreateMessage(
	ctx context.Context, req *pb.CreateMessageRequest,
) (
	*pb.CreateMessageResponse, error,
) {
	newID, err := service.CreateMessage(
		s.state.DB,
		s.state.IDGen,
		s.state.CacheCli,
		s.state.MQCli,
		dto.CreateMessageDTO{
			ChannelID: req.GetChannelId(),
			UserID:    req.GetSenderId(),
			Content:   req.GetContent(),
		},
	)

	if err != nil {
		return nil, err
	}

	return &pb.CreateMessageResponse{
		MessageId: newID,
	}, nil
}

func (s *serverImpl) ListChannelMessages(
	ctx context.Context, req *pb.ListChannelMessagesRequest,
) (
	*pb.ListChannelMessagesResponse, error,
) {
	messageVOs, err := service.GetMessagesByChannelID(
		s.state.DB,
		req.GetChannelId(),
		int(req.GetLimit()),
		req.GetLatestTime(),
	)

	if err != nil {
		return nil, err
	}

	var messages []*pb.Message

	for _, messageVO := range messageVOs {
		messages = append(messages, &pb.Message{
			MessageId: messageVO.ID,
			ChannelId: messageVO.ChannelID,
			SenderId:  messageVO.SenderID,
			Content:   messageVO.Content,
			CreatedAt: messageVO.CreatedAt,
		})
	}

	return &pb.ListChannelMessagesResponse{
		Messages: messages,
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

	lis, err := net.Listen("tcp", fmt.Sprintf("%s:%s", cfg.Host, cfg.Port))

	if err != nil {
		return err
	}

	grpcSrv := grpc.NewServer()

	pb.RegisterMessageServiceServer(grpcSrv, srv)

	slog.Info("gRPC server is running", "address", lis.Addr().String())

	return grpcSrv.Serve(lis)
}
