package dto

type RegisterUserDTO struct {
	Nickname string `json:"nickname"`
	Password string `json:"password"`
}

type LoginUserDTO struct {
	Nickname string `json:"nickname"`
	Password string `json:"password"`
}
