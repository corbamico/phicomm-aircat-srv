package main

import (
	"github.com/corbamico/phicomm-aircat-srv/server/internal"
	"log"
)

func main() {
	if err := internal.LoadConfig("config.json"); err != nil {
		log.Fatalln(err)
	}
	return
}
