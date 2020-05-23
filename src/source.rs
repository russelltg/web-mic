use std::{
    io,
    str,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{io::AsyncWrite, process::Command};

pub struct Source {
    fifo: tokio::fs::File,
    handle: i32,
}

impl Source {
    pub async fn new(sample_rate: u32, name: &str) -> io::Result<Source> {
        // get a unique filename
        let filename = format!(
            "/tmp/web_mic_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros()
        );

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

    pub fn writer(&mut self) -> &mut impl AsyncWrite {
        &mut self.fifo
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
