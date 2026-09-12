//! Tunnel wire protocol shared by the client (frpc) and the relay (frps).
//!
//! Frames travel as **binary WebSocket messages**. WebSocket preserves message
//! boundaries, so one message carries exactly one frame:
//!
//! ```text
//! [1 byte type][4 byte stream id][payload...]
//! ```
//!
//! * `type = 1` `Open(id)` -- relay -> client: open a new stream to the local endpoint.
//! * `type = 2` `Data(id, payload)` -- both directions: payload bytes for stream `id`.
//! * `type = 3` `Close(id)` -- both directions: stream `id` is finished, clean up.
//!
//! `Open`/`Close` carry no payload; for `Data` the payload is the remainder of the
//! message. This is a pure byte pump -- no request/response framing is assumed --
//! so long-lived streams (SSE) work unchanged.

use std::fmt;

/// `Open` -- relay asks the client to open a new stream.
pub const FRAME_OPEN: u8 = 1;
/// `Data` -- payload bytes for an existing stream.
pub const FRAME_DATA: u8 = 2;
/// `Close` -- an existing stream is finished.
pub const FRAME_CLOSE: u8 = 3;

/// Query parameter that carries the tunnel token on the control-channel handshake.
pub const TOKEN_QUERY_PARAM: &str = "token";

/// Header size: 1 byte type + 4 byte big-endian stream id.
pub const HEADER_LEN: usize = 5;

/// One frame on the tunnel control channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    /// Open a new stream with the given id.
    Open(u32),
    /// Payload bytes for the given stream.
    Data(u32, Vec<u8>),
    /// The given stream is closed; release its resources.
    Close(u32),
}

impl Frame {
    /// The stream this frame belongs to.
    pub fn stream_id(&self) -> u32 {
        match self {
            Frame::Open(id) | Frame::Data(id, _) | Frame::Close(id) => *id,
        }
    }

    /// The on-wire type byte.
    pub fn type_byte(&self) -> u8 {
        match self {
            Frame::Open(_) => FRAME_OPEN,
            Frame::Data(..) => FRAME_DATA,
            Frame::Close(_) => FRAME_CLOSE,
        }
    }

    /// Encode into exactly one WebSocket binary message.
    pub fn encode(&self) -> Vec<u8> {
        let payload: &[u8] = match self {
            Frame::Data(_, p) => p,
            Frame::Open(_) | Frame::Close(_) => &[],
        };
        let mut out = Vec::with_capacity(HEADER_LEN + payload.len());
        out.push(self.type_byte());
        out.extend_from_slice(&self.stream_id().to_be_bytes());
        out.extend_from_slice(payload);
        out
    }
}

/// Errors produced while decoding a frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    /// Message is shorter than the 5-byte header.
    Truncated { need: usize, got: usize },
    /// Type byte is not one of Open/Data/Close.
    UnknownFrameType(u8),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::Truncated { need, got } => {
                write!(f, "truncated frame: need {need} bytes, got {got}")
            }
            ProtocolError::UnknownFrameType(t) => write!(f, "unknown frame type: {t}"),
        }
    }
}

impl std::error::Error for ProtocolError {}

/// Decode one complete WebSocket message into a [`Frame`].
pub fn decode(msg: &[u8]) -> Result<Frame, ProtocolError> {
    if msg.len() < HEADER_LEN {
        return Err(ProtocolError::Truncated {
            need: HEADER_LEN,
            got: msg.len(),
        });
    }
    let mut id_bytes = [0u8; 4];
    id_bytes.copy_from_slice(&msg[1..HEADER_LEN]);
    let id = u32::from_be_bytes(id_bytes);
    match msg[0] {
        FRAME_OPEN => Ok(Frame::Open(id)),
        FRAME_CLOSE => Ok(Frame::Close(id)),
        FRAME_DATA => Ok(Frame::Data(id, msg[HEADER_LEN..].to_vec())),
        other => Err(ProtocolError::UnknownFrameType(other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_round_trip() {
        let f = Frame::Open(0x0102_0304);
        assert_eq!(f.encode(), vec![FRAME_OPEN, 1, 2, 3, 4]);
        assert_eq!(decode(&f.encode()).unwrap(), f);
    }

    #[test]
    fn data_round_trip() {
        let f = Frame::Data(7, b"hello".to_vec());
        let enc = f.encode();
        assert_eq!(&enc[..HEADER_LEN], &[FRAME_DATA, 0, 0, 0, 7]);
        assert_eq!(&enc[HEADER_LEN..], b"hello");
        assert_eq!(decode(&enc).unwrap(), f);
    }

    #[test]
    fn empty_data_round_trip() {
        let f = Frame::Data(9, Vec::new());
        assert_eq!(f.encode().len(), HEADER_LEN);
        assert_eq!(decode(&f.encode()).unwrap(), f);
    }

    #[test]
    fn close_round_trip() {
        let f = Frame::Close(42);
        assert_eq!(decode(&f.encode()).unwrap(), f);
    }

    #[test]
    fn stream_id_and_type_are_consistent() {
        assert_eq!(Frame::Open(3).stream_id(), 3);
        assert_eq!(Frame::Data(3, vec![]).stream_id(), 3);
        assert_eq!(Frame::Close(3).stream_id(), 3);
        assert_eq!(Frame::Open(3).type_byte(), FRAME_OPEN);
        assert_eq!(Frame::Data(3, vec![]).type_byte(), FRAME_DATA);
        assert_eq!(Frame::Close(3).type_byte(), FRAME_CLOSE);
    }

    #[test]
    fn truncated_message_is_rejected() {
        assert_eq!(
            decode(&[FRAME_OPEN, 0, 0]),
            Err(ProtocolError::Truncated {
                need: HEADER_LEN,
                got: 3
            })
        );
    }

    #[test]
    fn unknown_type_is_rejected() {
        assert_eq!(
            decode(&[9, 0, 0, 0, 1]),
            Err(ProtocolError::UnknownFrameType(9))
        );
    }

    #[test]
    fn interleaved_streams_decode_independently() {
        // Each WS message is decoded on its own; stream ids must not bleed across.
        let a = Frame::Data(1, b"aaa".to_vec()).encode();
        let b = Frame::Data(2, b"bbb".to_vec()).encode();
        assert_eq!(decode(&a).unwrap(), Frame::Data(1, b"aaa".to_vec()));
        assert_eq!(decode(&b).unwrap(), Frame::Data(2, b"bbb".to_vec()));
    }
}
