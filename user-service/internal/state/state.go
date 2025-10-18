package state

import (
	"user-service/internal/config"

	"github.com/bwmarrin/snowflake"
	"gorm.io/gorm"
)

type AppState struct {
	Cfg      *config.AppConfig
	DB       *gorm.DB
	SnowNode *snowflake.Node
}
