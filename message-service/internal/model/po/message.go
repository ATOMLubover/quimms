package po

type ChannelMessagePO struct {
	ID           string `gorm:"column:f_id;"`
	PKSendUserID string `gorm:"column:f_pk_send_user_id;"`
	Content      string `gorm:"column:f_content;"`
	CreatedAt    int64  `gorm:"column:f_created_at;"`
}

func (ChannelMessagePO) TableName() string {
	return "t_channel_message"
}

type ChannelMemberPO struct {
	ChannelID string `gorm:"column:f_channel_id;"`
	UserID    string `gorm:"column:f_user_id;"`
}

func (ChannelMemberPO) TableName() string {
	return "t_channel_member"
}
