use crate::aircat::aircatsrv::{Config, Message};

use hyper::{
    body,
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server, StatusCode,
};

use std::io::{Error, ErrorKind};
use std::net::{SocketAddr, ToSocketAddrs};

use tokio::sync::{mpsc, watch};

use log::{error, info};

type StdError = Box<(dyn std::error::Error + Send + Sync)>;

pub async fn run_rest_srv(
    c: &Config,
    tx_control: mpsc::Sender<Message>,
    watch_last_packet: watch::Receiver<Message>,
) -> Result<(), StdError> {
    let addr: SocketAddr = c
        .RESTServerAddr
        .clone()
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| Error::from(ErrorKind::InvalidInput))?;

    let service = make_service_fn(move |_socket: &AddrStream| {
        let tx_control = tx_control.clone();
        let watch_last_packet = watch_last_packet.clone();
        async move {
            let watch_last_packet = watch_last_packet.clone();
            Ok::<_, StdError>(service_fn(move |req: Request<Body>| {
                let tx_control = tx_control.clone();
                let watch_last_packet = watch_last_packet.clone();
                handler(req, tx_control, watch_last_packet)
            }))
        }
    });

    let server = Server::bind(&addr).serve(service);
    info!("restsrv run at {}", addr);
    server.await?;
    Ok(())
}

async fn handler(
    req: Request<Body>,
    mut tx_control: mpsc::Sender<Message>,
    mut watch_last_packet: watch::Receiver<Message>,
) -> Result<Response<Body>, StdError> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/v1/aircat") => {
            if let Some(Message::LastReport(bytes)) = watch_last_packet.recv().await {
                Ok(Response::new(Body::from(bytes)))
            } else {
                Ok(Response::default())
            }
        }

        (&Method::PUT, "/v1/aircat") => {
            let _ = tx_control
                .send(Message::Control(body::to_bytes(req.into_body()).await?))
                .await
                .map_err(|e| error!("send http json body to aircatsrv error: {:?}", e));
            Ok(Response::builder()
                .status(StatusCode::NO_CONTENT)
                .body(Body::empty())
                .unwrap())
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()),
    }
}
