package repo

import (
	"channel-service/internal/model/po"
	"time"

	"gorm.io/gorm"
)

func CreateChannel(db *gorm.DB, name string, newID string) error {
	newChannel := po.ChannelPO{
		ID:        newID,
		Name:      name,
		CreatedAt: time.Now(),
	}

	return db.Create(&newChannel).Error
}

func GetChannelDetailsByIDs(db *gorm.DB, ids []string) ([]po.ChannelPO, []po.ChannelMemberPO, error) {
	var channels []po.ChannelPO

	err := db.Where("f_id IN ?", ids).Find(&channels).Error

	if err != nil {
		return nil, nil, err
	}

	var channel_members []po.ChannelMemberPO

	err = db.Where("f_pk_channel_id IN ?", ids).Find(&channel_members).Error

	if err != nil {
		return nil, nil, err
	}

	return channels, channel_members, nil
}
