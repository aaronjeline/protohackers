use crate::server;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::signal;
use tokio::sync::mpsc;
use tokio::{task, task::JoinHandle};
use tracing::Instrument;
use tracing::{event, info, info_span, span::Entered};

#[derive(Debug, Clone)]
pub enum ClientMessage {}

pub type Tx = mpsc::Sender<ClientMessage>;
pub type Rx = mpsc::Receiver<ClientMessage>;

pub async fn handler(
    mut stream: TcpStream,
    controller: server::Tx,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut buffer = [0; 1024];
    loop {
        let n = stream.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        stream.write_all(&buffer[..n]).await?;
    }
    Ok(())
}
