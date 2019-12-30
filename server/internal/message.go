package internal

import (
	"bytes"
	"encoding/binary"
	"errors"
)

//protocol for phicomm-m1 talks to server(port 9000)
type rawmessage struct {
	header rawheader
	json   string
	end    [5]uint8 //fixed as FF 23 45 4E 44 23
}

//sizeof rawheader = 16 + 12 = 28
const (
	sizeRawHeader     = 28
	sizeActiveMessage = 33
	sizeMinMessage    = 33
	sizeMaxMessage    = 150 //we guess
)

/*
   00 01 02 03 04 05 06 07   08  09 10 11  12 13 14 15
00 -------unknown---------   0B  00 00 00  00 00 00 00
16 ---------MAC-----------   len 00 00 typ
*/
type rawheader struct {
	unknown [16]uint8 //fixed for every device
	mac     [8]uint8  //00-mac-00
	length  uint8     //length of (padding + msgType + json)
	padding [2]uint8  //fixed as 00 00
	msgType uint8     //1:active,2:control,4:report
}

type message struct {
	mac     string
	msgType uint8
	json    string
}

func (m *message) readMsg(buf []byte) error {
	var header rawheader
	b := bytes.NewBuffer(buf)
	if err := binary.Read(b, binary.BigEndian, &header); err != nil {
		return err
	}
	jsonBegin := 28
	jsonEnd := jsonBegin + int(header.length) - 3

	if !(jsonBegin < len(buf) && jsonBegin <= jsonEnd && jsonBegin < len(buf)) {
		return errors.New("bad packet")
	}
	m.msgType = header.msgType
	m.json = string(buf[jsonBegin:jsonEnd])
	return nil
}
