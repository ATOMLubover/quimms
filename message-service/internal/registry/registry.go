package registry

import (
	"fmt"
	"net/http"

	consulapi "github.com/hashicorp/consul/api"
)

func RunRegistryClient(
	consulsAddr string,
	serviceID, serviceName string,
	serviceAddr string, servicePort int,
) error {
	cfg := consulapi.DefaultConfig()

	cfg.Address = consulsAddr

	client, err := consulapi.NewClient(cfg)

	if err != nil {
		return err
	}

	reg := &consulapi.AgentServiceRegistration{
		ID:      serviceID,
		Name:    serviceName,
		Address: serviceAddr,
		Port:    servicePort,
		Check: &consulapi.AgentServiceCheck{
			HTTP:                           fmt.Sprintf("http://%s:%d/health", serviceAddr, servicePort),
			Interval:                       "30s",
			DeregisterCriticalServiceAfter: "1m",
		},
	}

	if err := client.Agent().ServiceRegister(reg); err != nil {
		return err
	}

	// Detach a goroutine to periodically response to the TTL health check.
	go func() {
		http.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
			w.WriteHeader(http.StatusOK)
			w.Write([]byte("OK"))
		})

		http.ListenAndServe(fmt.Sprintf("%s:%d", serviceAddr, servicePort), nil)
	}()

	return nil
}
