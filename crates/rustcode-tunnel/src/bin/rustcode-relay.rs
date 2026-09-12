//! Relay binary: run the frps half of the self-hosted tunnel.
//!
//! ```text
//! rustcode-relay --control 0.0.0.0:7000 --public 0.0.0.0:8080 --token <secret>
//! ```
//!
//! Every flag can also come from the environment: `RUSTCODE_RELAY_CONTROL`,
//! `RUSTCODE_RELAY_PUBLIC`, `RUSTCODE_RELAY_TOKEN`. See `docs/relay.md`.

use anyhow::{bail, Context, Result};
use rustcode_tunnel::server::run_relay;

const DEFAULT_CONTROL: &str = "0.0.0.0:7000";
const DEFAULT_PUBLIC: &str = "0.0.0.0:8080";

const USAGE: &str = "\
rustcode-relay -- self-hosted relay (frps half) for the RustCode tunnel

USAGE:
    rustcode-relay [OPTIONS]

OPTIONS:
    --control <ADDR>   control channel listen address  [default: 0.0.0.0:7000]
    --public  <ADDR>   public port remote clients hit  [default: 0.0.0.0:8080]
    --token   <TOKEN>  shared secret the dev machine must present (required)
    -h, --help         print this help

ENV:
    RUSTCODE_RELAY_CONTROL, RUSTCODE_RELAY_PUBLIC, RUSTCODE_RELAY_TOKEN
";

fn main() -> Result<()> {
    let mut control =
        std::env::var("RUSTCODE_RELAY_CONTROL").unwrap_or_else(|_| DEFAULT_CONTROL.to_string());
    let mut public =
        std::env::var("RUSTCODE_RELAY_PUBLIC").unwrap_or_else(|_| DEFAULT_PUBLIC.to_string());
    let mut token = std::env::var("RUSTCODE_RELAY_TOKEN").ok();

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--control" => control = args.next().context("--control requires a value")?,
            "--public" => public = args.next().context("--public requires a value")?,
            "--token" => token = Some(args.next().context("--token requires a value")?),
            "-h" | "--help" => {
                print!("{USAGE}");
                return Ok(());
            }
            other => bail!("unknown argument '{other}' (try --help)"),
        }
    }

    let token = token.context("missing relay token: pass --token or set RUSTCODE_RELAY_TOKEN")?;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("build tokio runtime")?;
    rt.block_on(run_relay(&control, &public, &token))
}
