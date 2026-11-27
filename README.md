# Crabforge

[简体中文](README.zh-CN.md) · [Operator guide (Chinese)](CRABFORGE.md) · [Windows releases](docs/windows-release.en.md) · [Windows validation](docs/crabforge-validation.en.md)

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

## Import providers from Magpie

Use a `.magpie-backup` from `magpie backup`. The CLI prompts for its passphrase without echo. Run the first command to preview, then add `--apply` to save the result. Omit `--provider` to select all providers, or repeat it to select several.

```powershell
carbforge-cli provider import-magpie .\magpie.magpie-backup --provider deepseek --key-env deepseek=DEEPSEEK_API_KEY
carbforge-cli provider import-magpie .\magpie.magpie-backup --provider deepseek --key-env deepseek=DEEPSEEK_API_KEY --apply
```

The import supports providers with explicit model IDs and standard HTTP API endpoints. It preserves supported routing settings and requires unique provider IDs and aliases. Set the API key in the specified environment variable before starting Crabforge. The result lists other backup sections under `other_sections`. For backups above 64 MiB, create a smaller file with `magpie backup --no-usage --no-library`.

After configuring the providers, preview and apply routing groups from the same backup. Repeat `--group` for each group you want to add, including any nested group that is absent from the destination. Omit it to select every stored group.

```powershell
carbforge-cli group import-magpie .\magpie.magpie-backup --group work --group fallback
carbforge-cli group import-magpie .\magpie.magpie-backup --group work --group fallback --apply
```

The group import preserves IDs, former IDs, display order and supported routing settings. An ID conflict, unavailable member or unsupported setting such as a classifier rejects the entire selection. Portable JSON shares from `magpie group share <id> --text` can also be added with `carbforge-cli group import <file>`.

To restore providers and groups together, use `import-magpie` at the top level. This also works when a provider's fallback points to a group in the same backup. Omit `--provider` or `--group` to select every entry in that section. The preview lists the selected IDs and any other backup sections. `--apply` saves both selections in one write after all references pass validation.

```powershell
carbforge-cli import-magpie .\magpie.magpie-backup --key-env deepseek=DEEPSEEK_API_KEY
carbforge-cli import-magpie .\magpie.magpie-backup --key-env deepseek=DEEPSEEK_API_KEY --apply
```

## Manage client resources

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

Open **Skills** under **Client connections** to import a local folder with `SKILL.md` and its supporting files. Select Claude Code, Codex, Gemini CLI, OpenCode or Pi to publish a copy to the current profile. For Gemini CLI and Pi, Crabforge also publishes to the profile used by `carbforge-cli agent`. **Reimport files** reads the source again. **Apply skills** retries pending changes. An existing native folder or a locally edited copy stays in place and appears as a conflict.

```powershell
carbforge-cli skills import .\diagnostics --agent claude --agent codex
carbforge-cli skills list
carbforge-cli skills assign diagnostics --agent codex --revision '<revision from list>'
```

Use `skills import` with `--revision` to update a skill of the same name. `skills remove <name>` removes it from the library and removes unchanged managed copies.

For a public GitHub skill, paste its `SKILL.md` or folder URL into the source field or pass it to `skills import`. A repository URL selects the root skill or the only skill folder it contains. If the repository contains several skills, specify the folder you want. **Reimport files** fetches the saved ref and folder again. Use a commit URL to pin the source. For branch names that contain a slash, use `?ref=feature%2Fdocs#path/to/skill` on the repository URL.

**Installed skills** finds compatible folders in native and gateway client profiles. **Manage in Crabforge** checks the discovered content hash, saves a library snapshot and takes over the selected folder in place. Unassigning removes that folder if its files are unchanged. The CLI uses `skills discover` and `skills adopt <agent> <name> <path> --content <content from discover>`.

For other Git hosts, enter the repository's HTTPS or SSH clone URL. GitLab and Gitea folder links, `user@host:path` SSH addresses and absolute `file://` repository URLs also work. Append `?ref=feature%2Fdocs#path/to/skill` to pick a ref and folder. These sources require Git on `PATH` and use its credential helper or SSH agent. **Check for updates** compares the skills' files with the library snapshot. It reports unchanged files, new content or a missing `SKILL.md`. **Reimport files** applies an update. The CLI equivalent is `skills check <name> --revision <revision from list>`.

