package mq

import (
	"encoding/json"
	"log/slog"
	"message-service/internal/model/vo"
)

func (c *Client) PushMsg(
	msg vo.ChannelMessageVO,
) error {
	data, err := json.Marshal(msg)

	if err != nil {
		return err
	}

	// We try to push the message for 3 times before giving up.
	for i := 0; i < 3; i++ {
		err := c.conn.Publish(kTopicPushChanMsg, data)

		if err == nil {
			return nil
		}

		slog.Error("Failed to publish channel message",
			slog.String("error", err.Error()),
			slog.Int("attempt", i+1))
	}

	return err
}
