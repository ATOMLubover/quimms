package config

import (
	"fmt"
	"path/filepath"

	"github.com/spf13/viper"
)

type AppConfig struct {
	Host string
	Port string
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
