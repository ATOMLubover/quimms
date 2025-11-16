package service

import (
	"channel-service/internal/model/vo"
	"channel-service/internal/repo"

	"github.com/bwmarrin/snowflake"
	"gorm.io/gorm"
)

func CreateChannel(db *gorm.DB, node *snowflake.Node, name string) (string, error) {
	newID := "channel_" + node.Generate().Base64()

	return newID, repo.CreateChannel(db, name, newID)
}

func GetChannelDetailsByIDs(db *gorm.DB, ids []string) ([]vo.ChannelVO, error) {
	channelsPO, membersPO, err := repo.GetChannelDetailsByIDs(db, ids)

	if err != nil {
		return nil, err
	}

	memberMap := make(map[string][]vo.ChannelMemberVO)

	for _, member := range membersPO {
		memberMap[member.ChannelID] = append(memberMap[member.ChannelID], vo.ChannelMemberVO{
			ChannelID: member.ChannelID,
			UserID:    member.UserID,
			JoinedAt:  member.CreatedAt.Unix(),
		})
	}

	var channelVOs []vo.ChannelVO

	for _, channel := range channelsPO {
		channelVO := vo.ChannelVO{
			ID:        channel.ID,
			Name:      channel.Name,
			CreatedAt: channel.CreatedAt.Unix(),
			Members:   memberMap[channel.ID],
		}

		channelVOs = append(channelVOs, channelVO)
	}

	return channelVOs, nil
}

func JoinChannel(db *gorm.DB, node *snowflake.Node, channelID string, userID string) error {
	newID := "channel_member_" + node.Generate().Base64()

	return repo.JoinChannel(db, channelID, userID, newID)
}
