package vo

type UserInfoVO struct {
	ID        string `json:"id"`
	Nickname  string `json:"nickname"`
	CreatedAt int64  `json:"created_at"`
}
