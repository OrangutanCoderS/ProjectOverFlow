use serde::Deserialize;
use serde_json::Value;

/// Supported packet types arriving over the mesh socket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    Telemetry,
    PluginGraphUpdate,
    SecurityAlert,
}

impl PacketType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "TELEMETRY_PACKET" => Some(Self::Telemetry),
            "PLUGIN_GRAPH_UPDATE" => Some(Self::PluginGraphUpdate),
            "SECURITY_ALERT" => Some(Self::SecurityAlert),
            _ => None,
        }
    }
}

/// Canonical packet structure for mesh traffic.
#[derive(Debug, Deserialize)]
pub struct MeshPacket {
    pub packet_type: String,
    pub origin_id: String,
    pub timestamp: String,
    pub payload: Value,
    pub auth_token: String,
}

impl MeshPacket {
    /// Convert to strong PacketType, if valid.
    pub fn packet_kind(&self) -> Option<PacketType> {
        PacketType::from_str(self.packet_type.as_str())
    }

    /// Quick structural sanity checks without deep semantics.
    pub fn is_structurally_valid(&self) -> bool {
        !self.packet_type.is_empty()
            && !self.origin_id.is_empty()
            && !self.timestamp.is_empty()
            && !self.auth_token.is_empty()
    }
}
