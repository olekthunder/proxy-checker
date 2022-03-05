use std::net::Ipv4Addr;
use std::str::FromStr;

use anyhow::Context;
use futures::Stream;
use futures::TryFutureExt;
use reqwest::Proxy as ReqwestProxy;
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;

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

fn get_proxies() -> impl Stream<Item = String> {
    tokio_stream::iter(vec![
        "218.28.136.54:7302".to_string(),
        "217.147.92.74:53923".to_string(),
        "159.203.33.4:7108".to_string(),
        "72.206.181.123:4145".to_string(),
        "72.221.196.157:35904".to_string(),
        "165.22.223.82:39928".to_string(),
        "72.49.49.11:31034".to_string(),
        "23.94.73.246:1080".to_string(),
        "218.6.155.49:7302".to_string(),
        "8.214.69.237:10809".to_string(),
        "117.86.184.200:20024".to_string(),
        "180.141.90.181:7302".to_string(),
        "139.162.100.214:38796".to_string(),
        "111.75.57.236:20008".to_string(),
        "165.227.104.122:25426".to_string(),
        "192.111.135.21:4145".to_string(),
        "92.222.206.150:2020".to_string(),
        "222.237.243.156:5002".to_string(),
        "194.60.87.97:52050".to_string(),
        "91.207.60.17:26608".to_string(),
        "149.202.189.147:6554".to_string(),
        "117.86.175.88:20032".to_string(),
        "192.111.130.2:4145".to_string(),
        "111.75.57.88:20044".to_string(),
        "221.218.245.146:7300".to_string(),
        "159.65.106.46:9050".to_string(),
        "184.179.216.133:24630".to_string(),
        "194.60.87.97:62096".to_string(),
        "120.237.253.142:1080".to_string(),
        "31.22.7.188:47114".to_string(),
        "59.47.140.142:7302".to_string(),
        "198.8.94.170:4145".to_string(),
        "58.18.36.61:7300".to_string(),
        "45.61.138.165:31652".to_string(),
        "113.9.157.29:7302".to_string(),
        "210.22.78.202:7300".to_string(),
        "45.117.83.62:4040".to_string(),
        "192.111.130.5:17002".to_string(),
        "89.19.115.55:6655".to_string(),
        "117.89.131.218:1080".to_string(),
        "192.111.139.162:4145".to_string(),
        "128.199.164.111:1086".to_string(),
        "78.46.37.212:64286".to_string(),
        "192.252.211.197:14921".to_string(),
        "69.61.200.104:36181".to_string(),
        "111.199.68.233:1080".to_string(),
        "198.199.95.57:37266".to_string(),
        "186.126.52.135:1080".to_string(),
        "166.62.83.60:27410".to_string(),
        "139.162.100.214:48730".to_string(),
        "162.243.140.82:8086".to_string(),
        "216.245.192.130:15268".to_string(),
        "61.178.172.95:7300".to_string(),
        "107.152.32.226:50626".to_string(),
        "111.75.57.94:20003".to_string(),
        "50.62.35.16:59816".to_string(),
        "180.124.153.225:8902".to_string(),
        "51.254.44.184:36516".to_string(),
        "47.57.184.90:29134".to_string(),
        "149.129.39.3:31316".to_string(),
        "128.199.138.28:12341".to_string(),
        "120.211.6.105:7300".to_string(),
        "50.62.63.126:10230".to_string(),
        "107.170.50.49:24722".to_string(),
        "218.59.182.190:7302".to_string(),
        "70.166.167.55:57745".to_string(),
        "43.129.243.128:3001".to_string(),
        "167.71.170.135:64936".to_string(),
        "49.51.189.171:21127".to_string(),
        "176.103.50.24:35544".to_string(),
        "193.29.63.45:57299".to_string(),
        "122.193.18.164:7302".to_string(),
        "174.64.199.82:4145".to_string(),
        "50.62.30.5:39275".to_string(),
        "112.95.227.6:7302".to_string(),
        "182.101.207.165:7300".to_string(),
    ])
}

async fn refresh_proxies(db: DB) {
    db.write().await.clear();
    let mut s = get_proxies().filter_map(|p| p.parse::<Proxy>().ok());
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
