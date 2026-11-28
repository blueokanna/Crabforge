# Crabforge

[简体中文](README.zh-CN.md) · [Windows build](https://github.com/blueokanna/Crabforge/actions/workflows/windows-build.yml) · [Releases](https://github.com/blueokanna/Crabforge/releases)

Crabforge is a local, authenticated model gateway with a Rust core, a Flutter desktop console, and a CLI. Connect a client once, then change providers, model aliases, or routing policy from the gateway. The desktop manages providers, model discovery, route groups, request history, and supported clients’ configuration.

## Install on Windows

Download the Windows x64 ZIP and its `.sha256` file from the same release. Verify the checksum and GitHub build provenance before extracting. Run `desktop\crabforge.exe` from the extracted folder. The two CLI binaries at the archive root, `crabforge-cli.exe` and `carbforge-cli.exe`, expose the same commands. [Verification instructions](docs/windows-release.en.md)

## Connect a client

1. Set your provider credential in the environment that starts Crabforge. In **Providers**, choose a preset or enter a custom endpoint, enter the credential’s environment variable name, and save the provider with at least one model.
2. Start **Gateway** and copy the key. For OpenAI-compatible clients, the default base URL is `http://127.0.0.1:3425/v1`. Copy a provider-qualified model ID from **Models**.
3. For failover, add a **Route group** and select it in the client using its published model ID. Use **Routes** to inspect the provider and status of each attempt.

To start the same gateway with the CLI, run these commands from the extracted folder. The second command prints a local credential. Run `serve` in the terminal that has access to the upstream credentials.

```powershell
.\crabforge-cli.exe catalog
.\crabforge-cli.exe key
.\crabforge-cli.exe serve
```

Windows stores settings and the gateway key under `%LOCALAPPDATA%\com.blue.crabforge`. Set `CRABFORGE_HOME` to an absolute path, or use `--data-dir` with the CLI, to work with a separate configuration. Keep the gateway key private. The default listener is loopback. Browser clients also need an allowed origin.

## Protocols and routing

- Gateway endpoints include OpenAI Chat Completions and Responses, Anthropic Messages, and Gemini Developer API. The gateway forwards native requests where possible and translates supported generation requests when protocols differ.
- Route groups support ordered failover, rotation, weighted selection, and manual selection. Groups can define request-matching rules, first-content timeouts, and retry patience.
- Audio, image, and decision routes depend on the configured provider and model. Use the upstream model’s native endpoint when a request requires a field the destination protocol cannot represent.
- The CLI previews encrypted backups before applying selected entries. Run `crabforge-cli import-backup --help` for options. Portable route-group shares are available through `group share` and `group import`.

## Keep a provider directory current

In **Providers**, set a model directory refresh interval such as `10m`, `6h`, or `1d`. Ten minutes is the minimum. A provider created from the **Remote Gateway** preset uses ten minutes by default. Use `off` for manual refresh. The **Load model catalog** action on the provider card refreshes its saved directory. Add one inclusion rule per line to pick newly listed models automatically. Globs such as `gpt-*` ignore case, and regular expressions prefixed with `re:` match the entire model ID. A model you deselect stays deselected while it remains in the upstream directory, unless you add a new rule that matches it. The gateway applies directory changes while serving.

```powershell
crabforge-cli provider refresh-every <provider-id> 10m
crabforge-cli provider auto-include <provider-id> 'gpt-*' 're:claude-.*'
crabforge-cli provider refresh <provider-id>
```

## Antigravity

With `agy` on `PATH` and the gateway running, launch Antigravity with `crabforge-cli agent antigravity <model-id>`. The launcher uses Antigravity’s Gemini API authentication mode and passes the gateway key and model selection to that session. In **Client connections**, choose **Antigravity CLI** to copy its gateway root. [Antigravity setup](docs/antigravity.md)

Crabforge also manages Antigravity project instructions, MCP configuration, hooks, and subagents. Global skills and subagents use Antigravity’s shared configuration directory. [Antigravity configuration discovery](https://antigravity.google/docs/customization-discovery)

## Extract memories from sessions

Automatic memory extraction is off by default. In **Client connections**, open **Local sessions** and **Automatic memory extraction**. Select a model or route group and a lookback period, then save. While the gateway runs, Crabforge sends user messages from sessions that have been quiet for 30 minutes to that route. Each hourly run attempts up to 30 sessions. Use **Read now** to process all eligible sessions in the recent list and review the resulting candidates in **Memory**.

Use `crabforge-cli memory distillation status` to inspect the schedule. The `memory distillation` subcommands `enable <model-or-group> --days 7`, `run`, and `disable` configure, run, and stop it.

## Desktop appearance

The desktop uses Material 3 Expressive button shapes and emphasized title styles, along with Google’s Material Symbols Rounded. The Windows accent seeds the Expressive light and dark palettes, including high-contrast variants. In the toolbar’s font menu, Google Sans renders Latin text and HarmonyOS Sans SC supplies Han glyphs. Selecting HarmonyOS Sans SC uses it for both English and Chinese. [Bundled font sources and licenses](assets/fonts/manifest.json)

## Build from source

The Windows CI configuration pins Rust 1.99.1 and Flutter 3.47.9. On Windows, install the Visual Studio C++ desktop workload and Windows SDK, then run the checks and builds below.

```powershell
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo build --locked --release --package crabforge-cli
Set-Location gui
flutter pub get --enforce-lockfile
flutter analyze --no-pub
flutter test --no-pub
flutter build windows --release --no-pub
```

[Operator guide (中文)](CRABFORGE.md) · [Release workflow](.github/workflows/windows-release.yml) · [License](LICENSE)
