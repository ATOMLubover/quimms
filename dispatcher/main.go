package main

import (
	"context"
	"dispatcher/internal/config"
	"dispatcher/internal/mq"
	"dispatcher/internal/registry"
	"dispatcher/internal/state"
	"errors"
	"fmt"
	"log/slog"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/ATOMLubover/balancer-go"
	"github.com/joho/godotenv"
	"github.com/nats-io/nats.go"
	"github.com/panjf2000/ants"
	"github.com/redis/go-redis/v9"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

func main() {
	// Set global slog logger to Debug level
	logger := slog.New(slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{Level: slog.LevelDebug}))
	slog.SetDefault(logger)

	if err := initEnv(); err != nil {
		slog.Error("failed to initialize environment", "error", err.Error())
		return
	}

	cfg, err := initConfig()

	if err != nil {
		slog.Error("failed to initialize config", "error", err.Error())
		return
	}

	slog.Info("loaded app config successfully")

	natsConn, err := initNATS(cfg)

	if err != nil {
		slog.Error("failed to initialize NATS", "error", err.Error())
		return
	}

	slog.Info("connected to NATS server successfully")

	rdb, err := initRedis(cfg)

	if err != nil {
		slog.Error("failed to initialize Redis", "error", err.Error())
		return
	}

	slog.Info("connected to Redis server successfully")

	reg, err := initRegistry(cfg)

	if err != nil {
		slog.Error("failed to initialize registry", "error", err.Error())
		return
	}

	chanSrv, err := initChanSrv(reg)

	if err != nil {
		slog.Error("failed to initialize channel service", "error", err.Error())
		return
	}

	stopCh := make(chan struct{})

	connSrv, err := initConnSrv(cfg, stopCh)

	if err != nil {
		slog.Error("failed to initialize connector service", "error", err.Error())
		return
	}

	slog.Info("created service registry clients successfully")

	dispPool, err := initPool()

	if err != nil {
		slog.Error("failed to initialize dispatcher pool", "error", err.Error())
		return
	}

	state, err := initState(chanSrv, connSrv, natsConn, rdb, dispPool)

	if err != nil {
		slog.Error("failed to initialize state", "error", err.Error())
		return
	}

	defer func() {
		if err := state.Drop(); err != nil {
			slog.Error("error occurred when dropping app state", "error", err.Error())
		}
	}()

	sub, err := initMQ(state)

	if err != nil {
		slog.Error("failed to initialize MQ", "error", err.Error())
		return
	}

	defer func() {
		if err := sub.Unsubscribe(); err != nil {
			slog.Error("failed to unsubscribe from NATS subject", "error", err.Error())
		}
	}()

	slog.Info("started pulling messages from NATS successfully")

	slog.Info("dispatcher server is now running...")

	waitExitSig()

	slog.Info("shutting down dispatcher...")
}

func initEnv() error {
	return godotenv.Load()
}

func initConfig() (*config.AppConfig, error) {
	return config.LoadConfig()
}

func initNATS(cfg *config.AppConfig) (*nats.Conn, error) {
	return nats.Connect(cfg.NATSURL)
}

func initRedis(cfg *config.AppConfig) (*redis.Client, error) {
	redisPwd := os.Getenv("REDIS_PASSWORD")

	if redisPwd == "" {
		return nil, errors.New("REDIS_PASSWORD environment variable is not set")
	}

	rdb := redis.NewClient(&redis.Options{
		Addr:     cfg.RedisAddr,
		Password: redisPwd,
		DB:       0,
	})

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)

	defer cancel()

	return rdb, rdb.Ping(ctx).Err()
}

func initRegistry(cfg *config.AppConfig) (*registry.ConsulClient[*grpc.ClientConn], error) {
	return registry.NewConsulClient(
		cfg.ConsulAddr,
		transFunc,
		cleanFunc,
	)
}

func initChanSrv(reg *registry.ConsulClient[*grpc.ClientConn]) (*state.ChanSrvBalancer, error) {
	return balancer.NewBalancer(
		reg,
		"ChannelService",
		30,
	)
}

func initConnSrv(cfg *config.AppConfig, stopCh <-chan struct{}) (*registry.ConsulClient[*grpc.ClientConn], error) {
	return newConnSrv(cfg.ConsulAddr, stopCh)
}

func initPool() (*ants.Pool, error) {
	return ants.NewPool(64)
}

func initState(
	chanSrv *state.ChanSrvBalancer,
	connSrv *registry.ConsulClient[*grpc.ClientConn],
	natsConn *nats.Conn,
	rdb *redis.Client,
	dispPool *ants.Pool,
) (*state.AppState, error) {
	return state.NewAppState(
		chanSrv,
		connSrv,
		natsConn,
		rdb,
		dispPool,
	), nil
}

func initMQ(state *state.AppState) (*nats.Subscription, error) {
	return mq.RunPullMsg(state)
}

func waitExitSig() {
	sigCh := make(chan os.Signal, 1)

	signal.Notify(sigCh, os.Interrupt, syscall.SIGTERM)

	<-sigCh
}

func transFunc(inst *registry.ConsulSrvInst) (*grpc.ClientConn, error) {
	target := fmt.Sprintf("%s:%d", inst.Address, inst.Port)

	conn, err := grpc.NewClient(
		target,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)

	if err != nil {
		return nil, fmt.Errorf("failed to connect to gRPC server %s: %v", target, err)
	}

	return conn, nil
}

func cleanFunc(conn *grpc.ClientConn) error {
	return conn.Close()
}

func newConnSrv(consulAddr string, stopCh <-chan struct{}) (*registry.ConsulClient[*grpc.ClientConn], error) {
	connSrv, err := registry.NewConsulClient(
		consulAddr,
		transFunc,
		cleanFunc,
	)

	if err != nil {
		return nil, fmt.Errorf("failed to create Consul registry client: %v", err)
	}

	tck := time.NewTicker(30 * time.Second)

	go func() {
		defer tck.Stop()

		const kConnSrvPref = "ConnectorService"

		for {
			select {
			case <-tck.C:
				slog.Info("pulling connector service instances")

				if _, err := connSrv.PullInst(kConnSrvPref); err != nil {
					slog.Error("failed to pull instances from Consul server")
					return
				}

			case <-stopCh:
				slog.Info("stopping connector service puller")
				return
			}
		}
	}()

	return connSrv, nil
}
