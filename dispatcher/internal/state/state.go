package state

import "github.com/ATOMLubover/balancer-go"

type AppState struct {
	ChanSrvBal balancer.Balancer[]
}
