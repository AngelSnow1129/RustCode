//! Minimal self-hosted frp-style reverse tunnel.
//!
//! Two halves share one wire protocol (see [`protocol`]):
//!
//! * [`client`] (frpc) runs on the dev machine: opens a WebSocket control channel
//!   to the relay and forwards inbound streams to a local port.
//! * [`server`] (frps) is the relay: exposes a public port and multiplexes inbound
//!   connections to the connected client over the control channel.
//!
//! Wire format (binary WebSocket messages, big-endian):
//! `[1 byte type][4 byte stream id][payload]`, with type 1=Open, 2=Data, 3=Close.
//!
//! See `docs/relay.md` for the full design and the self-hosting guide.

pub mod client;
pub mod protocol;
pub mod server;
