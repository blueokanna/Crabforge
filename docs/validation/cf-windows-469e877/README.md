# Windows x64 构建与桌面外观验收 / Windows x64 build and appearance validation

## GitHub Actions

- 源码 / Source `469e877b68481db90180ef46e78273cea15874b5`
- Windows x64 runner、Rust `1.99.1`、Flutter `3.47.9`、Dart `3.13.7`
- 锁定依赖检查、Rust Clippy、125 项 Rust 测试和 4 项 Flutter 测试通过。需要外部夹具的 33 项 Rust 测试按默认设置跳过。Flutter 分析、Release 桌面与两个 Release CLI 编译及打包通过。
- Locked dependency checks, Rust Clippy, 125 Rust tests, and four Flutter tests passed. The default run skipped 33 Rust tests that require external fixtures. Flutter analysis, the Release desktop build, both Release CLI builds, and packaging passed.
- CI ZIP `crabforge-windows-x64-ci.zip` · 65,001,202 bytes · SHA-256 `573D2F7AE0772C96AEE48C767FEF515F7C70DC98FC1E9DB93BB296018851333C` · [Run 45946299641](https://github.com/blueokanna/Crabforge/actions/runs/45946299641) · [Artifact 15175193310](https://github.com/blueokanna/Crabforge/actions/runs/45946299641/artifacts/15175193310)
- 下载件的哈希与 GitHub API 一致。解压后两个 CLI 都报告 `0.1.0`，输出完全相同的 97 项预设。六个 PE 文件的 Authenticode 状态为 `NotSigned`。
- The downloaded artifact matched GitHub's digest. Both unpacked CLIs reported `0.1.0` and returned the same 97 preset records. All six PE files showed `NotSigned` under Authenticode.

后续两次 Windows x64 构建均通过，下载件的大小和 SHA-256 均与 GitHub 一致。两个包各含 311 个 Rust 许可目录和补录正文。解压后，两种 CLI 入口都返回相同的 97 项预设。

Both follow-up Windows x64 builds passed. The downloaded ZIP sizes and SHA-256 digests matched GitHub. Each package contains 311 Rust license directories and the supplemental texts. Both unpacked CLI entry points returned the same 97 presets.

| 源码 / Source | ZIP bytes | ZIP SHA-256 | GitHub Actions |
| --- | ---: | --- | --- |
| `ba09164c4a2265462ef736d64722f1b20b085206` | 65,007,925 | `E2499BD9C1C9052497A41BAA8A30C6241F95B5F418403A5E5273A69A267EC423` | [Run 45952941977](https://github.com/blueokanna/Crabforge/actions/runs/45952941977) · [Artifact 15179225625](https://github.com/blueokanna/Crabforge/actions/runs/45952941977/artifacts/15179225625) |
| `99c87e1449f7990fa348646f6724520330be7921` | 65,007,919 | `A414C719C0AD97F66EA7D71DB4BFD52D5BB6939AAEA5D6E402DF6508F8B3A1DA` | [Run 45954001089](https://github.com/blueokanna/Crabforge/actions/runs/45954001089) · [Artifact 15178467899](https://github.com/blueokanna/Crabforge/actions/runs/45954001089/artifacts/15178467899) |

RustSec 使用 `cargo-audit 0.23.0` 和公告库提交 `c9f6c91a44dbad9d71742329e9df061279f9c4be` 扫描该锁文件，发现零项已知漏洞和一项停止维护提示。`paste 1.0.15` 由 `if-watch` 的非 Windows netlink 依赖链引入，在 Windows x64 的全特性启用图中未出现。

RustSec scanned the lockfile with `cargo-audit 0.23.0` and advisory database commit `c9f6c91a44dbad9d71742329e9df061279f9c4be`. The scan reported zero known vulnerabilities and one unmaintained package. `paste 1.0.15` came through the non-Windows netlink chain under `if-watch`. The all-features Windows x64 dependency graph excludes it. [RustSec advisory](https://rustsec.org/advisories/RUSTSEC-2024-0436.html)

锁定依赖的全特性启用图覆盖运行、构建和测试依赖。所有目标共 417 个唯一包版本，Windows x64 为 339 个。两者均未启用 Serde 系列、Hyper 或 Reqwest，且均包含锁定的 Courierust `1.1.3`、NextJSON `0.1.5`、RustBinary `0.1.10`、recurse-x `0.2.6` 和 libp2p `0.57.1`。

The locked, all-features graphs include runtime, build and test dependencies. The audit counted 417 distinct package rows across all targets and 339 for Windows x64. Neither graph enabled packages from Serde, Hyper or Reqwest. Both used the pinned Courierust `1.1.3`, NextJSON `0.1.5`, RustBinary `0.1.10`, recurse-x `0.2.6` and libp2p `0.57.1`.

## 桌面 / Desktop

桌面以两家回环测试供应商和一个路由组启动。Windows 集成测试覆盖 1,200 和 560 逻辑像素宽度、浅色和深色、列表与网格下的六个主页面，共 48 张截图。九种界面语言均走过窄屏导航。测试还读取了原生 DWM 强调色，核对高对比度浅色和深色主题，并在新进程中确认恢复 HarmonyOS Sans SC。截图来自 Windows Debug 集成测试。

The fixture used two loopback providers and one route group. The Windows integration test captured 48 views across six main pages, widths of 1,200 and 560 logical pixels, both brightness settings, and list and grid layouts. All nine locales exercised narrow navigation. The test also read the native DWM accent, checked both high-contrast schemes, and verified that a new process restored HarmonyOS Sans SC. The screenshots come from the Windows Debug integration test.

[Light 1200](light-1200.jpg) · [Light 560](light-560.jpg) · [Dark 1200](dark-1200.jpg) · [Dark 560](dark-560.jpg) · [48 original PNGs](captures.zip) · [PNG checksums](captures.sha256)

GitHub Actions 的 Release 构建中，三套文字字体和产品 PNG 均与原件逐字节一致。包内的 Material Symbols Rounded 裁剪版为 229,580 字节，包含应用引用的 82 个唯一图标码点和 `FILL`、`GRAD`、`opsz`、`wght` 四条可变轴。

The three text fonts and the product PNG in the GitHub build match their source files byte for byte. The bundled Material Symbols Rounded subset is 229,580 bytes and retains all 82 distinct icon codepoints used by the app and the `FILL`, `GRAD`, `opsz`, and `wght` axes. [Font source manifest](../../../assets/fonts/manifest.json)

在解压后的 CI Release 桌面中，Windows Graphics Capture 记录了 24 次导航操作。从点击到首个窗口内容变化，中位数为 62.306 ms，范围为 55.134–89.929 ms。[原始帧记录](window-capture-frames.csv)

Windows Graphics Capture recorded 24 navigations in the unpacked CI Release desktop. The median time from click to first window content change was 62.306 ms, with a range of 55.134–89.929 ms. [Raw frame samples](window-capture-frames.csv)

独立的控制台集成测试通过了九语菜单顺序、供应商编辑、分组路由、通过解压 CLI 发送请求以及冷启动恢复。

A separate console integration run passed the nine-language menu ordering, provider editing, route-group changes, requests through the unpacked CLI, and cold-start restoration.

后续 Windows Debug 测试在独立配置下通过了项目 MCP、共享插件库、子代理和斜杠命令。规则与技能用例也通过了源码更新后的复测。技能用例确认删除库条目后，保留的原生副本仍可在列表与网格中查看，外部编辑的二进制附件逐字节保留。

Follow-up Windows Debug tests passed for project MCP, shared plugin bundles, subagents, and slash commands in isolated profiles. The rule and skill cases passed after the layout update. The skill case verified that both layouts show the retained native copy after the library entry is removed, and that the externally edited binary attachment is preserved byte for byte.

环境变量、Hooks 和 Claude Code 原生插件管理用例也通过。会话用例的早期试跑在切换 Codex 筛选时失败。调整选择器，使其点击菜单中可命中的选项，并用独立的 Windows 用户主目录和客户端配置复测后，用例通过。它验证了 Codex 网格筛选、回环模型提炼、候选记忆显式保存，以及发布和逐字节恢复用户指令。

The environment variable, Hooks and native Claude Code plugin cases also passed. Earlier session attempts failed when switching to the Codex filter. The successful rerun targeted the hit-testable menu option and used an isolated Windows user home and client profiles. It verified the Codex grid filter, distillation through a loopback model, explicit saving of the memory candidate, and publication and byte-for-byte recovery of user instructions. [Session test](../../../gui/integration_test/agent_resources_test.dart)

项目 MCP 复测在移除库条目前手动编辑了原生服务器配置。移除后，列表与网格均显示保留条目，项目配置与手动编辑后的字节完全一致。

The project MCP rerun edited a native server configuration before removing its library entry. Both layouts showed the retained entry, and the project configuration matched the manually edited bytes. [Project MCP test](../../../gui/integration_test/project_mcp_test.dart)

## 内存、流式请求和 P2P / Memory, streaming, and P2P

使用解压的 CI 产物，每个内存场景测三轮。下表为工作集／私有字节中位数，单位为 MiB。Magpie 的桌面结果包括 WebView2 子进程。

The unpacked CI artifact ran three rounds per memory workload. The table reports median working set / private bytes in MiB. Magpie's desktop totals include its WebView2 child processes.

| 场景 / Workload | Crabforge | Magpie |
| --- | ---: | ---: |
| 桌面空闲 / Idle desktop | 134.68 / 196.41 | 488.02 / 547.03 |
| 空目录网关 / Empty catalog gateway | 8.17 / 1.67 | 61.19 / 95.37 |
| 本地对话网关 / Loopback chat gateway | 9.55 / 2.10 | 69.23 / 98.25 |

[Desktop samples](desktop-comparison.csv) · [Catalog samples](gateway-catalog-memory.csv) · [Chat samples](gateway-chat-memory.csv)

回环 SSE 夹具在首事件和结束事件之间等候约 700 ms。三轮首事件分别为 11、5、17 ms，结束分别为 730、719、720 ms。同一主机的 IPv6 回环、已分配的全局 IPv6 地址及 QUIC 直连失败后经 Kubo 0.43.1 Circuit Relay v2 回退均得到未授权 401、授权 200。跨公网主机及不同 NAT 拓扑尚待实测。

The loopback SSE fixture delayed the final event by approximately 700 ms. First events arrived at 11, 5, and 17 ms. The streams ended at 730, 719, and 720 ms. Same-host tests over IPv6 loopback, an assigned global IPv6 address, and Kubo 0.43.1 Circuit Relay v2 after a failed direct QUIC dial all returned 401 without authorization and 200 with authorization. Tests across public hosts and separate NATs remain outstanding. [Streaming samples](forward-h1.csv) · [IPv6 samples](ipv6.csv)

## 签名与许可 / Signing and licenses

本地隔离测试使用临时自签名证书。SignTool 添加了 RFC 3161 时间戳，随后因证书链不受信任拒绝该文件。签名脚本清除了临时密钥，发布目录未创建。正式发布需要受信任的发布者密钥。

An isolated local test used a temporary self-signed certificate. SignTool applied an RFC 3161 timestamp and then rejected the binary because the certificate chain was untrusted. The signing script removed the temporary private key and created no publishing directory. A trusted publisher credential is required for the public release. [Signing guide](../../windows-release.en.md)

GitHub 发布试跑以 `ba09164c4a2265462ef736d64722f1b20b085206` 为源。`identify` 和 `build` 成功，`sign` 下载并校验 CI 包后报告三项签名设置缺失，`publish` 跳过。仓库 Release 列表为零条。

The GitHub release trial used source `ba09164c4a2265462ef736d64722f1b20b085206`. `identify` and `build` passed. `sign` downloaded and verified the CI archive, then reported that the three signing settings were missing. `publish` was skipped, and the repository had zero release entries. [Release trial 45952988583](https://github.com/blueokanna/Crabforge/actions/runs/45952988583) · [Signing job 140361568621](https://github.com/blueokanna/Crabforge/actions/runs/45952988583/job/140361568621)

CI 归档中有 311 个 Rust 许可目录。后续打包脚本为 28 个 crate 加入对应上游提交的许可正文。`base256emoji`、`base45`、`futures-bounded`、`if-watch`、`match-lookup`、`no-proxy` 和 `yaml-edit` 在对应提交的包目录至仓库根目录中只有 Cargo 许可声明可供归档。

The CI artifact has 311 Rust license directories. The subsequent packaging update adds license texts from the corresponding publication commits for 28 crates. For `base256emoji`, `base45`, `futures-bounded`, `if-watch`, `match-lookup`, `no-proxy`, and `yaml-edit`, the audit retained the Cargo license declaration after checking the package directory and its ancestors at the publication commit. [Supplemental license texts and source revisions](../../../licenses/rust-upstream.txt)
