package handler

import "github.com/kataras/iris/v12"

func HealthCheck(ctx iris.Context, state *appState) {
	ctx.WriteString("OK")
}
