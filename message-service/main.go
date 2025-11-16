package main

import (
	"fmt"
	"log/slog"
	"message-service/internal/pbserver"
	"os"

	"github.com/joho/godotenv"
)

func main() {
	if err := InitEnv(); err != nil {
		fmt.Println(err)
		return
	}

	// Set global slog logger to Debug level
	logger := slog.New(slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{Level: slog.LevelDebug}))
	slog.SetDefault(logger)

	if err := pbserver.RunServer(); err != nil {
		fmt.Println(err)
		return
	}
}

func InitEnv() error {
	if err := godotenv.Load(); err != nil {
		return fmt.Errorf("Error loading .env file: %s", err)
	}

	return nil
}
