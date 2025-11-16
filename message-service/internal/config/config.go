package config

import (
	"fmt"

	"github.com/spf13/viper"
)

type AppConfig struct {
	ServiceID            string `mapstructure:"service_id"`
	ServiceName          string `mapstructure:"service_name"`
	Host                 string `mapstructure:"host"`
	Port                 uint16 `mapstructure:"port"`
	MQURL                string `mapstructure:"mq_url"`
	RedisURL             string `mapstructure:"redis_url"`
	ConsulsAddr          string `mapstructure:"consuls_addr"`
	HealthTTLSeconds     uint16 `mapstructure:"health_ttl_seconds"`
	HealthRefreshSeconds uint16 `mapstructure:"health_refresh_seconds"`
}

func LoadConfig() (*AppConfig, error) {
	viper.SetConfigFile("app_config.json")
	viper.SetConfigType("json")

	if err := viper.ReadInConfig(); err != nil {
		return nil, fmt.Errorf("Error when reading in config file: %s", err)
	}

	cfg := &AppConfig{}

	if err := viper.Unmarshal(cfg); err != nil {
		return nil, fmt.Errorf("Error when unmarshaling config file: %s", err)
	}

	return cfg, nil
}
