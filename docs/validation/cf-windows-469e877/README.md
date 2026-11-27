# Windows x64 构建与桌面外观验收 / Windows x64 build and appearance validation

## GitHub Actions

- 源码 / Source `469e877b68481db90180ef46e78273cea15874b5`
- Windows x64 runner、Rust `1.99.1`、Flutter `3.47.9`、Dart `3.13.7`
- 锁定依赖检查、Rust Clippy、125 项 Rust 测试和 4 项 Flutter 测试通过。需要外部夹具的 33 项 Rust 测试按默认设置跳过。Flutter 分析、Release 桌面与两个 Release CLI 编译及打包通过。
- Locked dependency checks, Rust Clippy, 125 Rust tests, and four Flutter tests passed. The default run skipped 33 Rust tests that require external fixtures. Flutter analysis, the Release desktop build, both Release CLI builds, and packaging passed.
- CI ZIP `crabforge-windows-x64-ci.zip` · 65,001,202 bytes · SHA-256 `573D2F7AE0772C96AEE48C767FEF515F7C70DC98FC1E9DB93BB296018851333C` · [Run 45946299641](https://github.com/blueokanna/Crabforge/actions/runs/45946299641) · [Artifact 15175193310](https://github.com/blueokanna/Crabforge/actions/runs/45946299641/artifacts/15175193310)
- 下载件的哈希与 GitHub API 一致。解压后两个 CLI 都报告 `0.1.0`，输出完全相同的 97 项预设。六个 PE 文件的 Authenticode 状态为 `NotSigned`。
- The downloaded artifact matched GitHub's digest. Both unpacked CLIs reported `0.1.0` and returned the same 97 preset records. All six PE files showed `NotSigned` under Authenticode.

## 桌面 / Desktop

桌面以两家回环测试供应商和一个路由组启动。Windows 集成测试覆盖 1,200 和 560 逻辑像素宽度、浅色和深色、列表与网格下的六个主页面，共 48 张截图。九种界面语言均走过窄屏导航。测试还读取了原生 DWM 强调色，核对高对比度浅色和深色主题，并在新进程中确认恢复 HarmonyOS Sans SC。截图来自 Windows Debug 集成测试。

The fixture used two loopback providers and one route group. The Windows integration test captured 48 views across six main pages, widths of 1,200 and 560 logical pixels, both brightness settings, and list and grid layouts. All nine locales exercised narrow navigation. The test also read the native DWM accent, checked both high-contrast schemes, and verified that a new process restored HarmonyOS Sans SC. The screenshots come from the Windows Debug integration test.

[Light 1200](light-1200.jpg) · [Light 560](light-560.jpg) · [Dark 1200](dark-1200.jpg) · [Dark 560](dark-560.jpg) · [48 PNG checksums](captures.sha256)

GitHub Actions 的 Release 构建中，三套文字字体和产品 PNG 均与原件逐字节一致。包内的 Material Symbols Rounded 裁剪版为 229,580 字节，包含应用引用的 82 个唯一图标码点和 `FILL`、`GRAD`、`opsz`、`wght` 四条可变轴。

The three text fonts and the product PNG in the GitHub build match their source files byte for byte. The bundled Material Symbols Rounded subset is 229,580 bytes and retains all 82 distinct icon codepoints used by the app and the `FILL`, `GRAD`, `opsz`, and `wght` axes. [Font source manifest](../../../assets/fonts/manifest.json)

在解压后的 CI Release 桌面中，Windows Graphics Capture 记录了 24 次导航操作。从点击到首个窗口内容变化，中位数为 62.306 ms，范围为 55.134–89.929 ms。[原始帧记录](window-capture-frames.csv)

Windows Graphics Capture recorded 24 navigations in the unpacked CI Release desktop. The median time from click to first window content change was 62.306 ms, with a range of 55.134–89.929 ms. [Raw frame samples](window-capture-frames.csv)

独立的控制台集成测试通过了九语菜单顺序、供应商编辑、分组路由、通过解压 CLI 发送请求以及冷启动恢复。

A separate console integration run passed the nine-language menu ordering, provider editing, route-group changes, requests through the unpacked CLI, and cold-start restoration.

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

CI 归档中有 311 个 Rust 许可目录。后续打包脚本为 28 个 crate 加入对应上游提交的许可正文。`base256emoji`、`base45`、`futures-bounded`、`if-watch`、`match-lookup`、`no-proxy` 和 `yaml-edit` 在对应提交的包目录至仓库根目录中只有 Cargo 许可声明可供归档。

The CI artifact has 311 Rust license directories. The subsequent packaging update adds license texts from the corresponding publication commits for 28 crates. For `base256emoji`, `base45`, `futures-bounded`, `if-watch`, `match-lookup`, `no-proxy`, and `yaml-edit`, the audit retained the Cargo license declaration after checking the package directory and its ancestors at the publication commit. [Supplemental license texts and source revisions](../../../licenses/rust-upstream.txt)
