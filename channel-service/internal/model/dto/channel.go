package dto

// type CreateChannelDTO struct {
// 	Name string `json:"name"`
// }

type JoinChannelDTO struct {
	ChannelID string `json:"channel_id"`
	UserID    string `json:"name"`
}

type CreatedChannelDTO struct {
	ID   string `json:"id"`
	Name string `json:"name"`
}
