mod aircat;
use aircat::aircatsrv;
use aircat::restsrv;

use futures::future::join;
use tokio::sync::mpsc;
use tokio::sync::watch;

#[tokio::main]
async fn main() {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let (tx_control, rx_control) = mpsc::channel::<aircatsrv::Message>(4);
    let (report_last_packet, watch_last_packet) = watch::channel(aircatsrv::Message::Nop);
    let conf = aircatsrv::load_config("config.json").expect("config.json read error.");

    let srv1 = aircatsrv::run_aircat_srv(&conf, rx_control, report_last_packet);
    let srv2 = restsrv::run_rest_srv(&conf, tx_control, watch_last_packet);
    let _ = join(srv1, srv2).await;
}
