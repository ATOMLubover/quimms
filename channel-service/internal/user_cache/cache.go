package user_cache

import (
	"time"

	"github.com/hashicorp/golang-lru/v2/expirable"
)

type UserCache struct {
	local *expirable.LRU[string, struct{}]
}

func NewUserCache(size int, ttl time.Duration) *UserCache {
	local := expirable.NewLRU[string, struct{}](size, nil, ttl)

	return &UserCache{
		local: local,
	}
}

func (c *UserCache) Add(userID string) {
	c.local.Add(userID, struct{}{})
}

func (c *UserCache) Contain(userID string) bool {
	_, ok := c.local.Get(userID)
	return ok
}
