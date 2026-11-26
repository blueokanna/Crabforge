# Crabforge 0.1.0 Windows validation

[简体中文](crabforge-validation.md)

## Windows x64 project memory and local sessions snapshot `85fd8fa`

The ZIP built offline from `85fd8fa556eb2448ed702ebbb59b0b38a4e426f8` is 60,726,090 bytes with SHA-256 `42A5B7ED546586099C3708B4F184F8D13D96A8B59E85651D2C911B5FB20EEE45`. Validation covered 299 inputs, 666 unpacked files, 126 app assets, 308 Rust dependency notices and the same 97 presets from both CLI names. All 118 serial Rust tests passed, along with all-target Clippy, Flutter analysis and both widget tests. All 28 enabled dependency trees across Windows x64 and all Cargo targets were clear of Serde, Hyper and Reqwest. Flutter assemble stalled on the first Release build. After stopping the affected subprocess, the same frozen source built successfully. [Inputs](validation/cf-windows-85fd8fa/build-inputs.sha256) · [Archive manifest](validation/cf-windows-85fd8fa/SHA256SUMS) · [Rust tests](validation/cf-windows-85fd8fa/regression.log) · [Dependency audit](validation/cf-windows-85fd8fa/dependency-audit.csv)

The unpacked CLI passed eight memory process checks and verified Git project keys, digest publication with CRLF, stale revisions, republishing after archive and two consecutive restores. The first Gemini CLI 0.71.1 request to a localhost model fixture included the published digest. Its next request carried the stdio MCP `memory_search` result in a `functionResponse`. Claude Code and Codex each ran a localhost model turn. The unpacked CLI read the session IDs, first prompts and working directories from their native JSONL and filtered by agent. [Memory](validation/cf-windows-85fd8fa/memory-process.csv) · [Digest](validation/cf-windows-85fd8fa/digest-publication.csv) · [Gemini MCP](validation/cf-windows-85fd8fa/mcp-client.csv) · [Native sessions](validation/cf-windows-85fd8fa/sessions-process.csv)

| Workload, median of three samples | Crabforge working set / private bytes (MiB) | Magpie working set / private bytes (MiB) |
| --- | ---: | ---: |
| Empty catalog gateway | 7.62 / 1.50 | 58.60 / 92.81 |
| Chat gateway | 8.92 / 1.90 | 71.59 / 100.41 |
| Desktop main process | 151.39 / 225.62 | 78.84 / 95.56 |
| Desktop process tree | 151.39 / 225.62 | 498.73 / 421.03 |

Crabforge started each desktop round from the unpacked Release executable with a fresh data directory. Magpie reused its visible window and seven WebView2 children. The Chat gateways used the same localhost fixture, with two model IDs from Crabforge and one from Magpie. [Catalog](validation/cf-windows-85fd8fa/catalog.csv) · [Chat](validation/cf-windows-85fd8fa/chat.csv) · [Desktop](validation/cf-windows-85fd8fa/desktop-comparison.csv)

The first complete loopback SSE event arrived in 5, 29 and 15 ms. The final event arrived in 713, 740 and 724 ms with a fixed 700 ms upstream delay. IPv6 loopback, a global IPv6 address on the same host and Kubo fallback after a failed direct dial returned 401 for invalid keys and 200 with the expected catalog for valid keys. Cross-host public and different-NAT samples are pending. [SSE](validation/cf-windows-85fd8fa/forward-h1.csv) · [IPv6](validation/cf-windows-85fd8fa/ipv6.csv)

Windows Graphics Capture recorded 297 pixel region changes over 24 transitions in the unpacked Release desktop. The median click-to-first-change time was 41.88 ms. Within a transition, the median interval between changes was 16.67 ms, P90 was 16.68 ms and the maximum was 33.34 ms. GPU execution and physical display output still need direct measurement. [Window capture](validation/cf-windows-85fd8fa/window-capture-frames.csv)

Windows Debug integration tests from frozen source passed configuration, gateway routing, route history, an independent restart, Oh My Pi settings and restart, and wide and narrow layouts. Profile recorded 580 frames. UI build took 1.50 ms at P90 and 6.68 ms at P99. Total span was 2.96 ms at P90 and 7.58 ms at P99. Raster timings were 0–1 µs. [Console](validation/cf-windows-85fd8fa/console-0.log) · [Restart](validation/cf-windows-85fd8fa/console-1.log) · [Oh My Pi](validation/cf-windows-85fd8fa/omp_settings-0.log) · [Oh My Pi restart](validation/cf-windows-85fd8fa/omp_settings-1.log) · [Layout](validation/cf-windows-85fd8fa/console-layout.log) · [Profile frames](validation/cf-windows-85fd8fa/profile.json)

