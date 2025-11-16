package vo

type ChannelMemberVO struct {
	ChannelID string `json:"channel_id"`
	UserID    string `json:"user_id"`
	JoinedAt  int64  `json:"joined_at"`
}

type ChannelVO struct {
	ID        string            `json:"id"`
	Name      string            `json:"name"`
	CreatedAt int64             `json:"created_at"`
	Members   []ChannelMemberVO `json:"members"`
}
