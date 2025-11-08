package config

import (
	"fmt"

	"github.com/spf13/viper"
)

type AppConfig struct {
	NATSURL    string `mapstructure:"nats_url"`
	RedisAddr  string `mapstructure:"redis_address"`
	ConsulAddr string `mapstructure:"consuls_address"`
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
