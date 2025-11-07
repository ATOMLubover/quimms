package cache

import "context"

func (c *Client) GetUserConnect(ctx context.Context, userIDs []string) (map[string]string, error) {
	connAddrs, err := c.rdb.
		HMGet(ctx, "user:connector", userIDs...).
		Result()

	if err != nil {
		return nil, err
	}

	mapper := make(map[string]string)

	for i, addr := range connAddrs {
		if addrStr, ok := addr.(string); ok {
			mapper[userIDs[i]] = addrStr
		}
	}

	return mapper, nil
}
