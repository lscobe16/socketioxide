use crate::{adapter::Adapter, packet::Packet, socket::RetryablePacket, Socket};
use engineioxide::sid_generator::Sid;
use std::{
    fmt::{Debug, Display},
    sync::Arc,
};
use tokio::sync::oneshot;

/// Error type for socketio
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error serializing json packet: {0:?}")]
    SerializeError(#[from] serde_json::Error),

    #[error("invalid packet type")]
    InvalidPacketType,

    #[error("invalid event name")]
    InvalidEventName,

    #[error("cannot find socketio engine")]
    EngineGone,

    #[error("cannot find socketio socket")]
    SocketGone(Sid),

    /// An engineio error
    #[error("engineio error: {0}")]
    EngineIoError(#[from] engineioxide::errors::Error),

    #[error("adapter error: {0}")]
    Adapter(#[from] AdapterError),
}

/// Error type for ack responses
#[derive(thiserror::Error, Debug)]
pub enum AckError {
    /// The ack response cannot be parsed
    #[error("error serializing/deserializing json packet: {0:?}")]
    SerdeError(#[from] serde_json::Error),

    /// The ack response cannot be received correctly
    #[error("ack receive error")]
    AckReceiveError(#[from] oneshot::error::RecvError),

    /// The ack response timed out
    #[error("ack timeout error")]
    AckTimeoutError(#[from] tokio::time::error::Elapsed),

    /// Internal error
    #[error("internal error: {0}")]
    InternalError(#[from] Error),

    #[error("send channel error: {0:?}")]
    SendChannel(#[from] SendError),
}

/// Error type for broadcast operations.
#[derive(Debug, thiserror::Error)]
pub enum BroadcastError {
    /// An error occurred while sending packets.
    #[error("Sending error: {0:?}")]
    SendError(Vec<SendError>),

    /// An error occurred while serializing the JSON packet.
    #[error("Error serializing JSON packet: {0:?}")]
    Serialize(#[from] serde_json::Error),

    #[error("Adapter error: {0}")]
    Adapter(#[from] AdapterError),
}

impl From<Vec<SendError>> for BroadcastError {
    /// Converts a vector of `SendError` into a `BroadcastError`.
    ///
    /// # Arguments
    ///
    /// * `value` - A vector of `SendError` representing the sending errors.
    ///
    /// # Returns
    ///
    /// A `BroadcastError` containing the sending errors.
    fn from(value: Vec<SendError>) -> Self {
        Self::SendError(value)
    }
}

/// Error type for sending operations.
#[derive(thiserror::Error, Debug)]
pub enum SendError {
    /// An error occurred while serializing the JSON packet.
    #[error("Error serializing JSON packet: {0:?}")]
    Serialize(#[from] serde_json::Error),
    /// An error occurred during the transport of the packet.
    #[error("Transport error: {0:?}")]
    TransportError(#[from] TransportError),

    #[error("Adapter error: {0}")]
    AdapterError(#[from] AdapterError),
}

#[derive(thiserror::Error, Debug)]
pub enum TransportError {
    #[error("Failed to send failed bin payloads")]
    SendFailedBinPayloads(Option<Packet>),
    #[error("Sent to a closed socket channel")]
    SocketClosed,
    #[error("Failed to send main message")]
    SendMainPacket(RetryablePacket),
}

#[derive(thiserror::Error, Debug)]
pub enum AckSenderError<A: Adapter> {
    #[error("Failed to send ack message")]
    SendError {
        /// The specific error that occurred while sending the message.
        send_error: SendError,
        /// The socket associated with the error.
        socket: Arc<Socket<A>>,
    },
}

impl TransportError {
    /// If PacketSender::send_buffered_binaries fails before packet deserialization
    /// It is needed to return this owned packet using the method
    pub(crate) fn add_main_packet(self, packet: Packet) -> Self {
        if let TransportError::SendFailedBinPayloads(_) = self {
            TransportError::SendFailedBinPayloads(Some(packet))
        } else {
            self
        }
    }
}

/// Error type for the [`Adapter`] trait.
#[derive(Debug, thiserror::Error)]
pub struct AdapterError(#[from] pub Box<dyn std::error::Error + Send>);
impl Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}
