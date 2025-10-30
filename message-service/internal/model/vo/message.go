package vo

type ChannelMessageVO struct {
	ID        string
	ChannelID string
	SenderID  string
	Content   string
	CreatedAt int64
}
