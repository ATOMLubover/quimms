package state

import (
	"dispatcher/internal/registry"
	"log/slog"

	"github.com/ATOMLubover/balancer-go"
	"github.com/nats-io/nats.go"
	"github.com/panjf2000/ants"
	"github.com/redis/go-redis/v9"
	"google.golang.org/grpc"
)

type ChanSrvBalancer = balancer.Balancer[
	*grpc.ClientConn,
	*registry.ConsulClient[*grpc.ClientConn],
]

type ConnSrvClient = registry.ConsulClient[*grpc.ClientConn]

type AppState struct {
	ChanSrv  *ChanSrvBalancer
	ConnSrv  *ConnSrvClient
	NATSConn *nats.Conn
	Rdb      *redis.Client
	DispPool *ants.Pool
}

func NewAppState(
	chanSrv *ChanSrvBalancer,
	connSrv *ConnSrvClient,
	natsConn *nats.Conn,
	rdb *redis.Client,
	dispPool *ants.Pool,
) *AppState {
	return &AppState{
		ChanSrv:  chanSrv,
		ConnSrv:  connSrv,
		NATSConn: natsConn,
		Rdb:      rdb,
		DispPool: dispPool,
	}
}

func (s *AppState) Drop() error {
	if err := s.ChanSrv.Close(); err != nil {
		slog.Error(
			"error occurred when closing channel service balancer",
			"error", err.Error(),
		)
	}

	if err := s.ConnSrv.Close(); err != nil {
		slog.Error(
			"error occurred when closing connection service client",
			"error", err.Error(),
		)
	}

	if err := s.NATSConn.Drain(); err != nil {
		slog.Error(
			"error occurred when draining NATS connection",
			"error", err.Error(),
		)
	}

	s.DispPool.Release()

	if err := s.Rdb.Close(); err != nil {
		slog.Error(
			"error occurred when closing Redis client",
			"error", err.Error(),
		)
	}

	return nil
}
