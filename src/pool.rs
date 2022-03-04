use async_trait::async_trait;
use reqwest::Proxy as ReqwestProxy;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use futures::future::TryFutureExt;
use crate::Proxy;

#[derive(Error, Debug)]
pub enum ProxyCheckError {
    #[error("proxy has invalid format")]
    InvalidFormat,
    #[error("connection error")]
    ConnectionError,
    #[error("Ip mismatch")]
    IPMismatch
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

#[async_trait]
pub trait ProxyPool {
    async fn add(&mut self, proxy: Proxy);
}

#[derive(Debug, Default)]
pub struct DB {
    pub valid_proxies: RwLock<Vec<Proxy>>,
    pub all_proxies: RwLock<Vec<Proxy>>,
}

impl DB {
    pub fn new() -> DB {
        DB::default()
    }
}

#[derive(Debug)]
pub struct LocalProxyPool {
    pub db: Arc<DB>,
}

impl LocalProxyPool {
    pub fn new() -> LocalProxyPool {
        LocalProxyPool {
            db: Arc::new(DB::new()),
        }
    }
}

#[async_trait]
impl ProxyPool for LocalProxyPool {
    async fn add(&mut self, proxy: Proxy) {
        let db = self.db.clone();
        tokio::spawn(async move {
            db.all_proxies.write().await.push(proxy.clone());
            match check_proxy(&proxy).await {
                Ok(_) => {
                    db.valid_proxies.write().await.push(proxy);
                    println!("Valid!");
                },
                Err(e) => {
                    println!("Not valid: {}!", e);
                }
        }});
    }
}