## Windows x64 memory snapshot `c8eb96e`

The ZIP was built offline from `c8eb96e79048add9d1f94680cc5fd44c0d9f372b`. It is 60,611,407 bytes with SHA-256 `4E1CD367471290B5617351A3C1A41DDFF7FDB22773A0484C3C749679C4899AA1`. The build verified 296 inputs, 666 files after extraction, 126 app assets, 308 Rust dependency notices, and the same 97 presets from both CLI names. The serial Rust suite passed 115 tests, including ignored tests. Clippy, Flutter analysis and two widget tests passed. All 28 enabled dependency trees across Windows and all Cargo targets were clear of Serde, Hyper and Reqwest. The full commit was verified on remote `main` after pushing through the configured proxy. [Inputs](validation/cf-windows-c8eb96e/build-inputs.sha256) · [Archive manifest](validation/cf-windows-c8eb96e/SHA256SUMS) · [Rust tests](validation/cf-windows-c8eb96e/regression.log) · [Dependency audit](validation/cf-windows-c8eb96e/dependency-audit.csv)

Eight process checks with the unpacked CLI covered Markdown creation and search, stale revisions, and MCP initialization, tools, deduplication, scoping and archive. Gemini CLI 0.71.1 invoked `memory_search` over stdio and sent the returned note as a `functionResponse` in its next request to a localhost model fixture. The unpacked CLI also repeated custom instruction checks. Qoder and Qoder CN 1.1.99 launched with isolated configurations and stopped before the first model request because `/login` was required. [Process results](validation/cf-windows-c8eb96e/memory-process.csv) · [Gemini MCP](validation/cf-windows-c8eb96e/mcp-client.csv) · [Custom context](validation/cf-windows-c8eb96e/custom-context.csv)

The Windows Debug console and Oh My Pi editor each passed a first launch and a separate-process restart using the unpacked Release CLI. The separate layout run passed at both window widths and covered memory editing and switching between list and grid. [Console](validation/cf-windows-c8eb96e/console-0.log) · [Console restart](validation/cf-windows-c8eb96e/console-1.log) · [Oh My Pi](validation/cf-windows-c8eb96e/omp_settings-0.log) · [Oh My Pi restart](validation/cf-windows-c8eb96e/omp_settings-1.log) · [Layout](validation/cf-windows-c8eb96e/console-layout.log)

| Workload, median of three samples | Crabforge working set / private bytes (MiB) | Magpie working set / private bytes (MiB) |
| --- | ---: | ---: |
| Empty catalog gateway | 7.60 / 1.41 | 60.75 / 93.76 |
| Chat gateway | 8.88 / 1.84 | 69.30 / 97.93 |
| Desktop main process | 155.12 / 229.17 | 82.91 / 98.97 |
| Desktop process tree | 155.12 / 229.17 | 455.67 / 431.39 |

Crabforge started each desktop round from the unpacked Release executable with a fresh data directory. Magpie reused its visible window and seven WebView2 children. Both Chat gateways used the same local fixture, with two model IDs from Crabforge and one from Magpie. The three loopback SSE first events arrived in 15, 19 and 2 ms. Final events arrived in 724, 728 and 704 ms with a 700 ms upstream delay. The unpacked CLI passed IPv6 loopback, same-host global IPv6 and Kubo fallback, returning 401 for invalid keys and 200 with the expected catalog for valid ones. Cross-host samples are pending. [Catalog](validation/cf-windows-c8eb96e/catalog.csv) · [Chat](validation/cf-windows-c8eb96e/chat.csv) · [Desktop](validation/cf-windows-c8eb96e/desktop-comparison.csv) · [SSE](validation/cf-windows-c8eb96e/forward-h1.csv) · [IPv6](validation/cf-windows-c8eb96e/ipv6.csv)

## Windows x64 snapshot `9aeb7e5`

