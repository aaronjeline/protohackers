use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;
    echo_server(1337).await
}

#[tracing::instrument]
async fn echo_server(port: u32) -> anyhow::Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    info!("Bind succeeded");
    loop {
        let (socket, _) = listener.accept().await?;
        info!("Client connected");
        tokio::spawn(async move {
            process_socket(socket).await.unwrap();
        });
    }
    Ok(())
}

#[tracing::instrument]
async fn process_socket(mut socket: TcpStream) -> anyhow::Result<()> {
    let mut buf = [0; 1024];
    loop {
        let read = socket.read(&mut buf).await?;
        debug!("Read {} bytes", read);
        if read == 0 {
            break;
        }
        socket.write_all(&buf[..read]).await?;
    }
    info!("Client disconnecting...");
    Ok(())
}
