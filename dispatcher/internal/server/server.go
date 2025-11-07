package server

import (
	"context"
	"dispatcher/internal/cache"
	"dispatcher/internal/config"
	"dispatcher/internal/model/vo"
	"dispatcher/internal/mq"
	"dispatcher/internal/service"
	"dispatcher/internal/transfer"
	"errors"
	"fmt"
	"log/slog"
	"os"
	"time"

	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

func RunServer() error {
	cfg, err := config.LoadConfig()

	if err != nil {
		return err
	}

	msgCh := make(chan *vo.ChanMsgVO, 100)

	mqCli, err := mq.NewClient(cfg.MQURL, msgCh)

	if err != nil {
		return err
	}

	if err := mqCli.RunPullMsg(); err != nil {
		return err
	}

	redisCli, err := cache.NewClient(cfg.RedisAddr)

	if err != nil {
		return err
	}

	dsn := os.Getenv("DSN")

	if dsn == "" {
		return errors.New("Error when initializing DB")
	}

	db, err := gorm.Open(postgres.Open(dsn), &gorm.Config{})

	if err != nil {
		return fmt.Errorf("Error when connecting to DB: %s", err)
	}

	trans, err := transfer.NewClient(cfg.ConsulsAddr)

	if err != nil {
		return err
	}

	if err := runDispatch(msgCh, redisCli, trans, db); err != nil {
		return err
	}

	return nil
}

func runDispatch(
	msgCh <-chan *vo.ChanMsgVO,
	redisCli *cache.Client,
	transCli *transfer.Client,
	db *gorm.DB,
) error {
	for {
		msg := <-msgCh

		members, err := service.GetChanMembers(db, msg.ChanID)

		if err != nil {
			slog.Error(
				"Failed to get channel members",
				"channel_id", msg.ChanID,
				"error", err,
			)
			continue
		}

		ctx, cancel := context.WithTimeout(context.Background(), time.Second*2)

		connectAddr, err := redisCli.GetUserConnect(ctx, members)

		if err != nil {
			slog.Error(
				"Failed to get user connector addresses",
				"members", members,
				"error", err,
			)

			cancel()

			continue
		}

		for _, addr := range connectAddr {
			transCli.SendMsg(transfer.MsgRequest{
				Addr: addr,
				Msg:  msg,
			})
		}

		cancel()
	}
}