**Browse source skills** lists available skills in a local collection folder or at the selected Git ref. Skills already in the library and candidates you set aside stay out of later scans. Choose a skill, select clients and confirm the import. `skills catalog <folder or Git URL>` returns the candidates with their import sources, and `skills dismiss <source from catalog>` sets a candidate aside. Its source remains available for direct import.

**Client connections → Environment variables** saves a variable once and assigns it to Claude Code, Codex or Pi. Crabforge edits Claude’s `env` and Codex’s `shell_environment_policy.set`, and writes a managed extension for the native and gateway Pi profiles. Pi keeps values inherited from its shell. Crabforge preserves existing values and external edits in client settings. Values are stored in the local library and native configuration and are masked by default in the desktop editor.

**Client connections → Hooks** saves lifecycle commands and publishes compatible events to Claude Code, Codex, Gemini CLI or Cursor. Gemini uses both its native and Crabforge profiles. The library can adopt a compatible command in place and reports conflicting or externally modified entries. Clients run commands according to their own hook and trust settings. Use `carbforge-cli hooks list` to see revisions and installed entries. Use `hooks save` with a UTF-8 JSON object on standard input containing `name`, `event`, `command` and `agents`. The optional fields are `description`, `matcher` and `timeout` in seconds. Portable events are `turn-end`, `after-tool`, `before-tool`, `prompt` and `session-start`. Use an event such as `claude:SessionStart` for a client-specific matcher. `hooks adopt <key> <name>` takes over an unchanged native entry, `hooks remove <name> --revision <revision>` withdraws managed copies, and `hooks sync` retries assignments.

**Remove installed hook** removes the selected event entry and saves the original client settings file under `hooks/backups` in the Crabforge data directory. A grouped entry can contain multiple commands. Restore from **Hook backups** while the client file still matches its state immediately after removal. The CLI equivalents are `hooks remove-installed <key>` and `hooks restore <id>`, using the `installed` and `backups` fields from `hooks list`.

In **Hooks → Open project**, assign shared hooks to compatible clients. Claude Code reads `.claude/settings.local.json`, Codex reads `.codex/hooks.json`, Antigravity reads `.agents/hooks.json`, and Grok Build reads `.grok/hooks/crabforge.json`. Project assignments have their own revisions and follow changes to the shared definition. Clear every client in the assignment to withdraw it. In the CLI, use `hooks --project <folder> list` for the revision under `shared`, then `hooks --project <folder> assign <name> --revision <revision> --agent claude`. Use `codex`, `agy` or `grok` for the other clients, or omit `--agent` to withdraw. The same project option scopes native discovery, removal and backup restoration. Review new or edited Codex hooks in `/hooks` according to the client's trust settings.

User assignments use `~/.gemini/config/hooks.json` for Antigravity and `$GROK_HOME/hooks/crabforge.json` for Grok Build, with `~/.grok` as the default Grok home. Antigravity keeps entries in a named `crabforge` group and supports `agy:PreInvocation` and `agy:PostInvocation` as native events. Grok discovery also reads other `.json` files in the user or project hooks directory. Adopted user hooks stay in their original files. Antigravity and Grok use their own workspace trust settings for project hooks.

DeepSeek Harness user assignments target initialized profiles under `$DSH_HOME/profiles` (`~/.dsh/profiles` by default). With `dsh` and `pnpm` on `PATH`, Crabforge uses `dsh plugin --profile` to install the official hook plugin and protocol at the installed `dsh` version and adds a profile patch referencing `crabforge-hooks.json`. It preserves existing profile settings and reports a conflict if the profile already configures the plugin. Withdrawing the final hook removes the Crabforge profile entry. Start a new Harness process after synchronization so it reloads the hook configuration.

**Client connections → Subagents** publishes native Markdown or TOML to Claude Code, Codex, OpenCode, Gemini CLI, Qwen Code, Factory Droid, Grok Build, Cursor and Copilot CLI. Open a project in the same dialog to manage that folder's subagents. Project publication also supports Antigravity. Claude Code accepts skill references and reasoning effort. Claude Code and Factory Droid accept MCP server names. Codex accepts a model ID and reasoning effort.

