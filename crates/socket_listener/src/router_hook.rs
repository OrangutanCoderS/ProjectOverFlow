use async_trait::async_trait;

use crate::{errors::ListenerError, model::MeshPacket};

/// Abstraction for routing validated packets into the rest of OverFlow.
///
/// Implemented by the daemon / orchestrator crate, NOT by this crate.
/// This keeps socket_listener independent of Phase III/IV internals.
#[async_trait]
pub trait RouterHook: Send + Sync {
    async fn route(&self, packet: &MeshPacket) -> Result<(), ListenerError>;
}
