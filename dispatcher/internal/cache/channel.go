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

	connTokens, err := rdb.
		HMGet(
			ctx,
			"user:connector",
			userIDs...,
		).
		Result()

	if err != nil {
		return nil, fmt.Errorf("failed to get user connector addresses from Redis: %v", err)
	}

	tokens := make([]string, 0, len(connTokens))

	for i, tkn := range connTokens {
		if tkn == nil {
			slog.Warn(
				"User is offline or has no connector address",
				"user_id", userIDs[i],
			)

			continue
		}

		addr, ok := tkn.(string)

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