Open a project and select **Shared subagents** to assign entries from your user library to that project's clients. Changes and renames in the library carry through to the assigned projects. Clear the client selection to withdraw the assignment. With the CLI, use `subagents --project <folder> list` for the `shared` entries and their revisions, then `subagents --project <folder> assign <name> --revision <revision> --agent claude`. Repeat `--agent` for each client or omit it to withdraw the assignment. Unchanged managed files are withdrawn and external edits are kept.

**Client connections → Slash commands** publishes reusable prompts to Claude Code, OpenCode, Pi, Gemini CLI, Qwen Code and Factory Droid. Use `$ARGUMENTS` for all arguments, or `$1`, `$2` and so on for individual arguments in Claude Code, OpenCode and Pi. Crabforge converts the placeholders for each client and reports assignments that use unsupported arguments. For Codex, use a skill for a reusable prompt.

Select **Import slash commands** to preview external Markdown or TOML commands from a file, local folder or Git URL. Imports copy the prompt and description into the library and retain the source for update checks. Claude, Pi, Gemini and Qwen placeholders are converted when the file layout identifies their format. The CLI uses `commands probe-source <source>` and `commands import-source` with the same JSON request format as subagents.

Open **Client connections → Claude Code plugins** to manage installed plugins and the markets in your Claude Code profile. With Claude Code on `PATH`, add a local folder or remote marketplace, browse its catalog, install or update plugins, and uninstall while retaining plugin data. The CLI exposes `claude-plugins list`, `available`, `add-marketplace`, `update-marketplace`, `remove-marketplace`, `install`, `update` and `uninstall`. Use `enable` or `disable` with the `--revision` from `list` to change a user flag.

Select **Open project** to enable an installed plugin for one folder through `.claude/settings.local.json`. The CLI accepts `claude-plugins --project <folder> list` and `enable <name@marketplace> --revision <revision>` under the same project option. Withdrawing an assignment removes the key when Crabforge wrote it and the value is still `true`. The settings editor preserves other keys and existing project choices.

The plugin menu exports and imports JSON settings. In the CLI, save the output of `claude-plugins export-settings` and load it with `claude-plugins import-settings <file>`. The JSON carries GitHub or plain HTTPS marketplace declarations and their enabled plugin IDs, including enabled plugins from the official market. Local locations and sources with credentials, request headers or URL query parameters stay in the original profile. Import adds missing declarations and flags, keeps existing local choices, and leaves installation to Claude Code.

Open **Client connections → Shared plugin library** to preview a Claude Code, Codex, Antigravity or Pi plugin from its folder, manifest or Git URL. Saving records the full source snapshot, including attachments, and checks that the source still matches the preview. The update action opens a fresh preview and saves against the current library revision.

For a Claude-compatible marketplace, enter its directory, `.claude-plugin/marketplace.json` or Git URL and select **Browse plugins**. The CLI equivalent is `plugin-bundles marketplace <source>`. Choose a listed source to preview its manifest and files before saving.

Open a saved bundle, then select a component to review and copy an individual skill, subagent, command, MCP server or command hook. The destination resource library manages client and project assignments, updates and removal. Copies have their own lifecycle after creation. Definitions that reference the native plugin environment show that dependency during review. For Pi packages, the library lists declared `SKILL.md` files and Markdown prompt templates for copying. The snapshot also carries the declared extensions and themes for use with Pi.

Use `carbforge-cli plugin-bundles preview <source>` to obtain a content hash and `plugin-bundles save <source> --content <hash>` to store it. Add `--revision <revision>` from `plugin-bundles list` when updating. Use `plugin-bundles resources <name> <kind> <path> --revision <revision>` to inspect one component. Repeat with `--copy <key>` to add the reviewed resource to its library.

To move a saved snapshot to another Crabforge profile, choose **Export plugin archive** or run `plugin-bundles export <name> <file.tar> --revision <revision>`. Open the TAR in the destination's import dialog or pass it to `plugin-bundles preview` and `save`. The archive includes the original bundle files and attachments. Assignments and changes in the destination resource libraries are managed there.

