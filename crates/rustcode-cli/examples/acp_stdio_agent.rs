/// Manual sanity check: run with piped initialize frame to verify round-trip.
/// Usage: echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-01-01"}}' | cargo run -p rustcode --example acp_stdio_agent
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rustcode::acp::serve_stdio(rustcode::acp::AcpServeOptions::default()).await
}
