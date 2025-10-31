package mq

import (
	"context"
	"encoding/json"
	"log/slog"
	"message-service/internal/model/vo"

	"github.com/nats-io/nats.go"
)

func BuildPushMessageWorker(
	mqURL string,
	ctx context.Context,
	recv <-chan vo.ChannelMessageVO,
) error {
	natsConn, err := nats.Connect(mqURL)

	if err != nil {
		return err
	}

	go PushMessage(natsConn, ctx, recv)

	return nil
}

// The creation of NATS connection should not be delegated to a
// asynchronous goroutine, because the connection process may fail.
func PushMessage(
	natsConn *nats.Conn,
	ctx context.Context,
	recv <-chan vo.ChannelMessageVO,
) {
	sync.Map {}

	for {
		select {
		case msg := <-recv:
			data, err := json.Marshal(msg)

			if err != nil {
				slog.Error("Failed to marshal channel message VO",
					slog.String("error", err.Error()))
				continue
			}

			err = natsConn.Publish(TopicPushChannelMessage, data)

			if err != nil {
				// TODO: Consider retrying on failure(backed by a retry counter and
				//  repush into the channel), or just end the task
				// in case of an infinite retry?
				slog.Error("Failed to publish channel message",
					slog.String("error", err.Error()))
				continue
			}

			slog.Debug("Successfully pushed message",
				slog.String("message_id", msg.ID))

		case <-ctx.Done():
			slog.Info("Stopping pushing message to MQ")
			return
		}
	}
}
