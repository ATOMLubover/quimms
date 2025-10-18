package repo

import (
	"errors"
	"fmt"
	"os"

	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

func InitDB() (*gorm.DB, error) {
	dsn := os.Getenv("DSN")

	if dsn == "" {
		return nil, errors.New("Error when initializing DB")
	}

	db, err := gorm.Open(postgres.Open(dsn), &gorm.Config{})

	if err != nil {
		return nil, fmt.Errorf("Error when connecting to DB: %s", err)
	}

	return db, nil
}
