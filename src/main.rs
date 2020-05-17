use futures::FutureExt;
use tokio::{io::AsyncWriteExt, stream::StreamExt};

use std::{str, sync::Mutex};
use warp::{http::Response, ws::WebSocket, Filter};

#[cfg(unix)]
mod unix;
#[cfg(unix)]
use unix::*;

async fn handle_ws(mut ws: WebSocket) {
    let a = ws.try_next().await.unwrap().unwrap();
    let sample_rate: f64 = a.to_str().unwrap().parse().unwrap();

    log::info!("Sample rate is: {}", sample_rate);

    let mut src = Source::new(sample_rate as u32, "WebMic").await.unwrap();

    log::info!("Created fifo and pulse is connected");

    while let Ok(Some(pack)) = ws.try_next().await {
        if pack.is_binary() {
            src.write_all(pack.as_bytes()).await.unwrap();
        }
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let (s, r) = tokio::sync::oneshot::channel();
    let s = Mutex::new(Some(s));
    ctrlc::set_handler(move || {
        if let Some(s) = s.lock().unwrap().take() {
            s.send(()).unwrap();
        }
    })
    .unwrap();

    let index = warp::path::end().map(|| {
        Response::builder()
            .header("Content-Type", "text/html")
            .body(include_str!("../static/index.html"))
    });
    let main = warp::path!("dist" / "main.js").map(|| {
        Response::builder()
            .header("Content-Type", "application/javascript")
            .body(include_str!("../static/main.js"))
    });
    let worklet = warp::path!("dist" / "worklet.js").map(|| {
        Response::builder()
            .header("Content-Type", "application/javascript")
            .body(include_str!("../static/worklet.js"))
    });

    let st = index.or(main).or(worklet);

    let ws = warp::path("audio")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(handle_ws));

    let server_fut = warp::serve(ws.or(st))
        .tls()
        .cert_path("tls/server.crt")
        .key_path("tls/server.rsa")
        .run(([0, 0, 0, 0], 8000));

    tokio::select! {
        _ = server_fut.fuse() => {},
        _ = r.fuse() => {},
    };
}
