mod types;
mod verif;
use anyhow::Result;
use std::ops::ControlFlow;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info};
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
        let read = read_until_newline(&mut stream, &mut buf).await?;
        match read {
            None => break,
            Some(read) => {
                debug!("Read {read} bytes: {}", String::from_utf8_lossy(&buf));
                if let ControlFlow::Break(()) =
                    verif::process_requests(&buf[..read], &mut stream).await?
                {
                    break;
                }
            }
        }
    }
    info!("Client disconnected");
    Ok(())
}

async fn read_until_newline(socket: &mut TcpStream, buffer: &mut [u8]) -> Result<Option<usize>> {
    let mut ptr = 0;
    let timeout = Duration::from_secs(60);
    loop {
        let read = tokio::time::timeout(timeout, socket.read(&mut buffer[ptr..])).await??;
        if ptr == 0 && read == 0 {
            return Ok(None);
        } else if read == 0 {
            return Ok(Some(ptr));
        } else if buffer[ptr + read - 1] == b'\n' {
            return Ok(Some(read + ptr));
        } else {
            ptr += read;
        }
    }
}
