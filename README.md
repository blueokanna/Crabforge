# Crabforge

Crabforge is a local model gateway written in Rust with a Flutter desktop console. The Rust CLI, desktop app, and authenticated QUIC tunnels share the same provider and routing configuration.

Build the CLI with `cargo build --locked --release -p crabforge-cli`. To build the Windows app, run `flutter pub get` and `flutter build windows --release` from `gui`. Distribute the entire generated `Release` directory.

The desktop app uses the fonts and icon in `assets`. Its language picker supports Chinese, English, French, German, Hindi, Japanese, Korean, Portuguese and Spanish. The default follows the operating system, with English as the fallback.

[使用说明](CRABFORGE.md) · [Compatibility and validation](docs/crabforge-validation.md)
