package mq

import (
	"dispatcher/internal/model/vo"

	"github.com/nats-io/nats.go"
)

type Client struct {
	conn  *nats.Conn
	msgTx chan<- *vo.ChanMsgVO
}

func NewClient(mqURL string, tx chan<- *vo.ChanMsgVO) (*Client, error) {
	natsConn, err := nats.Connect(mqURL)

	if err != nil {
		return nil, err
	}

	return &Client{
		conn:  natsConn,
		msgTx: tx,
	}, nil
}
