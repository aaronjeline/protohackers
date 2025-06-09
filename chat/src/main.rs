mod client;
mod connection;
mod server;
use tokio::signal;
use tracing::{event, info};

#[tokio::main]
#[tracing::instrument]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    server::spawn().await;
    //connection::run_server().await.unwrap();
}
