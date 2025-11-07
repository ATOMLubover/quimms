package registry

import (
	"dispatcher/pb"
	"encoding/json"
	"fmt"
	"io"
	"net/http"

	"github.com/ATOMLubover/balancer-go"
)

type Client struct {
	httpCli http.Client
	regURL  string
}

func NewClient(regURL string) (*Client, error) {
	return &Client{
		regURL: regURL,
	}, nil
}

const kConnSrvName = "connector"

// Implement Registry interface.
func (c *Client) PullInst(srvPref string) ([]balancer.SrvInst[pb.ChannelServiceClient], error) {
	url := fmt.Sprintf("%s/v1/health/services/%s", c.regURL, kConnSrvName)

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

	var instances []*ServiceInstance

	if err := json.Unmarshal(body, &instances); err != nil {
		return nil, err
	}

	return instances, nil
}
