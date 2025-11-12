package service

import (
	"context"
	"dispatcher/internal/model/vo"
	"dispatcher/pb"
	"fmt"
	"time"
)

func TransMsg(
	cli pb.DispatchServiceClient,
	targetUserID string,
	msg *vo.ChanMsgVO,
) error {
	const kTmo = 2 * time.Second

	ctx, cancel := context.WithTimeout(context.Background(), kTmo)

	defer cancel()

	_, err := cli.DispatchMessage(
		ctx,
		&pb.DispatchMessageRequest{
			TargetUserId: targetUserID,
			MessageId:    msg.MsgID,
			UserId:       msg.UserID,
			ChannelId:    msg.ChanID,
			Content:      msg.Content,
			CreatedAt:    msg.CreatedAt,
		},
	)

	if err != nil {
		return fmt.Errorf("failed to dispatch message: %v", err)
	}

	return nil
}
