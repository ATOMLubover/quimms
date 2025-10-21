package state

import (
	"channel-service/internal/config"

	"github.com/bwmarrin/snowflake"
	"gorm.io/gorm"
)

type AppState struct {
	Cfg   *config.AppConfig
	DB    *gorm.DB
	IDGen *snowflake.Node
}
