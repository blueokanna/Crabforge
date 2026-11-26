# Crabforge

[简体中文](README.zh-CN.md) · [Operator guide (Chinese)](CRABFORGE.md) · [Compatibility and validation](docs/crabforge-validation.md)

Crabforge runs an authenticated model gateway on your computer. Its Flutter desktop console and CLI share a Rust core. Agents can keep one API address and gateway key while you switch providers or route a request through several models. The gateway reads upstream credentials from the environment.

## Connect your first client

1. Set your provider's API key in the environment that starts Crabforge, for example `DEEPSEEK_API_KEY`. Open **Providers**, choose a preset, enter the environment variable name, then fetch the catalog or enter model IDs and save.
2. Start the service under **Gateway** and copy its API address and key. The default OpenAI-compatible base URL is `http://127.0.0.1:3425/v1`. Copy a provider-qualified ID from **Models**, such as `deepseek/deepseek-chat`, into your client.
3. For failover, add models under **Route groups**, select a policy and point the client at a group such as `group/work`. **Routes** shows the provider and HTTP status for each attempt.

`carbforge-cli` and `crabforge-cli` expose the same commands. Replace the example model names with configured IDs. Run the client in a second terminal while `serve` is active.

```powershell
carbforge-cli catalog
carbforge-cli models deepseek
carbforge-cli group set work primary/model backup/model
carbforge-cli serve
```

```powershell
carbforge-cli agent claude group/work
```

On Windows, the configuration, gateway key and P2P identity live under `%LOCALAPPDATA%\com.blue.crabforge`. Set `CRABFORGE_HOME` to an absolute path or pass `--data-dir` to use an isolated configuration. `carbforge-cli key` prints the credential for clients of that gateway.

## Protocols and routing

- Clients can use OpenAI Chat Completions, OpenAI Responses, Anthropic Messages, Gemini and Ollama APIs. The validation record describes which streaming, tool, media and authentication paths have been exercised. Model availability depends on the provider and account.
- Groups support ordered, rotating, weighted, manual and nested routes. `carbforge-cli provider fallback primary group/work` configures a fallback for direct requests to `primary`. `carbforge-cli provider set backup fallback-for-others=no` prevents an ordinary provider's failed request from being sent to `backup`.
- The desktop console follows the system locale by default and offers Chinese, English, French, German, Hindi, Japanese, Korean, Portuguese and Spanish.
- Use the list or grid button on any of the six console pages. The other pages follow your choice, and the console restores it on the next launch.

## Reach another gateway

Run `carbforge-cli peer-id` on the client. Add the resulting ID to the gateway's trusted peers, configure its P2P listen address and start it. For direct connections, use a reachable IPv4 or IPv6 address and UDP port. For a Circuit Relay v2 reservation, configure a relay reachable from both hosts and set the gateway's listen address to `/ip6/RELAY_IP/udp/PORT/quic-v1/p2p/RELAY_ID/p2p-circuit` with the relay's actual address and identity.

Pass the full address advertised by the gateway to `carbforge-cli tunnel <address>` on the client. If you have both addresses, use `carbforge-cli tunnel <direct-address> --relay-fallback <circuit-address>`. Both addresses must end with the same gateway Peer ID. The console accepts the same optional relay address. The tunnel listens on `http://127.0.0.1:3426/v1` by default and uses the remote gateway key. Local tests cover bidirectional QUIC with Kubo, IPv6 relay fallback and a DCUtR upgrade followed by an authenticated request after the relay went offline. Public IPv6 and hole punching across independent NATs still need cross-host verification.

## Build for Windows x64

A three-round desktop comparison used the unpacked `efb2db7` build. Crabforge used a median 129.80 MiB working set and 221.44 MiB private bytes. Magpie and its seven WebView2 child processes used 528.54 MiB and 868.88 MiB respectively. Crabforge started with fresh settings each round. Magpie kept its existing visible window. [Desktop samples](docs/validation/cf-protocol-efb2db7/desktop.csv)

The frozen Rust suite passed 100 of 101 tests on its first run. The OmO client timed out before its first gateway request and passed all four protocols on a separate run. The unpacked gateway used a median 7.90 MiB working set and 1.49 MiB private bytes with an empty catalog. For the Chat workload those figures were 8.84 and 1.56 MiB. The validation record includes the ZIP checksum, raw measurements, and same-host IPv6 results. [Windows validation](docs/crabforge-validation.md)

Install the Rust MSVC toolchain, Flutter and the Visual Studio Desktop development with C++ workload. The recorded Windows builds used Rust 1.99.1, Flutter 3.47.9 and Dart 3.13.7.

```powershell
cargo build --release --locked -p crabforge-cli
cd gui
flutter pub get
flutter build windows --release
```

The CLI executables are in `target/release`. Distribute the entire `gui/build/windows/x64/runner/Release` directory, including `crabforge.exe`, the Flutter runtime, the Rust bridge and `data`. The Magpie port is in progress. Measurements and compatibility results are tied to the specific snapshots listed in the validation record.
