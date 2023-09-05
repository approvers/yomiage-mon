use anyhow::{bail, Context, Result};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::trace;

pub async fn convert_pcm_s16le(source: Vec<u8>) -> Result<Vec<u8>> {
    let mut child = Command::new("ffmpeg")
        //input args
        .args(["-i", "pipe:"])
        //format: 16-bit signed little endian
        .args(["-f", "s16le"])
        //channels: 1(mono)
        .args(["-ac", "1"])
        //sampling rate: 48000Hz
        .args(["-ar", "48000"])
        //codec: pcm
        .args(["-acodec", "pcm_s16le"])
        //output: stdout
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn ffmpeg")?;
    trace!("Spawned ffmpeg");

    //Write to stdin in another thread to avoid deadlock
    let mut stdin = child
        .stdin
        .take()
        .context("Failed to take stdin of ffmpeg")?;
    tokio::spawn(async move {
        stdin
            .write_all(&source)
            .await
            .expect("Failed to write to ffmpeg's stdin");
        trace!("Wrote to ffmpeg's stdin");
    });

    let out = child
        .wait_with_output()
        .await
        .context("Failed to wait for ffmpeg")?;
    trace!("Received ffmpeg's output");

    if !out.status.success() {
        bail!(
            "ffmpeg exited with status code {}, {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    Ok(out.stdout)
}
