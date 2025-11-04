package cache

import (
	"context"
	"encoding/json"
	"fmt"
	"math/rand/v2"
	"message-service/internal/model/vo"
	"time"

	"github.com/redis/go-redis/v9"
)

func cacheEntryToMsgVO(entry string) vo.ChannelMessageVO {
	var msg vo.ChannelMessageVO

	json.Unmarshal([]byte(entry), &msg)

	return msg
}

func msgVOToCacheEntry(msg vo.ChannelMessageVO) (string, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return "", err
	}

	return string(msgBytes), nil
}

func cacheListToMsgArr(l interface{}) []vo.ChannelMessageVO {
	msgArr := make([]vo.ChannelMessageVO, 0)

	for _, msgStr := range l.([]string) {
		msgArr = append(msgArr, cacheEntryToMsgVO(msgStr))
	}

	return msgArr
}

func pushMsgArr(
	rdb *redis.Client,
	channelID string,
	msgArr ...vo.ChannelMessageVO,
) error {
	key := kChanMsgPrefix + channelID

	msgIntrArr := make([]any, len(msgArr))

	for i, msg := range msgArr {
		msgStr, err := msgVOToCacheEntry(msg)

		if err != nil {
			return err
		}

		msgIntrArr[i] = msgStr
	}

	ctx, cancel := context.WithTimeout(context.Background(), time.Second)

	defer cancel()

	return rdb.Pipeline().BatchProcess(
		ctx,
		rdb.LPush(
			ctx,
			key,
			msgIntrArr...,
		),
		rdb.Expire(
			ctx,
			key,
			kChanMsgEx,
		),
	)
}

// We have to ensure that the expiration of the message list
// will be atomically refreshed when we try to get it.
const kExistAndExpScript = `
-- KEYS[1]: channel ID.
-- ARGV[1]: expiration time in seconds.

local isExisting = redis.call("EXISTS", KEYS[1])

if #isExisting == 0 then
	return 0
end

redis.call("EXPIRE", KEYS[1], ARGV[1])

return 1
`

func (c *Client) PushMsg(msg vo.ChannelMessageVO) error {
	ctx, cancel := context.WithTimeout(context.Background(), time.Second)

	defer cancel()

	// Check whether the target channel message list exists.
	_, err := redis.
		NewScript(kExistAndExpScript).
		Run(ctx, c.rdb, []string{
			kChanMsgPrefix + msg.ChannelID,
		}, kChanMsgEx).
		Result()

	if err == redis.Nil {
		// If not exists, create a new list with expiration.
		msgStr, err := msgVOToCacheEntry(msg)

		if err != nil {
			return err
		}

		ctx, cancel := context.WithTimeout(context.Background(), time.Second)

		defer cancel()

		_, err = c.rdb.LPush(ctx, msg.ChannelID, msgStr).Result()

		return err
	}

	if err != nil {
		// Some error occurred when running the script.
		return err
	}

	// If exists, push the new message to the list.
	msgStr, err := msgVOToCacheEntry(msg)

	if err != nil {
		return err
	}

	// Shadow the older ctx.
	ctx, cancel = context.WithTimeout(context.Background(), time.Second)

	defer cancel()

	_, err = c.rdb.LPush(ctx, msg.ChannelID, msgStr).Result()

	return err
}

const kGetMsgScript = `
-- KEYS[1]: channel ID.

local isExisting = redis.call("EXISTS", KEYS[1])

if #isExisting == 0 then
	return nil
end

return redis.call("LRANGE", KEYS[1], 0, -1)
`

func getFromCache(rdb *redis.Client, channelID string) ([]vo.ChannelMessageVO, error) {
	const kGetTimeout = time.Second

	ctx, cancel := context.WithTimeout(context.Background(), kGetTimeout)

	defer cancel()

	listIntr, err := redis.NewScript(kGetMsgScript).
		Run(ctx, rdb, []string{channelID}).
		Result()

	if err == redis.Nil {
		// TODO: No messages cached for this channel.
		return []vo.ChannelMessageVO{}, redis.Nil
	}

	if err != nil {
		return nil, err
	}

	return cacheListToMsgArr(listIntr), nil
}

type OnMsgListMissed func(channelID string) ([]vo.ChannelMessageVO, error)

const kDelLockScript = `
-- KEYS[1]: lock key.
-- ARGV[1]: expected lock value.

if redis.call("GET", KEYS[1]) == ARGV[1] then
	return redis.call("DEL", KEYS[1])
else
	return 0
end
`

func updateMsgList(
	fn OnMsgListMissed,
	rdb *redis.Client,
	channelID string,
) ([]vo.ChannelMessageVO, error) {
	randID := rand.Int64()

	ctx, cancel := context.WithTimeout(context.Background(), time.Second)

	defer cancel()

	// TODO: We may need watchdog here.
	suc, err := rdb.
		SetNX(
			ctx,
			kLockPrefix+channelID,
			fmt.Sprint(randID),
			kLockTimeout,
		).
		Result()

	defer func() {
		// Release the lock.
		ctx, cancel := context.WithTimeout(context.Background(), time.Second)

		defer cancel()

		_ = redis.
			NewScript(kDelLockScript).
			Run(
				ctx,
				rdb,
				[]string{kLockPrefix + channelID},
				fmt.Sprint(randID),
			).
			Err()
	}()

	if err != nil {
		return nil, err
	}

	if !suc {
		// Failed to acquire the lock.
		// Sleep for a while and retry getting from cache.
		time.Sleep(time.Millisecond * 300)

		msgList, err := getFromCache(rdb, channelID)

		if err == nil {
			// Successfully got from cache after waiting.
			return msgList, nil
		}

		// Still failed, return error.
		// In sacrifice of consistency, let client to retry.
		return nil, err

		// Note: another approach is to wait for a randomized time
		// and fetch from DB, avoiding thundering herd problem.
	}

	// Succeeded to acquire the lock.
	// Call the callback to fetch from DB.
	msgList, err := fn(channelID)

	if err != nil {
		return nil, err
	}

	return msgList, pushMsgArr(rdb, channelID, msgList...)
}

func (c *Client) GetRecentMsgs(
	onMiss OnMsgListMissed,
	channelID string,
) ([]vo.ChannelMessageVO, error) {
	ctx, cancel := context.WithTimeout(context.Background(), time.Second)

	defer cancel()

	listIntr, err := redis.NewScript(kGetMsgScript).
		Run(ctx, c.rdb, []string{channelID}).
		Result()

	if err == redis.Nil {
		// No messages cached for this channel.
		// Try to seek from DB with LOCK.
		return updateMsgList(onMiss, c.rdb, channelID)
	}

	if err != nil {
		return nil, err
	}

	return cacheListToMsgArr(listIntr), nil
}
