package service

import (
	"channel-service/internal/model/vo"
	"channel-service/internal/repo"

	"gorm.io/gorm"
)

func GetChannelIDsByUserID(db *gorm.DB, userID string) ([]string, error) {
	return repo.GetChannelIDsByUserID(db, userID)
}

func GetChannelMembers(db *gorm.DB, channelID string) ([]*vo.ChannelMemberVO, error) {
	members, err := repo.GetChannelMembers(db, channelID)

	if err != nil {
		return nil, err
	}

	memberVOs := make([]*vo.ChannelMemberVO, len(members))

	for i, member := range members {
		memberVOs[i] = &vo.ChannelMemberVO{
			UserID:   member.UserID,
			JoinedAt: member.CreatedAt.Unix(),
		}
	}

	return memberVOs, nil
}
