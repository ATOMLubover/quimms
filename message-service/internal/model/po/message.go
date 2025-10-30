package po

type NewChannelMessagePO struct {
	ID          string
	PKChannelID string
	PKUserID    string
	Content     string
}

func (NewChannelMessagePO) TableName() string {
	return "t_channel_message"
}

type ChannelMessagePO struct {
	ID          string `gorm:"primaryKey;column:f_id"`
	PKChannelID string `gorm:"column:f_pk_channel_id"`
	PKUserID    string `gorm:"column:f_pk_user_id"`
	Content     string `gorm:"column:f_content"`
	CreatedAt   int64  `gorm:"column:f_created_at"`
}

func (ChannelMessagePO) TableName() string {
	return "t_channel_message"
}
