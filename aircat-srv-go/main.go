package main

import (
	"github.com/corbamico/phicomm-aircat-srv/aircat-srv-go/internal/aircat"
	"log"
)

func main() {
	if err := aircat.LoadConfig("config.json"); err != nil {
		log.Fatalln(err)
	}
	s := aircat.NewAircatServer()
	s.Run()
	return
}
