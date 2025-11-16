package repo

import (
	"message-service/internal/model/po"
	"time"

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
		CreatedAt:   time.Now(),
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

	// latestTime is unix seconds; convert to time.Time for comparison
	latest := time.Unix(latestTime, 0)

	err := db.
		Where("f_pk_channel_id = ?", channelID).
		Order("f_created_at DESC").
		Where("f_created_at < ?", latest).
		Limit(limit).
		Find(&messages).Error

	return messages, err
}
