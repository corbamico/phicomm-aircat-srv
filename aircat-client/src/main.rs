use futures::future::{ready, select};
use futures::StreamExt;

use tokio::net::TcpStream;
use tokio::time::{self, Duration};
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite};

use bytes::Bytes;

/*Rawheader show as
   00 01 02 03 04 05 06 07   08  09 10 11  12 13 14 15
00 -------unknown---------   0B  00 00 00  00 00 00 00
16 ---------MAC-----------   len 00 00 typ
*/

#[tokio::main]
async fn main() {
    static PACKET: [u8; 34] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xBB, 0xCC, 0xDD, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x02, 0xFF, 0x23,
        0x45, 0x4E, 0x44, 0x23,
    ];
    static PACKET_JSON: [u8; 35] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xBB, 0xCC, 0xDD, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x04, 0x45, 0xFF,
        0x23, 0x45, 0x4E, 0x44, 0x23,
    ];
    let mut stream = TcpStream::connect("127.0.0.1:9000").await.unwrap();
    let (rd, wr) = stream.split();
    println!("[Device]Connect to 217.0.0.1:9000");
    let wr = FramedWrite::new(wr, BytesCodec::new());
    let interval = time::interval(Duration::from_secs(2));

    let writer = interval
        .map(move |_inst| {
            let b = Bytes::from(&PACKET_JSON[..]);
            println!("[Device]Sending {:?}", b);
            Ok(b)
        })
        .forward(wr);
    let reader = FramedRead::new(rd, BytesCodec::new())
        .map(|x| println!("recieve: {:?}", x))
        .collect::<()>();
    select(reader, writer).await;
}
