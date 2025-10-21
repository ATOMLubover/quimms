package service

import (
	"channel-service/internal/repo"

	"gorm.io/gorm"
)

func GetChannelIDsByUserID(db *gorm.DB, userID string) ([]string, error) {
	return repo.GetChannelIDsByUserID(db, userID)
}
