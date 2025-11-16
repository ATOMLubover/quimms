package po

import "time"

type ChannelMemberPO struct {
	ID        string    `gorm:"primaryKey;column:f_id"`
	ChannelID string    `gorm:"column:f_pk_channel_id"`
	UserID    string    `gorm:"column:f_pk_user_id"`
	CreatedAt time.Time `gorm:"column:f_created_at"`
}

func (ChannelMemberPO) TableName() string {
	return "t_channel_member"
}
