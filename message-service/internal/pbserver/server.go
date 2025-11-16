package pbserver

import (
	"context"
	"fmt"
	"log/slog"
	"message-service/internal/cache"
	"message-service/internal/config"
	"message-service/internal/model/dto"
	"message-service/internal/mq"
	"message-service/internal/registry"
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

	slog.Info("Database connected successfully")

	var serviceIDNum int

	_, err = fmt.Sscanf(cfg.ServiceID, "MessageService-%d", &serviceIDNum)

	if err != nil {
		return nil, err
	}

	node, err := snowflake.NewNode(int64(serviceIDNum))

	if err != nil {
		return nil, err
	}

	mqCli, err := mq.NewClient(cfg.MQURL)

	if err != nil {
		return nil, err
	}

	slog.Info("Message Queue connected successfully")

	cacheCli, err := cache.NewClient(cfg.RedisURL)

	if err != nil {
		return nil, err
	}

	slog.Info("Cache connected successfully")

	err = registry.RunRegistryClient(
		cfg.ConsulsAddr,
		cfg.ServiceID, cfg.ServiceName,
		cfg.Host, int(cfg.Port),
		int(cfg.HealthTTLSeconds), int(cfg.HealthRefreshSeconds),
	)

	if err != nil {
		return nil, err
	}

	slog.Info("Service registered successfully")

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
			UserID:    req.GetUserId(),
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

	messages := make([]*pb.Message, len(messageVOs))

	for i, messageVO := range messageVOs {
		messages[i] = &pb.Message{
			MessageId: messageVO.ID,
			ChannelId: messageVO.ChannelID,
			SenderId:  messageVO.SenderID,
			Content:   messageVO.Content,
			CreatedAt: messageVO.CreatedAt,
		}
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

	lis, err := net.Listen("tcp", fmt.Sprintf("%s:%d", cfg.Host, cfg.Port))

	if err != nil {
		return err
	}

	grpcSrv := grpc.NewServer()

	pb.RegisterMessageServiceServer(grpcSrv, srv)

	slog.Info("gRPC server is running", "address", lis.Addr().String())

	return grpcSrv.Serve(lis)
}
