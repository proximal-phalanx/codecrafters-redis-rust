use std::sync::Arc;

use anyhow::Error;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use futures::lock::Mutex;

use crate::{protocol::RObject, BUFFER_SIZE};

pub struct Broadcaster {
    pub subscribers: Vec<Arc<Mutex<TcpStream>>>
}

impl Broadcaster {

    pub fn subscribe(&mut self, target: TcpStream) {
        self.subscribers.push(
            Arc::new(Mutex::new(target))
        );
    }

    pub async fn broadcast(&mut self, message: &[u8]) -> Result<(), Error>{
        println!("Scheduled to broadcast: {:?}", std::str::from_utf8(message).unwrap());
        
        let mut futures = Vec::new();

        for subscriber in &self.subscribers {
            let subscriber = subscriber.clone();
            let future = async move {
                let mut subscriber = subscriber.lock().await;
                subscriber.write_all(message).await.expect("Failed to broadcast");
            };
            futures.push(future);
        }

        Ok(())
    }

    pub async fn check_sync(&mut self, expect_bytes: usize) -> Result<usize, Error> {
        let mut count = 0;

        // for each subscriber, sends REPLCONF GETACK *
        // then check if the response is equal to the expected bytes
        for subscriber in &mut self.subscribers {
            let mut subscriber = subscriber.lock().await;
            
            subscriber.write_all(
                RObject::Array(
                    vec![
                        RObject::BulkString("REPLCONF".to_string()),
                        RObject::BulkString("GETACK".to_string()),
                        RObject::BulkString("*".to_string())
                    ]
                ).to_string().as_bytes()
            ).await.expect("Failed to send REPLCONF GETACK");

            let mut buffer = [0; BUFFER_SIZE];
            subscriber.read(&mut buffer).await.expect("Failed to read REPLCONF GETACK response");
            eprintln!(
                "Received REPLCONF GETACK response: {:?} from {:?}",
                 String::from_utf8_lossy(buffer.as_ref()),
                subscriber.peer_addr().unwrap()
            );

            let response = match RObject::decode(std::str::from_utf8(&buffer).expect("Failed to decode REPLCONF GETACK response"), 0) {
                Ok((RObject::Array(a), _)) => a,
                _ => {
                    eprintln!("Failed to parse REPLCONF GETACK response");
                    continue;
                }
            };
            if let RObject::BulkString(s) = &response[2] {
                // if s equals to the expected bytes, increment count
                count += if s.parse::<usize>().expect("Failed to parse REPLCONF GETACK response") == expect_bytes {
                    1
                } else {
                    0
                };
            }
        }
    
        Ok(count)
    }
}

