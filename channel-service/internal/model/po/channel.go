package po

type ChannelPO struct {
	ID        string `gorm:"primaryKey;column:f_id"`
	Name      string `gorm:"column:f_name"`
	CreatedAt uint64 `gorm:"column:f_created_at"`
}

func (ChannelPO) TableName() string {
	return "t_channel"
}
