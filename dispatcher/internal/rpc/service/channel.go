package service

import (
	"context"
	"dispatcher/pb"
	"fmt"
	"time"
)

func ListChanMemIDs(cli pb.ChannelServiceClient, chanID string) ([]string, error) {
	const kTmo = 2 * time.Second

	ctx, cancel := context.WithTimeout(context.Background(), kTmo)

	defer cancel()

	rsp, err := cli.ListChannelMembers(
		ctx,
		&pb.ListChannelMembersRequest{
			ChannelId: chanID,
		},
	)

	if err != nil {
		return nil, fmt.Errorf("failed to list channel members: %s", err)
	}

	memIDs := make([]string, 0, len(rsp.Members))

	for _, m := range rsp.Members {
		memIDs = append(memIDs, m.UserId)
	}

	return memIDs, nil
}
