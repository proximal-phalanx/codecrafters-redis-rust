use std::sync::Arc;

use anyhow::{bail, Error};
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::RwLock};

use crate::{protocol::RObject, State};

pub async fn handle_info(
    args: &Vec<RObject>,
    state: Arc<RwLock<State>>,
    stream: &mut TcpStream
) -> Result<(), Error> {
    let specification = match args.get(1).expect("No specification") {
        RObject::BulkString(s) => s,
        _ => bail!("Expect a specification after the info command")
    };

    match specification.as_str() {
        "replication" => {
            stream.write(
                RObject::BulkString(
                    format!(
                        concat!(
                            "role:{}\n",
                            "master_replid:{}\n",
                            "master_repl_offset:{}\n"
                        ),
                        state.read().await.role,
                        state.read().await.master_replid,
                        state.read().await.master_repl_offset,
                    )
                ).to_string().as_bytes()
            ).await.expect("Failed to write to stream handling info replication.")
        }
        _ => bail!("Specification not allowed")
    };
    Ok(())
}