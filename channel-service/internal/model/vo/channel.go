package vo

type ChannelMemberInfoVO struct {
	ChannelID string `json:"channel_id"`
	UserID    string `json:"user_id"`
	JoinedAt  uint64 `json:"joined_at"`
}

type ChannelInfoVO struct {
	ID        string                `json:"id"`
	Name      string                `json:"name"`
	CreatedAt uint64                `json:"created_at"`
	Members   []ChannelMemberInfoVO `json:"members"`
}
