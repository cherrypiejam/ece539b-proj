use tokio::io::AsyncWriteExt;
use tokio::runtime::Runtime;
use tokio_stream::StreamExt;
use tokio::time::{self, Duration};
use tokio::fs::File;
use anyhow::Result;
use reqwest;

async fn download(url: &str, duration: Duration) -> Result<()> {
    let mut file = File::create("/dev/null").await?;
    let mut stream = reqwest::get(url).await?.bytes_stream();

    let sleep = time::sleep(duration);
    tokio::pin!(sleep);

    loop {
        tokio::select! {
            maybe_v = stream.next() => {
                if let Some(Ok(bytes)) = maybe_v {
                    let _ = file.write_all(&bytes).await;
                } else {
                    stream = reqwest::get(url).await?.bytes_stream()
                }
            }
            _ = &mut sleep => {
                break
            }
        }
    }

    Ok(())
}


pub fn burst(url: &str, duration: Duration) -> Result<()> {
    let rt = Runtime::new().expect("failed to start tokio runtime");
    rt.block_on(async move {
        let _ = download(url, duration).await;
    });

    Ok(())
}
