package main

import (
	"github.com/corbamico/phicomm-aircat-srv/aircat-srv-go/internal"
	"log"
)

func main() {
	if err := internal.LoadConfig("config.json"); err != nil {
		log.Fatalln(err)
	}
	s := internal.NewAircatServer()
	s.Run()
	return
}
