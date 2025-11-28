use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;

use crate::model::SyncMessage;
use crate::packet::WirePacket;
use crate::error::PeerSyncError;

/// Sends signed packets to a peer.
pub struct Dispatcher;

impl Dispatcher {
    pub async fn send(addr: &str, msg: SyncMessage) -> Result<(), PeerSyncError> {
        let mut stream = TcpStream::connect(addr).await?;
        let packet = WirePacket { msg };

        let buf = serde_json::to_vec(&packet)?;
        stream.write_all(&buf).await?;
        Ok(())
    }
}
