package service

import (
	"user-service/internal/model/dto"
	"user-service/internal/model/po"
	"user-service/internal/model/vo"
	"user-service/internal/repo"

	"github.com/bwmarrin/snowflake"
	"gorm.io/gorm"
)

func RegisterUser(db *gorm.DB, node *snowflake.Node, user dto.RegisterUserDTO) (string, error) {
	newID := "user_" + node.Generate().Base64()

	newUser := po.NewUserPO{
		ID:           newID,
		Nickname:     user.Nickname,
		PasswordHash: user.Password,
	}

	return newID, repo.CreateUser(db, newUser)
}

func LoginUser(db *gorm.DB, user dto.LoginUserDTO) (string, error) {
	cert, err := repo.GetUserCertByNickname(db, user.Nickname)

	if err != nil {
		return "", err
	}

	if cert.PasswordHash != user.Password {
		return "", nil
	}

	// TODO: generate and return a JWT token
	return cert.ID, nil
}

func GetUserInfo(db *gorm.DB, id string) (*vo.UserInfoVO, error) {
	userPO, err := repo.GetUserByID(db, id)

	if err != nil {
		return nil, err
	}

	userVO := &vo.UserInfoVO{
		ID:        userPO.ID,
		Nickname:  userPO.Nickname,
		CreatedAt: userPO.CreatedAt.Unix(),
	}

	return userVO, nil
}
