package vo

type ChanMsgVO struct {
	MsgID     string `json:"message_id"`
	ChanID    string `json:"channel_id"`
	UserID    string `json:"user_id"`
	Content   string `json:"content"`
	CreatedAt int64  `json:"created_at"`
}
