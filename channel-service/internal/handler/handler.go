package handler

import (
	"channel-service/internal/config"

	"github.com/kataras/iris/v12"
	"github.com/kataras/iris/v12/hero"
	"gorm.io/gorm"
)

// As the shared state of the application,
// appState should be thread-safe.
type appState struct {
	cfg *config.AppConfig
	db  *gorm.DB
}

func NewApp(cfg *config.AppConfig, db *gorm.DB) *iris.Application {
	state := &appState{
		cfg: cfg,
		db:  db,
	}

	hero.Register(state)

	app := iris.Default()

	app.Get("/health", hero.Handler(HealthCheck))

	return app
}
