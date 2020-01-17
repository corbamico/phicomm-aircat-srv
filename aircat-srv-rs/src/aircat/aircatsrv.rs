use crate::aircat::influxdb;
use crate::aircat::message;
use bytes::Bytes;

use futures::{
    future::{ready, select},
    pin_mut, FutureExt, StreamExt,
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
    net::{TcpListener, TcpStream},
    sync::{mpsc, watch},
};
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite};

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
    let msg_process = _rx.filter(Message::is_control).for_each(move |msg| {
        let _ = watch_tx.broadcast(msg);
        ready(())
    });
    tokio::spawn(msg_process);

    loop {
        let (socket, client_addr) = listener.accept().await?;
        let influxdb_addr: String = c.InfluxdbServer.clone();
        let watch_rx = watch_rx.clone();

        tokio::spawn(async move {
            println!("aircat client connect at {}", client_addr);
            process_client(socket, &influxdb_addr, watch_rx).await;
            println!("aircat client disconnect, which at {}", client_addr);
        });
    }
}

async fn process_client(
    mut socket: TcpStream,
    influxdb_addr: &str,
    watch_rx: watch::Receiver<Message>,
) {
    let (rd, wr) = socket.split();
    //step 1. we need first packet to record fixed_device field and mac field.
    let (first, stream) = FramedRead::new(rd, message::AirCatFramedCodec::new())
        .take_while(|p| ready(p.is_ok()))
        .filter_map(|p| ready(p.ok()))
        .into_future()
        .await;

    //step 2. read from aircat device report, and record LAST_JSON for restsrv.rs use, and send it to influxdb
    let reader = stream
        .filter(|p| ready(p.msg_type == 4 && !p.json.is_empty()))
        .for_each(|p| {
            {
                let mut last = LAST_JSON.write().unwrap();
                *last = p.json.clone();
            }
            influxdb::send_json(influxdb_addr, hex::encode(&p.mac[1..7]), p.json).map(|_| ())
        });
    //step 3. read from watch_rx(which sent by restsrv, and is Control message), then forward to aircat device tcp session.
    let writer = watch_rx
        .filter(Message::is_control)
        .map(|x| match x {
            Message::Control(msg) => {
                let bytes = message::gen_packet(&first.as_ref().unwrap(), msg);
                Ok::<_, io::Error>(bytes)
            }
            _ => Err(io::Error::from(io::ErrorKind::InvalidInput)),
        })
        .forward(FramedWrite::new(wr, BytesCodec::new()));

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
impl Message {
    fn is_control(&self) -> impl futures::Future<Output = bool> {
        match self {
            Message::Control(_) => ready(true),
            _ => ready(false),
        }
    }
}