In the shared plugin library, open the project packages or project plugins view for Pi, Claude Code, Codex or Antigravity. Choose a project folder and assign the reviewed snapshot. Saving a newer snapshot leaves the project pinned. Use **Sync assignments** to switch versions. Withdrawal preserves edits made outside Crabforge. Each client manages its own project trust.

- Pi references the snapshot in `.pi/settings.json`. Its native loader can discover skills, prompt templates, extensions and themes.
- Claude Code must be on `PATH`. Crabforge uses the official CLI to install the snapshot with `local` scope and withdraws it with `--keep-data`.
- Codex reads project marketplace and plugin entries from `.codex/config.toml` in a trusted project. Crabforge stages the pinned package in the selected Codex profile's plugin cache. Withdrawal removes the owned project entries. Staged snapshots remain on disk.
- Antigravity uses `.agents/plugins.json` to reference a snapshot staged inside the workspace. Withdrawal removes the owned discovery entry. Staged snapshots remain on disk.

Under `carbforge-cli plugin-bundles`, choose `pi-project`, `claude-project`, `codex-project` or `antigravity-project`. Run `<command> <folder> list` for the project `revision` and each package's `bundle_revision`. Use `assign <name> --project-revision <revision> --bundle-revision <bundle_revision>` to assign or sync and `withdraw <name> --project-revision <revision>` to withdraw.

To carry assignments, select **Export assignments** and then **Export plugin archive** on each assigned package in the source project. Import the TAR files into the destination library, open the destination project for the same client, and select **Apply assignments**. The JSON carries package names and content hashes. The destination verifies the hashes and creates local paths and ownership records. The matching CLI operations are `export --project-revision <revision>`, `archive <name> <file.tar> --project-revision <revision>` and `apply <assignments.json> --project-revision <revision>`.

`carbforge-cli subagents` and `carbforge-cli commands` support `list`, `save`, `remove`, and `sync`. `subagents --project <folder>` selects a project library. `save` reads UTF-8 JSON from standard input with `name`, `prompt`, and `agents`. Supply `description` for subagents. Optional subagent fields are `model`, `effort`, `skills` and `mcp`. Save this example as `check.json` and run `cmd.exe /d /c "carbforge-cli commands save < check.json"` to publish it from Windows.

```json
{"name":"check","description":"Review a file","prompt":"Review $ARGUMENTS for correctness.","agents":["claude","opencode","pi"]}
```

Use `save --old <name> --revision <revision>` to edit or rename an entry and `remove <name> --revision <revision>` to withdraw it. `list` returns current revisions and per-file publication status. Existing files and external edits are preserved. `sync` retries publication.

Choose **Find existing files** to scan the supported client directories. You can adopt an unchanged native Markdown or TOML file in place, including OpenCode's legacy `command/` and `agent/` directories and Qwen's TOML commands. In the CLI, `discover` returns each file's client ID, path and revision. Use `adopt <client> <path> --revision <revision>` to manage that file. Adoption keeps the original bytes until you edit the resource. Withdrawing an assignment can remove its unchanged file.

In the user subagent library, select **Import subagents** to scan a Markdown file, a local folder, or a Git repository. Select the definitions to copy and choose any client assignments for the new entries. Scanning the same source again shows updates, preserves locally edited descriptions and prompts, and keeps the saved client assignments. Import copies the name, description and prompt. Set client-specific options separately in the editor.

Use `carbforge-cli subagents probe-source <path-or-url>` to get the relative `path` and `revision` for each candidate. Pass those pairs in a UTF-8 JSON request to `carbforge-cli subagents import-source` on standard input. The request has a `source` string, a `files` array of `{ "path": "…", "revision": "…" }` objects, and an optional `agents` array of client IDs for new entries.

The CLI reads the exact UTF-8 value from standard input with `environment save <name> --agent <claude|codex|pi>`. Repeat `--agent` to assign several clients. For edits, pass the revision from `environment list`. Add `--rename-from <old-name>` to rename, use `environment remove <name> --revision <revision>` to remove, and `environment sync` to retry publication. Use `environment list --show-values` to reveal values.

For a supported client, open **Project MCP** under **Client connections** and select the project folder. Add a server using the client's native JSON format or a TOML table body for Codex. The editor shows the file it will update and checks its revision before saving or removing an entry. The client loads and runs the server when it opens the project.

