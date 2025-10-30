package repo

import (
	"message-service/internal/model/po"

	"gorm.io/gorm"
)

func CreateMessage(
	db *gorm.DB,
	newID string,
	channelID string,
	userID string,
	content string,
) error {
	newMessage := &po.NewChannelMessagePO{
		ID:          newID,
		PKChannelID: channelID,
		PKUserID:    userID,
		Content:     content,
	}

	return db.Create(newMessage).Error
}

func GetMessagesByChannelID(
	db *gorm.DB,
	channelID string,
	limit int,
	latestTime int64,
) ([]po.ChannelMessagePO, error) {
	var messages []po.ChannelMessagePO

	err := db.
		Where("f_pk_channel_id = ?", channelID).
		Order("f_created_at DESC").
		Where("f_created_at < ?", latestTime).
		Limit(limit).
		Find(&messages).Error

	return messages, err
}
