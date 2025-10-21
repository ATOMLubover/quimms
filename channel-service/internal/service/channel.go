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

func GetChannelDetailsByIDs(db *gorm.DB, ids []string) ([]vo.ChannelInfoVO, error) {
	channelsPO, membersPO, err := repo.GetChannelDetailsByIDs(db, ids)

	if err != nil {
		return nil, err
	}

	memberMap := make(map[string][]vo.ChannelMemberInfoVO)

	for _, member := range membersPO {
		memberMap[member.ChannelID] = append(memberMap[member.ChannelID], vo.ChannelMemberInfoVO{
			ChannelID: member.ChannelID,
			UserID:    member.UserID,
			JoinedAt:  member.CreatedAt,
		})
	}

	var channelVOs []vo.ChannelInfoVO

	for _, channel := range channelsPO {
		channelVO := vo.ChannelInfoVO{
			ID:      channel.ID,
			Name:    channel.Name,
			Members: memberMap[channel.ID],
		}

		channelVOs = append(channelVOs, channelVO)
	}

	return channelVOs, nil
}

func JoinChannel(db *gorm.DB, node *snowflake.Node, channelID string, userID string) error {
	newID := "channel_member_" + node.Generate().Base64()

	return repo.JoinChannel(db, channelID, userID, newID)
}
