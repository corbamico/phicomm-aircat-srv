//use futures::future::{ready, select};
use futures::StreamExt;

use tokio::net::TcpStream;
use tokio::time::{self, Duration};
use tokio_util::codec::{BytesCodec, FramedWrite};

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
    let stream = TcpStream::connect("127.0.0.1:9000").await.unwrap();
    println!("[Device]Connect to 217.0.0.1:9000");
    let wr = FramedWrite::new(stream, BytesCodec::new());
    let interval = time::interval(Duration::from_secs(2));

    interval
        .map(move |_inst| {
            let b = Bytes::from(&PACKET[..]);
            println!("[Device]Sending {:?}", b);
            Ok(b)
        })
        .forward(wr)
        .await
        .map_err(|e| println!("{}", e))
        .unwrap();
}
