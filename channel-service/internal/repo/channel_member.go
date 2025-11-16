package repo

import (
	"channel-service/internal/model/po"

	"gorm.io/gorm"
)

func JoinChannel(db *gorm.DB, channelID string, userID string, newID string) error {
	newChannelMember := po.ChannelMemberPO{
		ID:        newID,
		ChannelID: channelID,
		UserID:    userID,
	}

	return db.Create(&newChannelMember).Error
}

func GetChannelIDsByUserID(db *gorm.DB, userID string) ([]string, error) {
	var channelIDs []string

	err := db.
		Model(&po.ChannelMemberPO{}).
		Where("f_pk_user_id = ?", userID).
		Pluck("f_pk_channel_id", &channelIDs).Error

	return channelIDs, err
}

func GetChannelMembers(db *gorm.DB, channelID string) ([]*po.ChannelMemberPO, error) {
	var channelMemberPOs []*po.ChannelMemberPO

	err := db.
		Where("f_pk_channel_id = ?", channelID).
		Find(&channelMemberPOs).Error

	if err != nil {
		return nil, err
	}

	return channelMemberPOs, nil
}
