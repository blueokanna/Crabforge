# Windows x64 构建与桌面外观验收 / Windows x64 build and appearance validation

## GitHub Actions

- 源码 / Source `469e877b68481db90180ef46e78273cea15874b5`
- Windows x64 runner、Rust `1.99.1`、Flutter `3.47.9`、Dart `3.13.7`
- 锁定依赖检查、Rust Clippy、125 项 Rust 测试和 4 项 Flutter 测试通过。需要外部夹具的 33 项 Rust 测试按默认设置跳过。Flutter 分析、Release 桌面与两个 Release CLI 编译及打包通过。
- Locked dependency checks, Rust Clippy, 125 Rust tests, and four Flutter tests passed. The default run skipped 33 Rust tests that require external fixtures. Flutter analysis, the Release desktop build, both Release CLI builds, and packaging passed.
- CI ZIP `crabforge-windows-x64-ci.zip` · 65,001,202 bytes · SHA-256 `573D2F7AE0772C96AEE48C767FEF515F7C70DC98FC1E9DB93BB296018851333C` · [Run 45946299641](https://github.com/blueokanna/Crabforge/actions/runs/45946299641) · [Artifact 15175193310](https://github.com/blueokanna/Crabforge/actions/runs/45946299641/artifacts/15175193310)
- 下载件的哈希与 GitHub API 一致。解压后两个 CLI 都报告 `0.1.0`，输出完全相同的 97 项预设。六个 PE 文件的 Authenticode 状态为 `NotSigned`。
- The downloaded artifact matched GitHub's digest. Both unpacked CLIs reported `0.1.0` and returned the same 97 preset records. All six PE files showed `NotSigned` under Authenticode.

下表的 Windows x64 构建均通过，下载件的大小和 SHA-256 均与 GitHub 一致。每个包均含 311 个 Rust 许可目录和补录正文。解压后，两种 CLI 入口都返回相同的 97 项预设。

The Windows x64 builds below passed. The downloaded ZIP sizes and SHA-256 digests matched GitHub. Each package contains 311 Rust license directories and the supplemental texts. Both unpacked CLI entry points returned the same 97 presets.

| 源码 / Source | ZIP bytes | ZIP SHA-256 | GitHub Actions |
| --- | ---: | --- | --- |
| `ba09164c4a2265462ef736d64722f1b20b085206` | 65,007,925 | `E2499BD9C1C9052497A41BAA8A30C6241F95B5F418403A5E5273A69A267EC423` | [Run 45952941977](https://github.com/blueokanna/Crabforge/actions/runs/45952941977) · [Artifact 15179225625](https://github.com/blueokanna/Crabforge/actions/runs/45952941977/artifacts/15179225625) |
| `99c87e1449f7990fa348646f6724520330be7921` | 65,007,919 | `A414C719C0AD97F66EA7D71DB4BFD52D5BB6939AAEA5D6E402DF6508F8B3A1DA` | [Run 45954001089](https://github.com/blueokanna/Crabforge/actions/runs/45954001089) · [Artifact 15178467899](https://github.com/blueokanna/Crabforge/actions/runs/45954001089/artifacts/15178467899) |
| `26acbe6ac39bd140580d74762c4182ce64a21f46` | 65,008,011 | `0582F62878FF1DF612B206B7188E6B344B3DB6F6E20966ADE9F8A0D688B1C25C` | [Run 45955859685](https://github.com/blueokanna/Crabforge/actions/runs/45955859685) · [Artifact 15179309529](https://github.com/blueokanna/Crabforge/actions/runs/45955859685/artifacts/15179309529) |
| `6191cfd264c031e5cd4ed01c48b9a77ccc5560bf` | 65,009,120 | `A3E090CFA94A8841CEE7B35146F7DCEC4CAAAB0984474C1BEB5ECE94E2BE59AF` | [Run 45958239454](https://github.com/blueokanna/Crabforge/actions/runs/45958239454) · [Artifact 15180043889](https://github.com/blueokanna/Crabforge/actions/runs/45958239454/artifacts/15180043889) |
| `17cae79c1b4b8e2d7686340e6db6c71ff9ebb7aa` | 65,009,118 | `9420794F712625C1EC371F56A533490700E4C2DB2E5B2E3A26F9752F96351182` | [Run 45961563177](https://github.com/blueokanna/Crabforge/actions/runs/45961563177) · [Artifact 15181248050](https://github.com/blueokanna/Crabforge/actions/runs/45961563177/artifacts/15181248050) |

`26acbe6`、`6191cfd` 和 `17cae79` 的每个包均含 668 个文件，与解压后的 SHA-256 逐项一致。两个 CLI 均报告 `0.1.0`，六个 Windows PE 的 Authenticode 状态均为 `NotSigned`。产品 PNG、三套文字字体和图标字体通过字节核对。Rust 补录许可正文的 SHA-256 为 `4194156F90E6854E38C54445E280FE6380571EFE254F740493C1A8D4C363521E`。

All 668 archive entries in each package from `26acbe6`, `6191cfd` and `17cae79` matched their extracted file hashes. Both CLI executables reported `0.1.0`, and all six Windows PE files reported `NotSigned` under Authenticode. The product PNG, three text fonts and icon font passed byte verification. The supplemental Rust license texts had SHA-256 `4194156F90E6854E38C54445E280FE6380571EFE254F740493C1A8D4C363521E`.

RustSec 使用 `cargo-audit 0.23.0` 和公告库提交 `c9f6c91a44dbad9d71742329e9df061279f9c4be` 扫描该锁文件，发现零项已知漏洞和一项停止维护提示。`paste 1.0.15` 由 `if-watch` 的非 Windows netlink 依赖链引入，在 Windows x64 的全特性启用图中未出现。

RustSec scanned the lockfile with `cargo-audit 0.23.0` and advisory database commit `c9f6c91a44dbad9d71742329e9df061279f9c4be`. The scan reported zero known vulnerabilities and one unmaintained package. `paste 1.0.15` came through the non-Windows netlink chain under `if-watch`. The all-features Windows x64 dependency graph excludes it. [RustSec advisory](https://rustsec.org/advisories/RUSTSEC-2024-0436.html)

锁定依赖的全特性启用图覆盖运行、构建和测试依赖。所有目标共 417 个唯一包版本，Windows x64 为 339 个。两者均未启用 Serde 系列、Hyper 或 Reqwest，且均包含锁定的 Courierust `1.1.3`、NextJSON `0.1.5`、RustBinary `0.1.10`、recurse-x `0.2.6` 和 libp2p `0.57.1`。

The locked, all-features graphs include runtime, build and test dependencies. The audit counted 417 distinct package rows across all targets and 339 for Windows x64. Neither graph enabled packages from Serde, Hyper or Reqwest. Both used the pinned Courierust `1.1.3`, NextJSON `0.1.5`, RustBinary `0.1.10`, recurse-x `0.2.6` and libp2p `0.57.1`.

供应商备份导入增加 `rpassword 7.5.5` 和 `rtoolbox 0.0.6`，两者均随包提供 Apache-2.0 许可正文。依赖图重查为全目标 419 个、Windows x64 341 个，五个指定版本一致，禁用依赖为零。RustSec 公告库 `8b75c83c3c9142dd18e93d75b8d895a13d285861` 的扫描结果仍为零项已知漏洞及上述 `paste` 维护提示。

The provider backup import adds `rpassword 7.5.5` and `rtoolbox 0.0.6`. Both registry packages include their Apache-2.0 license text. The updated dependency graphs count 419 packages across targets and 341 on Windows x64, retain all five required versions and enable no disallowed packages. RustSec database revision `8b75c83c3c9142dd18e93d75b8d895a13d285861` reported zero known vulnerabilities and the same `paste` maintenance warning.

## 桌面 / Desktop

桌面以两家回环测试供应商和一个路由组启动。Windows 集成测试覆盖 1,200 和 560 逻辑像素宽度、浅色和深色、列表与网格下的六个主页面，共 48 张截图。九种界面语言均走过窄屏导航。测试还读取了原生 DWM 强调色，核对高对比度浅色和深色主题，并在新进程中确认恢复 HarmonyOS Sans SC。截图来自 Windows Debug 集成测试。

The fixture used two loopback providers and one route group. The Windows integration test captured 48 views across six main pages, widths of 1,200 and 560 logical pixels, both brightness settings, and list and grid layouts. All nine locales exercised narrow navigation. The test also read the native DWM accent, checked both high-contrast schemes, and verified that a new process restored HarmonyOS Sans SC. The screenshots come from the Windows Debug integration test.

[Light 1200](light-1200.jpg) · [Light 560](light-560.jpg) · [Dark 1200](dark-1200.jpg) · [Dark 560](dark-560.jpg) · [48 original PNGs](captures.zip) · [PNG checksums](captures.sha256)

GitHub Actions 的 Release 构建中，三套文字字体和产品 PNG 均与原件逐字节一致。两套主字体均覆盖英语译文用到的 47 个拉丁字母，简体中文译文中的 388 个汉字均由 HarmonyOS Sans SC 提供。包内的 Material Symbols Rounded 裁剪版为 229,580 字节，包含应用引用的 82 个唯一图标码点和 `FILL`、`GRAD`、`opsz`、`wght` 四条可变轴。

The three text fonts and the product PNG in the GitHub build match their source files byte for byte. Both primary fonts cover the 47 Latin letters used in the English locale. HarmonyOS Sans SC supplies all 388 Han characters in the Simplified Chinese locale. The bundled Material Symbols Rounded subset is 229,580 bytes and retains all 82 distinct icon codepoints used by the app and the `FILL`, `GRAD`, `opsz`, and `wght` axes. [Font source manifest](../../../assets/fonts/manifest.json)

在解压后的 CI Release 桌面中，Windows Graphics Capture 记录了 24 次导航操作。从点击到首个窗口内容变化，中位数为 62.306 ms，范围为 55.134–89.929 ms。[原始帧记录](window-capture-frames.csv)

Windows Graphics Capture recorded 24 navigations in the unpacked CI Release desktop. The median time from click to first window content change was 62.306 ms, with a range of 55.134–89.929 ms. [Raw frame samples](window-capture-frames.csv)

独立的控制台集成测试通过了九语菜单顺序、供应商编辑、分组路由、通过解压 CLI 发送请求以及冷启动恢复。

A separate console integration run passed the nine-language menu ordering, provider editing, route-group changes, requests through the unpacked CLI, and cold-start restoration.

`17cae79` 的 Windows Debug 复测通过 11 轮隔离进程。控制台测试使用 `6191cfd` 的解压 CLI 发请求，另启进程检查恢复。其余用例覆盖项目 MCP、Codex 会话、共享插件、环境变量、子代理、斜杠命令、规则、技能、Hooks 和 Claude Code 原生插件。

The Windows Debug rerun on `17cae79` passed 11 isolated invocations. The console case used the unpacked CLI from `6191cfd` and verified recovery in a second process. The other cases covered project MCP, Codex sessions, shared plugins, environment variables, subagents, slash commands, rules, skills, hooks and native Claude Code plugins. [Integration tests](../../../gui/integration_test)

后续 Windows Debug 测试在独立配置下通过了项目 MCP、共享插件库、子代理和斜杠命令。规则与技能用例也通过了源码更新后的复测。技能用例确认删除库条目后，保留的原生副本仍可在列表与网格中查看，外部编辑的二进制附件逐字节保留。

Follow-up Windows Debug tests passed for project MCP, shared plugin bundles, subagents, and slash commands in isolated profiles. The rule and skill cases passed after the layout update. The skill case verified that both layouts show the retained native copy after the library entry is removed, and that the externally edited binary attachment is preserved byte for byte.

环境变量、Hooks 和 Claude Code 原生插件管理用例也通过。会话用例的早期试跑在切换 Codex 筛选时失败。调整选择器，使其点击菜单中可命中的选项，并用独立的 Windows 用户主目录和客户端配置复测后，用例通过。它验证了 Codex 网格筛选、回环模型提炼、候选记忆显式保存，以及发布和逐字节恢复用户指令。

The environment variable, Hooks and native Claude Code plugin cases also passed. Earlier session attempts failed when switching to the Codex filter. The successful rerun targeted the hit-testable menu option and used an isolated Windows user home and client profiles. It verified the Codex grid filter, distillation through a loopback model, explicit saving of the memory candidate, and publication and byte-for-byte recovery of user instructions. [Session test](../../../gui/integration_test/agent_resources_test.dart)

项目 MCP 复测在移除库条目前手动编辑了原生服务器配置。移除后，列表与网格均显示保留条目，项目配置与手动编辑后的字节完全一致。

The project MCP rerun edited a native server configuration before removing its library entry. Both layouts showed the retained entry, and the project configuration matched the manually edited bytes. [Project MCP test](../../../gui/integration_test/project_mcp_test.dart)

Oh My Pi 19.2.1 配合 Bun 1.4.5，在独立 `work` 配置中保存了 Advisor、思考级别、模型角色和协议。冷启动首轮在 MSBuild 退出，重跑一度停在桌面测试中。将测试窗口设为 1,280 × 900 逻辑像素并限定动画等待后，在新的隔离配置上连续跑通首次设置和独立冷启动。新进程读回了网格布局及保存值，也通过了角色和协议恢复检查。

Oh My Pi 19.2.1 with Bun 1.4.5 saved advisor, thinking, role model and protocol settings in an isolated `work` profile. The first cold-start attempt stopped in MSBuild and the next timed out during the desktop test. With a 1,280 × 900 logical-pixel test viewport and bounded animation waits, the first-use and cold-start tests both passed on a fresh profile. The new process restored the grid and saved values, and passed the role and protocol reset checks. [Oh My Pi test](../../../gui/integration_test/omp_settings_test.dart)

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

`6191cfd` 在同一台 Windows 主机上通过了四项 P2P 回归。Kubo 0.43.1 与 Rust 节点双向传输鉴权请求，两项 IPv6 中继测试覆盖预约和直连失败后的回退，DCUtR 测试停止中继后仍能经直连发起新请求。

`6191cfd` passed four P2P regression cases on one Windows host. Kubo 0.43.1 and the Rust peer exchanged authenticated requests in both directions. Two IPv6 relay cases verified reservation and fallback after a failed direct dial. The DCUtR case sent a new request over the upgraded direct connection after shutting down the relay. [Kubo test](../../../crates/core/tests/kubo.rs) · [Relay tests](../../../crates/core/tests/relay.rs) · [DCUtR test](../../../crates/core/src/peer/tests.rs)

同一源码还通过了四项外部夹具测试。llama.cpp `b12722` 加载 `SmolLM2-135M-Instruct-Q4_K_M.gguf`，六个协议入口均取得实际模型输出，Ollama Python SDK `0.6.3` 成功列出模型并读取流式 token。Magpie `0.1.4936` 使用隔离配置通过两项 QUIC 测试，分别转发固定上游回复和本地模型推理结果。

The same source passed four additional fixture tests. llama.cpp `b12722` loaded `SmolLM2-135M-Instruct-Q4_K_M.gguf`, and six protocol entry points returned model output. The official Ollama Python SDK `0.6.3` listed models and read streamed tokens. Two QUIC tests with Magpie `0.1.4936` in an isolated profile passed, one forwarding a fixed upstream response and one generating output from local weights. [llama.cpp tests](../../../crates/core/tests/llama.rs) · [Magpie tests](../../../crates/cli/tests/magpie.rs)

`26acbe6` 的解压 CLI 用独立数据目录连接本机 HTTP 夹具。`models lab --sync` 刷新了 `free:lab/*` 路由组，Magpie 兼容 JSON 导入第二个目录后按其本地目录解析同一规则。嵌入请求在首选上游返回 429 后，经嵌套手选组一次回退取得 200。图像编辑、音频转录和翻译均返回 200。上游核对了改写后的模型 ID、文件名和原始二进制字段。

The unpacked CLI from `26acbe6` used an isolated profile and a loopback HTTP fixture. `models lab --sync` refreshed the `free:lab/*` route group. Importing the Magpie-compatible JSON into a second profile resolved the same rule against its local catalog. An embeddings request received 429 from its primary upstream and then 200 on one fallback through a nested manual group. Image edits, audio transcriptions, and audio translations all returned 200. The upstream fixture checked the rewritten model ID, the filename, and the unchanged binary bytes. [Routing tests](../../../crates/cli/tests/groups.rs) · [Multipart tests](../../../crates/core/tests/media.rs)

Magpie `0.1.4936` 与 `26acbe6` 的解压 CLI 在独立配置间实际往返导入一个加权组。往返生成的 JSON 一致，保留成员顺序、备注和权重。含自动推理级别分类器的 Magpie 导出被明确拒绝，接收方设置逐字节保持原样。整库备份恢复和分类器执行仍待移植。

Magpie `0.1.4936` and the unpacked CLI from `26acbe6` exchanged a weighted group between isolated profiles. The re-exported JSON matched the original and preserved member order, the note and weights. Crabforge rejected a native export with automatic effort classification and left the receiver's settings byte for byte intact. Full backup restore and classifier execution remain migration gaps. [Routing group import](../../../crates/core/src/config/groups/share.rs)

后续 Windows 实测用 Magpie `0.1.4936` 生成了带中文文件名和口令的原生加密备份。两个 CLI 入口都预览了指定供应商，应用后保留排序、旧 ID 和回退。使用旧 ID 发出的回环请求在主供应商返回 429 后仅回退一次并获 200。HTTP 夹具确认密钥来自指定环境变量。错误口令、密文篡改、已关闭供应商及 ID 冲突均被拒绝，目标设置逐字节保持原样。整库恢复仍待移植。[供应商导入说明](../../../README.zh-CN.md#从-magpie-导入供应商)

A subsequent Windows test created a native encrypted backup with Magpie `0.1.4936` using a Chinese filename and passphrase. Both CLI entry points previewed the selected providers. Applying the import preserved order, a former ID and fallback settings. A loopback request using the former ID received a 429 from the primary and a 200 from one fallback. The HTTP fixture confirmed that the API key came from the selected environment variable. Wrong passphrases, modified ciphertext, disabled providers and ID conflicts left the destination settings byte for byte intact. Full backup restoration remains outstanding. [Provider import guide](../../../README.md#import-providers-from-magpie)

同一包的 `mcp memory` 通过独立进程的标准输入输出完成 `2025-11-25` 协议协商、工具枚举、项目隔离和保存去重。第二项目不能归档第一项目的记录。MCP 拒绝改写 CLI 创建的记忆。客户端归档记录后，CLI 以当前修订恢复成功，再用旧修订提交时被拒绝。恢复的记录在新 MCP 进程中可检索。

The same package's `mcp memory` negotiated protocol `2025-11-25` over stdio, listed its tools, isolated projects and deduplicated saves. A process scoped to the second project could not archive a record from the first. MCP rejected a change to a record created by the CLI. After the client archived a record, the CLI restored it with the current revision and rejected a subsequent edit with the old revision. A new MCP process found the restored record. [Memory MCP tests](../../../crates/core/src/memory/mcp.rs)

## 签名与许可 / Signing and licenses

本地隔离测试使用临时自签名证书。SignTool 添加了 RFC 3161 时间戳，随后因证书链不受信任拒绝该文件。签名脚本清除了临时密钥，发布目录未创建。正式发布需要受信任的发布者密钥。

An isolated local test used a temporary self-signed certificate. SignTool applied an RFC 3161 timestamp and then rejected the binary because the certificate chain was untrusted. The signing script removed the temporary private key and created no publishing directory. A trusted publisher credential is required for the public release. [Signing guide](../../windows-release.en.md)

GitHub 发布试跑以 `ba09164c4a2265462ef736d64722f1b20b085206` 为源。`identify` 和 `build` 成功，`sign` 下载并校验 CI 包后报告三项签名设置缺失，`publish` 跳过。仓库 Release 列表为零条。

The GitHub release trial used source `ba09164c4a2265462ef736d64722f1b20b085206`. `identify` and `build` passed. `sign` downloaded and verified the CI archive, then reported that the three signing settings were missing. `publish` was skipped, and the repository had zero release entries. [Release trial 45952988583](https://github.com/blueokanna/Crabforge/actions/runs/45952988583) · [Signing job 140361568621](https://github.com/blueokanna/Crabforge/actions/runs/45952988583/job/140361568621)

双语发布指南的 PowerShell 示例通过了本地验证。有效时间戳签名和 UTF-8 路径可正确核对，空清单、缺失清单、篡改文件及未签名的 CI 二进制均被拒绝。

The bilingual release guides' PowerShell example verified a timestamped signature and a UTF-8 file path. It rejected missing and empty manifests, a modified file and an unsigned CI executable. [中文发布指南](../../windows-release.md) · [English release guide](../../windows-release.en.md)

CI 归档中有 311 个 Rust 许可目录。后续打包脚本为 28 个 crate 加入对应上游提交的许可正文。`base256emoji`、`base45`、`futures-bounded`、`if-watch`、`match-lookup`、`no-proxy` 和 `yaml-edit` 在对应提交的包目录至仓库根目录中只有 Cargo 许可声明可供归档。

The CI artifact has 311 Rust license directories. The subsequent packaging update adds license texts from the corresponding publication commits for 28 crates. For `base256emoji`, `base45`, `futures-bounded`, `if-watch`, `match-lookup`, `no-proxy`, and `yaml-edit`, the audit retained the Cargo license declaration after checking the package directory and its ancestors at the publication commit. [Supplemental license texts and source revisions](../../../licenses/rust-upstream.txt)
