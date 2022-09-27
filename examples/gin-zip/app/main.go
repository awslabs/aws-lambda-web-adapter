package main

import (
	"os"
	
	"github.com/gin-gonic/gin"
)

func main() {
	if env := os.Getenv("ENV"); env != "local" {
		gin.SetMode(gin.ReleaseMode)
	}

	r := gin.Default()

	r.GET("/", func(c *gin.Context) {
		c.JSON(200, gin.H{
			"message": "hello world",
		})
	})
	r.Run() // listen and serve on 0.0.0.0:8080
}
