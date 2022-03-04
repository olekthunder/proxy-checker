use async_trait::async_trait;
use reqwest::Proxy as ReqwestProxy;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{Proxy, ProxyCheckError, ProxyCheckResult};

async fn check_proxy(proxy: &Proxy) -> ProxyCheckResult {
    let reqwest_proxy =
        ReqwestProxy::all(proxy.scheme()).map_err(|_| ProxyCheckError::InvalidFormat)?;
    let client = reqwest::Client::builder()
        .proxy(reqwest_proxy)
        .build()
        .expect("client can be built");
    client
        .get("http://ifconfig.me")
        .send()
        .await
        .map_err(|_| ProxyCheckError::ConnectionError)?;
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
            if let Ok(_) = check_proxy(&proxy).await {
                db.valid_proxies.write().await.push(proxy);
                println!("Valid!");
            } else {
                println!("Not valid!");
            }
        });
    }
}
