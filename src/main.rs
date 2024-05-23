pub mod protocol;
pub mod handler;
pub mod config;

use std::collections::HashMap;
use std::sync::Arc;

use structopt::StructOpt;
use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use tokio::spawn;
use tokio::sync::RwLock;

use crate::handler::handle;
use crate::protocol::RObject;
pub use crate::config::Config;


#[derive(StructOpt)]
struct Cli {
    #[structopt(default_value = "6379", long)]
    port: u64
}


#[tokio::main]
async fn main() {
    let args = Cli::from_args();
    let port = args.port;

    let config = Arc::new(RwLock::new(Config {
        role: "master".to_string()
    }));

    let storage = Arc::new(RwLock::new(HashMap::<String, RObject>::new()));

    let listener = TcpListener::bind(
        format!("127.0.0.1:{}", port)
    ).await.unwrap();
    
    const BUFFER_SIZE: usize = 4096;
    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        let storage = Arc::clone(&storage);
        let config = Arc::clone(&config);
        spawn(async move {
            loop {
                let mut buf = [0; BUFFER_SIZE];
                let s = stream.read(&mut buf)
                    .await.expect("error reading from stream");
                if s != 0 {
                    handle(&buf[..s], &mut stream, Arc::clone(&storage), Arc::clone(&config))
                        .await.expect("error handling request");
                }
            }
        });
    }
}