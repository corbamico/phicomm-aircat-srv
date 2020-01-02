package aircat

import (
	"bytes"
	"encoding/binary"
	"errors"
)

//protocol for phicomm-m1 talks to server(port 9000)
type rawmessage struct {
	header Rawheader
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

/*Rawheader show as
   00 01 02 03 04 05 06 07   08  09 10 11  12 13 14 15
00 -------unknown---------   0B  00 00 00  00 00 00 00
16 ---------MAC-----------   len 00 00 typ
*/
type Rawheader struct {
	Unknown [16]uint8 //fixed for every device
	Mac     [8]uint8  //00-mac-00
	Length  uint8     //length of (padding + msgType + json)
	Padding [2]uint8  //fixed as 00 00
	MsgType uint8     //1:active,2:control,4:report
}

type message struct {
	header Rawheader
	json   string
}

func (m *message) readMsg(buf []byte) error {

	b := bytes.NewBuffer(buf)
	if err := binary.Read(b, binary.LittleEndian, &m.header); err != nil {
		return err
	}
	jsonBegin := 28
	jsonEnd := jsonBegin + int(m.header.Length) - 3

	if !(jsonBegin < len(buf) && jsonBegin <= jsonEnd && jsonBegin < len(buf)) {
		return errors.New("bad packet")
	}
	m.json = string(buf[jsonBegin:jsonEnd])
	return nil
}

func (m *message) controlMsg(json string) []byte {
	var header Rawheader
	var b bytes.Buffer
	header = m.header
	header.MsgType = 2
	header.Length = uint8(len(json)) + 3
	b.Write([]byte(header.Unknown[:]))
	b.Write([]byte(header.Mac[:]))
	b.WriteByte(header.Length)
	b.Write([]byte(header.Padding[:]))
	b.WriteByte(header.MsgType)
	b.WriteString(json)
	b.Write([]byte{0xFF, 0x23, 0x45, 0x4E, 0x44, 0x23})
	return b.Bytes()
}
