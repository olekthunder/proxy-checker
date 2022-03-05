use async_stream::stream;
use http::header::{HeaderValue, CONTENT_TYPE};
use itertools::Itertools;
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio_stream::StreamExt;
use warp::hyper::Body;

use crate::DB;
use warp::Filter;

fn with_db(db: DB) -> impl Filter<Extract = (DB,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

async fn handle_json(db: DB) -> Result<impl warp::Reply, Infallible> {
    let s = stream! {
        yield "[".to_string();
        for proxy in Itertools::intersperse(
            db.read().await.iter().cloned().map(|p| serde_json::to_string(&p).expect("can serialize to json")),
            ",".to_string(),
        ) {
            yield proxy;
        }
        yield "]".to_string();
    };
    let body = Body::wrap_stream(s.map(|i| Result::<String, anyhow::Error>::Ok(i)));
    let mut res = warp::reply::Response::new(body);
    res.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    Ok(res)
}

pub async fn serve(db: DB, addr: impl Into<SocketAddr>) {
    let get = warp::path!("json")
        .and(warp::get())
        .and(with_db(db))
        .and_then(handle_json);
    warp::serve(get).run(addr).await
}
