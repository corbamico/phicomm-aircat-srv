package internal

import (
	"log"
	"net/http"
)

/*
APIs:
	1.query current measure, ignore mac
	GET v1/aircat/{MAC}
	Response: {"temperature"=1,"humidity"=2,"value"=3,"hcho"=4}
	2.change brightness. currently ignore mac, we have only one aircat.
	PUT v1/aricat/{MAC}
		{"brightness":"100","type":2}
*/

//restServer run at port 9000
type restServer struct{}

//newrestServer create a restServer
func newrestServer() restServer {
	return restServer{}
}

//Run restServer at port 9000
func (s *restServer) Run() {
	go func() {
		http.HandleFunc("/v1/aircat", aricat)
		log.Fatalln(http.ListenAndServe(configs.RESTerverAddr, nil))
	}()
}

func aricat(w http.ResponseWriter, r *http.Request) {
	res := "{\"temperature\"=1,\"humidity\"=2,\"value\"=3,\"hcho\"=4}"

	if r.Method == http.MethodGet {
		w.Header().Add("Content-Type", "application/json")
		w.Write([]byte(res))
		return
	}
	if r.Method == http.MethodPut {
		w.WriteHeader(http.StatusNoContent)
		return
	}
}
