package internal

import (
	"encoding/json"
	"log"
	"net"
	"os"
)

//AircatServer run at port 9000
type AircatServer struct {
	w writer
}

//NewAircatServer create a AircatServer
func NewAircatServer() AircatServer {
	return AircatServer{
		w: influxdb{addr: configs.InfluxdbServer},
	}
}

//Run AircatServer at port 9000
func (s *AircatServer) Run() {
	listen, err := net.Listen("tcp", ":9000")
	if err != nil {
		log.Fatalln(err)
	}
	for {
		conn, err := listen.Accept()
		if err != nil {
			log.Println(err)
		}
		go s.handleConnection(conn)
	}
}
func (s *AircatServer) handleConnection(conn net.Conn) {
	buf := make([]byte, 10240)
	for {
		len, err := conn.Read(buf)
		var msg message
		if err != nil {
			break
		}
		if len < sizeMinMessage || len > sizeMaxMessage {
			continue
		}
		if err = msg.readMsg(buf[0:len]); err != nil {
			continue
		}
		//we got right packet
		//then we write to influxdb
		s.w.write(msg.mac, msg.json)
	}
	conn.Close()
}

//Config for running programme
//{
//  "ServerAddr":":8080",
//  "InfluxdbServer":"localhost:8086"
//}
type Config struct {
	ServerAddr     string
	RESTServer     bool
	RESTerverAddr  string
	Influxdb       bool
	InfluxdbServer string
}

var configs Config

//LoadConfig load config file
func LoadConfig(file string) error {
	configFile, err := os.Open(file)
	if err != nil {
		return err
	}
	defer configFile.Close()
	jsonParse := json.NewDecoder(configFile)
	if err = jsonParse.Decode(&configs); err != nil {
		return err
	}
	return nil
}
