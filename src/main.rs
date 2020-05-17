use futures::FutureExt;
use tokio::{io::AsyncWriteExt, process::Command, stream::StreamExt};

use std::io;
use std::{str, sync::Mutex};
use warp::ws::WebSocket;
use warp::{http::Response, Filter};

struct PulseSource {
    fifo: tokio::fs::File,
    handle: i32,
}

impl PulseSource {
    async fn new(sample_rate: u32, name: &str) -> io::Result<PulseSource> {
        let filename = format!("/tmp/{}", rand::random::<u64>());

        let pulse_fut = Command::new("pactl")
            .arg("load-module")
            .arg("module-pipe-source")
            .arg(format!("source_properties=device.description={}", name))
            .arg(format!("file={}", &filename))
            .arg("format=float32")
            .arg(format!("rate={}", sample_rate))
            .arg("channels=1")
            .output();

        let mut open_options = tokio::fs::OpenOptions::new();
        let fifo_fut = async {
            loop {
                if let Ok(file) = open_options.write(true).open(&filename).await {
                    break Ok(file);
                }
            }
        };

        let (fifo, output) = tokio::try_join!(fifo_fut, pulse_fut)?;

        Ok(PulseSource {
            fifo,
            handle: str::from_utf8(&output.stdout)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                .trim()
                .parse()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        })
    }
}

impl Drop for PulseSource {
    fn drop(&mut self) {
        std::process::Command::new("pactl")
            .arg("unload-module")
            .arg(format!("{}", self.handle))
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }
}

async fn handle_ws(mut ws: WebSocket) {
    let a = ws.try_next().await.unwrap().unwrap();
    let sample_rate: f64 = a.to_str().unwrap().parse().unwrap();

    log::info!("Sample rate is: {}", sample_rate);

    let mut src = PulseSource::new(sample_rate as u32, "WebMic")
        .await
        .unwrap();

    log::info!("Created fifo and pulse is connected");

    while let Ok(Some(pack)) = ws.try_next().await {
        if pack.is_binary() {
            src.fifo.write_all(pack.as_bytes()).await.unwrap();
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
