mod types;
mod verif;
use anyhow::Result;
use bytes::{BufMut, Bytes, BytesMut};
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
async fn client(stream: TcpStream) -> anyhow::Result<()> {
    let mut socket = SocketReader::new(stream);
    loop {
        let buf = socket.read().await?;
        match buf {
            None => break,
            Some(buf) => {
                debug!("Read bytes: {}", String::from_utf8_lossy(&buf));
                if let ControlFlow::Break(()) =
                    verif::process_requests(&buf, &mut socket.socket).await?
                {
                    break;
                }
            }
        }
    }
    info!("Client disconnected");
    Ok(())
}

struct SocketReader {
    socket: TcpStream,
    slop_buffer: BytesMut,
}

impl SocketReader {
    pub fn new(socket: TcpStream) -> Self {
        Self {
            socket,
            slop_buffer: BytesMut::with_capacity(1024),
        }
    }

    pub async fn read(&mut self) -> Result<Option<Vec<u8>>> {
        let read = self.socket.read_buf(&mut self.slop_buffer).await?;
        let p: &[u8] = &self.slop_buffer;
        debug!("Contents of slop: {:?}", p);
        if read == 0 {
            Ok(None)
        } else {
            match self.last_newline() {
                None => Ok(Some(vec![])),
                Some(last_newline_idx) => {
                    let to_return = self.slop_buffer.split_to(last_newline_idx);
                    let _ = self.slop_buffer.split_to(1); // Drop the trailing newline
                    Ok(Some(to_return.to_vec()))
                }
            }
        }
    }

    fn last_newline(&self) -> Option<usize> {
        let mut i = self.slop_buffer.len() - 1;
        while i != 0 {
            if self.slop_buffer[i] == b'\n' {
                return Some(i);
            }
            i -= 1;
        }
        None
    }
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
