package cache

import (
	"context"
	"fmt"
	"log/slog"
	"time"

	"github.com/redis/go-redis/v9"
)

func ListUsrConnTokens(rdb *redis.Client, userIDs []string) ([]string, error) {
	const kTmo = 2 * time.Second

	ctx, cancel := context.WithTimeout(context.Background(), kTmo)

	defer cancel()

	connAddrs, err := rdb.
		HMGet(
			ctx,
			"user:connectors",
			userIDs...,
		).
		Result()

	if err != nil {
		return nil, fmt.Errorf("failed to get user connector addresses from Redis: %v", err)
	}

	tokens := make([]string, 0, len(connAddrs))

	for i, addr := range connAddrs {
		if addr == nil {
			slog.Warn(
				"User is offline or has no connector address",
				"user_id", userIDs[i],
			)

			continue
		}

		addr, ok := addr.(string)

		if !ok || addr == "" {
			slog.Warn(
				"User is offline or has no connector address",
				"user_id", userIDs[i],
			)

			continue
		}

		tokens = append(tokens, addr)
	}

	return tokens, nil
}
