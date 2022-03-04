use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use warp::Filter;

use crate::pool::DB;

fn with_db(
    db: Arc<DB>,
) -> impl Filter<Extract = (Arc<DB>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

async fn handle_get(db: Arc<DB>) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::html(format!(
        "HELLO: {:?}",
        db.valid_proxies.read().await
    )))
}

pub async fn serve(db: Arc<DB>, addr: impl Into<SocketAddr>) {
    let get = warp::path!("get")
        .and(warp::get())
        .and(with_db(db))
        .and_then(handle_get);
    warp::serve(get).run(addr).await
}
