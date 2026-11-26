# Crabforge 0.1.0 Windows validation

[简体中文](crabforge-validation.md)

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

The earlier Windows Profile run recorded 580 Flutter frames over four navigation rounds. UI build time was 1.58 ms at P90 and 8.45 ms at P99. Total span was 3.31 ms at P90 and 10.32 ms at P99, with one frame above 16.67 ms. GPU presentation timing remains unverified. [Profile data](validation/cf-windows-a4a9e40/profile-spans.json)
