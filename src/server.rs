use std::convert::Infallible;
use std::net::SocketAddr;

use crate::DB;
use warp::Filter;

fn with_db(db: DB) -> impl Filter<Extract = (DB,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

async fn handle_json(db: DB) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::reply())
}

pub async fn serve(db: DB, addr: impl Into<SocketAddr>) {
    let get = warp::path!("json")
        .and(warp::get())
        .and(with_db(db))
        .and_then(handle_json);
    warp::serve(get).run(addr).await
}
