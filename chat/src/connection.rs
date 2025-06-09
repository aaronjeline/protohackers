use crate::client;
use crate::server;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tower::{service_fn, Service, ServiceBuilder};
use tracing::{error, info};

const PORT: u16 = 1337;
const LOCALHOST: &str = "0.0.0.0";

#[tracing::instrument]
pub async fn run_server(controller: server::Tx) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(format!("{LOCALHOST}:{PORT}")).await?;
    info!("listening on {PORT}...");
    loop {
        let (stream, addr) = listener.accept().await?;
        let tx_clone = controller.clone();
        info!("New connection from: {}", addr);
        let mut service =
            service_fn(move |stream| async move { client::handler(stream, tx_clone).await });
        tokio::spawn(async move {
            if let Err(e) = service.call(stream).await {
                error!("Error with {}:{}", addr, e);
            }
        });
    }
}
