use std::{collections::HashMap, sync::Arc};

use anyhow::{bail, Error};
use tokio::{net::TcpStream, sync::RwLock};

use crate::{handler::{handle_echo, handle_get, handle_ping, handle_set}, protocol::{self, RObject}};

pub async fn handle(request: &[u8], stream: &mut TcpStream, storage: Arc<RwLock<HashMap<String, RObject>>>) -> Result<(), Error> {
    
    let str_req = std::str::from_utf8(request)?;

    // eprintln!(
    //     "Handling request: {}",
    //     { str_req }
    // );

    let (parsed, _) = protocol::RObject::decode(str_req, 0)?;

    eprintln!("Parsed request: {:#?}", parsed); 

    if let protocol::RObject::Array(a) = parsed {
        let command = match a.get(0)
            .ok_or_else(|| anyhow::anyhow!("Empty array"))? {
                protocol::RObject::SimpleString(s) => s,
                protocol::RObject::BulkString(s) => s,
                _ => bail!("Expected string as command"),
            };
        match command.as_str() {
            "PING" => handle_ping(stream).await?,
            "ECHO" => handle_echo(&a, stream).await?,
            "SET" => handle_set(&a, stream, Arc::clone(&storage)).await?,
            "GET" => handle_get(&a, stream, Arc::clone(&storage)).await?,
            _ => bail!("Unknown command: {}", command),
        }
    } else {
        bail!("Expected array as request");
    }

    Ok(())
}