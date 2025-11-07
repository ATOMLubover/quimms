package mq

import (
	"dispatcher/internal/model/vo"
	"encoding/json"
	"log/slog"

	"github.com/nats-io/nats.go"
)

func (c *Client) RunPullMsg() error {
	// FIXME: close the sub.
	_, err := c.conn.QueueSubscribe(
		kTopicChanMessage,
		kChanMsgQueue,
		func(msg *nats.Msg) {
			msgVO := &vo.ChanMsgVO{}

			if err := json.Unmarshal(msg.Data, msgVO); err != nil {
				slog.Error(
					"failed to unmarshal channel message",
					slog.String("error", err.Error()),
				)
				return
			}

			c.msgTx <- msgVO
		},
	)

	return err
}

func handleNATSMsg(msg *nats.Msg, msgTx chan<- *vo.ChanMsgVO) {
	msgVO := &vo.ChanMsgVO{}

	if err := json.Unmarshal(msg.Data, msgVO); err != nil {
		slog.Error(
			"failed to unmarshal channel message",
			"error", err.Error(),
		)

		return
	}

	msgTx <- msgVO
}
