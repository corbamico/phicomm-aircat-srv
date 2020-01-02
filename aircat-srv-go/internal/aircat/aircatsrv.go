package aircat

import (
	"encoding/hex"
	"encoding/json"
	"log"
	"net"
	"os"
)

//AircatServer run at port 9000
type AircatServer struct {
	w           influxdb
	controlChan chan string   //used for send json/string from restSrv to aircatDevice.listenControl()
	device      *aircatDevice //currently, we only keep one device connection. maybe we create vec in future.
	restSrv     *restServer
}

//NewAircatServer create a AircatServer
func NewAircatServer() AircatServer {
	controlChan := make(chan string)

	return AircatServer{
		w:           influxdb{addr: configs.InfluxdbServer},
		controlChan: controlChan,
		device:      nil,
		restSrv:     newrestServer(controlChan),
	}
}

//Run AircatServer at port 9000
func (s *AircatServer) Run() {
	listen, err := net.Listen("tcp", configs.ServerAddr)
	if err != nil {
		log.Fatalln(err)
	}
	log.Printf("Server run at %s\n", listen.Addr().String())
	//run REST Server in backend go-routine
	s.restSrv.setAircatServer(s)
	s.restSrv.Run()

	//run recieve controlmsg in backend go-routine
	go s.listenControl()

	for {
		conn, err := listen.Accept()

		if err != nil {
			log.Println(err)
			continue
		}
		log.Printf("Cient connected at %s\n", conn.RemoteAddr().String())
		//Caution:
		//we only keep one device connection
		if s.device != nil {
			s.device.cleanup()
		}
		s.device = newAircatDevice(s, conn)
		go s.device.run()
	}
}

func (s *AircatServer) listenControl() {
	for {
		select {
		case json, ok := <-s.controlChan:
			if !ok {
				//chan closed.
				return
			}
			if s.device != nil {
				//we ignore error
				s.device.sendControl(json)
			}
		}
	}
}
func (s *AircatServer) getCurrentMessage() (res string) {
	if s.device != nil {
		res = s.device.msg.json
	}
	return
}

type aircatDevice struct {
	sever *AircatServer
	conn  net.Conn
	msg   message //last report message
}

func newAircatDevice(sever *AircatServer, conn net.Conn) *aircatDevice {
	return &aircatDevice{sever: sever, conn: conn}
}

func (client *aircatDevice) run() {
	buf := make([]byte, 10240)
	for {
		len, err := client.conn.Read(buf)
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
		if msg.header.MsgType == 4 {
			client.msg = msg
		}
		client.sever.w.write(hex.EncodeToString(msg.header.Mac[1:7]), msg.json)
	}
	client.conn.Close()
}

func (client *aircatDevice) cleanup() {
	if client.conn != nil {
		client.conn.Close()
	}
}
func (client *aircatDevice) sendControl(json string) {
	bytes := client.msg.controlMsg(json)
	client.conn.Write(bytes)
	// log.Println("\n", hex.Dump(bytes))
	// if n, err := client.conn.Write(bytes); err == nil {
	// 	log.Printf("send control message (size=%d)\n", n)
	// }
}

//Config for running programme
//{
//  "ServerAddr":":9000",
//  "InfluxdbServer":"localhost:8086"
//}
type Config struct {
	ServerAddr     string //default as ":9000"
	RESTServerAddr string //default as ":8080"
	Influxdb       bool
	InfluxdbServer string //default as "localhost:8086"
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
	if configs.ServerAddr == "" {
		configs.ServerAddr = ":9000"
	}
	if configs.RESTServerAddr == "" {
		configs.RESTServerAddr = ":8080"
	}
	// if configs.InfluxdbServer == "" {
	// 	configs.InfluxdbServer = "localhost:8086"
	// }
	return nil
}
