// use tokio::net::TcpListener;
// use tokio::prelude::*;
//use crate::aircat::aircatsrv;
mod aircat;
use aircat::aircatsrv;
use aircat::restsrv;

use futures::future::join;
use tokio::sync::mpsc::channel;

#[tokio::main]
async fn main() {
    let (tx, rx) = channel::<aircatsrv::Message>(4);
    let conf = aircatsrv::load_config("config.json").expect("config.json read error.");
    let srv1 = aircatsrv::run_aircat_srv(&conf, rx);
    let srv2 = restsrv::run_rest_srv(&conf, tx);

    let joined = join(srv1, srv2);
    let (run1, run2) = joined.await;
    let _ = run1.map_err(|e| println!("{}", e));
    let _ = run2.map_err(|e| println!("{}", e));
}
