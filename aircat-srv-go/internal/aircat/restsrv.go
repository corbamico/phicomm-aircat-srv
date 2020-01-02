package aircat

import (
	"io/ioutil"
	"log"
	"net/http"
)

/*
APIs:
	1.query current measure, ignore mac
	GET v1/aircat/{id}
	Response: {"temperature"=1,"humidity"=2,"value"=3,"hcho"=4}
	2.change brightness. currently ignore mac, we have only one aircat.
	PUT v1/aricat/{id}
		{"brightness":"100","type":2}
*/

//restServer run at port 9000
type restServer struct {
	controlChan  chan string
	aircatServer *AircatServer
}

//newrestServer create a restServer
func newrestServer(controlChan chan string) *restServer {
	return &restServer{controlChan: controlChan}
}

func (s *restServer) setAircatServer(aircatServer *AircatServer) {
	s.aircatServer = aircatServer
}

//Run restServer at port 9000
func (s *restServer) Run() {
	go func() {
		http.HandleFunc("/v1/aircat", handlerFunc(s))
		log.Printf("REST Server run at %s\n", configs.RESTServerAddr)
		log.Fatalln(http.ListenAndServe(configs.RESTServerAddr, nil))
	}()
}

func handlerFunc(s *restServer) func(w http.ResponseWriter, r *http.Request) {
	handler := func(w http.ResponseWriter, r *http.Request) {
		if r.Method == http.MethodGet {
			w.Header().Add("Content-Type", "application/json")
			w.Write([]byte(s.aircatServer.getCurrentMessage()))
			return
		}
		if r.Method == http.MethodPut {
			body, err := ioutil.ReadAll(r.Body)
			if err != nil {
				w.WriteHeader(http.StatusBadRequest)
			}
			s.controlChan <- string(body)
			w.WriteHeader(http.StatusNoContent)
			return
		}
		w.WriteHeader(http.StatusBadRequest)
		return
	}
	return handler
}
