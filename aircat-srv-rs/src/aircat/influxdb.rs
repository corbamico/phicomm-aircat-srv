use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io;

use hyper::{Body, Client, Method, Request};

pub async fn send_json<T: AsRef<str>, U: AsRef<str>>(
    addr: U,
    mac: T,
    json: Bytes,
) -> Result<(), Box<(dyn std::error::Error + std::marker::Send + std::marker::Sync + 'static)>> {
    let body = json2body(mac, json)?;
    let req = Request::builder()
        .method(Method::POST)
        .uri(addr.as_ref())
        .body(Body::from(body))?;
    Client::new().request(req).await?;
    Ok(())
}

//AirMeasure reported from device
#[derive(Default, Serialize, Deserialize)]
pub struct AirMeasure {
    humidity: String,
    temperature: String,
    value: String,
    hcho: String,
}

fn json2body<T: AsRef<str>>(mac: T, json: Bytes) -> io::Result<String> {
    let air: AirMeasure = serde_json::from_slice(json.as_ref())?;
    Ok(format!(
        r#"aircat,mac="{}" humidity={},temperature={},value={},hcho={}"#,
        mac.as_ref(),
        air.humidity,
        air.temperature,
        air.value,
        air.hcho
    ))
}
