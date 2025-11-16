package service

import (
	"log/slog"
	"message-service/internal/cache"
	"message-service/internal/model/dto"
	"message-service/internal/model/vo"
	"message-service/internal/mq"
	"message-service/internal/repo"
	"time"

	"github.com/bwmarrin/snowflake"
	"gorm.io/gorm"
)

func CreateMessage(
	db *gorm.DB,
	node *snowflake.Node,
	cacheCli *cache.Client,
	mqCli *mq.Client,
	data dto.CreateMessageDTO,
) (string, error) {
	newID := "message_" + node.Generate().Base64()

	msgVO := vo.ChannelMessageVO{
		MsgID:     newID,
		ChanID:    data.ChannelID,
		UserID:    data.UserID,
		Content:   data.Content,
		CreatedAt: time.Now().Unix(),
	}

	// Push the new message to the MQ.
	err := mqCli.PushMsg(msgVO)

	if err != nil {
		return "", err
	}

	// Update Redis.
	err = cacheCli.PushMsg(msgVO)

	if err != nil {
		return "", err
	}

	// Persist the new message to the database
	// asynchronously in background goroutine.
	go func() {
		// TODO: Maybe we can use a worker pool and batch insert messages for better
		// performance.
		if err := repo.CreateMessage(
			db,
			newID,
			data.ChannelID,
			data.UserID,
			data.Content,
		); err != nil {
			slog.Error(
				"Failed to persist new message to database",
				slog.String("error", err.Error()),
				slog.String("message_id", newID),
				slog.String("channel_id", data.ChannelID),
				slog.String("user_id", data.UserID),
			)
		}
	}()

	return newID, nil
}

func GetMessagesByChannelID(
	db *gorm.DB,
	channelID string,
	limit int,
	latestTime int64,
) ([]vo.ChannelMessageVO, error) {
	messagePOs, err := repo.GetMessagesByChannelID(
		db,
		channelID,
		limit,
		latestTime,
	)

	if err != nil {
		return nil, err
	}

	var messageVOs []vo.ChannelMessageVO

	for _, messagePO := range messagePOs {
		messageVOs = append(messageVOs, vo.ChannelMessageVO{
			MsgID:     messagePO.ID,
			ChanID:    messagePO.PKChannelID,
			UserID:    messagePO.PKUserID,
			Content:   messagePO.Content,
			CreatedAt: messagePO.CreatedAt.Unix(),
		})
	}

	return messageVOs, nil
}
