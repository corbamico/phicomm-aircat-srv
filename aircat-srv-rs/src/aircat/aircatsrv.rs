use crate::aircat::influxdb;
use crate::aircat::message;
use bytes::{Buf, Bytes};

use futures::{
    future::{ready, select},
    pin_mut, stream, FutureExt, StreamExt,
};

use hex;
use serde::{Deserialize, Serialize};
use serde_json;

use std::{
    io,
    sync::{Arc, RwLock},
};

use tokio::{
    self,
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    sync::{mpsc, watch},
};
use tokio_util::codec::FramedRead;

//TODO:
//Current, we just record last json reported from device (if there are more than one device)
lazy_static! {
    pub static ref LAST_JSON: Arc<RwLock<Bytes>> = Arc::new(Default::default());
}

pub async fn run_aircat_srv(c: &Config, mut _rx: mpsc::Receiver<Message>) -> io::Result<()> {
    let mut listener = TcpListener::bind(&c.ServerAddr).await?;
    println!("aircat run at {}", &c.ServerAddr);
    //we broadcast all json to every TCP Connection of device, performance issue,
    //need handle if large number device connected.
    let (watch_tx, watch_rx) = watch::channel(Message::Nop);
    tokio::spawn(async move {
        loop {
            if let Some(Message::Control(msg)) = _rx.recv().await {
                let _ = watch_tx.broadcast(Message::Control(msg));
            }
        }
    });
    loop {
        let (socket, client_addr) = listener.accept().await?;
        let influxdb_addr: String = c.InfluxdbServer.clone();
        let watch_rx = watch_rx.clone();

        println!("aircat client connect at {}", client_addr);
        tokio::spawn(async move {
            process_client(socket, &influxdb_addr, watch_rx).await;
            println!("aircat client disconnect, which at {}", client_addr);
        });
    }
}

async fn process_client(
    mut socket: TcpStream,
    influxdb_addr: &str,
    mut watch_rx: watch::Receiver<Message>,
) {
    let (rd, mut wr) = socket.split();
    let first_packet: Arc<RwLock<Option<message::AirCatPacket>>> = Arc::new(Default::default());
    let first_packet_writer = first_packet.clone();

    let reader = async move {
        FramedRead::new(rd, message::AirCatFramedCodec::new())
            .take_while(|p| futures::future::ready(p.is_ok())) //disconnect client if there is fault packet
            .filter_map(|p| ready(p.ok())) //map to Result<AirCatPacket> to AirCatPacket
            .zip(stream::iter(vec![true]).chain(stream::repeat(false))) //we record first packet for mac/fixed_device information
            .map(|(p, is_first)| {
                if is_first {
                    let mut first = first_packet_writer.write().unwrap();
                    *first = Some(p.clone());
                }
                p
            })
            .filter(|p| ready(!(p.msg_type == 4 && !p.json.is_empty())))
            .for_each(|p| {
                {
                    let mut last = LAST_JSON.write().unwrap();
                    *last = p.json.clone();
                }
                influxdb::send_json(influxdb_addr, hex::encode(&p.mac[1..7]), p.json).map(|_| ())
            })
            .await;
    };

    let writer = async move {
        loop {
            //let got = watch_rx.recv().await;
            match watch_rx.recv().await {
                Some(Message::Control(msg)) => {
                    let mut bytes: Bytes = Bytes::default();
                    {
                        //force first_packet drop in this {}
                        //let first_packet_writer = first_packet.clone();
                        let first = first_packet.read().unwrap();
                        if let Some(p) = &*first {
                            bytes = message::gen_packet(p, msg);
                        }
                    }
                    if !bytes.is_empty() {
                        let _ = wr.write_all(bytes.bytes()).await;
                    }
                }
                _ => {}
            }
        }
    };
    pin_mut!(reader);
    pin_mut!(writer);
    select(reader, writer).await;
}

#[allow(non_snake_case)]
#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    pub ServerAddr: String,
    pub RESTServerAddr: String,
    pub InfluxdbServer: String,
}

pub fn load_config<T: AsRef<str>>(file: T) -> io::Result<Config> {
    let content = std::fs::read_to_string(file.as_ref())?;
    let c: Config = serde_json::from_str(content.as_ref())?;
    Ok(c)
}

#[derive(Debug, Clone)]
pub enum Message {
    Nop,
    Control(Bytes),
}
