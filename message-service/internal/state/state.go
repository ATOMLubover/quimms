package state

import (
	"message-service/internal/cache"
	"message-service/internal/config"
	"message-service/internal/mq"

	"github.com/bwmarrin/snowflake"
	"gorm.io/gorm"
)

type AppState struct {
	Cfg      *config.AppConfig
	DB       *gorm.DB
	IDGen    *snowflake.Node
	MQCli    *mq.Client
	CacheCli *cache.Client
}
