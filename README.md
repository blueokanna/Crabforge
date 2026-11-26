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

Use the folder picker on a memory's **Scope** field to associate it with a project. Crabforge uses the Git remote to give clones and worktrees the same project key. A project without a network remote uses its local path. In **Project instructions → Project memory**, preview the active project memories and publish the digest to the selected Agent's file. The published Markdown is visible to anyone with access to that file. Archive a note and publish again to remove it from the digest. **Restore previous version** can undo a publication.

```powershell
carbforge-cli memory project-key D:\Projects\app
carbforge-cli memory digest D:\Projects\app
carbforge-cli memory publish D:\Projects\app codex
```

To let an Agent search and save memories, register the full path to `carbforge-cli.exe` as a local stdio MCP server, with arguments `mcp memory --agent <agent-id> --project <folder>`. You can pass `--project-key <project-key>` in place of `--project <folder>`. Set the same `CRABFORGE_HOME` in the client environment if you use a custom data directory. `memory_search` reads active records in the available scopes. `memory_save` creates records and can update agent-created ones. `memory_forget` archives a visible record, and `memory_context` reports the scopes. Run `mcp memory` without arguments to expose only global memory.

Open **Client connections → Local sessions** to browse up to 50 recent Claude Code and Codex sessions, filter by agent and copy the session ID. Run `carbforge-cli sessions --limit 50` for JSON, with `--agent claude` or `--agent codex` to filter. Crabforge reads native JSONL and Codex `.jsonl.zst` logs. It returns the session ID, prompt preview, timestamp, working folder and profile folder. The primary profiles come from `CLAUDE_CONFIG_DIR` and `CODEX_HOME` or the default `.claude` and `.codex` folders. Crabforge also looks for profiles containing session logs in Claude- or Codex-named folders next to the primary profiles and directly under the user home, including one level of profile subfolders.

In a session's details, select **Extract memories** to review the user messages and choose a configured model route. **Send to model** submits that preview to the route. Crabforge verifies the quoted evidence and stores up to five candidates. Review them in **Memory** and set their status to **Active** to include them in scoped digests and MCP search.

For the CLI, run `carbforge-cli memory distill claude <session-id>` to get the messages and preview revision. Then run `carbforge-cli memory distill claude <session-id> --model <route-id> --revision <revision>`. Use `--profile <profile-folder>` on both calls to target a discovered secondary profile, and `codex` for Codex sessions. `carbforge-cli memory search --status candidate` lists candidates for review.

Select Claude Code or Codex in **Client connections**, then open **User instructions** to edit the managed Markdown or preview and publish **User memory**. It includes active global notes and the selected agent's notes. Crabforge writes the digest to `CLAUDE.md` or to the Codex user `AGENTS.md`, giving an existing `AGENTS.override.md` precedence. The client reads that file into new conversations. Use **Restore previous version** to roll back the most recent Crabforge edit to the file.

In the CLI, `carbforge-cli memory context --agent codex` previews the same digest and `carbforge-cli memory agent-instructions codex` reports the current file and revision. Append `--publish` to publish, `--preview <UTF-8 file>` to require a saved preview to match, or `--restore` to restore. `--revision <revision>` checks that the instruction file still matches the inspected version.

Open **Rules** under **Client connections** to save Markdown guidance and assign it to Claude Code, Cline or Kiro CLI. Claude Code and Kiro CLI accept optional path patterns. Cline applies its rules to every conversation. The library shows each native file and its status. Editing, renaming or removing a rule affects the unchanged copies Crabforge manages. A pre-existing file or a local edit stays in place and appears as a conflict. **Apply rules** retries outstanding changes for the current agent profiles.

```powershell
carbforge-cli rules save diagnostics --from diagnostics.md --agent claude --agent kiro-cli --path 'src/**/*.rs'
carbforge-cli rules list
```

Pass `--old <name>` to edit an existing rule or rename it. Add `--revision <revision>` from `rules list` to require the same version. `rules remove <name>` removes the library entry and its unchanged managed copies.

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
