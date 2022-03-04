use std::net::Ipv4Addr;
use std::str::FromStr;

use anyhow::Context;
use thiserror::Error;
use tokio;

pub mod pool;
pub mod server;

use pool::{LocalProxyPool, ProxyPool};

#[derive(Error, Debug)]
pub enum ProxyCheckError {
    #[error("proxy has invalid format")]
    InvalidFormat,
    #[error("connection error")]
    ConnectionError,
}

type ProxyCheckResult = Result<(), ProxyCheckError>;

#[derive(Debug, Clone)]
pub enum ProxyType {
    Socks5,
}

#[derive(Debug, Clone)]
pub struct Proxy {
    pub proxy_type: ProxyType,
    pub ip: Ipv4Addr,
    pub port: i32,
}

impl FromStr for Proxy {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(":");
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let proxy: Proxy = "45.61.186.127:37607".parse()?;
    let mut pool = LocalProxyPool::new();
    pool.add(proxy).await;
    Ok(server::serve(pool.db, ([127, 0, 0, 1], 8080)).await)
}
