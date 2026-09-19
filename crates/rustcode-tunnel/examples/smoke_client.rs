//! Manual smoke driver: connect to a running relay and forward to a local port.
//!
//! ```text
//! cargo run -p rustcode-tunnel --example smoke_client -- <relay_url> <token> <local_port>
//! ```

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let relay = args
        .next()
        .expect("usage: smoke_client <relay_url> <token> <local_port>");
    let token = args.next().expect("missing <token>");
    let port: u16 = args
        .next()
        .expect("missing <local_port>")
        .parse()
        .expect("local_port must be a number");
    println!("smoke client: {relay} -> 127.0.0.1:{port}");
    rustcode_tunnel::client::start_tunnel_client(&relay, &token, port).await
}
