package mq

import (
	"dispatcher/internal/cache"
	"dispatcher/internal/model/vo"
	"dispatcher/internal/rpc/service"
	"dispatcher/internal/state"
	"dispatcher/pb"
	"encoding/json"
	"fmt"
	"log/slog"

	"github.com/nats-io/nats.go"
	"github.com/panjf2000/ants"
	"github.com/redis/go-redis/v9"
)

const (
	kTopicChanMsg = "channel.message"
	kChanMsgQue   = "dispatcher"
)

func RunPullMsg(s *state.AppState) (*nats.Subscription, error) {
	sub, err := s.NATSConn.QueueSubscribe(
		kTopicChanMsg,
		kChanMsgQue,
		func(msg *nats.Msg) {
			handleNATSMsg(
				msg,
				s.Rdb,
				s.ChanSrv,
				s.ConnSrv,
				s.DispPool,
			)
		},
	)

	if err != nil {
		return nil, fmt.Errorf("failed to subscript to channel message topic: %s", err.Error())
	}

	return sub, nil
}

func handleNATSMsg(
	msg *nats.Msg,
	rdb *redis.Client,
	chanSrv *state.ChanSrvBalancer,
	connSrv *state.ConnSrvClient,
	exePool *ants.Pool,
) {
	msgVO := &vo.ChanMsgVO{}

	if err := json.Unmarshal(msg.Data, msgVO); err != nil {
		slog.Error(
			"failed to unmarshal channel message",
			"error", err.Error(),
		)

		return
	}

	memIDs, err := listChanMemIDs(chanSrv, msgVO.ChanID)

	if err != nil {
		slog.Error(
			"failed to list channel members",
			"channel_id", msgVO.ChanID,
			"error", err.Error(),
		)

		return
	}

	tokens, err := getUsrConnTokens(rdb, memIDs)

	if err != nil {
		slog.Error(
			"failed to get user connector tokens",
			"channel_id", msgVO.ChanID,
			"error", err.Error(),
		)

		return
	}

	for i, t := range tokens {
		tkn := t
		target := memIDs[i]
		conn := connSrv.GetStore(t)

		// Rate limit message dispatching using goroutine pool.
		exePool.Submit(func() {
			cli := pb.NewDispatchServiceClient(conn)

			if err := service.TransMsg(cli, target, msgVO); err != nil {
				slog.Error(
					"failed to dispatch message to connector",
					"connector_token", tkn,
					"message_id", msgVO.MsgID,
					"error", err.Error(),
				)
			}
		})
	}
}

func listChanMemIDs(
	chanSrv *state.ChanSrvBalancer,
	chanID string,
) ([]string, error) {
	inst, err := chanSrv.Next()

	if err != nil {
		return nil, fmt.Errorf("failed to get next channel service instance: %s", err.Error())
	}

	cli := pb.NewChannelServiceClient(inst.Store)

	return service.ListChanMemIDs(cli, chanID)
}

func getUsrConnTokens(
	rdb *redis.Client,
	userIDs []string,
) ([]string, error) {
	return cache.ListUsrConnTokens(rdb, userIDs)
}
