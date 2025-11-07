package transfer

import (
	"context"
	"dispatcher/internal/model/vo"
	"dispatcher/internal/registry"
	"dispatcher/pb"
	"fmt"
	"log/slog"
	"sync"
	"time"

	"google.golang.org/grpc"
)

type Client struct {
	regCli  *registry.Client
	connMtx sync.RWMutex
	conns   map[string]pb.DispatcherClient
}

func NewClient(consulAddr string) (*Client, error) {
	regCli, err := registry.NewClient(consulAddr)

	if err != nil {
		return nil, err
	}

	return &Client{
		regCli:  regCli,
		connMtx: sync.RWMutex{},
		conns:   make(map[string]pb.DispatcherClient),
	}, nil
}

func (t *Client) UpdateConns() {
	instances, err := t.regCli.DiscConnSrv()

	if err != nil {
		return
	}

	newConns := make(map[string]pb.DispatcherClient)

	for _, inst := range instances {
		conn, err := grpc.NewClient(fmt.Sprintf("%s:%d", inst.Address, inst.Port))

		if err != nil {
			slog.Error("failed to create gRPC client", "error", err)
			continue
		}

		cli := pb.NewDispatcherClient(conn)

		newConns[inst.ID] = cli
	}

	// Minimize the critical section.
	t.connMtx.Lock()

	t.conns = newConns

	t.connMtx.Unlock()
}

type MsgRequest struct {
	Addr string
	Msg  *vo.ChanMsgVO
}

func (t *Client) SendMsg(msg ...MsgRequest) error {
	t.connMtx.RLock()
	defer t.connMtx.RUnlock()

	for _, m := range msg {
		cli, ok := t.conns[m.Addr]
		if !ok {
			slog.Error("no connection found for address", "addr", m.Addr)
			continue
		}

		ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
		defer cancel()

		in, err := cli.DispatchMessage(ctx)
		if err != nil {
			slog.Error("failed to dispatch message", "error", err)
			continue
		}

		if err := in.Send(&pb.DispatchMessageRequest{
			UserId:    m.Msg.UserID,
			Content:   m.Msg.Content,
			ChannelId: m.Msg.ChanID,
			CreatedAt: m.Msg.CreatedAt,
		}); err != nil {
			slog.Error("failed to send message", "error", err)
			continue
		}
	}

	return nil
}
