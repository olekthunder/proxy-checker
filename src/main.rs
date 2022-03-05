use std::net::Ipv4Addr;
use std::str::FromStr;

use anyhow::Context;
use async_stream::stream;
use futures::pin_mut;
use futures::Stream;
use futures::TryFutureExt;
use futures::TryStreamExt;
use reqwest::Proxy as ReqwestProxy;
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;
use tokio::io::AsyncBufReadExt;

use tokio::sync::RwLock;
use tokio::time::Duration;
use tokio_stream::StreamExt;

mod server;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    Socks5,
}

#[derive(Debug, Clone, Serialize)]
pub struct Proxy {
    pub proxy_type: ProxyType,
    pub ip: Ipv4Addr,
    pub port: i32,
}

impl FromStr for Proxy {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(':');
        let ip: Ipv4Addr = split.next().context("invalid format")?.parse()?;
        let port: i32 = split.next().context("invalid format")?.parse()?;
        Ok(Proxy {
            proxy_type: ProxyType::Socks5,
            ip,
            port,
        })
    }
}

impl Proxy {
    fn scheme(&self) -> String {
        match self.proxy_type {
            ProxyType::Socks5 => format!("socks5://{}:{}", self.ip, self.port),
        }
    }
}

type DB = Arc<RwLock<Vec<Proxy>>>;

#[derive(Error, Debug)]
pub enum ProxyCheckError {
    #[error("proxy has invalid format")]
    InvalidFormat,
    #[error("connection error")]
    ConnectionError,
    #[error("Ip mismatch")]
    IPMismatch,
}

type ProxyCheckResult = Result<(), ProxyCheckError>;

async fn check_proxy(proxy: &Proxy) -> ProxyCheckResult {
    let reqwest_proxy =
        ReqwestProxy::all(proxy.scheme()).map_err(|_| ProxyCheckError::InvalidFormat)?;
    let client = reqwest::Client::builder()
        .proxy(reqwest_proxy)
        .build()
        .expect("client can be built");
    let ip = client
        .get("http://ifconfig.me")
        .send()
        .and_then(|r| r.text())
        .await
        .map_err(|_| ProxyCheckError::ConnectionError)?;
    if ip != proxy.ip.to_string() {
        return Err(ProxyCheckError::IPMismatch);
    }
    Ok(())
}

fn convert_err(err: reqwest::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Interrupted, err)
}

async fn get_proxies() -> impl Stream<Item = String> {
    let resp = reqwest::get("http://0.0.0.0:8000/socks5.txt")
        .await
        .unwrap()
        .bytes_stream();
    let reader = tokio_util::io::StreamReader::new(resp.map_err(convert_err));
    let mut s = reader.lines();
    stream! {
        while let Ok(Some(line)) = s.next_line().await {
            yield line;
        }
    }
}

async fn refresh_proxies(db: DB) {
    db.write().await.clear();
    let s = get_proxies().await.filter_map(|p| p.parse::<Proxy>().ok());
    pin_mut!(s);
    while let Some(proxy) = s.next().await {
        let db = db.clone();
        tokio::spawn(async move {
            if check_proxy(&proxy).await.is_ok() {
                println!("{}", &proxy.ip);
                db.write().await.push(proxy);
            }
        });
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = DB::default();
    let db1 = db.clone();
    tokio::spawn(async move {
        loop {
            refresh_proxies(db1.clone()).await;
            tokio::time::sleep(Duration::from_secs(5 * 60)).await;
        }
    });
    server::serve(db.clone(), ([127, 0, 0, 1], 8080)).await;
    Ok(())
}
