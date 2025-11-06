package vo

type ChannelMessageVO struct {
	ID        string `json:"id"`
	ChannelID string `json:"channel_id"`
	SenderID  string `json:"sender_id"`
	Content   string `json:"content"`
	CreatedAt int64  `json:"created_at"`
}
