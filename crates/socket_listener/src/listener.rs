use std::{
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};

use tokio::{
    io::{AsyncReadExt},
    net::TcpListener,
    time::timeout,
};
use tokio_rustls::TlsAcceptor;
use tracing::{error, info};

use crate::{
    errors::ListenerError,
    model::MeshPacket,
    rate_limit::RateLimiter,
    replay::ReplayProtection,
    router_hook::RouterHook,
};

/// Maximum bytes accepted for a single packet.
/// Prevents memory blowup from malicious peers.
const MAX_PACKET_BYTES: usize = 1_048_576; // 1 MiB

/// Main async TLS listener for mesh packets.
pub struct SocketListener {
    addr: SocketAddr,
    tls_acceptor: TlsAcceptor,
    rate_limiter: Arc<RateLimiter>,
    replay_protection: Arc<ReplayProtection>,
    router: Arc<dyn RouterHook>,
    read_timeout: Duration,
}

impl SocketListener {
    /// Create a new listener instance.
    pub fn new(
        addr: SocketAddr,
        tls_acceptor: TlsAcceptor,
        rate_limiter: Arc<RateLimiter>,
        replay_protection: Arc<ReplayProtection>,
        router: Arc<dyn RouterHook>,
        read_timeout: Duration,
    ) -> Self {
        Self {
            addr,
            tls_acceptor,
            rate_limiter,
            replay_protection,
            router,
            read_timeout,
        }
    }

    /// Run accept loop. This should normally be spawned inside tokio runtime.
    pub async fn run(&self) -> Result<(), ListenerError> {
        let listener = TcpListener::bind(self.addr).await?;

        info!("socket_listener: listening on {}", self.addr);

        loop {
            let (tcp_stream, peer_addr) = listener.accept().await?;

            let ip = peer_addr.ip();
            if self.rate_limiter.is_limited(&ip) {
                // We silently drop over-limit peers; no hint is given.
                error!("socket_listener: rate limit exceeded for {}", peer_addr);
                continue;
            }

            let tls = self.tls_acceptor.clone();
            let rl = Arc::clone(&self.rate_limiter);
            let rp = Arc::clone(&self.replay_protection);
            let router = Arc::clone(&self.router);
            let timeout_dur = self.read_timeout;

            tokio::spawn(async move {
                if let Err(e) = handle_connection(tls, rl, rp, router, tcp_stream, timeout_dur).await {
                    error!("socket_listener: connection error: {:?}", e);
                }
            });
        }
    }
}

async fn handle_connection(
    tls_acceptor: TlsAcceptor,
    _rate_limiter: Arc<RateLimiter>,
    replay: Arc<ReplayProtection>,
    router: Arc<dyn RouterHook>,
    tcp_stream: tokio::net::TcpStream,
    read_timeout: Duration,
) -> Result<(), ListenerError> {
    // TLS handshake
    let mut tls_stream = tls_acceptor
        .accept(tcp_stream)
        .await
        .map_err(|e| ListenerError::TlsHandshake(e.to_string()))?;

    let mut buf = Vec::new();

    // Hard bound on total size to avoid memory abuse
    let read_future = async {
        loop {
            let mut chunk = vec![0u8; 4096];
            let n = tls_stream.read(&mut chunk).await?;

            if n == 0 {
                break;
            }

            buf.extend_from_slice(&chunk[..n]);

            if buf.len() > MAX_PACKET_BYTES {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "packet too large",
                ));
            }
        }
        Ok::<(), std::io::Error>(())
    };

    timeout(read_timeout, read_future)
        .await
        .map_err(|_| ListenerError::Timeout)??;

    let packet: MeshPacket = serde_json::from_slice(&buf)
        .map_err(|e| ListenerError::InvalidPacket(e.to_string()))?;

    if !packet.is_structurally_valid() {
        return Err(ListenerError::InvalidPacket(
            "missing required fields".to_string(),
        ));
    }

    // Replay protection
    if !replay.validate(&packet.origin_id, &packet.timestamp) {
        return Err(ListenerError::ReplayAttack);
    }

    // Router decides auth + downstream handling.
    router
        .route(&packet)
        .await
        .map_err(|e| ListenerError::Router(e.to_string()))?;

    Ok(())
}
