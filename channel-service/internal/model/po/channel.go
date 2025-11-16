package po

import "time"

type ChannelPO struct {
	ID        string    `gorm:"primaryKey;column:f_id"`
	Name      string    `gorm:"column:f_name"`
	CreatedAt time.Time `gorm:"column:f_created_at"`
}

func (ChannelPO) TableName() string {
	return "t_channel"
}
