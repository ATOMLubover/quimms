package cache

import (
	"errors"
	"os"

	"github.com/redis/go-redis/v9"
)

type Client struct {
	rdb *redis.Client
}

func NewClient(
	redisURL string,
) (*Client, error) {
	redisPwd := os.Getenv("REDIS_PASSWORD")

	if redisPwd == "" {
		return nil, errors.New("REDIS_PASSWORD environment variable is not set")
	}

	redisCli := redis.NewClient(&redis.Options{
		Addr:     redisURL,
		Password: redisPwd,
		DB:       0,
	})

	if redisCli == nil {
		return nil, errors.New("failed to create Redis client")
	}

	return &Client{
		rdb: redisCli,
	}, nil
}
