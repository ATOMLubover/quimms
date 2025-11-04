package mq

import "github.com/nats-io/nats.go"

type Client struct {
	conn *nats.Conn
}

func NewClient(mqURL string) (*Client, error) {
	natsConn, err := nats.Connect(mqURL)

	if err != nil {
		return nil, err
	}

	return &Client{
		conn: natsConn,
	}, nil
}
