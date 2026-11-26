# Crabforge

[简体中文](README.zh-CN.md) · [Operator guide (Chinese)](CRABFORGE.md) · [Windows validation](docs/crabforge-validation.en.md)

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

Open **Client connections → Project instructions** for a supported agent, then choose a project folder. Crabforge edits the Magpie-compatible Markdown block in the selected agent's file. **Restore previous version** restores the previous file after checking that the current version still matches the backup.

For Gemini CLI, Qwen Code, Qoder and Qoder CN, Crabforge checks `context.fileName` in the project settings, then the applicable Crabforge profile and the Agent's user settings. A list uses its first nonempty name. The file must be inside the project.

```powershell
carbforge-cli project-instructions D:\Projects\app codex
carbforge-cli project-instructions D:\Projects\app codex --from instructions.md
carbforge-cli project-instructions D:\Projects\app codex --restore
```

Open **Client connections → Memory** to add, edit, pin or archive Markdown memories. Search by text in the desktop console or filter by scope, kind and tags from the CLI. Each memory is a Markdown file with YAML front matter under `memory/` in the Crabforge data directory. To bring over Magpie Markdown memories, copy the individual `.md` files into that directory.

```powershell
carbforge-cli memory add preference.md --kind preference --tag workflows
carbforge-cli memory search --kind preference --status active
carbforge-cli memory edit <id> --status archived
```

To share those records with an Agent, register the full path to `carbforge-cli.exe` as a local stdio MCP server, with arguments `mcp memory --agent <agent-id> --project-key <project-key>`. Set the same `CRABFORGE_HOME` in the client environment if you use a custom data directory. `memory_search` reads active records in the available scopes. `memory_save` creates records and can update agent-created ones. `memory_forget` archives a visible record, and `memory_context` reports the scopes. Omit the two optional arguments for global memory only. Use the same project key for all clients working on one project.

## Protocols and routing

- Clients can use OpenAI Chat Completions, OpenAI Responses, Anthropic Messages, Gemini and Ollama APIs. The validation record describes which streaming, tool, media and authentication paths have been exercised. Model availability depends on the provider and account.
- Groups support ordered, rotating, weighted, manual and nested routes. `carbforge-cli provider fallback primary group/work` configures a fallback for direct requests to `primary`. `carbforge-cli provider set backup fallback-for-others=no` prevents an ordinary provider's failed request from being sent to `backup`.
- The desktop console follows the system locale by default and offers Chinese, English, French, German, Hindi, Japanese, Korean, Portuguese and Spanish.
- Switch between list and grid on any console page, in the routing group member editor, or when choosing an Oh My Pi profile, protocol, role, or model. The app remembers the layout on restart. Switching while editing a group keeps the weight you have entered.

## Reach another gateway

Run `carbforge-cli peer-id` on the client. Add the resulting ID to the gateway's trusted peers, configure its P2P listen address and start it. For direct connections, use a reachable IPv4 or IPv6 address and UDP port. For a Circuit Relay v2 reservation, configure a relay reachable from both hosts and set the gateway's listen address to `/ip6/RELAY_IP/udp/PORT/quic-v1/p2p/RELAY_ID/p2p-circuit` with the relay's actual address and identity.

Pass the full address advertised by the gateway to `carbforge-cli tunnel <address>` on the client. If you have both addresses, use `carbforge-cli tunnel <direct-address> --relay-fallback <circuit-address>`. Both addresses must end with the same gateway Peer ID. The console accepts the same optional relay address. The tunnel listens on `http://127.0.0.1:3426/v1` by default and uses the remote gateway key. Local tests cover bidirectional QUIC with Kubo, IPv6 relay fallback and a DCUtR upgrade followed by an authenticated request after the relay went offline. Public IPv6 and hole punching across independent NATs still need cross-host verification.

## Build for Windows x64

The unpacked `9aeb7e5` gateway had median working set and private bytes of 7.53 / 1.42 MiB for an empty catalog and 8.85 / 1.86 MiB for Chat. With a 700 ms delay between upstream events, the first complete SSE event arrived in 6, 2 and 2 ms. Three desktop cold starts from the unpacked ZIP had a median working set of 154.39 MiB and 227.91 MiB of private bytes. The frozen source passed 113 Rust tests and all 28 dependency checks across Windows x64 and all Cargo targets. The checked runtime, build and test trees had no enabled Serde, Hyper or Reqwest packages. [Dependency audit](docs/validation/cf-windows-9aeb7e5/dependency-audit.csv) · [Windows validation](docs/crabforge-validation.en.md)

Install the Rust MSVC toolchain, Flutter and the Visual Studio Desktop development with C++ workload. The recorded Windows builds used Rust 1.99.1, Flutter 3.47.9 and Dart 3.13.7.

```powershell
cargo build --release --locked -p crabforge-cli
cd gui
flutter pub get
flutter build windows --release
```

The CLI executables are in `target/release`. Distribute the entire `gui/build/windows/x64/runner/Release` directory, including `crabforge.exe`, the Flutter runtime, the Rust bridge and `data`. The Magpie port is in progress. Measurements and compatibility results are tied to the specific snapshots listed in the validation record.
