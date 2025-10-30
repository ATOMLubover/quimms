package service

import (
	"message-service/internal/model/vo"
	"message-service/internal/repo"

	"github.com/bwmarrin/snowflake"
	"gorm.io/gorm"
)

func CreateMessage(
	db *gorm.DB,
	node *snowflake.Node,
	channelID string,
	userID string,
	content string,
) (string, error) {
	newID := "message_" + node.Generate().Base64()

	return newID, repo.CreateMessage(
		db,
		newID,
		channelID,
		userID,
		content,
	)
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
			ID:        messagePO.ID,
			ChannelID: messagePO.PKChannelID,
			SenderID:  messagePO.PKUserID,
			Content:   messagePO.Content,
			CreatedAt: messagePO.CreatedAt,
		})
	}

	return messageVOs, nil
}
