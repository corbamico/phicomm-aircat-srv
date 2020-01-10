use crate::aircat::aircatsrv::{Config, Message};

use hyper::{
    body,
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server, StatusCode,
};

use std::io::{Error, ErrorKind};
use std::net::{SocketAddr, ToSocketAddrs};

use tokio::sync::mpsc::Sender;

type StdError = Box<(dyn std::error::Error + Send + Sync)>;

pub async fn run_rest_srv(c: &Config, tx: Sender<Message>) -> Result<(), StdError> {
    let addr: SocketAddr = c
        .RESTServerAddr
        .clone()
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| Error::from(ErrorKind::InvalidInput))?;

    let service = make_service_fn(move |_socket: &AddrStream| {
        let tx1 = tx.clone();
        async move {
            Ok::<_, StdError>(service_fn(move |req: Request<Body>| {
                let tx2 = tx1.clone();
                handler(req, tx2)
            }))
        }
    });

    let server = Server::bind(&addr).serve(service);
    println!("restsrv run at {}", addr);
    server.await?;
    Ok(())
}

async fn handler(req: Request<Body>, mut tx: Sender<Message>) -> Result<Response<Body>, StdError> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/v1/aircat") => Ok(Response::new(Body::from("200 OK"))),

        (&Method::PUT, "/v1/aircat") => {
            let _ = tx
                .send(Message::Control(body::to_bytes(req.into_body()).await?))
                .await;
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
