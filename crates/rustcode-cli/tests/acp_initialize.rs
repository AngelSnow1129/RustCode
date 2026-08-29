// Minimal smoke: serve_stdio must exist and AcpServeOptions must be constructible.
#[test]
fn serve_stdio_symbol_exists() {
    // Compile-time proof the public surface exists; behavior covered in Task 11.
    let _f: fn(rustcode::acp::AcpServeOptions) -> _ = rustcode::acp::serve_stdio;
    let _opts = rustcode::acp::AcpServeOptions {
        engine: None,
        provider_factory: None,
        auto_approve: false,
        ..Default::default()
    };
}
