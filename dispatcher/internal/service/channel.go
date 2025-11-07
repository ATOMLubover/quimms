package service

import "gorm.io/gorm"

func GetChanMembers(db *gorm.DB, channelID string) ([]string, error) {
	var members []string

	if err := db.Table("channel_members").
		Where("f_pk_channel_id = ?", channelID).
		Pluck("f_pk_user_id", &members).Error; err != nil {
		return nil, err
	}

	return members, nil
}
