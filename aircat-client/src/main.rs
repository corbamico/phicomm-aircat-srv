use futures::{
    future::{ready, select},
    stream,
    stream::repeat,
    StreamExt,
};

use tokio::net::TcpStream;
use tokio::time::{self, Duration};
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite};

use bytes::Bytes;

use clap::App;

/*Rawheader show as
   00 01 02 03 04 05 06 07   08  09 10 11  12 13 14 15
00 -------unknown---------   0B  00 00 00  00 00 00 00
16 ---------MAC-----------   len 00 00 typ
*/

#[tokio::main]
async fn main() {
    static _ACTIVE : [u8; 34] = *b"\xaa\x0f\x01\x88\x0c29\x8f\x0b\0\0\0\0\0\0\0\0\xb0\xf8\x93#\xc0\x88\0\x03\0\0\x01\xff#END#";
    static _REPORT : [u8; 110]= *b"\xaa\x0f\x01\x88\x0c29\x8f\x0b\0\0\0\0\0\0\0\0\xb0\xf8\x93#\xc0\x88\0O\0\0\x04{ \"humidity\": \"46.15\", \"temperature\": \"24.68\", \"value\": \"20\", \"hcho\": \"30\" }\xff#END#";

    let matches = App::new("aircat-client")
        .version("1.0")
        .author("corbamico <corbamico@163.com>")
        .args_from_usage(
            "[init-delay] -i, --init-delay=[INIT]  'report measure after initial delay, default 3s'
            [interval] -r, --repeat=[INTERVAL]  'report measure every INTERVAL seconds,default 5s'
            [server] -s, --server=[SERVER] 'report measure to server address, default 127.0.0.1:9000'")
        .get_matches();

    let init_delay = matches
        .value_of("init-delay")
        .unwrap_or("3")
        .parse::<u64>()
        .unwrap();
    let interval = matches
        .value_of("interval")
        .unwrap_or("5")
        .parse::<u64>()
        .unwrap();
    let server = matches.value_of("127.0.0.1:9000").unwrap_or("5");

    println!(
        "[Device]Connect to {}, with interval {}s, after delay {}s",
        server, interval, init_delay
    );
    std::thread::sleep(Duration::from_secs(init_delay));

    let mut stream = TcpStream::connect(server).await.unwrap();
    let (rd, wr) = stream.split();

    let wr = FramedWrite::new(wr, BytesCodec::new());
    let interval = time::interval(Duration::from_secs(interval));

    let writer = interval
        .zip(stream::iter(vec![Bytes::from(&_ACTIVE[..])]).chain(repeat(Bytes::from(&_REPORT[..]))))
        .map(move |(_, b)| {
            println!("[Device]Sending {:?}", b);
            Ok(b)
        })
        .forward(wr);
    let reader = FramedRead::new(rd, BytesCodec::new())
        .take_while(|p| ready(p.is_ok()))
        .map(|x| println!("[Device]Recieve: {:?}", x))
        .collect::<()>();
    select(reader, writer).await;
}
