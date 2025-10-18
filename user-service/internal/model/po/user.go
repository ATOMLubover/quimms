package po

type NewUserPO struct {
	ID           string `gorm:"primaryKey;column:f_id"`
	Nickname     string `gorm:"column:f_nickname"`
	PasswordHash string `gorm:"column:f_password_hash"`
}

func (NewUserPO) TableName() string {
	return "t_user"
}

type UserCertPO struct {
	ID           string `gorm:"primaryKey;column:f_id"`
	PasswordHash string `gorm:"column:f_password_hash"`
}

func (UserCertPO) TableName() string {
	return "t_user"
}

type UserInfoPO struct {
	ID        string `gorm:"primaryKey;column:f_id"`
	Nickname  string `gorm:"column:f_nickname"`
	CreatedAt int64  `gorm:"column:f_created_at"`
}

func (UserInfoPO) TableName() string {
	return "t_user"
}
