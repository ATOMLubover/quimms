package cache

import "time"

const (
	kMsgChanBufSize = 512
	kChanMsgEx      = 3600 * 24 // 1 day.
	kChanMsgPrefix  = "channel:messages:"

	kLockPrefix  = "lock:"
	kLockTimeout = 10 * time.Second // 10 seconds
)
