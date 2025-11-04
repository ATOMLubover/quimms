package main

import (
	"fmt"

	"github.com/joho/godotenv"
)

func main() {
	if err := InitEnv(); err != nil {
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
