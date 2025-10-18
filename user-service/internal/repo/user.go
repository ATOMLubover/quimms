package repo

import (
	"user-service/internal/model/po"

	"gorm.io/gorm"
)

func CreateUser(db *gorm.DB, newUser po.NewUserPO) error {
	return db.Create(&newUser).Error
}

func GetUserByID(db *gorm.DB, id string) (*po.UserInfoPO, error) {
	var user po.UserInfoPO

	err := db.First(&user, "f_id = ?", id).Error

	return &user, err
}

func GetUserCertByNickname(db *gorm.DB, nickname string) (*po.UserCertPO, error) {
	var user po.UserCertPO

	err := db.
		Table("t_user").
		Where("f_nickname = ?", nickname).
		First(&user).Error

	return &user, err
}
