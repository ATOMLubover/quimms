package pbserver

import (
	"context"
	"math/rand/v2"
	"message-service/internal/config"
	"message-service/internal/repo"
	"message-service/internal/service"
	"message-service/internal/state"
	"message-service/pb"

	"github.com/bwmarrin/snowflake"
)

type serverImpl struct {
	pb.UnimplementedMessageServiceServer
	state *state.AppState
}

func NewServer(cfg *config.AppConfig) (pb.MessageServiceServer, error) {
	db, err := repo.InitDB()

	if err != nil {
		return nil, err
	}

	node, err := snowflake.NewNode(rand.Int64()%1024 + 200)

	return &serverImpl{
		state: &state.AppState{
			Cfg:   cfg,
			DB:    db,
			IDGen: node,
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
		req.GetChannelId(),
		req.GetSenderId(),
		req.GetContent(),
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
