package registry

import (
	"encoding/json"
	"fmt"
	"io"
	"log/slog"
	"net/http"
	"sync"

	"github.com/ATOMLubover/balancer-go"
)

type ConsulClient[T any] struct {
	httpCli http.Client
	regURL  string

	trans TransFunc[T]
	clean CleanFunc[T]

	mu       sync.RWMutex
	isClosed bool
	storeMap map[string]T
}

type TransFunc[T any] = func(inst *ConsulSrvInst) (T, error)
type CleanFunc[T any] = func(store T) error

func NewConsulClient[T any](
	consulURL string,
	transFunc TransFunc[T],
	cleanFunc CleanFunc[T],
) (*ConsulClient[T], error) {
	return &ConsulClient[T]{
		regURL:   consulURL,
		trans:    transFunc,
		clean:    cleanFunc,
		isClosed: false,
		storeMap: make(map[string]T),
	}, nil
}

type ConsulSrvInst struct {
	ServiceID   string `json:"ServiceID"`
	ServiceName string `json:"ServiceName"`
	Address     string `json:"Address"`
	Port        uint16 `json:"Port"`
}

// Implement Registry interface.
func (c *ConsulClient[T]) PullInst(srvPref string) ([]balancer.SrvInst[T], error) {
	url := fmt.Sprintf("%s/v1/health/services/%s", c.regURL, srvPref)

	rsp, err := c.httpCli.Get(url)

	if err != nil {
		return nil, err
	}

	defer rsp.Body.Close()

	if rsp.StatusCode != 200 {
		return nil, fmt.Errorf(
			"failed to discover connector service, status code: %d",
			rsp.StatusCode,
		)
	}

	body, err := io.ReadAll(rsp.Body)

	if err != nil {
		return nil, err
	}

	var instLst []ConsulSrvInst

	if err := json.Unmarshal(body, &instLst); err != nil {
		return nil, err
	}

	resLst := make([]balancer.SrvInst[T], 0, len(instLst))
	newMap := make(map[string]T)

	for _, inst := range instLst {
		store, err := c.trans(&inst)

		if err != nil {
			// For a better availability, just log the error and continue.
			slog.Error(
				"Failed to transform connector service instance",
				"service_id", inst.ServiceID,
				"service_name", inst.ServiceName,
				"error", err,
			)

			continue
		}

		newMap[inst.ServiceName+":"+inst.ServiceID] = store

		resLst = append(resLst, balancer.SrvInst[T]{
			SrvID:   inst.ServiceName,
			SrvName: inst.ServiceName,
			Addr:    inst.Address,
			Port:    inst.Port,
			Store:   store,
		})
	}

	c.mu.Lock()

	c.storeMap = newMap

	c.mu.Unlock()

	return resLst, nil
}

func (c *ConsulClient[T]) GetStore(key string) T {
	c.mu.RLock()

	defer c.mu.RUnlock()

	return c.storeMap[key]
}

func (c *ConsulClient[T]) RmvStore(key string) {
	c.mu.Lock()

	defer c.mu.Unlock()

	delete(c.storeMap, key)
}

func (c *ConsulClient[T]) Close() error {
	if c.isClosed {
		return nil
	}

	c.mu.Lock()

	defer c.mu.Unlock()

	c.isClosed = true

	// Clean up all stores.
	if c.clean != nil {
		for _, store := range c.storeMap {
			if err := c.clean(store); err != nil {
				slog.Error(
					"Failed to clean up connector service client",
					"error", err,
				)
			}
		}
	}

	return nil
}
