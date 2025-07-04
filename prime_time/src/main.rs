mod types;
mod verif;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    spawn_server().await?;
    Ok(())
}

#[tracing::instrument]
async fn spawn_server() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:1337").await?;
    info!("Listening...");
    loop {
        let (socket, _) = listener.accept().await?;
        info!("Client connected");
        tokio::spawn(async move { client(socket).await.unwrap() });
    }
}

#[tracing::instrument]
async fn client(mut stream: TcpStream) -> anyhow::Result<()> {
    let mut buf = [0; 1024];
    loop {
        let read = stream.read(&mut buf).await?;
        if read == 0 {
            break;
        }
        debug!("Read {read} bytes: {}", String::from_utf8_lossy(&buf));
        match verif::process_request(&buf[..read]) {
            Ok(mut buf) => stream.write_all(&mut buf).await?,
            Err(err) => {
                warn!("Failed! {err}");
                let buf = err.to_string();
                stream.write(buf.as_bytes()).await?;
                break;
            }
        }
    }
    info!("Client disconnected");
    Ok(())
}
