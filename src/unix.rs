use std::{
    io,
    pin::Pin,
    str,
    task::{Context, Poll},
};
use tokio::{io::AsyncWrite, process::Command};

pub struct Source {
    fifo: tokio::fs::File,
    handle: i32,
}

impl Source {
    pub async fn new(sample_rate: u32, name: &str) -> io::Result<Source> {
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

        Ok(Source {
            fifo,
            handle: str::from_utf8(&output.stdout)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                .trim()
                .parse()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        })
    }
}

impl AsyncWrite for Source {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.get_mut().fifo).poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().fifo).poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().fifo).poll_shutdown(cx)
    }
}

impl Drop for Source {
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
