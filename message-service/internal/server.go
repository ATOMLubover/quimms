package internal

import (
	"poprako-list/internal/config"
	"poprako-list/internal/handler"
	"poprako-list/internal/repo"
)

func Serve() error {
	cfg, err := config.LoadConfig()

	if err != nil {
		return err
	}

	db, err := repo.InitDB()

	if err != nil {
		return err
	}

	app := handler.NewApp(cfg, db)

	addr := cfg.Host + ":" + cfg.Port

	return app.Listen(addr)
}
