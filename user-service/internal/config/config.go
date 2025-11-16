package config

import (
	"fmt"
	"path/filepath"

	"github.com/spf13/viper"
)

type AppConfig struct {
	ServiceID            string `mapstructure:"service_id"`
	ServiceName          string `mapstructure:"service_name"`
	Host                 string `mapstructure:"host"`
	Port                 uint16 `mapstructure:"port"`
	ConsulsAddr          string `mapstructure:"consuls_addr"`
	HealthTTLSeconds     uint16 `mapstructure:"health_ttl_seconds"`
	HealthRefreshSeconds uint16 `mapstructure:"health_refresh_seconds"`
}

func LoadConfig() (*AppConfig, error) {
	cfgPath := "app_config.json"

	cfgAbsPath, _ := filepath.Abs(cfgPath)

	viper.SetConfigFile(cfgAbsPath)
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