```powershell
carbforge-cli project-mcp list . gemini
carbforge-cli project-mcp save . gemini diagnostics --from .\server.json
carbforge-cli project-mcp remove . gemini diagnostics --revision '<revision from list>'
```

To edit an existing entry with the CLI, pass `--replace --revision <revision>` to `project-mcp save`.

Open **MCP library** from a loaded project to save a reusable server and assign it to the selected client. A library definition uses `transport` (`stdio`, `http` or `sse`) with `command`, `args` and `env` for stdio, or `url` and `headers` for a remote server. Crabforge writes each assignment in the client's native format. Sync preserves existing same-name entries and externally edited placements and reports conflicts.

```powershell
carbforge-cli project-mcp library save diagnostics --from .\shared-server.json
carbforge-cli project-mcp library assign diagnostics . gemini --revision '<revision from library list>'
carbforge-cli project-mcp library sync
```

## Protocols and routing

- Clients can use OpenAI Chat Completions, OpenAI Responses, Anthropic Messages, Gemini and Ollama APIs. The validation record describes which streaming, tool, media and authentication paths have been exercised. Model availability depends on the provider and account.
- Groups support ordered, rotating, weighted, manual and nested routes. `carbforge-cli provider fallback primary group/work` configures a fallback for direct requests to `primary`. `carbforge-cli provider set backup fallback-for-others=no` prevents an ordinary provider's failed request from being sent to `backup`.
- The desktop console follows the system locale by default and offers Chinese, English, French, German, Hindi, Japanese, Korean, Portuguese and Spanish.
- Switch between list and grid on any console page, in the routing group member editor, or when choosing an Oh My Pi profile, protocol, role, or model. The app remembers the layout on restart. Switching while editing a group keeps the weight you have entered.

## Reach another gateway

Run `carbforge-cli peer-id` on the client. Add the resulting ID to the gateway's trusted peers, configure its P2P listen address and start it. For direct connections, use a reachable IPv4 or IPv6 address and UDP port. For a Circuit Relay v2 reservation, configure a relay reachable from both hosts and set the gateway's listen address to `/ip6/RELAY_IP/udp/PORT/quic-v1/p2p/RELAY_ID/p2p-circuit` with the relay's actual address and identity.

Pass the full address advertised by the gateway to `carbforge-cli tunnel <address>` on the client. If you have both addresses, use `carbforge-cli tunnel <direct-address> --relay-fallback <circuit-address>`. Both addresses must end with the same gateway Peer ID. The console accepts the same optional relay address. The tunnel listens on `http://127.0.0.1:3426/v1` by default and uses the remote gateway key. Local tests cover bidirectional QUIC with Kubo, IPv6 relay fallback and a DCUtR upgrade followed by an authenticated request after the relay went offline. Public IPv6 and hole punching across independent NATs still need cross-host verification.

## Build for Windows x64

To run the Windows x64 ZIP, extract the full archive and launch `desktop\crabforge.exe`. The two CLI executables are at the archive root. The target computer requires the x64 Microsoft Visual C++ v14 Redistributable. [Microsoft runtime download](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170)

GitHub Actions builds the Windows x64 desktop and both CLI executables. Each successful run uploads `crabforge-windows-x64-ci.zip` as an unsigned CI artifact. The signed release workflow builds from the release commit and publishes the package after Authenticode verification. [Windows builds](https://github.com/blueokanna/Crabforge/actions/workflows/windows-build.yml) · [Release guide](docs/windows-release.en.md) · [Build and desktop validation](docs/validation/cf-windows-469e877/README.md)

Install the Rust MSVC toolchain, Flutter and the Visual Studio Desktop development with C++ workload. The recorded Windows builds used Rust 1.99.1, Flutter 3.47.9 and Dart 3.13.7.

```powershell
cargo build --release --locked -p crabforge-cli
cd gui
flutter pub get
flutter build windows --release
```

The CLI executables are in `target/release`. Distribute the entire `gui/build/windows/x64/runner/Release` directory, including `crabforge.exe`, the Flutter runtime, the file selector plugin, the Rust bridge and `data`. The Magpie port is in progress. Measurements and compatibility results are tied to the specific snapshots listed in the validation record.
