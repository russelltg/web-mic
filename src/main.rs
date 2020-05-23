#![deny(unsafe_code)]

use tokio::{io::AsyncWriteExt, stream::StreamExt};

use std::{io, str, sync::Mutex};
use warp::{http::Response, path, path::Tail, ws::WebSocket, Filter, Rejection, Reply};

use rust_embed::RustEmbed;

mod cert;
mod source;

fn warp2io(e: warp::Error) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e)
}

async fn handle_ws(mut ws: WebSocket) -> io::Result<()> {
    let a = ws
        .try_next()
        .await
        .map_err(warp2io)?
        .expect("Websocket closed before sending the sample rate");

    let sample_rate: f64 = a
        .to_str()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "First message was not text"))?
        .parse()
        .unwrap();

    log::info!("Sample rate is: {}", sample_rate);

    let mut src = source::Source::new(sample_rate as u32, "WebMic").await?;

    log::info!("Created fifo and pulse is connected");

    while let Some(pack) = ws.try_next().await.map_err(warp2io)? {
        if pack.is_binary() {
            src.writer().write_all(pack.as_bytes()).await?;
        }
    }
    Ok(())
}

// serve serve /static directory into /dist
#[derive(RustEmbed)]
#[folder = "static"]
struct Static;

fn serve_static(path: &str) -> Result<impl Reply, Rejection> {
    Ok(Response::builder()
        .header(
            "Content-Type",
            mime_guess::from_path(path).first_or_octet_stream().as_ref(),
        )
        .body(Static::get(path).ok_or_else(warp::reject::not_found)?))
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // handle ctrl+c correctly
    let (send_close, recv_close) = tokio::sync::oneshot::channel();
    let s = Mutex::new(Some(send_close));
    ctrlc::set_handler(move || {
        if let Some(s) = s.lock().unwrap().take() {
            s.send(()).unwrap();
        }
    })
    .unwrap();

    // serve static files
    let index = path::end().and_then(|| async { serve_static("index.html") });
    let st = path("dist")
        .and(path::tail())
        .and_then(|path: Tail| async move { serve_static(path.as_str()) });

    // serve websocket
    let ws = warp::path("audio")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(|ws| async { handle_ws(ws).await.unwrap() }));

    // get tls certs
    let (cert, key) = cert::get_cert_key();

    // make the server future
    let server = warp::serve(ws.or(index).or(st))
        .tls()
        .cert(cert)
        .key(key)
        .bind(([0, 0, 0, 0], 8000));

    // exit on ctrl+c
    tokio::select! {
        _ = server => {}
        _ = recv_close => {}
    }
}
