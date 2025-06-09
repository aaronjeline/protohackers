use crate::client;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::signal;
use tokio::sync::mpsc;
use tokio::{task, task::JoinHandle};
use tower::{service_fn, Service, ServiceBuilder};
use tracing::Instrument;
use tracing::{event, info, info_span, span::Entered};

#[derive(Debug, Clone)]
pub enum ServerMsg {}

pub type Rx = mpsc::Receiver<ServerMsg>;
pub type Tx = mpsc::Sender<ServerMsg>;

const PORT: u16 = 1337;
const LOCALHOST: &str = "0.0.0.0";

pub struct Server {
    rx: Rx,
    tx: Tx
    listener : TcpListener,
}

pub fn spawn() -> JoinHandle<()> {
    task::spawn(async { Server::init().await.server_main().await })
}

impl Server {
    async fn init() -> Self {
        let (tx, rx) = mpsc::channel(32);
        let listener = TcpListener::bind(format!("{LOCALHOST}:{PORT}")).await.unwrap();
        let mut server = Server { rx, tx, listener };
        server
    }

    async fn server_main(&mut self) {
        self.enter().instrument(info_span!("controller")).await
    }

    async fn enter(&mut self) {
        info!("Controller spawned");
        loop {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    info!("Received Ctrl+c, shutting down...");
                    break;
                },
                msg = self.rx.recv() => {
                    match msg.unwrap() {
                    }
                },
                conn = self.listener.accept() => {
                    let (stream, addr) = conn.unwrap();
                    info!("Connection from {addr}");
                    let tx_clone = self.tx.clone();
                    let mut service = service_fn(move |stream| {
                        let tx = tx_clone.clone();
                        async move { client::handler(stream, tx).await 
                        }
                    });
                    tokio::spawn(async move {
                        if let Err(e) = service.call(stream).await {
                            error!("{addr} - {e}");
                        }
                    });
                }
            }
        }
    }

}