The Windows ZIP was built offline from `9aeb7e58423929f0eabeac2b1be4f49fde085ccf`. It is 60,230,597 bytes with SHA-256 `AB20EB4DEB65646DBBD174F8631B4FA9DFBDE4D266095DA7E5B59163F15318C9`. All 292 build inputs matched before and after the build. The extracted archive contained 666 hash-verified files and 126 verified application assets, including 308 Rust dependency notices. Both CLI names returned the same 97 provider presets. The serial Rust suite passed 113 tests, including normally ignored tests. Clippy, Flutter analysis and both widget tests passed. All 28 enabled dependency trees across Windows x64 and all Cargo targets were clear of Serde, Hyper and Reqwest. [Inputs](validation/cf-windows-9aeb7e5/build-inputs.sha256) · [Archive manifest](validation/cf-windows-9aeb7e5/SHA256SUMS) · [Rust tests](validation/cf-windows-9aeb7e5/regression.log) · [Dependency audit](validation/cf-windows-9aeb7e5/dependency-audit.csv)

| Workload | Crabforge working set / private bytes (MiB) | Magpie working set / private bytes (MiB) |
| --- | ---: | ---: |
| Empty catalog gateway | 7.53 / 1.42 | 57.33 / 91.20 |
| Chat gateway | 8.85 / 1.86 | 68.91 / 97.56 |
| Desktop main process | 154.39 / 227.91 | 81.20 / 96.48 |
| Desktop process tree | 154.39 / 227.91 | 453.75 / 429.78 |

These are medians of three measurements. Crabforge started each desktop round from the unpacked Release executable with a fresh data directory. Magpie reused its visible window and seven WebView2 children. The Chat gateways used the same localhost fixture. Crabforge published two model IDs and Magpie published one. [Catalog samples](validation/cf-windows-9aeb7e5/catalog.csv) · [Chat samples](validation/cf-windows-9aeb7e5/chat.csv) · [Desktop samples](validation/cf-windows-9aeb7e5/desktop-comparison.csv)

The unpacked CLI returned 401 with an invalid key and 200 with the expected catalog using IPv6 loopback, direct QUIC to the local machine's assigned global IPv6 address, and fallback through Kubo 0.43.1 after a failed direct connection. In a separate localhost SSE fixture the upstream delayed its final event by 700 ms. The first complete events arrived in 6, 2 and 2 ms, and the final events in 719, 710 and 713 ms. Cross-host public IPv6 and direct, hole-punched and relayed paths across independent NATs still require a second host. [IPv6 samples](validation/cf-windows-9aeb7e5/ipv6.csv) · [SSE samples](validation/cf-windows-9aeb7e5/forward-h1.csv)

Gemini CLI, Qwen Code, Qoder and Qoder CN passed custom `context.fileName` selection, BOM import, CRLF preservation, stale revision rejection and restore through the unpacked CLI. Gemini CLI 0.71.1 and Qwen Code 0.28.4 sent the custom file content to a localhost model fixture. Gemini also sent the default `GEMINI.md` content. Native Qoder and Qoder CN client behavior remains unverified. [Client results](validation/cf-windows-9aeb7e5/custom-context.csv)

The Windows debug console and Oh My Pi editor each passed first use and a separate-process restart using the unpacked Release CLI. The initial console run encountered a tunnel button outside the test viewport. The same frozen production source passed with a fixed test viewport and explicit button targeting. [Initial run](validation/cf-windows-9aeb7e5/console-0.log) · [Console](validation/cf-windows-9aeb7e5/console-retry-0.log) · [Console restart](validation/cf-windows-9aeb7e5/console-retry-1.log) · [Oh My Pi](validation/cf-windows-9aeb7e5/omp_settings-retry-0.log) · [Oh My Pi restart](validation/cf-windows-9aeb7e5/omp_settings-retry-1.log)

The frozen Windows layout test passed at both window widths. Windows Graphics Capture sampled the unpacked Release window across 24 transitions at the wide layout. The sampled pixel region changed in 288 captures. Median time from posting the click to the first changed capture was 45.59 ms. Intervals between changed captures within one transition had a median of 16.67 ms, P90 of 16.68 ms, and a maximum of 33.35 ms. [Layout test](validation/cf-windows-9aeb7e5/console-layout.log) · [Capture timestamps](validation/cf-windows-9aeb7e5/window-capture-frames.csv)

The separate Windows Profile run recorded 580 Flutter frames over four navigation rounds. UI build time was 1.58 ms at P90 and 8.45 ms at P99. Total span was 3.31 ms at P90 and 10.32 ms at P99, with one frame above 16.67 ms. PresentMon ETW access and DXGI output duplication were unavailable on this host. Per-frame GPU execution and display scanout remain unverified. [Profile data](validation/cf-windows-a4a9e40/profile-spans.json)
