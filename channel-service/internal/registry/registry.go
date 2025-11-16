package registry

import (
	"fmt"
	"time"

	consulapi "github.com/hashicorp/consul/api"
)

func RunRegistryClient(
	consulsAddr string,
	serviceID, serviceName string,
	serviceAddr string, servicePort int,
	ttlSeconds int, refreshSeconds int,
) error {
	cfg := consulapi.DefaultConfig()

	cfg.Address = consulsAddr

	client, err := consulapi.NewClient(cfg)

	if err != nil {
		return err
	}

	checkID := fmt.Sprintf("%s-%s", serviceName, serviceID)
	ttlStr := fmt.Sprintf("%ds", ttlSeconds)

	reg := &consulapi.AgentServiceRegistration{
		ID:      serviceID,
		Name:    serviceName,
		Address: serviceAddr,
		Port:    servicePort,
		Check: &consulapi.AgentServiceCheck{
			CheckID:                        checkID,
			Name:                           serviceName,
			TTL:                            ttlStr,
			DeregisterCriticalServiceAfter: "1m",
		},
	}

	if err := client.Agent().ServiceRegister(reg); err != nil {
		return err
	}

	// Active TTL refresh loop
	go func() {
		if refreshSeconds <= 0 {
			refreshSeconds = 5
		}
		ticker := time.NewTicker(time.Duration(refreshSeconds) * time.Second)
		defer ticker.Stop()
		for range ticker.C {
			_ = client.Agent().PassTTL(checkID, "ok")
		}
	}()

	return nil
}
