use bytes::{Buf, BufMut, Bytes, BytesMut};

use std::io;
use tokio_util::codec::{Decoder, Encoder};

/*Framed show as
   00 01 02 03 04 05 06 07   08  09 10 11  12 13 14 15
00 -------unknown---------   0B  00 00 00  00 00 00 00
16 ---------MAC-----------   len 00 00 typ ----json---
24 FF 23 45 4E 44 23
*/
//current we set max_frame_length = 150 for safe.
#[derive(Debug, Default, Clone)]
pub struct AirCatPacket {
    pub device_fixed: [u8; 16],
    pub msg_type: u8, //1:active,2:control,4:report
    pub mac: [u8; 8],
    pub json: Bytes,
}

impl AirCatPacket {
    const MIN_PACKET_LENGTH: usize = 33;
    const _MAX_PACKET_LENGTH: usize = 156;
    fn from(src: BytesMut) -> io::Result<AirCatPacket> {
        let b = src.bytes();
        if b.len() < AirCatPacket::MIN_PACKET_LENGTH {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }

        let begin: usize = 28;
        let end: usize = 28usize - 3usize + b[24] as usize;

        if !(begin <= end && end <= b.len()) {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }
        Ok(AirCatPacket::new(
            &b[0..16],
            b[27],
            &b[16..24],
            &b[begin..end],
        ))
    }
    fn new(device_fixed: &[u8], msg_type: u8, mac: &[u8], json: &[u8]) -> AirCatPacket {
        let mut a = AirCatPacket::default();
        a.device_fixed.copy_from_slice(device_fixed);
        a.msg_type = msg_type;
        a.mac.copy_from_slice(mac);
        a.json = Bytes::copy_from_slice(json);
        a
    }
}

//#[allow(dead_code)]
pub struct AirCatFramedCodec;

impl AirCatFramedCodec {
    pub fn new() -> AirCatFramedCodec {
        AirCatFramedCodec {}
    }
}

impl Decoder for AirCatFramedCodec {
    type Item = AirCatPacket;
    type Error = io::Error;
    /// We always read whole one packet from src: BytesMut    
    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<AirCatPacket>> {
        if src.len() <= 0 {
            //wait read more from FramedRead...
            //src.reserve(AirCatPacket::MAX_PACKET_LENGTH);
            Ok(None)
        } else {
            //always eat all bytes in read buffer.
            let bytes_mut = src.split_to(src.len());
            //src.reserve(AirCatPacket::MAX_PACKET_LENGTH);
            AirCatPacket::from(bytes_mut).map(Some)
        }
    }
}
impl Encoder for AirCatFramedCodec {
    type Item = Bytes;
    type Error = io::Error;
    fn encode(&mut self, _data: Bytes, _dst: &mut BytesMut) -> io::Result<()> {
        unimplemented!()
    }
}

pub fn gen_packet(p: &AirCatPacket, json: Bytes) -> Bytes {
    let mut b: BytesMut = BytesMut::new();
    let end: [u8; 6] = [0xFF, 0x23, 0x45, 0x4E, 0x44, 0x23];
    let len = 3 + json.len() as u8;
    b.put(&p.device_fixed[..]);
    b.put(&p.mac[..]);
    b.put_u8(len);
    b.put_u8(0x00);
    b.put_u8(0x00);
    b.put_u8(0x02); //type
    b.put(json);
    b.put(&end[..]);
    b.to_bytes()
}
