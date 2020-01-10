use bytes::{Buf, BufMut, Bytes, BytesMut};

use std::io;
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};

/*Framed show as
   00 01 02 03 04 05 06 07   08  09 10 11  12 13 14 15
00 -------unknown---------   0B  00 00 00  00 00 00 00
16 ---------MAC-----------   len 00 00 typ ----json---
24 FF 23 45 4E 44 23
*/
#[derive(Default, Clone)]
pub struct AirCatPacket {
    pub device_fixed: [u8; 16],
    pub msg_type: u8, //1:active,2:control,4:report
    pub mac: [u8; 8],
    pub json: String,
}

#[allow(dead_code)]
pub struct AirCatFramedCodec {
    codec: LengthDelimitedCodec,
}

impl AirCatFramedCodec {
    pub fn new() -> AirCatFramedCodec {
        AirCatFramedCodec {
            codec: LengthDelimitedCodec::builder()
                .length_field_offset(24)
                .length_field_length(1)
                .length_adjustment(31)
                .num_skip(0)
                .new_codec(),
        }
    }
}

impl Decoder for AirCatFramedCodec {
    type Item = AirCatPacket;
    type Error = io::Error;
    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<AirCatPacket>> {
        use std::convert::TryInto;
        self.codec.decode(src).map(|x| match x {
            None => None,
            Some(b) => {
                let b = b.bytes();
                let mut air: AirCatPacket = Default::default();
                air.device_fixed.copy_from_slice(&b[0..16]);
                air.mac.copy_from_slice(&b[16..24]);
                air.msg_type = b[27];
                let begin: usize = 28;
                let end: usize = b[24].try_into().unwrap();
                let end: usize = 28usize - 3usize + end;
                if let Ok(s) = String::from_utf8(b[begin..end].to_vec()) {
                    air.json = s;
                    Some(air)
                } else {
                    None
                }
            }
        })
    }
}
impl Encoder for AirCatFramedCodec {
    type Item = Bytes;
    type Error = io::Error;
    fn encode(&mut self, _data: Bytes, _dst: &mut BytesMut) -> io::Result<()> {
        //unimplemented!()
        self.codec.encode(_data, _dst)
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
