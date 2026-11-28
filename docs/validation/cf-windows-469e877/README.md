# Windows x64 构建与桌面外观验收 / Windows x64 build and appearance validation

## GitHub Actions

- 源码 / Source `469e877b68481db90180ef46e78273cea15874b5`
- Windows x64 runner、Rust `1.99.1`、Flutter `3.47.9`、Dart `3.13.7`
- 锁定依赖检查、Rust Clippy、125 项 Rust 测试和 4 项 Flutter 测试通过。需要外部夹具的 33 项 Rust 测试按默认设置跳过。Flutter 分析、Release 桌面与两个 Release CLI 编译及打包通过。
- Locked dependency checks, Rust Clippy, 125 Rust tests, and four Flutter tests passed. The default run skipped 33 Rust tests that require external fixtures. Flutter analysis, the Release desktop build, both Release CLI builds, and packaging passed.
- CI ZIP `crabforge-windows-x64-ci.zip` · 65,001,202 bytes · SHA-256 `573D2F7AE0772C96AEE48C767FEF515F7C70DC98FC1E9DB93BB296018851333C` · [Run 45946299641](https://github.com/blueokanna/Crabforge/actions/runs/45946299641) · [Artifact 15175193310](https://github.com/blueokanna/Crabforge/actions/runs/45946299641/artifacts/15175193310)
- 下载件的哈希与 GitHub API 一致。解压后两个 CLI 都报告 `0.1.0`，输出完全相同的 97 项预设。六个 PE 文件的 Authenticode 状态为 `NotSigned`。
- The downloaded artifact matched GitHub's digest. Both unpacked CLIs reported `0.1.0` and returned the same 97 preset records. All six PE files showed `NotSigned` under Authenticode.

下表的 Windows x64 构建均通过，下载件的大小和 SHA-256 均与 GitHub 一致。解压后，两种 CLI 入口都返回相同的 97 项预设。

The Windows x64 builds below passed. The downloaded ZIP sizes and SHA-256 digests matched GitHub. Both unpacked CLI entry points returned the same 97 presets.

| 源码 / Source | ZIP bytes | ZIP SHA-256 | GitHub Actions |
| --- | ---: | --- | --- |
| `ba09164c4a2265462ef736d64722f1b20b085206` | 65,007,925 | `E2499BD9C1C9052497A41BAA8A30C6241F95B5F418403A5E5273A69A267EC423` | [Run 45952941977](https://github.com/blueokanna/Crabforge/actions/runs/45952941977) · [Artifact 15179225625](https://github.com/blueokanna/Crabforge/actions/runs/45952941977/artifacts/15179225625) |
| `99c87e1449f7990fa348646f6724520330be7921` | 65,007,919 | `A414C719C0AD97F66EA7D71DB4BFD52D5BB6939AAEA5D6E402DF6508F8B3A1DA` | [Run 45954001089](https://github.com/blueokanna/Crabforge/actions/runs/45954001089) · [Artifact 15178467899](https://github.com/blueokanna/Crabforge/actions/runs/45954001089/artifacts/15178467899) |
| `26acbe6ac39bd140580d74762c4182ce64a21f46` | 65,008,011 | `0582F62878FF1DF612B206B7188E6B344B3DB6F6E20966ADE9F8A0D688B1C25C` | [Run 45955859685](https://github.com/blueokanna/Crabforge/actions/runs/45955859685) · [Artifact 15179309529](https://github.com/blueokanna/Crabforge/actions/runs/45955859685/artifacts/15179309529) |
| `6191cfd264c031e5cd4ed01c48b9a77ccc5560bf` | 65,009,120 | `A3E090CFA94A8841CEE7B35146F7DCEC4CAAAB0984474C1BEB5ECE94E2BE59AF` | [Run 45958239454](https://github.com/blueokanna/Crabforge/actions/runs/45958239454) · [Artifact 15180043889](https://github.com/blueokanna/Crabforge/actions/runs/45958239454/artifacts/15180043889) |
| `17cae79c1b4b8e2d7686340e6db6c71ff9ebb7aa` | 65,009,118 | `9420794F712625C1EC371F56A533490700E4C2DB2E5B2E3A26F9752F96351182` | [Run 45961563177](https://github.com/blueokanna/Crabforge/actions/runs/45961563177) · [Artifact 15181248050](https://github.com/blueokanna/Crabforge/actions/runs/45961563177/artifacts/15181248050) |
| `1faa92f51ca75f7a5fd5a7a328a7c63d35440ceb` | 65,133,307 | `8FD429BF06FE3FCD405B29A6A7CC5A06469C0604CA5512BE4CCB6721A229AFE2` | [Run 45964710149](https://github.com/blueokanna/Crabforge/actions/runs/45964710149) · [Artifact 15184091576](https://github.com/blueokanna/Crabforge/actions/runs/45964710149/artifacts/15184091576) |
| `95298d29c5fc900dc281d3ea828aa33e94475200` | 65,178,839 | `915113AC3E370E53187CA0FEFBD16903910425EFD413E31FD276E63B775CE04C` | [Run 45968626265](https://github.com/blueokanna/Crabforge/actions/runs/45968626265) · [Artifact 15184403493](https://github.com/blueokanna/Crabforge/actions/runs/45968626265/artifacts/15184403493) |

`26acbe6`、`6191cfd` 和 `17cae79` 的每个包均含 668 个文件和 311 个 Rust 许可目录，归档内容与解压后的 SHA-256 逐项一致。两个 CLI 均报告 `0.1.0`，六个 Windows PE 的 Authenticode 状态均为 `NotSigned`。产品 PNG、三套文字字体和图标字体通过字节核对。Rust 补录许可正文的 SHA-256 为 `4194156F90E6854E38C54445E280FE6380571EFE254F740493C1A8D4C363521E`。

Each package from `26acbe6`, `6191cfd` and `17cae79` included 311 Rust license directories. All 668 archive entries matched their extracted file hashes. Both CLI executables reported `0.1.0`, and all six Windows PE files reported `NotSigned` under Authenticode. The product PNG, three text fonts and icon font passed byte verification. The supplemental Rust license texts had SHA-256 `4194156F90E6854E38C54445E280FE6380571EFE254F740493C1A8D4C363521E`.

`1faa92f` 的 GitHub runner 通过 126 项 Rust 测试、Clippy、Flutter 分析和 4 项 Flutter 测试。33 项 Rust 外部夹具测试按默认设置跳过。下载件的 670 个文件与解压后的 SHA-256 逐项一致，313 个 Rust 许可目录含两个新增的 Apache-2.0 正文。产品 PNG、三套文字字体、图标字体和补录许可均通过字节核对。两个解压 CLI 均报告 `0.1.0`，97 项预设完全一致，六个 PE 均为 `NotSigned`。

The GitHub runner for `1faa92f` passed 126 Rust tests, Clippy, Flutter analysis and four Flutter tests. The default suite skipped 33 Rust fixture tests. All 670 archive entries matched their extracted SHA-256 hashes. The 313 Rust license directories included the two new Apache-2.0 license texts. The product PNG, three text fonts, icon font and supplemental licenses matched byte for byte. Both unpacked CLIs reported `0.1.0` and identical catalogs of 97 presets. All six PE files reported `NotSigned`.

`95298d2` 的 GitHub runner 通过 127 项 Rust 测试、Clippy、Flutter 分析、4 项 Flutter 测试及 Release 打包。33 项外部夹具测试按 GitHub 构建的默认配置跳过，并在本地另行通过。下载包的 670 个文件通过 SHA-256 核对，包含 313 个 Rust 许可目录。产品 PNG、三套文字字体和图标字体与原件逐字节一致。两个 CLI 均报告 `0.1.0` 并列出相同的 97 项预设，六个 AMD64 PE 均为 `NotSigned`。

The GitHub runner for `95298d2` passed 127 Rust tests, Clippy, Flutter analysis, four Flutter tests and Release packaging. The default GitHub build skipped 33 external fixture tests, which passed in separate local runs. All 670 package files passed SHA-256 verification. The package contains 313 Rust license directories. The product PNG, three text fonts and icon font matched their originals byte for byte. Both CLIs reported `0.1.0` and the same 97 presets. All six AMD64 PE files reported `NotSigned`.

RustSec 使用 `cargo-audit 0.23.0` 和公告库提交 `c9f6c91a44dbad9d71742329e9df061279f9c4be` 扫描该锁文件，发现零项已知漏洞和一项停止维护提示。`paste 1.0.15` 由 `if-watch` 的非 Windows netlink 依赖链引入，在 Windows x64 的全特性启用图中未出现。

RustSec scanned the lockfile with `cargo-audit 0.23.0` and advisory database commit `c9f6c91a44dbad9d71742329e9df061279f9c4be`. The scan reported zero known vulnerabilities and one unmaintained package. `paste 1.0.15` came through the non-Windows netlink chain under `if-watch`. The all-features Windows x64 dependency graph excludes it. [RustSec advisory](https://rustsec.org/advisories/RUSTSEC-2024-0436.html)

锁定依赖的全特性启用图覆盖运行、构建和测试依赖。所有目标共 417 个唯一包版本，Windows x64 为 339 个。两者均未启用 Serde 系列、Hyper 或 Reqwest，且均包含锁定的 Courierust `1.1.3`、NextJSON `0.1.5`、RustBinary `0.1.10`、recurse-x `0.2.6` 和 libp2p `0.57.1`。

The locked, all-features graphs include runtime, build and test dependencies. The audit counted 417 distinct package rows across all targets and 339 for Windows x64. Neither graph enabled packages from Serde, Hyper or Reqwest. Both used the pinned Courierust `1.1.3`, NextJSON `0.1.5`, RustBinary `0.1.10`, recurse-x `0.2.6` and libp2p `0.57.1`.

供应商备份导入增加 `rpassword 7.5.5` 和 `rtoolbox 0.0.6`，两者均随包提供 Apache-2.0 许可正文。依赖图重查为全目标 419 个、Windows x64 341 个，五个指定版本一致，禁用依赖为零。RustSec 公告库 `8b75c83c3c9142dd18e93d75b8d895a13d285861` 的扫描结果仍为零项已知漏洞及上述 `paste` 维护提示。

The provider backup import adds `rpassword 7.5.5` and `rtoolbox 0.0.6`. Both registry packages include their Apache-2.0 license text. The updated dependency graphs count 419 packages across targets and 341 on Windows x64, retain all five required versions and enable no disallowed packages. RustSec database revision `8b75c83c3c9142dd18e93d75b8d895a13d285861` reported zero known vulnerabilities and the same `paste` maintenance warning.

路由组导入提交沿用相同锁文件和依赖图。重查本地工作区 78,315 个目录，未发现 `vendor`。扫描识别到 24 个由 Flutter 生成并被 Git 忽略的 `file_selector_windows` 插件目录链接，均指向工作区外。

The group import commit retains the same lockfile and dependency graphs. A local scan of 78,315 workspace directories found no `vendor` directory. All 24 `file_selector_windows` directory links were generated by Flutter, ignored by Git and resolved outside the checkout.

### Expressive 桌面与 Antigravity 构建 / Expressive desktop and Antigravity build

`56b7a0da63adf6a436f57ad0cbeae67243fddf5f` 的 Windows Run `46107310744` 通过 163 项 Rust 测试，30 项忽略。Artifact `15249086982` 为 66,028,395 字节，ZIP SHA-256 为 `e7f8a806c6c38236f1c46bcea81718d6cb3f4766651c7aa3b70c1e40d611b342`。555 项解包清单的 SHA-256 为 `90691a02a0df50d56419ca0bf8d1bef23c9a816efdb5d5397a7e293d1bfdd672`。六个 PE 均为 `NotSigned`，Release 桌面在关闭请求后 0.15 秒退出。两种 CLI 入口通过 `import-backup`、ConPTY 无回显及路由、媒体与决策回环夹具。Antigravity `1.5.18` 经两个入口读取项目指令、发现 MCP 工具，并通过 Gemini 流式网关收到回复。Pi `1.6.0`、Grok `1.0.92` 和 Oh My Pi `19.2.1` 的会话夹具通过。该版本移除了 Gemini CLI 启动、会话及资源集成，两种 CLI 仍通过 Gemini 目录和网关测试。包内缺少产品 PNG，双 CLI 将旧式托管指令块预览为 `create`。验后远端 `main` 为 `f489e94db5232003d856085d0b449161b3f8f9f2`，Git 核实该构建 SHA 是其祖先。 [Job](https://github.com/blueokanna/Crabforge/actions/runs/46107310744/job/140862831313) · [Artifact](https://github.com/blueokanna/Crabforge/actions/runs/46107310744/artifacts/15249086982)

Windows run `46107310744` built `56b7a0da63adf6a436f57ad0cbeae67243fddf5f` and passed 163 Rust tests with 30 ignored. The 66,028,395-byte ZIP from artifact `15249086982` has SHA-256 `e7f8a806c6c38236f1c46bcea81718d6cb3f4766651c7aa3b70c1e40d611b342`. The 555-entry extracted manifest has SHA-256 `90691a02a0df50d56419ca0bf8d1bef23c9a816efdb5d5397a7e293d1bfdd672`. All six PE files reported `NotSigned`. The Release desktop exited 0.15 seconds after the close request. Both CLI entry points passed `import-backup`, no-echo ConPTY input, and loopback routing, media and decision fixtures. Through each entry point, Antigravity `1.5.18` read project instructions, discovered MCP tools and received a streaming Gemini reply. Session fixtures passed for Pi `1.6.0`, Grok `1.0.92` and Oh My Pi `19.2.1`. This revision removed Gemini CLI launch, session and resource integration. Both entry points passed the Gemini gateway and catalog fixtures. The package omitted the product PNG, and both CLIs previewed a legacy managed instruction block as `create`. After verification, remote `main` was `f489e94db5232003d856085d0b449161b3f8f9f2`, and Git confirmed that it contained the full build SHA. [Job](https://github.com/blueokanna/Crabforge/actions/runs/46107310744/job/140862831313) · [Artifact](https://github.com/blueokanna/Crabforge/actions/runs/46107310744/artifacts/15249086982)

`f489e94db5232003d856085d0b449161b3f8f9f2` 的原始 push Run `46107809374` 通过 163 项 Rust 测试，30 项忽略。Artifact `15248733979` 为 66,028,397 字节，ZIP SHA-256 为 `6e58674e6bcf634ad3a5bffa3f760e81091bfba6c01c2a06c8d357626b267b48`。555 项解包清单的 SHA-256 为 `19f27e1dd805e5ed872afb51ac388fab7140be922ebb472745d5c7d454bdd73f`。六个 PE 均为 `NotSigned`，Release 桌面关闭耗时 0.15 秒。两种 CLI 入口复跑上述迁移、网关和客户端夹具，再次复现两项回归。验后远端 `main` 已前进到含两项修复的 `cf76ad373c90a14abfc32a3803ec970767404d8c`，Git 核实该分支包含本次构建 SHA。 [Job](https://github.com/blueokanna/Crabforge/actions/runs/46107809374/job/140864397115) · [Artifact](https://github.com/blueokanna/Crabforge/actions/runs/46107809374/artifacts/15248733979)

Original push run `46107809374` built `f489e94db5232003d856085d0b449161b3f8f9f2` and passed 163 Rust tests with 30 ignored. The 66,028,397-byte ZIP from artifact `15248733979` has SHA-256 `6e58674e6bcf634ad3a5bffa3f760e81091bfba6c01c2a06c8d357626b267b48`. The 555-entry extracted manifest has SHA-256 `19f27e1dd805e5ed872afb51ac388fab7140be922ebb472745d5c7d454bdd73f`. All six PE files reported `NotSigned`. The Release desktop exited 0.15 seconds after the close request. Both CLI entry points repeated the migration, gateway and client fixtures and reproduced both regressions. At verification, remote `main` had advanced to `cf76ad373c90a14abfc32a3803ec970767404d8c` with both fixes, and Git confirmed that it contained the full build SHA. [Job](https://github.com/blueokanna/Crabforge/actions/runs/46107809374/job/140864397115) · [Artifact](https://github.com/blueokanna/Crabforge/actions/runs/46107809374/artifacts/15248733979)

## 桌面 / Desktop

桌面以两家回环测试供应商和一个路由组启动。Windows 集成测试覆盖 1,200 和 560 逻辑像素宽度、浅色和深色、列表与网格下的六个主页面，共 48 张截图。九种界面语言均走过窄屏导航。测试还读取了原生 DWM 强调色，核对高对比度浅色和深色主题，并在新进程中确认恢复 HarmonyOS Sans SC。截图来自 Windows Debug 集成测试。

The fixture used two loopback providers and one route group. The Windows integration test captured 48 views across six main pages, widths of 1,200 and 560 logical pixels, both brightness settings, and list and grid layouts. All nine locales exercised narrow navigation. The test also read the native DWM accent, checked both high-contrast schemes, and verified that a new process restored HarmonyOS Sans SC. The screenshots come from the Windows Debug integration test.

[Light 1200](light-1200.jpg) · [Light 560](light-560.jpg) · [Dark 1200](dark-1200.jpg) · [Dark 560](dark-560.jpg) · [48 original PNGs](captures.zip) · [PNG checksums](captures.sha256)

恢复产品 PNG 后，本地 Windows 集成测试重新验证两种宽度、浅色和深色、List/Grid 及九语窄屏导航，共生成 48 张 PNG。独立进程复测确认 HarmonyOS Sans SC 的保存偏好。截图库 ZIP SHA-256 为 `a383713a16ddbfab11f5735aacbefbf66465301fc8dc4f7f1c91c693027eae71`，本地交付目录为 `dist/crabforge-expressive-layouts/`。

After restoring the product PNG, local Windows integration tests repeated both widths, both brightness settings, list/grid layouts and nine-locale narrow navigation, producing 48 PNGs. A separate process confirmed the saved HarmonyOS Sans SC preference. The local capture archive under `dist/crabforge-expressive-layouts/` has SHA-256 `a383713a16ddbfab11f5735aacbefbf66465301fc8dc4f7f1c91c693027eae71`.

Windows Actions 从 `cf76ad373c90a14abfc32a3803ec970767404d8c` 构建，164 项 Rust 测试通过，30 项忽略。下载的 Release 桌面通过启动与退出检查，包内 PNG 与原件逐字节一致。双 CLI 通过旧式指令覆盖的冲突预览、缺失或过期修订拒绝及确认后原位保留 CRLF。37 项夹具开关对应的检查全部通过，六个 PE 均为 `NotSigned`。验收结束时经代理核对远端 `main` 为 `465897d447039211b379f7ab4c733259e9e0dfad`，Git 确认它包含构建提交。 [Windows job](https://github.com/blueokanna/Crabforge/actions/runs/46110437610/job/140872535031) · [下载件](https://github.com/blueokanna/Crabforge/actions/runs/46110437610/artifacts/15249104431)

Windows Actions built `cf76ad373c90a14abfc32a3803ec970767404d8c` and passed 164 Rust tests with 30 ignored. The downloaded Release desktop opened and exited, and its PNG matched the original byte for byte. Both CLIs previewed a legacy instruction conflict, rejected missing and stale revisions, and preserved CRLF during the approved in-place edit. Checks selected by all 37 fixture flags passed. All six PE files reported `NotSigned`. The proxied ref check at the end of acceptance found remote `main` at `465897d447039211b379f7ab4c733259e9e0dfad`, and Git confirmed it contained the build commit. [Windows job](https://github.com/blueokanna/Crabforge/actions/runs/46110437610/job/140872535031) · [Artifact](https://github.com/blueokanna/Crabforge/actions/runs/46110437610/artifacts/15249104431)

| ZIP bytes | ZIP SHA-256 | 解包项数 / Extracted files | 清单 SHA-256 / Manifest SHA-256 |
| ---: | --- | ---: | --- |
| 67,031,411 | `c8838936cde17e5719ff53ed907e5eca0ab03c814fe0745ad8186a7a0fcb8ae6` | 556 | `8f8a19773dd19bf4cf8a25d67cccfa7cc254b7b13555d67b993c45fd05b7edfc` |

GitHub Actions 的 Release 构建中，三套文字字体和产品 PNG 均与原件逐字节一致。两套主字体均覆盖英语译文用到的 47 个拉丁字母，简体中文译文中的 388 个汉字均由 HarmonyOS Sans SC 提供。包内的 Material Symbols Rounded 裁剪版为 229,580 字节，包含应用引用的 82 个唯一图标码点和 `FILL`、`GRAD`、`opsz`、`wght` 四条可变轴。

The three text fonts and the product PNG in the GitHub build match their source files byte for byte. Both primary fonts cover the 47 Latin letters used in the English locale. HarmonyOS Sans SC supplies all 388 Han characters in the Simplified Chinese locale. The bundled Material Symbols Rounded subset is 229,580 bytes and retains all 82 distinct icon codepoints used by the app and the `FILL`, `GRAD`, `opsz`, and `wght` axes. [Font source manifest](../../../assets/fonts/manifest.json)

在解压后的 CI Release 桌面中，Windows Graphics Capture 记录了 24 次导航操作。从点击到首个窗口内容变化，中位数为 62.306 ms，范围为 55.134–89.929 ms。[原始帧记录](window-capture-frames.csv)

Windows Graphics Capture recorded 24 navigations in the unpacked CI Release desktop. The median time from click to first window content change was 62.306 ms, with a range of 55.134–89.929 ms. [Raw frame samples](window-capture-frames.csv)

`1faa92f` 的解压桌面复测覆盖同样六个页面，共 24 次导航、234 次窗口内容变化。Windows Graphics Capture 测得点击至首个变化中位数为 43.878 ms，范围为 39.308–121.129 ms。PresentMon 因当前账号无权启动跟踪会话，GPU 执行与物理显示时序仍待测。

The unpacked `1faa92f` desktop reran 24 navigations across the same six pages and recorded 234 window content changes. Windows Graphics Capture measured a median of 43.878 ms from click to first change, with a range of 39.308–121.129 ms. PresentMon could not start its trace session with the current account, so GPU execution and physical display timing remain unmeasured. [Navigation samples](1faa92f-navigation.csv)

`95298d2` 的解压桌面也录得 24 次导航和 230 次窗口内容变化，首变化中位数为 53.809 ms，范围为 41.531–86.481 ms。

The unpacked `95298d2` desktop recorded 24 navigations and 230 window content changes. First change arrived at a median of 53.809 ms, with a range of 41.531–86.481 ms. [Navigation samples](95298d2-navigation.csv)

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

The same source passed four additional fixture tests. llama.cpp `b12722` loaded `SmolLM2-135M-Instruct-Q4_K_M.gguf`, and six protocol entry points returned model output. The official Ollama Python SDK `0.6.3` listed models and read streamed tokens. Two QUIC tests with Magpie `0.1.4936` in an isolated profile passed, one forwarding a fixed upstream response and one generating output from local weights. [llama.cpp tests](../../../crates/core/tests/llama.rs) · [Magpie tests at 6191cfd](https://github.com/blueokanna/Crabforge/blob/6191cfd264c031e5cd4ed01c48b9a77ccc5560bf/crates/cli/tests/magpie.rs)

`26acbe6` 的解压 CLI 用独立数据目录连接本机 HTTP 夹具。`models lab --sync` 刷新了 `free:lab/*` 路由组，Magpie 兼容 JSON 导入第二个目录后按其本地目录解析同一规则。嵌入请求在首选上游返回 429 后，经嵌套手选组一次回退取得 200。图像编辑、音频转录和翻译均返回 200。上游核对了改写后的模型 ID、文件名和原始二进制字段。

The unpacked CLI from `26acbe6` used an isolated profile and a loopback HTTP fixture. `models lab --sync` refreshed the `free:lab/*` route group. Importing the Magpie-compatible JSON into a second profile resolved the same rule against its local catalog. An embeddings request received 429 from its primary upstream and then 200 on one fallback through a nested manual group. Image edits, audio transcriptions, and audio translations all returned 200. The upstream fixture checked the rewritten model ID, the filename, and the unchanged binary bytes. [Routing tests](../../../crates/cli/tests/groups.rs) · [Multipart tests](../../../crates/core/tests/media.rs)

Magpie `0.1.4936` 与 `26acbe6` 的解压 CLI 在独立配置间实际往返导入一个加权组。往返生成的 JSON 一致，保留成员顺序、备注和权重。含自动推理级别分类器的 Magpie 导出被明确拒绝，接收方设置逐字节保持原样。整库备份恢复和分类器执行仍待移植。

Magpie `0.1.4936` and the unpacked CLI from `26acbe6` exchanged a weighted group between isolated profiles. The re-exported JSON matched the original and preserved member order, the note and weights. Crabforge rejected a native export with automatic effort classification and left the receiver's settings byte for byte intact. Full backup restore and classifier execution remain migration gaps. [Routing group import](../../../crates/core/src/config/groups/share.rs)

后续 Windows 实测用 Magpie `0.1.4936` 生成了带中文文件名和口令的原生加密备份。两个 CLI 入口都预览了指定供应商，应用后保留排序、旧 ID 和回退。使用旧 ID 发出的回环请求在主供应商返回 429 后仅回退一次并获 200。HTTP 夹具确认密钥来自指定环境变量。错误口令、密文篡改、已关闭供应商及 ID 冲突均被拒绝，目标设置逐字节保持原样。整库恢复仍待移植。[供应商导入说明](../../../README.zh-CN.md#从-magpie-导入供应商)

A subsequent Windows test created a native encrypted backup with Magpie `0.1.4936` using a Chinese filename and passphrase. Both CLI entry points previewed the selected providers. Applying the import preserved order, a former ID and fallback settings. A loopback request using the former ID received a 429 from the primary and a 200 from one fallback. The HTTP fixture confirmed that the API key came from the selected environment variable. Wrong passphrases, modified ciphertext, disabled providers and ID conflicts left the destination settings byte for byte intact. Full backup restoration remains outstanding. [Provider import guide](../../../README.md#import-providers-from-magpie)

`1faa92f` 的解压包复跑了供应商备份导入。同一备份经四个进程同时应用，只有一次写入成功，最终配置完整。两个 CLI 均通过 Windows ConPTY 口令提示、无回显及只读预览检查。

The unpacked `1faa92f` build repeated the provider backup import. Four concurrent processes attempted to apply the same backup. Exactly one write succeeded and the resulting configuration was complete. Both CLI entry points passed the Windows ConPTY checks for a visible prompt, no passphrase echo and a read-only preview.

`1faa92f` 解压 CLI 通过八项记忆检查，包括检索、`2025-11-25` 协商、项目和 Agent 隔离、去重、归档、恢复和拒绝旧修订。Claude Code 2.1.332 在项目批准后连接 stdio、Streamable HTTP 和 SSE。两种 HTTP 传输验证自定义请求头，三种传输均在项目外不可见，且撤回后消失。原批准文件逐字节保留。

The unpacked `1faa92f` CLI passed eight memory checks covering search, `2025-11-25` negotiation, project and agent isolation, deduplication, archiving, restore and stale revision rejection. Claude Code 2.1.332 connected over stdio, Streamable HTTP and SSE after project approval. Both HTTP transports verified the configured header. All three servers were absent outside the project and after withdrawal, and the approval file remained byte for byte intact. [Memory checks](1faa92f-memory-process.csv) · [Native MCP transport checks](1faa92f-project-mcp-transports.csv)

Codex CLI 0.172.0 使用 `1faa92f` 解压包完成 12 轮对话。每轮从新进程启动，交替使用两个 Crabforge CLI，第二轮起按同一个原生会话 ID 恢复，第七轮前重启网关。回环 Responses 夹具核对了每轮请求中的全部既往提示和本次提示，每轮恰好收到一个请求。两种 CLI 的 `sessions --agent codex` 均从隔离配置读回同一会话和项目路径。

Codex CLI 0.172.0 completed 12 turns through the unpacked `1faa92f` build. Each turn started a new client process, alternating Crabforge entry points. Turns after the first resumed the same native session ID, including after a gateway restart before turn seven. The loopback Responses fixture received one request per turn and found the current and preceding prompt markers in each request. `sessions --agent codex` found the same session and project path in the isolated profile through both entry points. [Session samples](1faa92f-codex-session-resume.csv)

同一解压 CLI 的三轮回环 SSE 首事件为 5.424、2.974、2.234 ms，结束为 706.124、703.181、703.197 ms。每轮由独立网关进程处理，调用上游一次，代理未收到请求。

The same unpacked CLI received the first loopback SSE event at 5.424, 2.974 and 2.234 ms, and the final event at 706.124, 703.181 and 703.197 ms. Each round started its own gateway process, made one upstream request and sent no traffic to the proxy. [Streaming samples](1faa92f-forward-h1.csv)

`1faa92f` 在 Windows 10 x64 build 19045 上重复采样三轮。工作集／私有字节中位数以 MiB 计，Crabforge 空闲桌面为 159.95／233.75。空目录网关为 Crabforge 8.24／1.65、Magpie 0.1.4936 58.84／92.60。回环对话为 Crabforge 9.56／2.11、Magpie 70.85／99.91，分别列出两个和一个模型 ID。该轮只采样 Crabforge 桌面进程。

The `1faa92f` package ran three sampling rounds on Windows 10 x64 build 19045. Median working set and private bytes in MiB were 159.95 and 233.75 for the idle Crabforge desktop. For the empty gateway catalog they were 8.24 and 1.65 for Crabforge and 58.84 and 92.60 for Magpie 0.1.4936. For loopback chat they were 9.56 and 2.11 for Crabforge and 70.85 and 99.91 for Magpie, with two and one listed model IDs respectively. This desktop rerun sampled Crabforge alone. [Desktop samples](1faa92f-desktop-working-set.csv) · [Gateway samples](1faa92f-gateway-memory.csv)

另用已打开的 Magpie 0.1.4936 桌面配对采样三轮。工作集／私有字节中位数以 MiB 计，Crabforge 为 158.30／233.24，Magpie 主进程为 181.15／230.86，计入 WebView2 后为 919.47／1169.46。Crabforge 每轮使用新的隔离配置，Magpie 三轮均使用同一已运行实例。

A separate three-round desktop comparison sampled an existing Magpie 0.1.4936 window. Median working set and private bytes in MiB were 158.30 and 233.24 for Crabforge, 181.15 and 230.86 for Magpie's main process, and 919.47 and 1169.46 with WebView2 included. Each Crabforge round used a fresh profile, and all Magpie rounds used the same running instance. [Paired process samples](1faa92f-desktop-paired.csv)

`1faa92f51ca75f7a5fd5a7a328a7c63d35440ceb` 的本地 Windows 源码构建另通过了 18 项按默认配置跳过的进程测试。Kubo 0.43.1 双向交换鉴权请求，llama.cpp b12722 使用相同 GGUF 权重通过六协议推理，Ollama Python SDK 0.6.3 读取模型目录及流式 token，Magpie 0.1.4936 通过两项 QUIC 转发。其余 13 项覆盖 Claude Code 2.1.332、Codex 0.172.0、Gemini 0.71.1、Pi 1.6.0 及 Oh My Pi 19.2.1 和 Bun 1.4.5 的实际进程。所有 P2P 节点均位于同一主机。

The local Windows source build of `1faa92f51ca75f7a5fd5a7a328a7c63d35440ceb` passed 18 process tests that the default suite skips. Kubo 0.43.1 exchanged authenticated requests in both directions. llama.cpp b12722 used the same GGUF weights for six protocols. The Ollama Python SDK 0.6.3 read the model list and streamed tokens, and Magpie 0.1.4936 passed both QUIC forwarding cases. The other 13 cases exercised Claude Code 2.1.332, Codex 0.172.0, Gemini 0.71.1, Pi 1.6.0 and Oh My Pi 19.2.1 with Bun 1.4.5. All P2P peers ran on the same host. [Agent process tests](../../../crates/cli/tests/agents.rs)

路由组导入提交 `95298d29c5fc900dc281d3ea828aa33e94475200` 的本地 Windows 源码构建复跑并通过全部 33 项默认跳过的外部夹具测试。其中 25 项验证真实 Agent 客户端的回环对话或 Oh My Pi 配置，1 项验证 DevEco 原生配置和鉴权端点，2 项使用 Chrome 和 ChromeDriver 158.0.8127.37，另有 5 项覆盖 Kubo、llama.cpp、Ollama SDK 和 Magpie QUIC。Mistral Vibe 3.1.5 初次试跑遇到已失效的 Python 路径，使用新的隔离 CPython 3.13.18 环境后通过。

The local Windows source build of group import commit `95298d29c5fc900dc281d3ea828aa33e94475200` passed all 33 external fixture tests skipped by default. Twenty-five exercised native client conversations over loopback or Oh My Pi settings. One inspected the DevEco configuration and authenticated endpoint, two used Chrome and ChromeDriver 158.0.8127.37, and five covered Kubo, llama.cpp, the Ollama SDK and Magpie QUIC. Mistral Vibe 3.1.5 initially failed because its saved Python path was invalid, then passed in a fresh isolated CPython 3.13.18 environment. [Native client tests](../../../crates/cli/tests/agents.rs) · [Browser fixtures](../../../crates/core/src/webchat.rs)

本地 Windows 路由组导入测试使用 Magpie 0.1.4936 CLI 写出的原生加密备份。两个 Crabforge CLI 都通过预览。一次原子应用保留三个组的顺序、旧 ID、停用状态、加权配置、匹配成员排列和中文备注。向旧组 ID 发起的实际网关请求命中手选的嵌套加权组并获得 HTTP 200。缺子组、错误口令、冲突和带脱敏规则的批次均被拒绝，目标设置逐字节一致。

The local Windows group import test used a native encrypted backup written by Magpie 0.1.4936. Both Crabforge CLIs previewed it. One atomic apply preserved the three groups' order, former IDs, disabled state, weights, matched-member placement and Chinese note. A gateway request using the former group ID reached its manually selected nested weighted group and returned HTTP 200. A missing child, wrong passphrase, ID collision and redaction setting each caused rejection and left the destination settings byte for byte intact. [Group import guide](../../../README.md#import-providers-from-magpie)

`95298d2` 下载包的两个 CLI 均通过原生 Windows ConPTY 路由组预览。控制台显示提示但未回显中文口令，三条组记录按备份顺序返回，预览前后的设置字节一致。

Both CLIs from the `95298d2` download passed the group import preview under native Windows ConPTY. The prompt appeared without echoing the Chinese passphrase. The preview returned three groups in backup order and preserved the settings bytes.

同一解压包复跑了 Magpie 原生备份导入及嵌套路由请求。四个进程同时应用同一批组时只有一次成功，结果与单进程应用逐字节一致。另通过八项记忆检查，以及 Claude Code 2.1.332 的 stdio、Streamable HTTP 和 SSE 连接、项目隔离及撤回。

The same unpacked build repeated the native Magpie backup import and nested route request. One of four concurrent group applies succeeded. The resulting settings matched the single-process apply byte for byte. Eight memory checks and the Claude Code 2.1.332 stdio, Streamable HTTP and SSE connection, project isolation and withdrawal checks also passed.

Codex CLI 0.172.0 在该包上重复通过 12 轮原生会话恢复。第七轮前重启网关后，仍读回同一会话和之前的提示。

Codex CLI 0.172.0 repeated the 12-turn native session test on this package. It resumed the same session with its earlier prompts after the gateway restarted before turn seven. [Session samples](95298d2-codex-session-resume.csv)

Claude Code 2.1.332 在同一解压包上独立完成 12 轮。每轮启动新进程，交替调用两个 CLI，第二轮起按同一原生 ID 恢复，第七轮前重启网关。回环 Anthropic Messages 夹具每轮收到一次请求，核对了全部历史提示。两个 CLI 均读回同一会话和项目路径。

Claude Code 2.1.332 completed a separate 12-turn run on the same unpacked build. Each turn started a new process, alternating the two CLI entry points. Turns after the first resumed the same native ID, including after the gateway restart before turn seven. The loopback Anthropic Messages fixture received one request per turn and verified every prior prompt. Both CLI entry points read back the same session and project path. [Claude Code session samples](95298d2-claude-session-resume.csv)

同一解压包从 Claude Code 2.1.332 和 Codex CLI 0.172.0 的原生会话日志分别提取 12 轮用户提示。回环模型夹具验证请求中只有用户消息。每个会话产生一条有原文引用的候选记忆，缺少真实引用的建议被丢弃，重复请求未产生重复候选。过期修订在调用模型前被拒绝。审核激活后，记忆可写入项目指令，撤回时逐字节恢复含 BOM 和 CRLF 的原文件。

The same unpacked build extracted 12 user prompts per session from native Claude Code 2.1.332 and Codex CLI 0.172.0 logs. The loopback model fixture checked that the request contained user messages only. Each session produced one candidate with a verified quote. The fabricated quote was rejected and repeating the proposal created no duplicate. A stale revision was rejected before the model call. After approval, the project digest was published to the client's instruction file and then restored byte for byte, including its BOM and CRLF line endings. [Memory proposal checks](95298d2-memory-distill.csv)

该包首轮三次回环 SSE 测得首事件为 17.716–67.334 ms，末事件为 718.565–768.151 ms。每次均调用上游一次，代理未收到请求。内存采样三轮，桌面空闲、空目录网关和回环对话网关的工作集／私有字节中位数分别为 159.59／232.96、8.18／1.64 和 9.52／2.05 MiB。

Three initial loopback SSE trials on this build returned first events in 17.716–67.334 ms and final events in 718.565–768.151 ms. Each trial sent one upstream request and no proxy traffic. Across three memory samples, median working set and private bytes in MiB were 159.59 and 232.96 for the idle desktop, 8.18 and 1.64 for the empty catalog gateway, and 9.52 and 2.05 for the loopback chat gateway. [Streaming samples](95298d2-forward-h1.csv) · [Memory samples](95298d2-memory.csv)

使用同一回环夹具按 `1faa92f`、`95298d2`、`95298d2`、`1faa92f` 顺序复测，每批三次。两版本首事件中位数分别为 4.389 和 4.067 ms。范围分别为 3.235–8.987 和 2.911–77.363 ms。较新包出现两次首事件超过 50 ms，原因未确定。12 次仍各只有一个上游请求，代理均未收到请求。

The same loopback fixture ran three trials per batch in the order `1faa92f`, `95298d2`, `95298d2`, `1faa92f`. Median first event times were 4.389 and 4.067 ms. The ranges were 3.235–8.987 and 2.911–77.363 ms, respectively. Two first events on the newer build exceeded 50 ms. The cause remains undetermined. Each of the 12 trials sent one upstream request and no proxy traffic. [Paired streaming samples](forward-h1-paired.csv)

分段复测为每个包启动 20 次新网关，每轮交换两包的执行顺序。`1faa92f` 和 `95298d2` 的首事件中位数分别为 8.889 和 11.006 ms。四次超过 50 ms 的样本中，从客户端发起请求到回环上游读完请求约为 49.858–87.889 ms，从上游开始发送到客户端读到首事件为 0.440–1.144 ms。40 次均返回 HTTP 200，每次一次上游请求，代理请求为零。前段延迟包含请求处理、回环连接及进程调度，尚未进一步定位。

The instrumented rerun started 20 fresh gateways per package and alternated which package went first. Median first event latency was 8.889 ms for `1faa92f` and 11.006 ms for `95298d2`. In the four trials above 50 ms, the interval between the client starting the request and the loopback upstream reading it was 49.858–87.889 ms. The interval between the upstream starting its send and the client reading the first event was 0.440–1.144 ms. All 40 trials returned HTTP 200 with one upstream request and zero proxy requests each. The first interval includes request processing, the loopback connection and process scheduling. Its cause remains unresolved. [Instrumented streaming samples](forward-h1-stage-pair.csv)

Python 3.15.2 在同一 Windows 10 上为 `95298d2` 和 `30ce06a` 各启动 20 次新网关，交替测试顺序，并记录客户端连接、写入返回、上游接受连接和读取请求头的时刻。首事件中位数分别为 2.666 和 3.106 ms，最大值为 17.092 和 20.754 ms。每个包最慢的一轮中，客户端连接分别耗时 15.038 和 15.846 ms。40 轮均返回 HTTP 200，每轮只请求一次上游，代理请求为零。此前超过 50 ms 的样本在本轮未复现。

Python 3.15.2 started 20 fresh gateways each for `95298d2` and `30ce06a` on the same Windows 10 host. The test alternated package order and timed the client connection, return from the request write, upstream accept and upstream header read. Median first events were 2.666 and 3.106 ms, and maxima were 17.092 and 20.754 ms. The client connection took 15.038 and 15.846 ms in the slowest trial of each package. All 40 trials returned HTTP 200 with one upstream request and zero proxy requests each. The earlier samples above 50 ms did not recur. [Connection and streaming samples](forward-h1-timing-detail.csv)

又在同一主机交替启动 `30ce06a` 和 `5cc51ab` 各 20 次，并拆分客户端地址解析、套接字初始化和 `connect` 调用。这轮与本机桌面验收并行。两个版本首事件中位数分别为 2.852 和 2.905 ms，三个超过 50 ms 的样本为 66.120、83.829 和 99.248 ms。客户端请求写入返回至上游接受连接经过 55.393–62.005 ms，上游接受连接至读完请求头经过 9.327–25.722 ms，上游开始发送至客户端读到首事件经过 0.438–0.483 ms。最长的一轮另有 21.770 ms 耗在客户端 `connect`，地址解析在三个慢样本中均不超过 0.059 ms。上游接受连接的时间点在 Python 进程测得，尚无法将主要间隔归因于网关、TCP 或进程调度。40 轮均为 HTTP 200，每轮一次上游、零代理请求。 [原始分段样本](forward-h1-windows-connect.csv)

Another alternating run started 20 gateways each for `30ce06a` and `5cc51ab` on the same host alongside desktop acceptance tests. The client timed address resolution, socket setup and its `connect` call. Median first events were 2.852 and 2.905 ms. Three samples exceeded 50 ms at 66.120, 83.829 and 99.248 ms. From return of the client request write to upstream accept took 55.393–62.005 ms. Upstream accept to header read took 9.327–25.722 ms, and the upstream starting its send to the client's first event took 0.438–0.483 ms. The slowest sample also spent 21.770 ms in the client `connect` call. Address resolution took at most 0.059 ms in all three outliers. The upstream accept timestamp was measured in Python, leaving gateway execution, TCP and process scheduling unresolved within the longest interval. All 40 trials returned HTTP 200 with one upstream request and zero proxy requests each. [Raw segmented samples](forward-h1-windows-connect.csv)

## 原生备份联合导入 / Combined native backup import

Windows Debug 双 CLI 读取 Magpie 0.1.4936 CLI 生成的加密备份。供应商的回退目标引用同一备份中的旧组 ID。两个入口生成了相同的预览，应用时一次保存供应商和路由组。缺少回退目标、组含不受支持的脱敏规则或只导入供应商时，原有设置逐字节保持一致。四个并发应用进程中只有一个成功，写入结果与单进程应用一致。向旧供应商 ID 发送回环 Chat Completions 请求时，上游返回 429，随后通过导入的嵌套组恰好回退一次并返回 200。

The two Windows Debug CLI entry points read an encrypted backup generated by Magpie 0.1.4936. The provider's fallback referenced a former group ID from the same backup. Both entry points returned the same preview, and apply saved the providers and groups together. An unavailable fallback, a group with an unsupported redaction rule, or applying the provider alone left the original settings byte for byte intact. One of four concurrent applies succeeded, producing the same bytes as a single apply. A loopback Chat Completions request to the former provider ID received an upstream 429, then returned 200 through exactly one fallback to the imported nested group. [Import commands](../../../README.md#import-providers-from-magpie)

`30ce06a` 本地 Debug 双 CLI 通过联合导入的 Windows ConPTY 检查。中文口令提示可见，输入无回显。预览按备份顺序返回选中的供应商和路由组，设置文件逐字节一致。该源码版本的 33 项外部夹具均通过。MiMo Code 0.1.17 和 Mistral Vibe 3.1.5 在批量试跑中的失败和超时，经单独复跑后通过。

Both local `30ce06a` Debug CLIs passed the combined import in Windows ConPTY. The prompt was visible and the Chinese passphrase was entered without echo. The preview returned the selected providers and groups in backup order and preserved the settings bytes. All 33 external fixtures passed for this source revision. MiMo Code 0.1.17 and Mistral Vibe 3.1.5 passed in separate reruns after a failure and timeout during batch execution. [Native client tests](../../../crates/cli/tests/agents.rs)

Windows Actions 从 `30ce06a61a9a7d8f36332e754e7757180abbf911` 构建并通过 128 项 Rust 测试、Clippy、依赖门禁、Flutter 分析及桌面测试。下载包有 670 个文件，全部与解压后的 SHA-256 一致。双 CLI 均报告 `0.1.0`，输出同样的 97 项预设。六个 PE 的 Authenticode 状态为 `NotSigned`。两种 CLI 复跑分项和联合预览、原子应用、四进程竞争以及旧 ID 的网关回退。验包完成后再次查询远端 `main`，完整 SHA 与构建一致。

Windows Actions built `30ce06a61a9a7d8f36332e754e7757180abbf911` and passed 128 Rust tests, Clippy, the dependency gate, Flutter analysis and desktop tests. SHA-256 checks matched all 670 archive files with their unpacked copies. Both CLIs reported `0.1.0` and returned the same 97 presets. The six PE files had Authenticode status `NotSigned`. The unpacked CLIs repeated individual and combined previews, atomic apply, four-process contention and gateway fallback using former IDs. The remote `main` SHA matched the build when verification finished. [Actions run 45974408067](https://github.com/blueokanna/Crabforge/actions/runs/45974408067)

- CI ZIP `crabforge-windows-x64-ci.zip` · 65,197,885 bytes · SHA-256 `51B7EA1605DA43B855D143431BC001885E97B452CD8DCA98211364B9CF054D09` · [Artifact 15187966673](https://github.com/blueokanna/Crabforge/actions/runs/45974408067/artifacts/15187966673)

解压后的两个 CLI 也通过 Windows ConPTY 联合导入预览，输入中文口令时无回显，预览保留原设置的字节。

Both unpacked CLIs also passed the combined import preview in Windows ConPTY. The Chinese passphrase was entered without echo and the preview preserved the settings bytes.

Pi 1.6.0 在同一解压包上启动 12 个独立进程，交替使用两个 CLI，每轮沿用同一原生会话文件和 ID。第七轮前用另一 CLI 重启网关。回环 Chat Completions 夹具每轮仅收到一次请求，并核对全部历史用户提示和助手回复。原生会话文件保留了每轮内容。Claude Code 和 Codex 的记忆候选、审核发布和撤回也在该下载包上复测通过。

Pi 1.6.0 started 12 separate processes on the same unpacked build, alternating CLI entry points and resuming the same native session file and ID. The gateway restarted before turn seven using the other CLI. The loopback Chat Completions fixture received one request per turn and verified all user prompts and prior assistant replies. The native session file kept the full exchange. Claude Code and Codex also repeated the memory proposal, reviewed publication and withdrawal checks on this download. [Pi session samples](30ce06a-pi-session-resume.csv)

Gemini CLI 0.71.1 在同一 Windows 下载包上使用 `--resume latest` 完成 12 轮原生会话。每轮由新进程启动，交替使用两个 CLI，第七轮前重启网关。Gemini JSON 输出的会话 ID 始终一致。回环 Gemini `streamGenerateContent` 上游每轮收到一次请求，包含全部用户提示及此前的助手回复。

Gemini CLI 0.71.1 used `--resume latest` to complete 12 turns through the same unpacked build. Each turn started a new process, alternating the two CLIs, and the gateway restarted before turn seven. Gemini's JSON output kept the original session ID. The loopback Gemini `streamGenerateContent` fixture received one request per turn, containing every user prompt and prior assistant reply. [Gemini session samples](30ce06a-gemini-session-resume.csv)

接入 Gemini CLI 会话读取后，两个 Windows Debug CLI 在同一原生会话重跑 12 轮，使用 Gemini CLI 0.71.1 和 Node.js 26.13.0。通过 `sessions --agent gemini` 找到原生会话 ID 与项目路径，再用 `memory distill gemini` 逐条核对 12 条用户原话。测试还通过 `GEMINI_CLI_HOME` 读取自定义配置、检查相同目录去重并拒绝未配置目录。预览后原生日志字节一致。Rust 回归覆盖旧版 JSON、撤回、提示编辑、用户可见文本、工具结果和子代理排除。 [原始样本](gemini-native-session-discovery.csv)

After adding Gemini CLI session discovery, the two Windows Debug CLIs repeated 12 turns in one native session using Gemini CLI 0.71.1 and Node.js 26.13.0. `sessions --agent gemini` returned the original session ID and project path. `memory distill gemini` returned each of the 12 original user prompts. The test also discovered a custom profile through `GEMINI_CLI_HOME`, deduplicated the same folder and rejected an unconfigured profile. The preview preserved the native log bytes. Rust tests covered legacy JSON, rewinds, edited prompts, visible user text, tool results and subagent exclusion. [Raw samples](gemini-native-session-discovery.csv)

Windows Debug 桌面在 560 逻辑像素的网格中通过 Gemini CLI 筛选、会话详情和只读记忆预览。测试确认预览期间未向模型发送请求。

The Windows Debug desktop passed Gemini CLI filtering, session details and read-only memory preview in the grid at 560 logical pixels. The preview made zero model requests. [Desktop integration test](../../../gui/integration_test/agent_resources_test.dart)

同一 Windows Debug 测试批次还通过 560／1,200 逻辑像素下六个页面的列表与网格复验、独立进程恢复 HarmonyOS Sans SC、Flutter 分析和四项 Flutter 单测。

The same Windows Debug batch passed the six-page list and grid rerun at 560 and 1,200 logical pixels, restored HarmonyOS Sans SC in a new process, and passed Flutter analysis and four Flutter unit tests. [Layout and restart test](../../../gui/integration_test/console_layout_test.dart)

控制台用例点击路由卡片标题进入编辑，保留列表内选择器的独立操作。Windows 实测从手选切到轮询、重新启用成员，再切回手选并重启网关。真实 HTTP 请求返回 200，路由记录显示 `group/parent → group/renamed → local`。同轮还通过 P2P 隧道连接、带鉴权的模型目录读取、断开与独立冷启动。

The console test opens the route editor from the card title so the embedded selector remains a separate action. The Windows run switched from manual selection to rotation, re-enabled a member, restored manual selection and restarted the gateway. An HTTP request returned 200 and recorded `group/parent → group/renamed → local`. The same run verified the P2P tunnel, its authenticated model catalog, disconnect and a separate cold start. [Console test](../../../gui/integration_test/console_test.dart)

该包的 Windows Graphics Capture 采集了 24 次导航切换和 262 次内容变化。从投递点击到首个采集变化的最小值、中位数和最大值分别为 47.109、64.186 和 94.920 ms。

Windows Graphics Capture observed 262 content changes across 24 navigation transitions on this build. The first captured change after the click was posted arrived at 47.109 ms minimum, 64.186 ms median and 94.920 ms maximum. [Navigation samples](30ce06a-navigation.csv)

同一包在 Windows 10 x64 build 19045 上每个场景采样三轮。空闲桌面、空目录网关和回环 Chat 网关的工作集／私有字节中位数分别为 153.42／228.10、8.18／1.62 和 9.54／2.07 MiB。同机 Magpie 0.1.4936 的空目录和回环 Chat 网关为 62.43／96.63 和 69.59／98.77 MiB。回环 Chat 中 Crabforge 与 Magpie 分别列出两个和一个模型 ID。独立的三轮 SSE 请求，首事件为 3.370–3.813 ms，末事件为 703.790–704.350 ms。每轮只请求一次回环上游，代理请求为零。

The same package sampled each workload three times on Windows 10 x64 build 19045. Median working set and private bytes for the idle desktop, empty catalog gateway and loopback Chat gateway were 153.42 / 228.10, 8.18 / 1.62 and 9.54 / 2.07 MiB. Magpie 0.1.4936 on the same host used 62.43 / 96.63 MiB for the empty catalog and 69.59 / 98.77 MiB for loopback Chat. The Chat catalogs listed two model IDs in Crabforge and one in Magpie. Three separate SSE trials received the first event in 3.370–3.813 ms and the final event in 703.790–704.350 ms. Each trial made one loopback upstream request and no proxy request. [Memory samples](30ce06a-memory.csv) · [Streaming samples](30ce06a-forward-h1.csv)

接入 Pi 会话读取后，本地 Windows Debug 双 CLI 使用 Pi 1.6.0 在 Crabforge 管理的配置中再跑 12 轮。两个入口都找到同一原生会话。记忆预览逐条核对 12 条用户提示，模型只收到这 12 条。测试拒绝过期修订和引用助手回复的候选，重复提炼未新增候选。显式审核后可写入项目 `AGENTS.md`，撤回逐字节恢复 BOM 和 CRLF。整个流程保留原会话文件。分支回归覆盖被放弃的提示、上下文编辑和旧版线性日志。

After Pi session discovery was added, both local Windows Debug CLIs exercised another 12 turns with Pi 1.6.0 in the Crabforge-managed profile. Both entry points found the same native session. The memory preview matched all 12 user prompts and the loopback model received those exact messages. The check rejected a stale revision and a proposal citing an assistant reply. A repeated request created no duplicate. Explicit approval enabled publication to the project's `AGENTS.md`, and withdrawal restored the BOM and CRLF bytes. The session file stayed byte for byte intact. Regression coverage includes abandoned branches, context edits and legacy linear logs. [Session reader](../../../crates/core/src/sessions/pi.rs)

Windows Debug 桌面集成测试在 560 逻辑像素宽度显示 Claude Code、Codex 和 Pi 的本地会话。用例切换网格，按 Pi 筛选，打开会话详情和只读的记忆预览，再切回 Codex 完成候选审核和用户指令撤回。

The Windows Debug desktop integration test showed local Claude Code, Codex and Pi sessions at 560 logical pixels. It switched to the grid, filtered for Pi and opened the session details and read-only memory preview. It then switched to Codex to review a candidate and withdraw the publication from user instructions. [Desktop integration test](../../../gui/integration_test/agent_resources_test.dart)

Windows Actions `45981941463` 从 `7471c961c39acc62277f614a9a656e4486a0c30b` 构建，132 项 Rust 测试、依赖门禁、Clippy 和 Flutter 检查通过。归档 `15190149052` 为 65,254,809 字节，SHA-256 为 `cb14dd982fe26c6f3515aaf60f4bc51c7c2fc723e207614f8157a3e40a0bfba0`。670 个解压文件逐一匹配归档哈希，313 个 Rust 许可目录通过核验，六个 PE 的签名状态为 `NotSigned`。下载包的两个 CLI 通过 Pi 1.6.0 的 12 轮原生会话恢复、记忆预览和审核发布撤回，以及原版加密备份迁移和 ConPTY 中文口令无回显。验包结束时远端 `main` 匹配上述完整 SHA。

Windows Actions run `45981941463` built `7471c961c39acc62277f614a9a656e4486a0c30b`. The dependency gate, Clippy, 132 Rust tests and Flutter checks passed. Artifact `15190149052` was 65,254,809 bytes with SHA-256 `cb14dd982fe26c6f3515aaf60f4bc51c7c2fc723e207614f8157a3e40a0bfba0`. All 670 extracted files matched the archive hashes, 313 Rust license directories passed verification, and all six PE files had Authenticode status `NotSigned`. The packaged CLIs passed 12 turns of native Pi 1.6.0 resumption, memory preview, reviewed publication and withdrawal. Native encrypted backup migration and Chinese passphrase entry without echo in ConPTY also passed. Remote `main` matched the full build SHA at the end of verification. [Actions run](https://github.com/blueokanna/Crabforge/actions/runs/45981941463) · [Artifact](https://github.com/blueokanna/Crabforge/actions/runs/45981941463/artifacts/15190149052) · [Pi session samples](7471c96-pi-session-resume.csv)

## 网关设置迁移 / Gateway settings migration

Magpie 0.1.4936 写出的加密备份包含网关端口、浏览器来源及等价模型映射。本地 Windows Debug 双 CLI 通过 `--settings-only` 和 `--settings` 的预览与应用。预览保留原设置字节，旧来源被替换，已有等价模型映射保留。网关在导入的回环端口启动。导入的来源无密钥返回 401，移除的来源即使带密钥也返回 401。导入来源使用有效密钥时，上游先返回 429，再经新映射回退一次并返回 200。测试还检查了非法端口、通配来源和冲突映射均被拒绝且保留原设置。

An encrypted backup written by Magpie 0.1.4936 contained the gateway port, browser origins and equivalent model names. Both local Windows Debug CLIs previewed and applied `--settings-only` and `--settings`. The previews preserved the original settings bytes. Apply replaced the origin list and retained the existing equivalent model mapping. The gateway started on the imported loopback port. The imported origin without a key and the removed origin with a key each received 401. The imported origin with a valid key received 200 after one fallback through the imported equivalent model mapping following an upstream 429. Regression tests rejected invalid ports, wildcard origins and conflicting mappings and checked that the settings file stayed byte for byte intact. [Import commands](../../../README.md#import-providers-from-magpie)

Windows Actions `45978145720` 从完整提交 `5cc51ab575b27a333aa007188c636f4b70508d93` 构建。130 项 Rust 测试及依赖门禁、Clippy、Flutter 分析和桌面构建通过。归档 `15189647470` 为 65,226,059 字节，SHA-256 为 `32fcedfb8fbdee30b0e01a8acaefdbd0280cb85a980743566bb9480e4d181cea`。归档与解压目录的 670 个文件哈希逐一一致，313 个 Rust 许可目录通过核验，六个 PE 的签名状态为 `NotSigned`。两个下载包 CLI 均通过原版加密备份的单项与联合迁移，包含 CORS 鉴权、429 后等价模型回退和四进程竞争。Windows ConPTY 联合预览经验证不会回显中文口令。验包结束时远端 `main` 与上述完整 SHA 一致。

Windows Actions run `45978145720` built `5cc51ab575b27a333aa007188c636f4b70508d93`. The dependency gate, Clippy, 130 Rust tests, Flutter analysis and the desktop build passed. Artifact `15189647470` was 65,226,059 bytes with SHA-256 `32fcedfb8fbdee30b0e01a8acaefdbd0280cb85a980743566bb9480e4d181cea`. All 670 extracted files matched the archive digests, 313 Rust license directories passed verification, and the six PE files had Authenticode status `NotSigned`. Both packaged CLIs passed separate and combined imports from the native encrypted backup, including CORS authentication, equivalent-model fallback after HTTP 429 and four competing processes. The combined preview in Windows ConPTY passed with a Chinese passphrase and no echo. Remote `main` still matched the complete build SHA when verification finished. [Actions run](https://github.com/blueokanna/Crabforge/actions/runs/45978145720) · [Artifact](https://github.com/blueokanna/Crabforge/actions/runs/45978145720/artifacts/15189647470)

## 记忆备份迁移 / Memory backup migration

Magpie 0.1.4936 从隔离的便携配置生成采用中文文件名和中文口令的加密备份。两个 Windows Debug CLI 预览均未创建目标目录，错误口令也未写入文件。四个并发应用中，一个创建四条记忆，其他三个报告文件相同。导入后读取到活跃、候选、归档和已替代四种状态。范围、时间戳、标签、置顶状态及来源引文保留。项目摘要仅包含活跃项目记忆，项目 `AGENTS.md` 保留原始 BOM 和 CRLF。已有不同内容的 ID 使整批应用失败，保留原文件字节。Rust 回归还检查了备份中的删除标记和无效文件名。

Magpie 0.1.4936 wrote an encrypted backup from an isolated portable profile using a Unicode filename and Chinese passphrase. Both Windows Debug CLIs previewed without creating the destination directory, and a wrong passphrase produced no files. One of four competing applies created all four memories, and the other three found identical files. The import preserved the active, candidate, archived and superseded states along with scope, timestamps, tags, pinning and source quotes. The digest included only active project memory, and the project's `AGENTS.md` kept its original BOM and CRLF. A conflicting ID blocked the batch and preserved the existing file bytes. The Rust regression also checked a deletion recorded in the backup and an invalid filename. [Import command](../../../README.md#manage-client-resources)

Windows Actions `45984372547` 从 `38ac6ba98ec0f13317b58060feeff2648cd4c73a` 构建，通过依赖门禁、Clippy、133 项 Rust 测试和 Flutter 检查。归档 `15192330345` 为 65,317,859 字节，SHA-256 为 `38fcaa9c3e6b7ab6dff5babd169a73f04e0d5ea97057b51a41f0d29f5d6090ae`。下载并逐项校验了 670 个文件、313 个 Rust 许可目录、产品资源和双 CLI。六个 PE 均为 `NotSigned`。下载包中的 Release 桌面窗口正常响应并退出，两个 CLI 通过四种记忆状态、原始证据、项目摘要和并发应用回归。验包结束时远端 `main` 匹配上述完整 SHA。

Windows Actions run `45984372547` built `38ac6ba98ec0f13317b58060feeff2648cd4c73a`. The dependency gate, Clippy, 133 Rust tests and Flutter checks passed. Artifact `15192330345` was 65,317,859 bytes with SHA-256 `38fcaa9c3e6b7ab6dff5babd169a73f04e0d5ea97057b51a41f0d29f5d6090ae`. Download verification covered all 670 files, 313 Rust license directories, product resources and both CLIs. All six PE files reported `NotSigned`. The unpacked Release desktop opened a responsive window and exited cleanly. Both CLIs passed the four memory states, original evidence, project digest and concurrent apply regression. Remote `main` matched the full build SHA at the end of verification. [Actions run](https://github.com/blueokanna/Crabforge/actions/runs/45984372547) · [Artifact](https://github.com/blueokanna/Crabforge/actions/runs/45984372547/artifacts/15192330345)

同一归档首次执行四进程联合路由导入时，Windows 偶发拒绝替换 `settings.json`，返回 `os error 5`。重复测试记录了完整错误。固定复现用例持有一个设置读句柄并同时保存。修复后的测试验证原读句柄仍读到旧内容，新读句柄读到完整的新配置，使用旧快照的写入被拒绝。修复后的 Debug CLI 连续通过六轮原版备份回归，每轮分别对供应商、路由组和联合导入做四进程竞争。

The first four-process combined routing import on that archive intermittently failed to replace `settings.json` with Windows `os error 5`. A repeated run captured the full error. A deterministic regression holds a settings reader open during a save. With the fix, the open handle still reads the previous bytes, a new reader gets the complete updated settings, and a save based on the old snapshot is rejected. The updated Debug CLIs completed six consecutive native-backup runs, each exercising four competing processes for the provider, group and combined imports. [Windows regression](../../../crates/core/src/config.rs)

## 子代理与斜杠命令迁移 / Subagent and slash command migration

Magpie 0.1.4936 写出的加密备份包含两条命令和两个子代理。两个 Windows Debug CLI 通过只读预览、按名称选择、拒绝错误口令，以及冲突时保留资源库原始字节。每个区段分别由四个进程竞争应用，一次写入、三次报告相同定义。显式同步保留已有 Claude 命令的 CRLF 文件，Pi 模板保留位置参数，Gemini TOML 使用 `{{args}}`，Codex 和 Claude 读取到各自支持的子代理字段。预览列出未迁移的 `from` 和 `fromHash` 字段。

An encrypted backup written by Magpie 0.1.4936 contained two commands and two subagents. Both Windows Debug CLIs passed read-only preview, name selection, wrong-passphrase rejection and byte-for-byte library preservation on conflict. Four processes competed to apply each section. One wrote the definitions and three reported identical entries. Explicit sync preserved an existing Claude command with CRLF line endings, kept Pi's positional argument, wrote `{{args}}` to Gemini TOML, and rendered the supported subagent fields for Codex and Claude. The preview reported the skipped `from` and `fromHash` fields. [Import commands](../../../README.md#manage-client-resources)

Windows Actions `45987998078` 为 `2ec7e74adc15f96fa1b8ee12328d460235a1ed06` 通过 136 项 Rust 测试、依赖门禁、Clippy 和 Flutter 检查。ZIP `15193886613` 为 65,407,847 字节，SHA-256 为 `a84e180183c818dc076f5b4b84f2de44a237bfc4fe48ce0014c7f6f8af329c5b`，670 项逐文件哈希清单的 SHA-256 为 `ff80dd67d9687c8832fe459e6738d3d4eede1192980190e273cf74c6ce9bc932`。六个 PE 均返回 `NotSigned`。下载包的桌面能启动并正常退出，两个 CLI 通过供应商、路由、设置、ConPTY、记忆及资源迁移，分别恢复 Pi 与 Gemini 的 12 轮原生会话。回环 SSE 首事件三次分别为 `18.419`、`36.299`、`11.225 ms`。验包结束时远端 `main` 匹配该完整 SHA。

Windows Actions run `45987998078` built `2ec7e74adc15f96fa1b8ee12328d460235a1ed06`. The dependency gate, Clippy, 136 Rust tests and Flutter checks passed. Artifact `15193886613` was 65,407,847 bytes with SHA-256 `a84e180183c818dc076f5b4b84f2de44a237bfc4fe48ce0014c7f6f8af329c5b`. The extracted manifest covers 670 files and has SHA-256 `ff80dd67d9687c8832fe459e6738d3d4eede1192980190e273cf74c6ce9bc932`. All six PE files reported `NotSigned`. The packaged desktop opened and closed cleanly. The CLIs passed provider, routing, settings, ConPTY, memory and resource migrations, plus 12 rounds of native resume each for Pi and Gemini. Loopback SSE first-event times were `18.419`, `36.299` and `11.225 ms`. Remote `main` matched the full source SHA at the end of verification. [Actions run](https://github.com/blueokanna/Crabforge/actions/runs/45987998078) · [Artifact](https://github.com/blueokanna/Crabforge/actions/runs/45987998078/artifacts/15193886613)

## 规则迁移 / Rule migration

Magpie 0.1.4936 生成包含两条规则的原生加密备份。两个 Windows Debug CLI 验证只读预览、按名称选取、错误口令、整批冲突检查和四进程并发。并发时一个进程写入完整规则库，其余三个报告相同条目。显式同步将文件匹配规则写入 Kiro，将全局规则写入 Cline。已有同名 Claude 文件保持原始 CRLF 字节，界面状态数据报告冲突。

Magpie 0.1.4936 exported an encrypted backup with two rules. Both Windows Debug CLIs passed read-only preview, name selection, wrong-passphrase rejection, whole-batch conflict checks and four competing applies. One process wrote the complete library and three reported identical entries. Explicit sync published scoped guidance to Kiro and global guidance to Cline. It reported a conflict for an existing Claude file and preserved its original CRLF bytes. [Rules importer](../../../crates/core/src/rules/backup.rs)

环境变量回归分别使用 Magpie 0.1.4936 生成的普通备份和 `--no-keys` 备份。双 CLI 验证按名筛选、值遮蔽、空值、UTF-8 多行值和 Claude Code、Codex、Pi 分配。四进程同时应用时只写入一次，冲突批次保留原库字节。显式同步把值写入 Codex 和 Pi，同时保留 Claude Code 已有的值。

Both CLIs tested environment variables from Magpie 0.1.4936 backups with and without values. They verified name filtering, masked reports, empty values, multiline UTF-8, and client assignments. Four competing applies wrote the batch once, and a conflicting batch preserved the original library bytes. Explicit sync published to Codex and Pi while preserving Claude Code's existing value. [Environment importer](../../../crates/core/src/environment/backup.rs)

## MCP 备份迁移 / MCP backup migration

双 Windows Debug CLI 从 Magpie 0.1.4936 加密备份导入 stdio、HTTP 和 SSE 定义。预览隐藏命令参数、URL、环境变量值和请求头。四进程并发只写入一次，已有不同定义和备份中启用的工具限制均阻止写入。导入后显式分配到含中文及空格的项目目录，Codex、Gemini 和 Claude Code 分别获得对应的原生配置。

Both Windows Debug CLIs imported stdio, HTTP and SSE definitions from a Magpie 0.1.4936 encrypted backup. Reports omit command arguments, URLs, environment values and headers. Four competing processes wrote the library once. Conflicting definitions and active tool restrictions blocked the import. Explicit assignments generated native Codex, Gemini and Claude Code configuration in a project path containing Chinese characters and a space. [MCP importer](../../../crates/core/src/project_mcp/library/backup.rs)

Windows Actions `45992223719` 从 `7fd0552e6b2a069381627241e16e0f30f964cba6` 构建，通过依赖门禁、Clippy、141 项 Rust 测试和 Flutter 检查。ZIP `15195319324` 为 65,532,592 字节，SHA-256 为 `6a1501d8826f2ffda797457fcd07e670fe4168222ffc8d01e371520764255412`。670 项逐文件清单的 SHA-256 为 `1c858d42390a6b9f0ccfbd1f3b51b79c4dc13061737c75de33d0c00a5b319d4d`。桌面通过启动和退出测试，六个 PE 均返回 `NotSigned`。双 CLI 通过供应商、路由、设置、口令无回显、记忆及命令、子代理、规则、环境、MCP 回归。Pi 和 Gemini 分别恢复 12 轮原生会话。验收结束时远端 `main` 匹配上述完整 SHA。

Windows Actions run `45992223719` built `7fd0552e6b2a069381627241e16e0f30f964cba6`. The dependency gate, Clippy, 141 Rust tests and Flutter checks passed. Artifact `15195319324` was 65,532,592 bytes with SHA-256 `6a1501d8826f2ffda797457fcd07e670fe4168222ffc8d01e371520764255412`. The 670-file manifest has SHA-256 `1c858d42390a6b9f0ccfbd1f3b51b79c4dc13061737c75de33d0c00a5b319d4d`. The desktop opened and exited cleanly, and all six PE files reported `NotSigned`. Both CLIs passed provider, routing, settings, hidden passphrase entry, memory, commands, subagents, rules, environment and MCP regressions. Pi and Gemini each resumed 12 native turns. Remote `main` matched the full build SHA at the end of verification. [Actions run](https://github.com/blueokanna/Crabforge/actions/runs/45992223719) · [Artifact](https://github.com/blueokanna/Crabforge/actions/runs/45992223719/artifacts/15195319324)

下载包的回环 SSE 首事件为 `20.984`、`14.220` 和 `6.363 ms`。每轮返回 HTTP 200、向上游请求一次且未使用代理。

The unpacked CLI's three loopback SSE first events arrived at `20.984`, `14.220` and `6.363 ms`. Each trial returned HTTP 200, made one upstream request and sent zero proxy requests. [Raw timings](7fd0552-forward-h1.csv)

## 插件备份迁移 / Plugin backup migration

两个 Windows Debug CLI 读取了便携版 Magpie 0.1.4936 生成的加密备份。Claude Code 配置保留已有停用项和同名本地市场，报告受影响的插件，重复应用保持文件字节一致。共享插件的四个并发应用仅创建一次快照，导出的 TAR 逐文件匹配原件，包括 CRLF 和中文路径的二进制附件。测试覆盖同名不同内容的冲突、仅有远程来源的条目，以及 Codex 项目的显式分配和撤回。

Both Windows Debug CLIs read an encrypted backup produced by portable Magpie 0.1.4936. The Claude Code import preserved a disabled plugin and a colliding local marketplace, reported affected plugins, and left the file byte-identical on reapply. Four competing shared bundle imports created one snapshot. The exported TAR matched every original file, including CRLF and a binary attachment with a Chinese filename. The fixture covered same-name content conflicts, a source-only entry, and explicit Codex project assignment and withdrawal. [Claude Code settings](../../../crates/core/src/claude_plugins/transfer.rs) · [Shared bundles](../../../crates/core/src/plugin_bundles/backup.rs)

Windows Actions `46000848429` 从 `7bbf8f670ca873bb121c598535157d842beb1cd0` 构建，job `140511793049` 通过依赖门禁、Clippy、143 项 Rust 测试和 Flutter 检查。ZIP `15200610934` 为 65,732,126 字节，SHA-256 为 `5633e843bfd7f92c5fe1b49827c959f36392689defbbf2278bba4eadc7b1f435`。670 项逐文件清单的 SHA-256 为 `77407e342201eb982c8125b28e14b5d697d3a7d4a5ac95bc7757b32c8f466390`。桌面正常启动和退出，六个 PE 均返回 `NotSigned`。双 CLI 通过原版备份的插件与先前资源迁移回归，Pi 和 Gemini 各恢复 12 轮原生会话。验收末尾远端 `main` 匹配该完整构建 SHA。

Windows Actions run `46000848429` built `7bbf8f670ca873bb121c598535157d842beb1cd0`. Job `140511793049` passed the dependency gate, Clippy, 143 Rust tests and Flutter checks. Artifact `15200610934` was 65,732,126 bytes with SHA-256 `5633e843bfd7f92c5fe1b49827c959f36392689defbbf2278bba4eadc7b1f435`. The 670-file manifest has SHA-256 `77407e342201eb982c8125b28e14b5d697d3a7d4a5ac95bc7757b32c8f466390`. The desktop opened and closed cleanly, and all six PE files reported `NotSigned`. Both CLIs passed the native plugin backup and earlier resource migrations. Pi and Gemini each resumed 12 native turns. Remote `main` matched the full build SHA at the end of verification. [Job](https://github.com/blueokanna/Crabforge/actions/runs/46000848429/job/140511793049) · [Artifact](https://github.com/blueokanna/Crabforge/actions/runs/46000848429/artifacts/15200610934)

### 确定性路由规则 / Deterministic routing rules

回环网关回归从外层组进入轮询子组，核对按消息、推理级别、工作目录、子代理和图像进行选路。首选成员返回 503 时尝试规则内后备成员，暂停的成员被跳过。手选成员保持优先，整个组暂停时返回 503。夏令时回归核对纽约周五跨午夜时段在换季前后的 UTC 起止点。Windows Debug 桌面通过 `flutter_rust_bridge` 在 560 和 1,200 逻辑像素下保存规则。四项 Flutter 单测与分析通过。 [网关测试](../../../crates/core/tests/gateway.rs) · [桌面测试](../../../gui/integration_test/console_layout_test.dart)

The loopback gateway regression reached a rotating subgroup through its parent and verified routing on message text, reasoning effort, working folder, subagents and images. A preferred member's 503 led to the rule's next member, and matching pauses excluded their members. Manual selection took precedence, and a fully paused group returned 503. The daylight-saving check verified the UTC boundaries of a Friday overnight window in New York before and after the transition. The Windows Debug desktop saved rules through `flutter_rust_bridge` at 560 and 1,200 logical pixels. Flutter analysis and four unit tests passed. [Gateway test](../../../crates/core/tests/gateway.rs) · [Desktop test](../../../gui/integration_test/console_layout_test.dart)

Windows 工作区通过 149 项 Rust 测试，33 项外部或专项测试保持忽略。全特性 Clippy 通过。两个 Debug CLI 均从 Magpie 0.1.4936 原版加密备份恢复规则，并验证分享导入、组复制和改名、供应商改名、无效成员原子拒绝、清空和删除。带条件与不带条件的流式请求均按预期选路，保留上游 SSE 字节。Windows Debug CLI 将命令行解析放到具有独立 4 MiB 栈的线程上，两个入口的启动和子命令回归均通过。 [CLI](../../../crates/cli/src/lib.rs) · [备份测试](../../../crates/core/src/config/backup.rs)

The Windows workspace passed 149 Rust tests, with 33 external or specialized tests ignored. All-feature Clippy passed. Both Debug CLI entry points restored rules from an encrypted Magpie 0.1.4936 backup and verified share reimport, group copy and rename, provider rename, atomic rejection of an invalid member, clearing and removal. Matching and nonmatching requests selected the expected routes and preserved upstream SSE bytes. Windows command-line parsing now uses a dedicated thread with a 4 MiB stack. Both Debug entry points passed startup and subcommand checks. [CLI](../../../crates/cli/src/lib.rs) · [Backup test](../../../crates/core/src/config/backup.rs)

全目标和 Windows x64 依赖图分别有 423 和 343 个启用的包版本，包含五项精确锁定依赖。两图均无 Serde 系列、Hyper 和 Reqwest。Jiff、Jiff Core 与 Jiff TZDB 原件均附 `LICENSE-MIT`，其 SHA-256 为 `0f96a83840e146e43c0ec96a22ec1f392e0680e6c1226e6f3ba87e0740af850f`。RustSec 快照 `8b75c83c3c9142dd18e93d75b8d895a13d285861` 对锁文件报告零已知漏洞，并继续报告非 Windows netlink 依赖 `paste 1.0.15` 停止维护。 [许可证打包](../../../.github/scripts/stage-windows.ps1) · [RUSTSEC-2024-0436](https://rustsec.org/advisories/RUSTSEC-2024-0436.html)

The all-target and Windows x64 dependency graphs contained 423 and 343 active package versions. Both graphs included the five pinned dependencies and were free of Serde-family crates, Hyper and Reqwest. The upstream `LICENSE-MIT` files for Jiff, Jiff Core and Jiff TZDB share SHA-256 `0f96a83840e146e43c0ec96a22ec1f392e0680e6c1226e6f3ba87e0740af850f`. RustSec snapshot `8b75c83c3c9142dd18e93d75b8d895a13d285861` reported zero known vulnerabilities in the lockfile and retained the maintenance warning for the non-Windows netlink dependency `paste 1.0.15`. [License staging](../../../.github/scripts/stage-windows.ps1) · [RUSTSEC-2024-0436](https://rustsec.org/advisories/RUSTSEC-2024-0436.html)

### 供应商繁忙重试 / Direct provider retries

两个 Windows Debug CLI 均从 Magpie 0.1.4936 加密备份保留供应商的 `patience`。验收覆盖 `0`、`-1`、超出范围时设置字节不变、现有 `fallback-for-others` 命令、直接请求 503 后按预算重试并返回完整 SSE，以及路由组关闭重试。149 项工作区 Rust 测试、全特性 Clippy、Flutter 分析和四项单测通过。Windows Debug 桌面通过 `flutter_rust_bridge` 在 560 和 1,200 逻辑像素下分别保存 `0` 和 `7` 秒。 [网关回归](../../../crates/core/tests/gateway.rs) · [桌面回归](../../../gui/integration_test/console_layout_test.dart)

Both Windows Debug CLIs preserved provider `patience` from an encrypted Magpie 0.1.4936 backup. The checks covered `0`, `-1`, unchanged settings bytes after an out-of-range input, the existing `fallback-for-others` command, a direct request that retried a 503 and returned the complete SSE response, and a group that disabled retries. The workspace passed 149 Rust tests, all-feature Clippy, Flutter analysis and four Flutter unit tests. The Windows Debug desktop saved `0` and `7` seconds at 560 and 1,200 logical pixels respectively through `flutter_rust_bridge`. [Gateway regression](../../../crates/core/tests/gateway.rs) · [Desktop regression](../../../gui/integration_test/console_layout_test.dart)

### 意图分类与候补 / Intent classification and backup classifiers

两版 Windows Debug CLI 从 Magpie 0.1.4936 生成的加密备份迁移 `intent`、`classifier`、`classifierMore`、`classifyPrompt` 和 `classifyTimeout`。回归通过分享导回、分类器组改名、规则对象编辑、无效引用及超时值原子拒绝、无效回复转候补、`0` 回复和未进入的后备组不调用分类器。网关回归另覆盖主分类器超时、手选优先和上一轮对话节选。Rust 通过 150 项，忽略 33 项，全目标 Clippy 通过。Flutter 分析和四项单测通过，Windows 桌面通过 FRB 在 560 和 1,200 逻辑像素下保存意图规则和网络超时。 [网关回归](../../../crates/core/tests/gateway.rs) · [桌面回归](../../../gui/integration_test/console_layout_test.dart)

Both Windows Debug CLIs imported `intent`, `classifier`, `classifierMore`, `classifyPrompt` and `classifyTimeout` from an encrypted backup generated by Magpie 0.1.4936. Checks covered round-tripping a share, renaming a classifier group, editing a rule object, byte-identical settings after invalid references or timeouts, trying a backup after an invalid answer, accepting `0` and leaving unused backup classifiers uncalled. The gateway regression also covered a primary classifier timeout, manual selection and excerpts of the previous exchange. Rust reported 150 passed and 33 ignored, and all-target Clippy passed. Flutter analysis and four unit tests passed. The Windows desktop saved intent rules and network timeouts through FRB at 560 and 1,200 logical pixels. [Gateway regression](../../../crates/core/tests/gateway.rs) · [Desktop regression](../../../gui/integration_test/console_layout_test.dart)

同一会话两次并发请求只启动一轮主分类器及候补，追加工具结果时复用同轮判断，用户消息变化后重新判断。两个 Debug CLI 均通过正向意图和 `0` 答复的工具续轮检查。Rust 通过 150 项，忽略 33 项，Clippy 通过。全目标和 Windows x64 的启用依赖分别为 423 和 343 个唯一版本，五项锁定依赖均启用，禁用项为零。 [网关回归](../../../crates/core/tests/gateway.rs)

Two concurrent requests for the same named session and user turn shared one primary and backup classifier sequence. A request with appended tool results reused the verdict, and a different user message classified again. Both Debug CLIs passed tool continuation checks for a matched intent and a `0` verdict. Rust reported 150 passed and 33 ignored, and Clippy passed. The all-target and Windows x64 graphs contained 423 and 343 unique package versions respectively, including all five pinned dependencies and zero prohibited packages. [Gateway regression](../../../crates/core/tests/gateway.rs)

### 路由规则推理等级 / Reasoning levels in routing rules

两个 Windows Debug CLI 均从 Magpie 0.1.4936 原版加密备份恢复 `useEffort` 和 `modelEfforts`。回归核对分享、供应商改名、自定义等级的设置与清除、上游目录的无效等级和重复项过滤，以及目录等级优先。网关回归核对每次尝试的实际请求，覆盖嵌套回退、同距向上拟合、未知模型等级、关闭推理的 Responses 请求，以及 Anthropic 自适应模式和预算上限。Rust 通过 154 项，忽略 33 项，全目标 Clippy 通过。Flutter 分析与四项单测通过。 [网关回归](../../../crates/core/tests/gateway.rs)

Both Windows Debug CLIs restored `useEffort` and `modelEfforts` from a native encrypted Magpie 0.1.4936 backup. The fixture checked sharing, provider rename, setting and clearing custom levels, filtering invalid and duplicate catalog levels, and catalog precedence. The gateway regression inspected the actual requests for nested fallback, an upward tie, unknown levels, a Responses request with reasoning disabled, and adaptive and budget-limited Anthropic thinking. Rust reported 154 passed and 33 ignored. All-target Clippy, Flutter analysis and four Flutter unit tests passed. [Gateway regression](../../../crates/core/tests/gateway.rs)

Windows Debug 桌面通过 `flutter_rust_bridge` 在 1,200 和 560 逻辑像素宽度下分别保存 `medium` 和 `high`，两次编辑供应商时均保留已有的目录推理等级。 [桌面回归](../../../gui/integration_test/console_layout_test.dart)

The Windows Debug desktop saved `medium` and `high` at 1,200 and 560 logical pixels respectively through `flutter_rust_bridge`. Editing the provider at either width preserved its saved catalog levels. [Desktop regression](../../../gui/integration_test/console_layout_test.dart)

### 自动推理等级 / Automatic reasoning effort

Rust 全工作区通过 155 项测试，忽略 33 项，全目标 Clippy 通过。网关回归覆盖无规则组、无效分类输出后的备选分类器、意图与等级合并判定、工具结果续轮和嵌套继承。双 Debug CLI 从 Magpie 0.1.4936 原版加密备份恢复 `effort: auto`，经分享往返后实测同轮复用、目录等级拟合及显式关闭推理。 [网关回归](../../../crates/core/tests/gateway.rs)

Rust passed 155 workspace tests with 33 ignored. All-target Clippy passed. The gateway regression covered a group without rules, fallback after malformed classifier output, combined intent and effort classification, tool-result continuation, and nested inheritance. Both Debug CLIs restored `effort: auto` from a native encrypted Magpie 0.1.4936 backup, round-tripped it through sharing, and checked same-turn reuse, fitting to catalog levels, and explicitly disabled reasoning. [Gateway regression](../../../crates/core/tests/gateway.rs)

Windows Debug 桌面通过完整控制台集成测试，经 `flutter_rust_bridge` 在 1,200 和 560 逻辑像素下保存 `effort: auto`、分类器、规则和重试时限。测试窗口被最小化时未继续渲染，恢复窗口后测试完成。 [桌面回归](../../../gui/integration_test/console_layout_test.dart)

The Windows Debug desktop passed the full console integration test. At 1,200 and 560 logical pixels it saved `effort: auto`, the classifier, rules, and retry limits through `flutter_rust_bridge`. Rendering stalled while the test window was minimized. The test completed after the window was restored. [Desktop regression](../../../gui/integration_test/console_layout_test.dart)

### 工具续轮的路由决策 / Routing decisions across tool calls

Windows Debug 的 156 项 Rust 测试和全目标 Clippy 通过，33 项测试忽略。回归核对工具续轮保留组规则和推理等级、按会话隔离、实时暂停、用户新消息重新路由及容量增长后切换。两个 CLI 均从 Magpie 0.1.4936 的加密备份导入供应商级和模型级 `contexts`，并验证请求达到阈值后选择容量更大的模型。 [网关回归](../../../crates/core/tests/gateway.rs)

Windows Debug passed 156 Rust tests and all-target Clippy, with 33 tests ignored. The regression checked that tool continuations retain the group's rule and reasoning level, decisions stay scoped to a session, pauses apply immediately, a new user message routes afresh, and an oversized request can switch to a larger context. Both CLIs imported provider and model `contexts` from an encrypted Magpie 0.1.4936 backup and verified that the request switched models at the threshold. [Gateway regression](../../../crates/core/tests/gateway.rs)

Windows Actions Run `46043024572` 构建了 `c9ac02f8974057761a9fc17758aaef2a7d27ac45`。CI 通过 156 项 Rust 测试，忽略 33 项。ZIP 为 67,042,883 字节，SHA-256 为 `c0a5b26f6b3bfc24abd1f75bc9b5bcc62826c820a142e0e41872f3649298e4b3`。676 项解包清单的 SHA-256 为 `84d8b33a0cf48ef1514b7924206ee2dcfc58cdee2f02486344acd5b3b28d713f`，六个 PE 均为 `NotSigned`。Release 桌面正常启动和退出，双 CLI 通过工具续轮和容量切换夹具。Pi `1.6.0`、Gemini CLI `0.71.1`、Grok `1.0.92` 和 Oh My Pi `19.2.1` 各通过 12 轮。恢复验收时保留了 Magpie `0.1.4987` 已通过的备份和设置结果，记忆、资源、Grok 钩子和路由用 `0.1.4936` 验证。验后通过代理核对远端 `main` 与完整构建 SHA 一致。 [Job](https://github.com/blueokanna/Crabforge/actions/runs/46043024572/job/140655254788) · [Artifact](https://github.com/blueokanna/Crabforge/actions/runs/46043024572/artifacts/15221444943)

Windows Actions run `46043024572` built `c9ac02f8974057761a9fc17758aaef2a7d27ac45` and passed 156 Rust tests with 33 ignored. The 67,042,883-byte ZIP has SHA-256 `c0a5b26f6b3bfc24abd1f75bc9b5bcc62826c820a142e0e41872f3649298e4b3`. The 676-entry extracted manifest has SHA-256 `84d8b33a0cf48ef1514b7924206ee2dcfc58cdee2f02486344acd5b3b28d713f`. All six PE files reported `NotSigned`. The Release desktop opened and closed cleanly, and both CLIs passed the tool continuation and context growth fixture. Pi `1.6.0`, Gemini CLI `0.71.1`, Grok `1.0.92` and Oh My Pi `19.2.1` each passed 12 turns. The resumed acceptance retained the passing Magpie `0.1.4987` backup and settings results. Memory, resources, Grok hooks and routing used `0.1.4936`. The proxied remote `main` check matched the full build SHA after acceptance. [Job](https://github.com/blueokanna/Crabforge/actions/runs/46043024572/job/140655254788) · [Artifact](https://github.com/blueokanna/Crabforge/actions/runs/46043024572/artifacts/15221444943)

### 原生 Gemini 推理 / Native Gemini reasoning

双 Windows CLI 从 Magpie 0.1.4936 原版加密备份导入组规则。回环上游逐次记录 Gemini 4 Neon 的 `XHIGH`、Gemini 2.5 Flash 回退的 `24576` token 预算和客户端显式传入的 `thinkingBudget: 0`。Rust 回归覆盖 Gemini 3.1 Pro 的自动 `MEDIUM` 及两种字段命名。真实 Google 端点仍待有凭据的验收。 [网关回归](../../../crates/core/tests/gateway.rs) · [Gemini 思考配置](https://ai.google.dev/gemini-api/docs/thinking)

Both Windows CLIs imported the group rule from an encrypted Magpie 0.1.4936 backup. The loopback upstream recorded `XHIGH` on Gemini 4 Neon, a 24,576-token budget on the Gemini 2.5 Flash fallback, and the client's explicit `thinkingBudget: 0`. The Rust regression covered automatic `MEDIUM` on Gemini 3.1 Pro and both naming conventions. Live Google endpoints still require credentialed acceptance. [Gateway regression](../../../crates/core/tests/gateway.rs) · [Gemini thinking configuration](https://ai.google.dev/gemini-api/docs/thinking)

Windows Actions Run `46103929325` 构建了 `ce6283dd467b383c0cf76d9a5d7f935e44e91891`，157 项 Rust 测试通过，33 项忽略。ZIP 为 67,055,119 字节，SHA-256 为 `acb96d4f67451a1b92b514dd24e0e52b4dab92521c39b8beff8b2af1accce4b6`。676 项解包清单的 SHA-256 为 `f2b6386dc1836899ea5c9f3ce267887aa1ff1083108899296098c115337de753`，六个 PE 均为 `NotSigned`。Release 桌面首次关闭超过 15 秒，同包复测正常退出。两个下载版 CLI 通过原生 Gemini 推理及路由续轮夹具。Pi `1.6.0`、Gemini CLI `0.71.1`、Grok `1.0.92` 和 Oh My Pi `19.2.1` 各通过 12 轮。Grok 复测让回环上游等待钩子标记后再结束响应。验收使用 Magpie `0.1.4936`，验后经代理复核远端 `main` 与完整构建 SHA 一致。 [Job](https://github.com/blueokanna/Crabforge/actions/runs/46103929325/job/140852336250) · [Artifact](https://github.com/blueokanna/Crabforge/actions/runs/46103929325/artifacts/15247036818)

Windows Actions run `46103929325` built `ce6283dd467b383c0cf76d9a5d7f935e44e91891` and passed 157 Rust tests with 33 ignored. The 67,055,119-byte ZIP has SHA-256 `acb96d4f67451a1b92b514dd24e0e52b4dab92521c39b8beff8b2af1accce4b6`. The 676-entry extracted manifest has SHA-256 `f2b6386dc1836899ea5c9f3ce267887aa1ff1083108899296098c115337de753`. All six PE files reported `NotSigned`. The Release desktop exceeded the initial 15-second close timeout and exited cleanly on a same-package retry. Both downloaded CLIs passed the native Gemini reasoning and tool continuation fixtures. Pi `1.6.0`, Gemini CLI `0.71.1`, Grok `1.0.92` and Oh My Pi `19.2.1` each passed 12 turns. On the Grok retry, the loopback upstream waited for the hook markers before ending its response. Acceptance used Magpie `0.1.4936`, and the proxied remote `main` check matched the full build SHA after acceptance. [Job](https://github.com/blueokanna/Crabforge/actions/runs/46103929325/job/140852336250) · [Artifact](https://github.com/blueokanna/Crabforge/actions/runs/46103929325/artifacts/15247036818)

### 跨协议推理 / Cross-protocol reasoning

Windows Debug 通过 158 项 Rust 测试和全目标 Clippy，33 项测试忽略。原生 Gemini 夹具核对了 SSE 回退前后的请求和响应字节，并检查未知模型沿用客户端参数。跨协议夹具核对 OpenAI Chat、Responses、Anthropic Messages、Gemini 的目标参数和显式关闭。十二个流式转换方向均在上游结束前交付首个正文事件。转译响应保留可表示的正文与用量，推理块、摘要和签名留在原生协议内。 [网关回归](../../../crates/core/tests/gateway.rs) · [协议回归](../../../crates/core/tests/protocols.rs)

Windows Debug passed 158 Rust tests and all-target Clippy, with 33 tests ignored. The native Gemini fixture checked the requests and SSE response body across fallback, and verified that an unknown model receives the client's settings. The translated fixture checked destination reasoning controls and explicit off across OpenAI Chat, Responses, Anthropic Messages and Gemini. All twelve streaming directions delivered visible text before the upstream finished. Translated replies included representable content and usage. Reasoning blocks, summaries and signatures require their native protocol. [Gateway regression](../../../crates/core/tests/gateway.rs) · [Protocol regression](../../../crates/core/tests/protocols.rs)

两个 Windows CLI 都从同一原版备份导入路由，核对 Chat `low` 在 Gemini 4 Neon 上发送 `XHIGH`，回退时在 Gemini 2.5 Flash 上发送 `24576` token 预算。Chat `none` 跳过无法关闭推理的 Neon，对 Flash 发送零预算并收到转换后的 Chat 回复。

Both Windows CLIs imported the route from the original backup. Chat `low` sent `XHIGH` to Gemini 4 Neon and a 24,576-token budget to Gemini 2.5 Flash on fallback. Chat `none` skipped Neon, sent a zero budget to Flash and returned a translated Chat reply.

Windows Actions Run `46106961315` 构建了 `96e695b1691bf3498928ab91bf7051507b387a5c`，158 项 Rust 测试通过，33 项忽略。ZIP 为 67,090,161 字节，SHA-256 为 `320af3404c4d4beab5cba8778768329146409e5626dcc4f6b057984e9b22abcc`。676 项解包清单的 SHA-256 为 `9cd668f56106136e58f4cf335f5ac85f34f191ef0e200b31facd1603cefcf7e1`，六个 PE 均为 `NotSigned`。Release 桌面正常启动并在收到关闭请求后 0.23 秒退出。两个下载版 CLI 均通过跨协议 Gemini 推理回退和显式关闭测试，Pi `1.6.0`、Gemini CLI `0.71.1`、Grok `1.0.92` 和 Oh My Pi `19.2.1` 各通过 12 轮。验后经代理复核远端 `main` 已前进到 `f489e94db5232003d856085d0b449161b3f8f9f2`，git 核实该分支含完整构建 SHA。 [Job](https://github.com/blueokanna/Crabforge/actions/runs/46106961315/job/140861761143) · [Artifact](https://github.com/blueokanna/Crabforge/actions/runs/46106961315/artifacts/15248512305)

Windows Actions run `46106961315` built `96e695b1691bf3498928ab91bf7051507b387a5c` and passed 158 Rust tests with 33 ignored. The 67,090,161-byte ZIP has SHA-256 `320af3404c4d4beab5cba8778768329146409e5626dcc4f6b057984e9b22abcc`. The 676-entry extracted manifest has SHA-256 `9cd668f56106136e58f4cf335f5ac85f34f191ef0e200b31facd1603cefcf7e1`. All six PE files reported `NotSigned`. The Release desktop opened and exited 0.23 seconds after the close request. Both downloaded CLIs passed cross-protocol Gemini reasoning fallback and explicit-off tests. Pi `1.6.0`, Gemini CLI `0.71.1`, Grok `1.0.92` and Oh My Pi `19.2.1` each passed 12 turns. After acceptance, the proxied check found remote `main` at `f489e94db5232003d856085d0b449161b3f8f9f2`, and Git confirmed the full build SHA was an ancestor. [Job](https://github.com/blueokanna/Crabforge/actions/runs/46106961315/job/140861761143) · [Artifact](https://github.com/blueokanna/Crabforge/actions/runs/46106961315/artifacts/15248512305)

### 对话附件转译 / Conversation attachment translation

Windows Debug 通过 159 项 Rust 测试和全目标 Clippy，33 项忽略。回环测试在四种协议的十二个转换方向中核对了内联 PNG、MIME 类型和正文顺序，并验证可表示的 HTTP 图片 URL 直传及无法表示的字段返回 400。双 CLI 均通过 Chat 和 Gemini 之间的 PNG 和 WAV 转译，核对了工具结果在录音前发送、Chat SSE 结束帧及 Gemini JSON 回复。OpenCode Go 的原协议回退仍逐字节保留显式图片细节设置。 [协议回归](../../../crates/core/tests/protocols.rs) · [原协议回退](../../../crates/core/src/gateway/console_test.rs)

Windows Debug passed 159 Rust tests and all-target Clippy, with 33 tests ignored. Loopback tests checked inline PNG, MIME type, and text order in all twelve directions between the four protocols. They also verified that representable HTTP image URLs pass through and unrepresentable fields return HTTP 400. Both CLIs converted PNG and WAV input between Chat and Gemini, preserved tool-result order, and returned the Chat SSE terminator and a Gemini JSON reply. OpenCode Go's original-protocol fallback preserved the explicit image-detail setting byte for byte. [Protocol regression](../../../crates/core/tests/protocols.rs) · [Original-protocol fallback](../../../crates/core/src/gateway/console_test.rs)

历史验证 Run `46108753077` 构建了 `657f6361f2cc816fb653a6ea978825f205a307e9`。159 项 Rust 测试通过，33 项忽略。下载包通过 27 项夹具开关对应的检查，包括双 CLI 媒体转译。六个 PE 均为 `NotSigned`。验后经代理复核远端 `main`，并确认它包含该构建 SHA。

Historical validation run `46108753077` built `657f6361f2cc816fb653a6ea978825f205a307e9`. It passed 159 Rust tests with 33 ignored. The downloaded package passed the checks selected by 27 fixture flags, including media translation through both CLIs. All six PE files reported `NotSigned`. The proxied check confirmed that remote `main` contained the build SHA after acceptance.

| ZIP bytes | ZIP SHA-256 | 解包项数 / Extracted files | 清单 SHA-256 / Manifest SHA-256 | 验后远端 main / Remote main after acceptance | GitHub Actions |
| ---: | --- | ---: | --- | --- | --- |
| 67,118,138 | `a3964adfe9409732d8c09b0cc2d383a032f2158a379786b58dd2f77e7f023a52` | 676 | `61d5d478ae3af3a58125fa0268b7e27bdfbbc701351c58c8323087d640dc6c43` | `cf76ad373c90a14abfc32a3803ec970767404d8c` | [Job 140867331943](https://github.com/blueokanna/Crabforge/actions/runs/46108753077/job/140867331943) · [Artifact 15248859269](https://github.com/blueokanna/Crabforge/actions/runs/46108753077/artifacts/15248859269) |

### Gemini 语音合成 / Gemini speech synthesis

Windows Debug 通过 160 项 Rust 测试及全目标 Clippy，33 项忽略。回环夹具通过两个 CLI 核对 `generateContent` 的音色和朗读提示、WAV 文件头、PCM 原始采样及 SSE 请求的 400 响应。Rust 回归还验证了畸形 PCM 触发组回退、带额外 RIFF 块的 WAV 可以转换为 PCM，以及原生 OpenAI 兼容中转返回原音频。Google AI Studio 和 Vertex AI 实际端点的验收需要供应商凭据。 [媒体回归](../../../crates/core/tests/media.rs)

Windows Debug passed 160 Rust tests and all-target Clippy, with 33 tests ignored. A loopback fixture exercised both CLIs and checked the `generateContent` voice and spoken prompt, the WAV header, exact PCM samples, and HTTP 400 for an SSE request. The Rust regression also covered group fallback after malformed PCM, PCM extraction from WAV with an extra RIFF chunk, and audio passthrough on a native OpenAI-compatible relay. Live Google AI Studio and Vertex AI verification requires provider credentials. [Media regression](../../../crates/core/tests/media.rs)

### 意图置信度 / Intent confidence

Windows Debug 通过 160 项 Rust 测试和全目标 Clippy，33 项忽略。两个 CLI 均从 Magpie 0.1.4936 原版加密备份导入 `intentSure`，核对了分享、拒绝越界阈值并保留磁盘设置，以及置信度恰好达到阈值时命中意图。Rust 回归覆盖低置信度续轮、非法置信度触发候补、显式 `0` 和自动推理等级。Windows 桌面通过 `flutter_rust_bridge` 在 1,200 和 560 逻辑像素下分别保存 `0.6` 和 `0`。Flutter 分析、四项单测和桌面布局集成测试均通过。 [网关回归](../../../crates/core/tests/gateway.rs) · [桌面测试](../../../gui/integration_test/console_layout_test.dart)

Windows Debug passed 160 Rust tests and all-target Clippy, with 33 tests ignored. Both CLIs imported `intentSure` from an encrypted Magpie 0.1.4936 backup, checked sharing, rejected an out-of-range threshold without changing the saved configuration, and matched an intent at the exact threshold. The Rust regression covered low-confidence tool continuations, backup classifiers after invalid confidence, explicit `0`, and automatic reasoning effort. The Windows desktop saved `0.6` at 1,200 logical pixels and `0` at 560 through `flutter_rust_bridge`. Flutter analysis, four unit tests, and the Windows layout integration test passed. [Gateway regression](../../../crates/core/tests/gateway.rs) · [Desktop test](../../../gui/integration_test/console_layout_test.dart)

### DashScope 图片生成与编辑 / DashScope image generation and edits

Windows Debug 通过 161 项 Rust 测试和全目标 Clippy，33 项忽略。回环夹具验证两个 CLI 从配置的 Anthropic 兼容地址选择 DashScope 原生路径，使用 Bearer 认证，返回生成 URL，并将 multipart 图片转成原生请求。`b64_json` 返回的 PNG 与夹具字节一致，下载请求未携带供应商令牌。Rust 回归另覆盖 Chat 兼容地址、多图顺序、不支持的遮罩、非允许来源及重定向拒绝，以及上游未返回图片时的组回退。实际 DashScope 端点验收需要供应商凭据。 [媒体回归](../../../crates/core/tests/media.rs)

Windows Debug passed 161 Rust tests and all-target Clippy, with 33 tests ignored. The loopback fixture verified that both CLIs selected the native DashScope path from an Anthropic-compatible base, sent Bearer authentication, returned the generated URL, and converted a multipart file to native input. The `b64_json` PNG matched the fixture bytes. The download carried no provider credential. Rust coverage also included a Chat-compatible base, image order, mask rejection, download origin and redirect checks, and group fallback when the upstream returned no image. Live DashScope acceptance requires provider credentials. [Media regression](../../../crates/core/tests/media.rs)

### DashScope 语音合成与转写 / DashScope speech synthesis and transcription

Windows Debug 通过 162 项 Rust 测试和全目标 Clippy，33 项忽略。两个 CLI 通过回环验证默认音色、朗读指令、语速下限、MP3 下载字节和 multipart WAV 转写，并核对 `verbose_json` 的语言、时长和正文。Rust 回归还覆盖语速上限、内联 PCM、畸形 Base64 触发组回退、普通 JSON 和纯文本转写，以及 SSE、FLAC、SRT 和翻译请求的 400 响应。实际语音模型验收需要 DashScope 凭据。 [媒体回归](../../../crates/core/tests/media.rs)

Windows Debug passed 162 Rust tests and all-target Clippy, with 33 tests ignored. Both CLIs verified the default voice, speech instructions, lower rate bound, downloaded MP3 bytes, and multipart WAV transcription on loopback. The `verbose_json` response preserved the language hint, duration, and text. Rust coverage also included the upper rate bound, inline PCM, group fallback after malformed Base64, JSON and plain text transcripts, and HTTP 400 for SSE, FLAC, SRT, and translation requests. Live speech model acceptance requires DashScope credentials. [Media regression](../../../crates/core/tests/media.rs)

### MiniMax 与火山方舟图片 / MiniMax and Volcengine Ark images

Windows TLS 夹具验证 MiniMax 在已知区域端点返回 HTTP 200 和业务码 `1002` 时，网关以 429 记录该次尝试并切换组成员，路由记录为 `[429, 200]`。火山方舟夹具验证 multipart 编辑转 JSON、参考图顺序、三次绘图中的两次成功和用量合并。Seedream 5 Pro 的 `4K` 与 `16:9` 请求先发送 `2816x1584`，收到 HTTP 422 后改用 `2848x1600` 并返回上游图片链接。真实端点测试需要对应供应商凭据。 [TLS 回归](../../../crates/core/src/gateway/console_test.rs)

The Windows TLS fixture verified that MiniMax business code `1002` in an HTTP 200 response records a 429 and switches group members, producing attempt statuses `[429, 200]`. The Ark fixture checked multipart edits as JSON, reference image order, two successful drawings out of three, and combined usage. A `4K`, `16:9` request for Seedream 5 Pro first sent `2816x1584`. After HTTP 422 the gateway sent `2848x1600` and returned the upstream image URL. Live tests require provider credentials. [TLS regressions](../../../crates/core/src/gateway/console_test.rs)

### 媒体构建下载验收 / Media build download acceptance

下表两次验收均核对下载 ZIP、逐文件清单、产品 PNG 及 Release 桌面启动和退出。六个 PE 均为 `NotSigned`。`42dce75` 的 Grok 夹具曾缺少 `new` 过滤钩子标记，同包从该阶段续跑后通过。`465897d` 完成全部 37 项累计夹具检查，验后远端 `main` 与构建 SHA 一致。

Both download checks verified the ZIP, file manifest, product PNG, and Release desktop startup and exit. All six PE files reported `NotSigned`. The Grok fixture for `42dce75` initially lacked a filtered `new` hook marker and passed when resumed on the same package. All 37 accumulated fixture checks passed for `465897d`, and remote `main` matched its full build SHA after acceptance.

| 构建 SHA / Build SHA | Rust 通过 / 忽略 | ZIP bytes | ZIP SHA-256 | 文件数 / Files | 清单 SHA-256 / Manifest SHA-256 | 验后远端 main / Remote main after acceptance | GitHub Actions |
| --- | ---: | ---: | --- | ---: | --- | --- | --- |
| `42dce757303df9acb1c72f8a08af8efc706923e5` | 160 / 33 | 67,164,527 | `ebeac7138619f34ac97622c8415a099845c9ab1c7f69c636f105b6e262387f14` | 676 | `27b6dd8ef6a655ac2390845b23c31515c605d6c6e6a23bc369fe26c04e59ecd3` | `465897d447039211b379f7ab4c733259e9e0dfad` | [Job](https://github.com/blueokanna/Crabforge/actions/runs/46110796580/job/140873656832) · [Artifact](https://github.com/blueokanna/Crabforge/actions/runs/46110796580/artifacts/15250441872) |
| `465897d447039211b379f7ab4c733259e9e0dfad` | 167 / 30 | 67,072,838 | `07820cbca0a7c26ee470f6a50c0c56e0ba80af98a6292888fe3704e73a57d7d2` | 556 | `fd7d7851299c561e576c6a11ec5776ce739185f5552ef2cdbfa195e28ecf4d57` | `465897d447039211b379f7ab4c733259e9e0dfad` | [Job](https://github.com/blueokanna/Crabforge/actions/runs/46111496870/job/140875811777) · [Artifact](https://github.com/blueokanna/Crabforge/actions/runs/46111496870/artifacts/15250871596) |

### 决策接口与原生分类 / Decision APIs and native classification

Windows 本地通过 164 项 Rust 测试及全目标 Clippy，33 项忽略。双 CLI 通过回环核对原生 Decisions 请求字节、Bearer 认证、与 System One 的双向转换、429 组回退及原生分类阈值。Rust 回归还核对题目顺序、布尔选项、评分分布、缺答回退为 refusal、不可转换字段的 HTTP 400、决策模型目录，以及 Magpie 备份中的 `decide`。Flutter 分析和四项单测通过，Windows 集成测试在 560 和 1,200 逻辑像素下经 `flutter_rust_bridge` 保存两种端点。 [网关测试](../../../crates/core/tests/gateway.rs) · [目录测试](../../../crates/core/tests/catalog.rs) · [桌面测试](../../../gui/integration_test/console_layout_test.dart)

Local Windows validation passed 164 Rust tests and all-target Clippy, with 33 tests ignored. Both CLIs checked native Decisions request bytes, Bearer authentication, conversion to and from System One, group fallback after a 429, and native classifier thresholds on loopback. Rust coverage also checked question order, Boolean choices, score distributions, missing answers as refusals, HTTP 400 for fields the target cannot carry, decision model discovery, and `decide` in Magpie backups. Flutter analysis and four unit tests passed. The Windows integration test saved both endpoints through `flutter_rust_bridge` at 560 and 1,200 logical pixels. [Gateway tests](../../../crates/core/tests/gateway.rs) · [Catalog tests](../../../crates/core/tests/catalog.rs) · [Desktop test](../../../gui/integration_test/console_layout_test.dart)

加入 Gemini Developer API 适配后，Windows 本地通过 165 项 Rust 测试及全目标 Clippy，33 项忽略，双 CLI 通过回环决策夹具。Gemini 协议回归核对两个入口、`x-goog-api-key`、`gemini-decision-1` 目录、按顺序排列的文本、内联 PNG 和 WAV URL、判断题和选择题条件、用量换算及拦截响应的 HTTP 400。实际 Gemini 决策调用待提供 AI Studio 密钥。 [协议测试](../../../crates/core/src/gateway/decision.rs) · [目录测试](../../../crates/core/tests/catalog.rs)

After the Gemini Developer API adapter was added, local Windows validation passed 165 Rust tests and all-target Clippy, with 33 tests ignored. Both CLIs passed the loopback decision fixture. The Gemini regressions checked both routes, `x-goog-api-key`, the `gemini-decision-1` catalog, ordered text, inline PNG and a WAV URL, predicate and choice criteria, usage conversion, and HTTP 400 for blocked input. Live Gemini Decisions acceptance requires an AI Studio API key. [Protocol tests](../../../crates/core/src/gateway/decision.rs) · [Catalog tests](../../../crates/core/tests/catalog.rs)

接入 Vercel AI Gateway Evaluation 后，Windows 本地通过 165 项 Rust 测试及全目标 Clippy，33 项忽略。双 CLI 通过回环 `--vercel-decisions`，核对 Bearer、四个 Evaluation 请求头、两种决策入口、布尔题 0.8、选择题校准置信度 0.75、42／1 token 用量及决策模型的 Chat 400 响应。加密备份回归验证旧版 Vercel 和 Gemini `decide` 基址迁移。实际 Vercel 调用待提供供应商密钥。 [网关测试](../../../crates/core/tests/gateway.rs) · [迁移测试](../../../crates/core/src/config/backup.rs)

With Vercel AI Gateway Evaluation, local Windows validation passed 165 Rust tests and all-target Clippy, with 33 tests ignored. Both CLIs passed `--vercel-decisions` on loopback, covering Bearer authentication, the four evaluation headers, both decision routes, a Boolean probability of 0.8, calibrated choice confidence of 0.75, 42 input and one output token, and HTTP 400 when the decision model was called through Chat. The encrypted backup regression covered migration of Vercel and Gemini `decide` bases. Live Vercel acceptance requires a provider key. [Gateway tests](../../../crates/core/tests/gateway.rs) · [Migration tests](../../../crates/core/src/config/backup.rs)

接入 Cloudflare Workers AI 后，Windows 本地通过 165 项 Rust 测试及全目标 Clippy，33 项忽略。双 CLI 通过回环 `--cloudflare-decisions`。每个 CLI 的两次并发 System One 请求只发起一次带 Bearer 的账户查询，并分别向 Jev 发送封装请求。使用显式账户 ID 的 Decisions 请求直接发往 Clef 专用路径，两种请求均解包答案和用量。Rust 回归覆盖模型目录及带账户占位符的加密备份。实际 Workers AI 调用待提供 API token。 [网关测试](../../../crates/core/tests/gateway.rs) · [迁移测试](../../../crates/core/src/config/backup.rs)

With Cloudflare Workers AI, local Windows validation passed 165 Rust tests and all-target Clippy, with 33 tests ignored. Both CLIs passed `--cloudflare-decisions` on loopback. Two concurrent System One requests per CLI shared one authenticated account lookup and sent separate wrapped requests to Jev. A Decisions request with an explicit account ID went directly to the Clef model path. Both routes unwrapped answers and usage. Rust regressions covered the model catalog and an encrypted backup with an account placeholder. Live Workers AI acceptance requires an API token. [Gateway tests](../../../crates/core/tests/gateway.rs) · [Migration tests](../../../crates/core/src/config/backup.rs)

### 推理设置之前的下载验收 / Download acceptance before reasoning controls

提交 `76db05fd973171dce234a1152927ececb2f245fa` 的 Windows Run `46031034848` 通过 153 项 Rust 测试，33 项忽略。ZIP 为 66,957,505 字节，SHA-256 为 `79074f67d989ddb5f18056bbe12607b176b66316f56685ba5483d6b96e0680fc`。676 项逐文件清单的 SHA-256 为 `228ee3af7d4d47fc7d7e82d772b1893eb84e7d690a4b2e8d92b9b451815fe710`。六个 PE 均为 `NotSigned`。Release 桌面正常启动及退出，双 CLI 通过路由、重试、分类恢复、SSE 保活和原版备份导入，Pi、Gemini、Grok 和 Oh My Pi 各通过 12 轮。验后经代理核对远端 `main` 匹配完整构建 SHA，再推送后继。 [Job 140614575279](https://github.com/blueokanna/Crabforge/actions/runs/46031034848/job/140614575279) · [Artifact 15216770297](https://github.com/blueokanna/Crabforge/actions/runs/46031034848/artifacts/15216770297)

Windows run `46031034848` built `76db05fd973171dce234a1152927ececb2f245fa` and passed 153 Rust tests with 33 ignored. The 66,957,505-byte ZIP has SHA-256 `79074f67d989ddb5f18056bbe12607b176b66316f56685ba5483d6b96e0680fc`. The 676-entry manifest has SHA-256 `228ee3af7d4d47fc7d7e82d772b1893eb84e7d690a4b2e8d92b9b451815fe710`. All six PE files reported `NotSigned`. The Release desktop opened and closed cleanly. Both CLIs passed routing, retries, classifier recovery, SSE keepalive, and Magpie backup import. Pi, Gemini, Grok, and Oh My Pi each passed 12 turns. A proxied ref check after acceptance matched remote `main` to the full build SHA before the successor was pushed. [Job 140614575279](https://github.com/blueokanna/Crabforge/actions/runs/46031034848/job/140614575279) · [Artifact 15216770297](https://github.com/blueokanna/Crabforge/actions/runs/46031034848/artifacts/15216770297)

提交 `7827da4d49b7b114d32a99b1b402c376cac32a93` 的 Windows Run `46035999419` 通过 154 项 Rust 测试，33 项忽略。ZIP 为 67,021,838 字节，SHA-256 为 `ed6f045f259376e774d29ea9822ee0f5b6ed91295055cc21a5b0677e907e25ff`。676 项逐文件清单的 SHA-256 为 `25e16c33c1a49c50c98f754549f41d9961d39280291985b5c2486d5d94f2d230`。六个 PE 均为 `NotSigned`。Release 桌面正常启动及退出，双 CLI 通过原版 `useEffort` 和 `modelEfforts` 加密备份、目录同步、供应商改名及原生推理等级拟合。Pi、Gemini、Grok 和 Oh My Pi 各通过 12 轮，验后通过代理核对远端 `main` 与完整构建 SHA 一致。 [Job 140631439515](https://github.com/blueokanna/Crabforge/actions/runs/46035999419/job/140631439515) · [Artifact 15219855229](https://github.com/blueokanna/Crabforge/actions/runs/46035999419/artifacts/15219855229)

Windows run `46035999419` built `7827da4d49b7b114d32a99b1b402c376cac32a93` and passed 154 Rust tests with 33 ignored. The 67,021,838-byte ZIP has SHA-256 `ed6f045f259376e774d29ea9822ee0f5b6ed91295055cc21a5b0677e907e25ff`. The 676-entry manifest has SHA-256 `25e16c33c1a49c50c98f754549f41d9961d39280291985b5c2486d5d94f2d230`. All six PE files reported `NotSigned`. The Release desktop opened and closed cleanly. Both CLIs passed encrypted Magpie backup handling for `useEffort` and `modelEfforts`, catalog sync, provider rename, and effort fitting in the native request. Pi, Gemini, Grok, and Oh My Pi each passed 12 turns. The proxied remote `main` check after acceptance matched the full build SHA. [Job 140631439515](https://github.com/blueokanna/Crabforge/actions/runs/46035999419/job/140631439515) · [Artifact 15219855229](https://github.com/blueokanna/Crabforge/actions/runs/46035999419/artifacts/15219855229)

提交 `4f5df391d37377f5f0a27a1862a76b19073f3963` 的 Windows Run `46039698411` 通过 155 项 Rust 测试，33 项忽略。ZIP 为 67,023,537 字节，SHA-256 为 `4fd33e50a03f21df4fd9112b109f3e4eda0f9f3328c744548d91708ad7635467`。676 项逐文件清单的 SHA-256 为 `ef7055deb0123080b0fc924169603d7fc746ce0f85f032f5f9fff08d38ddfe9c`。六个 PE 均为 `NotSigned`。Release 桌面正常启动和退出。双 CLI 通过加密备份导入和分享、意图与自动推理合并判断、同轮复用和显式关闭推理。Pi、Gemini、Grok 和 Oh My Pi 各通过 12 轮，验后通过代理核对远端 `main` 与完整构建 SHA 一致。 [Job 140643983031](https://github.com/blueokanna/Crabforge/actions/runs/46039698411/job/140643983031) · [Artifact 15220626573](https://github.com/blueokanna/Crabforge/actions/runs/46039698411/artifacts/15220626573)

Windows run `46039698411` built `4f5df391d37377f5f0a27a1862a76b19073f3963` and passed 155 Rust tests with 33 ignored. The 67,023,537-byte ZIP has SHA-256 `4fd33e50a03f21df4fd9112b109f3e4eda0f9f3328c744548d91708ad7635467`. The 676-entry manifest has SHA-256 `ef7055deb0123080b0fc924169603d7fc746ce0f85f032f5f9fff08d38ddfe9c`. All six PE files reported `NotSigned`. The Release desktop opened and closed cleanly. Both CLIs passed the encrypted backup import and share fixture, joint intent and reasoning classification, same-turn reuse, and explicit reasoning disablement. Pi, Gemini, Grok, and Oh My Pi each passed 12 turns. The proxied remote `main` check after acceptance matched the full build SHA. [Job 140643983031](https://github.com/blueokanna/Crabforge/actions/runs/46039698411/job/140643983031) · [Artifact 15220626573](https://github.com/blueokanna/Crabforge/actions/runs/46039698411/artifacts/15220626573)

### 供应商重试历史构建 / Historical provider retry build

按完整 SHA 补验 `649c036b07986275aeef4529d1228dc4fec7d3ae` 时，Run `46024892156` 在 Rust 检查阶段失败。`config.rs:441` 将重试时限校验插入了环境变量校验的表达式，导致语法错误，Windows 包未生成。后继 `dfd58df4c3c3461d8969f5a654a40c437c15bb13` 已修正校验位置，其下载验收包括供应商重试。 [Windows job 140593634231](https://github.com/blueokanna/Crabforge/actions/runs/46024892156/job/140593634231)

The Windows check for `649c036b07986275aeef4529d1228dc4fec7d3ae` failed in run `46024892156`. The retry-patience check interrupted the environment-variable expression at `config.rs:441`, causing a syntax error before the Windows package could be built. The successor `dfd58df4c3c3461d8969f5a654a40c437c15bb13` moves the check to its own statement. Its download fixture includes provider retry coverage. [Windows job 140593634231](https://github.com/blueokanna/Crabforge/actions/runs/46024892156/job/140593634231)

修复提交 `dfd58df4c3c3461d8969f5a654a40c437c15bb13` 的 Windows Run `46024944128` 通过 150 项 Rust 测试，33 项忽略。ZIP 为 66,840,697 字节，SHA-256 为 `f8fe161f705d1278c574626a004fef88d404dc884410e995d87772aae5123ece`，676 项清单的 SHA-256 为 `562580f8b02febb02626ba68cf91fe9f8c487ba0f6cb6713610d6b8f4a69a241`。六个 PE 均为 `NotSigned`。首次设置夹具遇到 `WinError 10053`，同包单项和整套重跑均通过。两套 CLI 均验证了供应商重试和组优先级。验后查询的远端 `main` 为 `8fab8d3bb7f3cfd0af7a7fbb9389cf1f6d612ec9`，包含构建 SHA。 [Job 140593804622](https://github.com/blueokanna/Crabforge/actions/runs/46024944128/job/140593804622) · [Artifact 15212497048](https://github.com/blueokanna/Crabforge/actions/runs/46024944128/artifacts/15212497048)

Windows run `46024944128` for `dfd58df4c3c3461d8969f5a654a40c437c15bb13` passed 150 Rust tests with 33 ignored. The 66,840,697-byte ZIP has SHA-256 `f8fe161f705d1278c574626a004fef88d404dc884410e995d87772aae5123ece`. The 676-entry manifest has SHA-256 `562580f8b02febb02626ba68cf91fe9f8c487ba0f6cb6713610d6b8f4a69a241`. All six PE files reported `NotSigned`. The first settings fixture encountered `WinError 10053`. The individual fixture and complete suite passed on the same package in subsequent runs. Both CLIs verified provider retries and group precedence. The final remote `main` check returned `8fab8d3bb7f3cfd0af7a7fbb9389cf1f6d612ec9`, which includes the build SHA. [Job 140593804622](https://github.com/blueokanna/Crabforge/actions/runs/46024944128/job/140593804622) · [Artifact 15212497048](https://github.com/blueokanna/Crabforge/actions/runs/46024944128/artifacts/15212497048)

### 分类缓存历史构建 / Historical classifier cache build

提交 `8adc702f93062b2c07052cd0c044e1377df86530` 的 Windows Run `46024970247` 通过 150 项 Rust 测试，33 项忽略。ZIP 为 66,860,970 字节，SHA-256 为 `8fd00c093aa825ca1e9aa55fb0ce5c3eb934222b4515217fec43fc8524cc6825`。676 项清单的 SHA-256 为 `be4aa7b8919b4492d1278a16881a253722eab157de1e1f69be07c13e3c1f13d6`。六个 PE 均为 `NotSigned`。Release 桌面启动和退出正常。两个 CLI 均验证了工具续轮复用分类命中及显式 `0`，Pi、Gemini、Grok 和 Oh My Pi 各通过 12 轮。验后经代理核对远端 `main` 为 `8fab8d3bb7f3cfd0af7a7fbb9389cf1f6d612ec9`，包含构建 SHA。 [Job 140593899677](https://github.com/blueokanna/Crabforge/actions/runs/46024970247/job/140593899677) · [Artifact 15212941493](https://github.com/blueokanna/Crabforge/actions/runs/46024970247/artifacts/15212941493)

Windows run `46024970247` for `8adc702f93062b2c07052cd0c044e1377df86530` passed 150 Rust tests with 33 ignored. The 66,860,970-byte ZIP has SHA-256 `8fd00c093aa825ca1e9aa55fb0ce5c3eb934222b4515217fec43fc8524cc6825`. The 676-entry manifest has SHA-256 `be4aa7b8919b4492d1278a16881a253722eab157de1e1f69be07c13e3c1f13d6`. All six PE files reported `NotSigned`. The Release desktop opened and closed cleanly. Both CLIs reused positive and explicit `0` classifier results across tool continuations. Pi, Gemini, Grok, and Oh My Pi each passed 12 turns. The proxied remote `main` check returned `8fab8d3bb7f3cfd0af7a7fbb9389cf1f6d612ec9`, which includes the build SHA. [Job 140593899677](https://github.com/blueokanna/Crabforge/actions/runs/46024970247/job/140593899677) · [Artifact 15212941493](https://github.com/blueokanna/Crabforge/actions/runs/46024970247/artifacts/15212941493)

### SSE 保活历史构建 / Historical SSE keepalive build

提交 `cc4865d8ddbae357d1b07007d278483a2441482a` 的 Windows Run `46025000819` 通过 153 项 Rust 测试，33 项忽略。ZIP 为 66,953,595 字节，SHA-256 为 `fb0b335792706ce2d3e04c385b4a6d373eade7f85f358f59465da3a3ae78a83c`。676 项清单的 SHA-256 为 `c045e468c27f1f5967d700ddde2fc888ee9ba1f25554d7c46095bf26c0fa8da6`。六个 PE 均为 `NotSigned`。Release 桌面通过启动和退出，两个 CLI 均检查了连续等待注释、完整回退 SSE 和延迟错误。Pi、Gemini、Grok 和 Oh My Pi 各通过 12 轮。验后经代理核对远端 `main` 为 `8fab8d3bb7f3cfd0af7a7fbb9389cf1f6d612ec9`，包含构建 SHA。 [Job 140594001232](https://github.com/blueokanna/Crabforge/actions/runs/46025000819/job/140594001232) · [Artifact 15213471448](https://github.com/blueokanna/Crabforge/actions/runs/46025000819/artifacts/15213471448)

Windows run `46025000819` for `cc4865d8ddbae357d1b07007d278483a2441482a` passed 153 Rust tests with 33 ignored. The 66,953,595-byte ZIP has SHA-256 `fb0b335792706ce2d3e04c385b4a6d373eade7f85f358f59465da3a3ae78a83c`. The 676-entry manifest has SHA-256 `c045e468c27f1f5967d700ddde2fc888ee9ba1f25554d7c46095bf26c0fa8da6`. All six PE files reported `NotSigned`. The Release desktop opened and closed cleanly. Both CLIs checked repeated wait comments, complete fallback SSE, and late errors. Pi, Gemini, Grok, and Oh My Pi each passed 12 turns. The proxied remote `main` check returned `8fab8d3bb7f3cfd0af7a7fbb9389cf1f6d612ec9`, which includes the build SHA. [Job 140594001232](https://github.com/blueokanna/Crabforge/actions/runs/46025000819/job/140594001232) · [Artifact 15213471448](https://github.com/blueokanna/Crabforge/actions/runs/46025000819/artifacts/15213471448)

### 高级路由历史下载包 / Historical conditional routing package

提交 `2446bb2ac33b326362d57532ff3052818223174f` 通过 Windows Run `46022086824`，149 项 Rust 测试通过，33 项忽略。ZIP 为 66,695,421 字节，SHA-256 为 `2be9d84d5eafe69c09a8a841e6217534c280b8ce2f19519540497e5fbd9e5c29`。676 项清单的 SHA-256 为 `233d28110482aa740c1cf90d166f7a90b2c1b17d7f283eeb3c8153e77afda430`。六个 PE 均为 `NotSigned`。Release 桌面、双 CLI、条件路由、组重试及四个客户端各 12 轮通过。验后经代理复核远端 `main` 为 `8fab8d3bb7f3cfd0af7a7fbb9389cf1f6d612ec9`，包含构建 SHA。 [Job 140584006318](https://github.com/blueokanna/Crabforge/actions/runs/46022086824/job/140584006318) · [Artifact 15212745581](https://github.com/blueokanna/Crabforge/actions/runs/46022086824/artifacts/15212745581)

Windows run `46022086824` built `2446bb2ac33b326362d57532ff3052818223174f` and passed 149 Rust tests with 33 ignored. The 66,695,421-byte ZIP has SHA-256 `2be9d84d5eafe69c09a8a841e6217534c280b8ce2f19519540497e5fbd9e5c29`. The 676-entry manifest has SHA-256 `233d28110482aa740c1cf90d166f7a90b2c1b17d7f283eeb3c8153e77afda430`. All six PE files reported `NotSigned`. The Release desktop, both CLIs, conditional routing, group retries and 12 turns in each of four clients passed. The proxied remote `main` check returned `8fab8d3bb7f3cfd0af7a7fbb9389cf1f6d612ec9`, which includes the build SHA. [Job 140584006318](https://github.com/blueokanna/Crabforge/actions/runs/46022086824/job/140584006318) · [Artifact 15212745581](https://github.com/blueokanna/Crabforge/actions/runs/46022086824/artifacts/15212745581)

## Grok Build 会话 / Grok Build sessions

Grok Build 1.0.92 通过回环夹具启动原生会话，并用同一个会话 ID 继续到第 12 轮。每轮上游收到之前的用户提示。两个 Windows Debug CLI 均列出原生会话 ID、项目和配置路径，从日志恢复完全相同的 12 条用户原话和预览修订，并拒绝未经发现的配置目录。Rust 回归覆盖分段提示、客户端内部操作和子代理日志。

Grok Build 1.0.92 started a native session against a loopback fixture and resumed the same session ID through turn 12. The upstream received the earlier user prompts on each turn. Both Windows Debug CLIs listed the native ID, project and profile, extracted the same 12 original prompts and preview revision, and rejected an unconfigured profile. The Rust regression covers split prompts, host turns and subagent logs. [Session reader](../../../crates/core/src/sessions/grok.rs)

提交 `bac0f89cae4b0a9edc3cebf57011366cfaaa0ab8` 的 Windows Actions 包通过桌面启动和两版 CLI 验收。Run `46005007334`、job `140525857272`、Artifact `15203095203`，ZIP 共 65,749,896 字节，SHA-256 为 `4de17c7750ef858c4e6bb2b63f78919423e38f2911516b918a764250a03da9e2`。670 个文件逐项核对，清单 SHA-256 为 `47e2bece1e8f6555b77b8f33cbf40e8485f84b8271111896adc9799680ff42cd`。Rust 通过 144 项，忽略 33 项。六个 PE 均为 `NotSigned`。Grok 的首轮夹具在第 10 轮仅取最后一个上游请求，因未找到全部提示词而中断。原生文件保留了 10 条记录。复测经下载版网关转发并核对每轮全部上游请求，区分只带一条提示词的附加调用，确认所有 12 轮均存在包含完整历史的请求，两版 CLI 读出同一原生 ID、12 条原话和相同预览修订。复测结束后，经代理核对远端 `main` 完整 SHA 与该包一致。 [Windows job](https://github.com/blueokanna/Crabforge/actions/runs/46005007334/job/140525857272)

The Windows Actions package for `bac0f89cae4b0a9edc3cebf57011366cfaaa0ab8` passed desktop startup and checks with both CLIs. Run `46005007334`, job `140525857272` and artifact `15203095203` identify the 65,749,896-byte ZIP with SHA-256 `4de17c7750ef858c4e6bb2b63f78919423e38f2911516b918a764250a03da9e2`. All 670 files matched the extracted manifest, whose SHA-256 was `47e2bece1e8f6555b77b8f33cbf40e8485f84b8271111896adc9799680ff42cd`. Rust reported 144 passed and 33 ignored. All six PE files reported `NotSigned`. The first Grok fixture stopped at turn 10 after checking only the last upstream request. The native session still contained all ten prompts. The retest forwarded through the downloaded gateway and inspected every upstream request per turn, including ancillary requests that carried one prompt. All 12 turns sent the complete history in a generation request. Both CLIs recovered the same native ID, 12 original prompts and preview revision. After acceptance, a proxied remote ref check matched the package's full SHA. [Windows job](https://github.com/blueokanna/Crabforge/actions/runs/46005007334/job/140525857272)

Windows 下载包 `46005007334` 与 `45992223719` 各用 30 次交替的回环 Chat SSE 请求复测。每次均返回 200、只向模拟模型发出一次请求、代理流量为零，且逐条检查了首事件和结束帧。首事件最小、中位和最大耗时分别为 `2.385/3.067/7.953 ms` 与 `2.229/2.962/5.084 ms`。同期一秒间隔的 47 个 CPU 样本最小、中位和最大为 `75.00/89.65/99.22%`，均值 `89.62%`。运行期间有其他任务占用 CPU，仍需低负载复测。 [请求数据](sse-repeat-46005007334.csv) · [CPU 样本](sse-repeat-46005007334-cpu.csv)

The downloaded Windows builds from runs `46005007334` and `45992223719` each received 30 alternating loopback Chat SSE requests. Every request returned 200, reached the model fixture once, bypassed the proxy and delivered the first event and final frame. Minimum, median and maximum first-event times were `2.385/3.067/7.953 ms` and `2.229/2.962/5.084 ms` respectively. The 47 one-second CPU samples had a minimum of `75.00%`, a median of `89.65%`, a maximum of `99.22%` and a mean of `89.62%`. Other workloads were active throughout the run, so low-load confirmation is still pending. [Request samples](sse-repeat-46005007334.csv) · [CPU samples](sse-repeat-46005007334-cpu.csv)

## Oh My Pi 会话 / Oh My Pi sessions

Windows 10 上使用 Oh My Pi `19.2.1` 和 Bun `1.4.5`，经 Actions `45996621864` 下载包的两个 CLI 分别完成六轮，对同一个命名配置下的原生会话连续恢复到第 12 轮。回环上游在每轮请求中收到此前的提示，并另答复客户端自动发出的七个模型探测请求。新增读取功能在两个本地 Debug CLI 中返回相同的原生 ID、项目、配置路径、12 条提示和预览修订。读取前后的会话文件字节一致。Rust 回归覆盖旧格式、固定标题记录、清空边界和日志中最后一次保存的活跃分支。

On Windows 10, Oh My Pi `19.2.1` with Bun `1.4.5` resumed a native session in a named profile through turn 12. Each of the two packaged CLIs from Actions run `45996621864` forwarded six turns. The loopback upstream received the earlier prompts on every turn and separately answered seven native model probes. Both local Debug CLIs returned the same session ID, project, profile, 12 prompts and preview revision. Reading left the session file byte-identical. The Rust regression covers legacy records, the title record, clear boundaries and the saved active branch. [Session readers](../../../crates/core/src/sessions.rs) · [Pi and Oh My Pi parser](../../../crates/core/src/sessions/pi.rs)

双 CLI 在共用会话目录中分别识别 Pi 与 Oh My Pi，并跳过下层子代理日志。Pi `1.6.0` 另通过 12 轮原生续接、记忆提炼、审核、发布和逐字节撤回回归。

Both CLIs distinguished Pi and Oh My Pi in a shared session directory and skipped a nested subagent log. A separate Pi `1.6.0` regression verified 12 native turns, memory extraction, review, publication and byte-identical withdrawal.

提交 `260c00a22a2a2a1f22b6a47b65e2817e745da12f` 的 Windows 包通过桌面启动，以及 Pi、Gemini、Grok 和 Oh My Pi 各 12 轮。解压版双 CLI 从 Oh My Pi 会话读取原话，共有七次模型探测，读取前后会话文件字节一致。Run `46014541751` 通过 146 项 Rust 测试，忽略 33 项。ZIP 为 65,831,256 字节，SHA-256 为 `400cb6cee24df3dd481e86484354c2ff707f43cf30e854155a2ffbe56b34b821`。670 项逐文件清单的 SHA-256 为 `ece044b3d9f2ab86c87f0d7a302a58f118698ad92aab8f71e0abd9bed40bfa89`，六个 PE 均为 `NotSigned`。夹具完成后通过代理复核远端 `main` 为 `8fab8d3bb7f3cfd0af7a7fbb9389cf1f6d612ec9`，并验证构建 SHA 是它的祖先。 [Job 140558184135](https://github.com/blueokanna/Crabforge/actions/runs/46014541751/job/140558184135) · [Artifact 15207111741](https://github.com/blueokanna/Crabforge/actions/runs/46014541751/artifacts/15207111741)

The Windows package built from `260c00a22a2a2a1f22b6a47b65e2817e745da12f` passed desktop startup and 12 native turns each with Pi, Gemini, Grok and Oh My Pi. Both extracted CLIs recovered the original Oh My Pi prompts. The upstream answered seven model probes, and reading left the session file byte-identical. Run `46014541751` passed 146 Rust tests with 33 ignored. The 65,831,256-byte ZIP has SHA-256 `400cb6cee24df3dd481e86484354c2ff707f43cf30e854155a2ffbe56b34b821`. The 670-file manifest has SHA-256 `ece044b3d9f2ab86c87f0d7a302a58f118698ad92aab8f71e0abd9bed40bfa89`, and all six PE files reported `NotSigned`. After the fixtures finished, a proxied ref check found remote `main` at `8fab8d3bb7f3cfd0af7a7fbb9389cf1f6d612ec9` and confirmed that the build SHA was its ancestor. [Job 140558184135](https://github.com/blueokanna/Crabforge/actions/runs/46014541751/job/140558184135) · [Artifact 15207111741](https://github.com/blueokanna/Crabforge/actions/runs/46014541751/artifacts/15207111741)

## 技能与钩子备份迁移 / Skill and hook backup migration

用 Magpie 0.1.4936 的便携配置生成加密备份。双 Windows Debug CLI 只读预览技能，保留 `SKILL.md` 的 CRLF、含中文路径的二进制附件及客户端分配。四进程并发仅写入一次。显式同步发布到 Claude Code、Codex 和 Pi，撤销保留了外部编辑。钩子用例覆盖命令遮蔽、密码错误、选名、并发、批次冲突及不兼容会话匹配条件。同步后核对 Claude Code 的 `Bash`、Codex 的 `resume` 和 Gemini 的 `run_shell_command` 及毫秒超时，并验证撤销。

Both Windows Debug CLIs read an encrypted backup written by portable Magpie 0.1.4936. Skill imports preserved CRLF in `SKILL.md`, a binary attachment with a Chinese filename, and client assignments. Four competing applies wrote the library once. Explicit sync published to Claude Code, Codex and Pi, and withdrawal preserved an external edit. Hook fixtures covered command masking, a wrong passphrase, name selection, concurrent apply, conflicts and incompatible session filters. Native configuration after sync showed Claude Code's `Bash`, Codex's `resume` and Gemini's `run_shell_command` with a millisecond timeout. The fixture also checked withdrawal. [Skill importer](../../../crates/core/src/skills/backup.rs) · [Hook importer](../../../crates/core/src/hooks/backup_import.rs)

Grok `1.0.92` 在回环模型夹具中恢复同一个原生会话。新建会话触发导入的 `new` 钩子，按 ID 恢复触发 `load` 钩子，无匹配条件的对照钩子触发两次。第二轮上游请求包含第一轮提示，两轮都收到模型回复。Antigravity 的会话来源转换由 Rust 单测覆盖。

Grok `1.0.92` resumed the same native session against a loopback model fixture. The imported `new` hook ran on startup, the `load` hook ran on resume by ID, and the unfiltered control ran twice. The second model request retained the first prompt, and both turns received a reply. Rust unit coverage checks the Antigravity session source conversion. [Client hook encoding](../../../crates/core/src/hooks/native.rs)

Windows Actions `45996621864` 从 `802e64a4fbad30846a1079c713b61fd046aedaba` 构建，job `140497503621` 通过依赖门禁、Clippy、143 项 Rust 测试及 Flutter 检查。ZIP `15197592899` 为 65,669,070 字节，SHA-256 为 `9d94246c4811d83cc07fb8f56e0759cadb8659278854aa422e9079b01479479f`。670 项逐文件清单的 SHA-256 为 `0f25c3d4f791de6b7557bef10fe11b81025c305bf4342d1dd93f7ea06bb0614a`。桌面通过启动和退出测试，六个 PE 均返回 `NotSigned`。双 CLI 通过技能、钩子、用户指令和先前的迁移回归，Pi 和 Gemini 各恢复 12 轮原生会话。验收末尾远端 `main` 匹配该完整构建 SHA。

Windows Actions run `45996621864` built `802e64a4fbad30846a1079c713b61fd046aedaba`. Job `140497503621` passed the dependency gate, Clippy, 143 Rust tests and Flutter checks. Artifact `15197592899` was 65,669,070 bytes with SHA-256 `9d94246c4811d83cc07fb8f56e0759cadb8659278854aa422e9079b01479479f`. The 670-file manifest has SHA-256 `0f25c3d4f791de6b7557bef10fe11b81025c305bf4342d1dd93f7ea06bb0614a`. The desktop opened and exited cleanly, and all six PE files reported `NotSigned`. Both CLIs passed skill, hook, user instruction and earlier migration regressions. Pi and Gemini each resumed 12 native turns. Remote `main` matched the full build SHA at the end of verification. [Job](https://github.com/blueokanna/Crabforge/actions/runs/45996621864/job/140497503621) · [Artifact](https://github.com/blueokanna/Crabforge/actions/runs/45996621864/artifacts/15197592899)

## 用户指令备份迁移 / User instruction backup migration

双 Windows Debug CLI 从 Magpie 0.1.4936 加密备份预览活跃指令集、默认集及 Claude Code、Codex 各自的附加文本。应用保留用户文档的 BOM、CRLF 和托管区块外的内容。Codex 使用现有 `AGENTS.override.md`。区块已有不同内容时，须核对当前文件版本后才能替换。测试验证了错误口令、未知指令集、重复导入及外部编辑后的陈旧修订拒绝。

Both Windows Debug CLIs previewed the active and default instruction sets plus client-specific additions from a native Magpie 0.1.4936 encrypted backup. Apply preserved BOM, CRLF and all bytes outside the managed block. Codex used the existing `AGENTS.override.md`. Replacing different managed text required a current file revision. The fixture covered a wrong passphrase, an unknown set, repeat import and stale revision rejection after an external edit. [Instruction importer](../../../crates/core/src/project_instructions/backup.rs)

本地修复重跑了双 CLI 加密导入。使用旧标记和 CRLF 的 `AGENTS.override.md` 预览为 `conflict`，拒绝无修订号的应用和外部编辑后的陈旧修订号，最终用当前修订号原位替换并逐字节核对其余内容。Rust 回归还覆盖旧块删除、同时存在两代块及混用首尾标记。

The local fix reran encrypted import through both CLIs. An existing CRLF `AGENTS.override.md` with legacy markers previewed as `conflict`. Apply rejected a missing revision and then a stale revision after an external edit. Applying the current revision replaced the text in place and preserved the remaining bytes. Rust regression coverage also includes clearing a legacy block, duplicate blocks from both generations, and mismatched opening and closing markers. [Marker handling and regression](../../../crates/core/src/project_instructions.rs)

## Windows 首事件分段时序 / Windows first-event timing

同一 Windows 10 主机在开发构建期间交替采样。`5cc51ab575b27a333aa007188c636f4b70508d93` 的 Actions 包和基于 `38ac6ba98ec0f13317b58060feeff2648cd4c73a` 加入 QPC 探针的本地 Release 各测 45 次。每次启动新网关进程，均返回 HTTP 200、调用上游一次、代理流量为零。首事件最小／中位／最大值分别为 `14.454/90.838/286.718 ms` 和 `14.601/102.876/290.461 ms`。

The Windows 10 host alternated 45 samples each from the `5cc51ab575b27a333aa007188c636f4b70508d93` Actions package and a local Release build of `38ac6ba98ec0f13317b58060feeff2648cd4c73a` with QPC markers. Development builds were running on the host. Every sample started a fresh gateway process, returned HTTP 200, made one upstream request and sent no proxy traffic. First-event minimum, median and maximum were `14.454/90.838/286.718 ms` and `14.601/102.876/290.461 ms`, respectively. [90 raw samples](forward-h1-windows-qpc.csv)

36 个带探针的样本超过 50 ms。阻塞任务从提交到开始的中位数为 `45.144 ms`，从开始到发起上游连接为 `39.561 ms`，TCP 连接调用为 `0.814 ms`，从 Python 接受连接到读完请求头为 `21.892 ms`。从网关首次读到上游字节到客户端收到完整首事件的中位数为 `0.479 ms`。采样定位了这些等待区间，各区间的线程调度与计算耗时仍需单独区分。

Thirty-six instrumented samples took over 50 ms. Median intervals were `45.144 ms` from submitting the blocking task to its start, `39.561 ms` from task start to the upstream connect call, `0.814 ms` for the connect call, and `21.892 ms` from Python accepting the socket to reading the request headers. The median from the gateway's first upstream read to the complete client event was `0.479 ms`. These timestamps locate the waits. Distinguishing thread scheduling from execution within those intervals requires further measurement.

`ureq` 在启用解析超时时会为每个地址启动解析线程。网关现在直接处理可解析为 socket 地址的 IPv4 和 IPv6 字面量，域名继续使用带超时的默认解析。以同一 `38ac6ba` 为基础构建两个带探针的 Release 版本，各交替测 24 次。阻塞任务开始到发起上游连接的中位数从 `43.165 ms` 降至 `0.117 ms`。首事件中位数从 `116.626 ms` 降至 `53.222 ms`，最大值分别为 `324.734 ms` 和 `353.668 ms`。改动后最慢样本仍含 `200.676 ms` 的阻塞任务启动等待和 `149.332 ms` 的上游接受至完成读头时间。

With a resolution timeout, `ureq` starts a resolver thread even for a numeric address. The gateway now handles IPv4 and IPv6 socket literals directly while keeping the default timed resolver for hostnames. Two instrumented Release builds based on `38ac6ba` alternated 24 trials each. The median from worker start to the upstream connect call fell from `43.165 ms` to `0.117 ms`. The first-event median went from `116.626 ms` to `53.222 ms`, with maxima of `324.734 ms` and `353.668 ms`. The slowest updated sample still spent `200.676 ms` waiting for the worker to start and `149.332 ms` between upstream accept and request headers. [48 paired samples](forward-h1-windows-address-pair.csv) · [Resolver](../../../crates/core/src/upstream/forward/socket.rs)

两个本地 Release 探针的 CLI SHA-256 如下。采样主机为 Windows 10 19045，Intel Core i7-11850H，8 核 16 线程。

The instrumented CLI hashes identify the exact binaries in the paired series. The host ran Windows 10 build 19045 on an Intel Core i7-11850H with 8 cores and 16 threads.

| Resolver | SHA-256 |
| --- | --- |
| Default | `688d29f615f1fdf8e8abfc2e501bdf33cfc7f453bde5181a361d86f999ead4c0` |
| Numeric literals resolved inline | `bc1463ace11be3730beeb2481f76d848b644075ed0f3099066cc6932096e9ea9` |

同一主机对 `2ec7e74` 和 `7fd0552` 的下载包各做 30 次交替回环探针。首事件最小／中位／最大值分别为 `1.763/2.481/29.156 ms` 和 `1.916/2.518/26.263 ms`。60 次均为 HTTP 200、一次上游请求、零代理请求。46 个 CPU 样本为 `38.71–71.53%`，均值 `55.33%`。两版最慢样本分别在上游读完请求头前用时 `28.536` 和 `25.798 ms`，其中客户端 TCP 连接调用为 `12.656` 和 `23.631 ms`。对应的上游响应准备只用 `0.108` 和 `0.137 ms`。

The same host alternated 30 loopback trials each from the `2ec7e74` and `7fd0552` Actions packages. First-event minimum, median and maximum were `1.763/2.481/29.156 ms` and `1.916/2.518/26.263 ms`. All 60 returned HTTP 200 with one upstream request and zero proxy requests. The 46 CPU samples ranged from `38.71%` to `71.53%` and averaged `55.33%`. In the slowest sample for each build, `28.536` and `25.798 ms` elapsed before the upstream finished reading the request headers. The client TCP connect calls accounted for `12.656` and `23.631 ms` of those times. Upstream response preparation took `0.108` and `0.137 ms`. [60 paired samples](forward-h1-windows-actions-pair.csv) · [CPU samples](forward-h1-windows-actions-cpu.csv)

## 繁忙重试时限 / Retry patience

Windows 本机两个 Debug CLI 读取了 Magpie `0.1.4936` 生成的原版加密备份，验证 `patience` 和 `groupPatience` 的预览、应用、编辑、复制、改名和删除。回环夹具在首个 HTTP 503 后返回完整 SSE，关闭重试的对照组保留 HTTP 503、原始错误正文和 `Retry-After`。Rust 网关回归覆盖候选顺序、嵌套组禁用、配额耗尽、超过预算的 HTTP 日期及每次尝试的状态记录。Windows 桌面集成测试通过 Rust 桥接在 1,200 和 560 逻辑像素下保存组级和网关默认值。

组重试对应的历史构建 SHA 为 `e39b2c2537d4edaf355ff4a99c6f8f64def4990a`，通过临时分支触发 Run `46017272731`。Windows CI 通过 147 项 Rust 测试，忽略 33 项。ZIP 为 65,897,733 字节，SHA-256 为 `d3da144257da9c39690e9a4cc71eb707a03a9b04b001bad604e32f3c9f9e50df`。670 项清单的 SHA-256 为 `79d45f5b73a16c3f0b5e7bf116cbde6d9afd2aa612158908630f27437dbbab7d`。Release 桌面与双 CLI 通过，六个 PE 均为 `NotSigned`。首次 Grok 试跑缺少第二轮的 `load` 钩子记录，同包复跑通过钩子和 12 轮续接，随后通过组重试及 Oh My Pi 夹具。验后远端 `main` 为 `8fab8d3bb7f3cfd0af7a7fbb9389cf1f6d612ec9`，包含该构建 SHA。 [Job 140567500726](https://github.com/blueokanna/Crabforge/actions/runs/46017272731/job/140567500726) · [Artifact 15207448487](https://github.com/blueokanna/Crabforge/actions/runs/46017272731/artifacts/15207448487)

Run `46017272731` built the historical group retry revision `e39b2c2537d4edaf355ff4a99c6f8f64def4990a` from a temporary branch. Windows CI passed 147 Rust tests with 33 ignored. The 65,897,733-byte ZIP has SHA-256 `d3da144257da9c39690e9a4cc71eb707a03a9b04b001bad604e32f3c9f9e50df`. The 670-entry manifest has SHA-256 `79d45f5b73a16c3f0b5e7bf116cbde6d9afd2aa612158908630f27437dbbab7d`. The Release desktop and both CLIs passed, and all six PE files reported `NotSigned`. The first Grok run lacked a record of the second turn's `load` hook. A rerun of the same package passed the hook checks and all 12 turns, followed by the group retry and Oh My Pi fixtures. Remote `main` was checked after acceptance at `8fab8d3bb7f3cfd0af7a7fbb9389cf1f6d612ec9` and included the build SHA. [Job 140567500726](https://github.com/blueokanna/Crabforge/actions/runs/46017272731/job/140567500726) · [Artifact 15207448487](https://github.com/blueokanna/Crabforge/actions/runs/46017272731/artifacts/15207448487)

Both local Windows Debug CLIs read an encrypted backup produced by Magpie `0.1.4936` and verified preview, apply, edit, copy, rename and removal for `patience` and `groupPatience`. The loopback fixture returned a complete SSE stream after an initial HTTP 503. The group with retries disabled returned HTTP 503 with its original error body and `Retry-After`. The Rust gateway regression covers member order, a nested opt-out, exhausted quota, an HTTP date beyond the retry budget and the status recorded for each attempt. The Windows desktop integration test saved the group and gateway defaults through the Rust bridge at widths of 1,200 and 560 logical pixels. [Gateway test](../../../crates/core/tests/gateway.rs) · [Desktop test](../../../gui/integration_test/console_layout_test.dart)

## 首段内容超时 / First-content timeout

Windows 本地测试使用 Magpie `0.1.4936` 生成的原版加密备份。两个 Debug CLI 验证只读预览及 `groupFirstToken`、`firstToken` 的导入、复制、改名和删除。回环 Chat 夹具在只有角色元数据时触发 HTTP 504 和回退，随后逐字节保留后备流。在收到首段内容后，流可以超过原时限继续输出。回归同时覆盖最后一个兼容成员、非流式请求、内层等待覆盖、嵌套轮询顺序以及组分享往返。桌面集成测试在 1,200 和 560 逻辑像素下通过 Rust 桥接保存了组级和网关默认值，并运行了九种语言的窄屏导航。

Local Windows testing used an encrypted backup produced by Magpie `0.1.4936`. Both Debug CLIs verified the read-only preview, imported `groupFirstToken` and `firstToken`, and exercised group copy, rename and removal. The loopback Chat fixture recorded HTTP 504 after receiving only role metadata, then preserved the fallback stream byte for byte. Once content arrived, the stream could continue beyond the original deadline. The regression also covered the last compatible member, non-streaming requests, a nested wait override, nested rotation order and group-share round trips. The desktop integration test saved group and gateway values through the Rust bridge at widths of 1,200 and 560 logical pixels, and exercised narrow navigation in all nine languages. [Gateway regression](../../../crates/core/tests/gateway.rs) · [Windows layouts](../../../gui/integration_test/console_layout_test.dart)

已下载并验收 `4968112838e68326b601f2d413ae42a4b0cac49e` 的 Windows Actions 包。Run `46009593581`、job `140541409874`、artifact `15203569739`。ZIP 为 65,809,862 字节，SHA-256 为 `3e57920eda348d3cb63f3e2896aa4d0d96f5112f123fb66727632dd3c3e14227`。670 项解压文件与归档逐项匹配，清单 SHA-256 为 `52c422ff59df1a3d750121295083250ece80060bc8ce5b61ffbfcfdf25fa38e6`，六个 PE 均为 `NotSigned`。CI 通过 145 项 Rust 测试，忽略 33 项。下载版桌面成功启动和退出，两版 CLI 通过原版超时备份、回退和编辑检查，Pi、Gemini、Grok 各完成 12 轮。验收结束经代理查询，远端 `main` 与构建的完整 SHA 一致。 [Windows job](https://github.com/blueokanna/Crabforge/actions/runs/46009593581/job/140541409874) · [下载件](https://github.com/blueokanna/Crabforge/actions/runs/46009593581/artifacts/15203569739)

The downloaded Windows Actions package for `4968112838e68326b601f2d413ae42a4b0cac49e` passed acceptance. Run `46009593581`, job `140541409874` and artifact `15203569739` identify the 65,809,862-byte ZIP with SHA-256 `3e57920eda348d3cb63f3e2896aa4d0d96f5112f123fb66727632dd3c3e14227`. All 670 files matched the archive contents. The extracted manifest had SHA-256 `52c422ff59df1a3d750121295083250ece80060bc8ce5b61ffbfcfdf25fa38e6`, and all six PE files reported `NotSigned`. CI reported 145 Rust tests passed and 33 ignored. The downloaded desktop launched and closed cleanly, and both CLIs passed native backup, timeout fallback and editing checks. Pi, Gemini and Grok passed 12 turns each. The proxied remote `main` check at the end of acceptance matched the full build SHA. [Windows job](https://github.com/blueokanna/Crabforge/actions/runs/46009593581/job/140541409874) · [Artifact](https://github.com/blueokanna/Crabforge/actions/runs/46009593581/artifacts/15203569739)

## 分类缓存候选变化与恢复 / Classifier cache scope and recovery

提交 `8fab8d3bb7f3cfd0af7a7fbb9389cf1f6d612ec9` 的解压版双 CLI 通过分类候选变化、显式 `0` 复用、临时 503 后恢复和 SSE 等待夹具。Release 桌面正常启动和退出，Pi、Gemini、Grok 和 Oh My Pi 各通过 12 轮。Run `46015492103` 通过 153 项 Rust 测试，忽略 33 项。ZIP 为 66,957,507 字节，SHA-256 为 `8f469035f59901bb6ddf8624e9cdd71be8a126ee7547e0907fcd587dd4884dcc`。676 项清单的 SHA-256 为 `baafc1b2ad686285d187c635c24394a29a874faabfe2c21327182883ee9733aa`。六个 PE 均为 `NotSigned`。验收结束后经代理复核，远端 `main` 匹配该完整构建 SHA。 [Job 140561423475](https://github.com/blueokanna/Crabforge/actions/runs/46015492103/job/140561423475) · [Artifact 15206494701](https://github.com/blueokanna/Crabforge/actions/runs/46015492103/artifacts/15206494701)

Both CLIs extracted from `8fab8d3bb7f3cfd0af7a7fbb9389cf1f6d612ec9` passed fixtures for changed classifier candidates, reuse of an explicit `0`, recovery from temporary HTTP 503 responses, and SSE keepalive. The Release desktop opened and closed cleanly. Pi, Gemini, Grok, and Oh My Pi each passed 12 turns. Run `46015492103` passed 153 Rust tests with 33 ignored. The 66,957,507-byte ZIP has SHA-256 `8f469035f59901bb6ddf8624e9cdd71be8a126ee7547e0907fcd587dd4884dcc`. The 676-entry manifest has SHA-256 `baafc1b2ad686285d187c635c24394a29a874faabfe2c21327182883ee9733aa`. All six PE files reported `NotSigned`. A proxied check at the end of acceptance matched remote `main` to the full build SHA. [Job 140561423475](https://github.com/blueokanna/Crabforge/actions/runs/46015492103/job/140561423475) · [Artifact 15206494701](https://github.com/blueokanna/Crabforge/actions/runs/46015492103/artifacts/15206494701)

同一会话、同一用户轮的候选意图列表变化时会重新分类。列表恢复后可复用此前明确的 `0`，新列表的正向结果也可独立复用。分类器全部返回 HTTP 503 时先按普通路由处理，随后同轮请求再次分类，恢复后的判断可复用。Windows 网关回归和全量 153 项 Rust 测试通过，忽略 33 项，全目标 Clippy 通过。 [网关回归](../../../crates/core/tests/gateway.rs)

The gateway classifies again when the list of eligible intents changes within the same user turn. It can reuse the explicit `0` when the original list returns and independently reuse a positive answer for the new list. When every classifier returns HTTP 503, the gateway uses ordinary routing for that request, classifies again on the next request in the turn and reuses the recovered decision. The Windows gateway regression and all 153 enabled Rust tests passed, with 33 ignored. Clippy passed for all targets. [Gateway regression](../../../crates/core/tests/gateway.rs)

## SSE 保活 / SSE keepalives

Windows 本机通过 153 项 Rust 测试，忽略 33 项，全目标 Clippy 通过。保活回归覆盖跨块的 UTF-8 BOM 和 CRLF、完整事件边界及三种协议的晚到错误。两版 Debug CLI 均在回环 Chat 夹具中收到两次等待注释，然后在九秒首段超时后收到完整后备流。启用等待保活时，晚到的上游 HTTP 503 和 `Retry-After` 出现在 SSE 错误事件中。默认请求保留 HTTP 503、重试头与原始错误正文。 [网关测试](../../../crates/core/tests/gateway.rs) · [SSE 单测](../../../crates/core/src/gateway/keepalive.rs)

On Windows, 153 Rust tests passed, 33 were ignored, and Clippy passed for all targets. The keepalive regressions covered a split UTF-8 BOM, split CRLF, complete event boundaries and late error frames in all three protocols. Both Debug CLIs received two wait comments from the loopback Chat fixture and the complete fallback stream after the nine-second first-content deadline. With wait keepalives enabled, the late upstream HTTP 503 and `Retry-After` arrived in the SSE error event. The default request preserved HTTP 503, the retry header and the original error body. [Gateway test](../../../crates/core/tests/gateway.rs) · [SSE unit tests](../../../crates/core/src/gateway/keepalive.rs)

Windows 低负载复测观察了 30 分钟。采样每 30 秒查看一次约十秒的全机 CPU 忙碌率。观察期内各采样窗均高于探针采用的 25% 启动阈值，因此未启动新的配对流测试。此前的 60 次回环请求和同期 CPU 数据仍可复核。 [请求 CSV](sse-repeat-46005007334.csv) · [CPU CSV](sse-repeat-46005007334-cpu.csv)

The Windows low-load probe observed the host for 30 minutes. About every 30 seconds, it measured system-wide CPU busy time over roughly ten seconds. Every sampled window exceeded the probe's 25% start threshold, so the probe did not start a new paired stream run. The earlier 60 loopback requests and their concurrent CPU measurements remain available. [Request CSV](sse-repeat-46005007334.csv) · [CPU CSV](sse-repeat-46005007334-cpu.csv)

后续将下载的 `465897d` 和 `cf76ad3` 解包后各做 20 次回环 SSE 请求。首事件最小值／中位数／最大值分别为 `3.660 / 19.713 / 285.920 ms` 和 `3.834 / 31.132 / 189.335 ms`。两包均全部返回 HTTP 200，每次只发出一次上游请求，代理收到零次 CONNECT。分段计时显示长尾主要发生在上游收到请求之前。这一批未记录同期 CPU，低负载验收仍待完成。 [原始 CSV](465897d-cf76ad3-forward-h1-repeat.csv) · [465897d 构建](https://github.com/blueokanna/Crabforge/actions/runs/46111496870) · [cf76ad3 构建](https://github.com/blueokanna/Crabforge/actions/runs/46110437610)

A later loopback SSE run sent 20 requests each through the downloaded `465897d` and `cf76ad3` packages. The minimum, median, and maximum times to the first event were `3.660 / 19.713 / 285.920 ms` and `3.834 / 31.132 / 189.335 ms`, respectively. All requests returned HTTP 200, made one upstream request, and sent zero CONNECT requests to the proxy. Timing by request phase placed most of the long tail before the upstream received the request. This run did not capture concurrent CPU measurements, so low-load acceptance is still pending. [Raw CSV](465897d-cf76ad3-forward-h1-repeat.csv) · [465897d build](https://github.com/blueokanna/Crabforge/actions/runs/46111496870) · [cf76ad3 build](https://github.com/blueokanna/Crabforge/actions/runs/46110437610)

同期 CPU 复测交替运行两包各 20 次。`465897d` 和 `cf76ad3` 的首事件最小值／中位数／最大值分别为 `2.351 / 3.643 / 13.456 ms` 和 `2.170 / 4.110 / 23.187 ms`。40 次均返回 HTTP 200，每次各一次上游请求、零次代理 CONNECT。全机 CPU 忙碌率基于 Windows `GetSystemTimes`，采样窗包含网关启动、请求和退出。两包的采样范围分别为 `63.963–98.798%` 和 `59.707–96.354%`，均超过 25% 的低负载阈值。 [原始 CSV](465897d-cf76ad3-forward-h1-with-cpu.csv)

The repeat with concurrent CPU measurement alternated 20 trials per package. First-event minimum, median, and maximum were `2.351 / 3.643 / 13.456 ms` for `465897d` and `2.170 / 4.110 / 23.187 ms` for `cf76ad3`. All 40 returned HTTP 200 with one upstream request and zero proxy CONNECT requests per trial. Windows `GetSystemTimes` measured system-wide CPU busy time over each trial, including gateway startup, request, and shutdown. The ranges were `63.963–98.798%` and `59.707–96.354%`, both above the 25% low-load threshold. The CSV SHA-256 is `58ded33c438156bd9e4f33f9891ad20207b92fe94a4fef4835ed6321d5d1a6d9`. [Raw CSV](465897d-cf76ad3-forward-h1-with-cpu.csv)

2026 年 11 月 28 日 07:43 至 07:57 UTC，本机观察了 30 个 CPU 采样窗。忙碌率为 `55.225–97.894%`，均值 `73.779%`。所有采样窗均超过 `25%` 启动阈值，未启动配对 SSE 请求。观察记录的 SHA-256 为 `366f459d498f69a48c9959c627fe35391f9bdfac21747bbc0d8723f9e74da810`。 [CPU 观察 CSV](465897d-cf76ad3-low-load-windows.csv)

On 28 November 2026 from 07:43 to 07:57 UTC, the Windows probe collected 30 CPU observation windows. Host CPU busy time ranged from `55.225%` to `97.894%` and averaged `73.779%`. Every window exceeded the `25%` start threshold, so the probe issued no paired SSE requests. The observation CSV SHA-256 is `366f459d498f69a48c9959c627fe35391f9bdfac21747bbc0d8723f9e74da810`. [CPU observations](465897d-cf76ad3-low-load-windows.csv)

## 自动记忆提炼 / Automatic memory distillation

提交 `0a759eaec3daa0fbe19f7bc3157f65eb5dd552a3` 的 Windows Debug 双 CLI 通过默认关闭、网关启动后调度、按路由组推理、会话静默筛选、只接受用户原话支持的候选、显式审核、重启去重和禁用检查。Rust 全工作区 170 项通过，30 项忽略。调度回归验证中断后继续使用已保存的小时尝试计数。 [调度实现和回归](../../../crates/core/src/memory/distill/schedule.rs)

Both Windows Debug CLIs from `0a759eaec3daa0fbe19f7bc3157f65eb5dd552a3` verified that distillation is off by default, starts with the gateway when enabled, routes through the selected group, waits for quiet sessions, requires a supporting user quote, leaves candidates for explicit review, deduplicates after restart, and can be disabled. The Rust workspace passed 170 tests with 30 ignored. The scheduling regression verified that an interrupted run resumes with the saved hourly attempt count. [Scheduler and regression](../../../crates/core/src/memory/distill/schedule.rs)

Windows 桌面集成在 1200 和 560 像素宽分别启用与关闭提炼，核对组选择和磁盘状态，并打开全部九种语言的设置。并行读取 `settings.json` 曾与 Windows 原子替换产生共享冲突。改为等待保存完成后经 Rust 桥读取配置，集成测试通过。Flutter 分析无问题，四项 Flutter 单测通过。 [桌面集成测试](../../../gui/integration_test/console_layout_test.dart)

The Windows desktop integration enabled and disabled distillation at widths of 1200 and 560 pixels, verified the selected group and persisted state, and opened the settings in all nine languages. The earlier direct read of `settings.json` hit a Windows sharing violation during atomic replacement. The rerun waited for the save to finish and read the persisted settings through the Rust bridge. The integration passed, Flutter analysis reported no issues, and all four Flutter unit tests passed. [Desktop integration](../../../gui/integration_test/console_layout_test.dart)

## `f497750` 历史包 / Historical package `f497750`

Run `46110378934` 为 `f4977507bb2f5cad4123af39cfd6c227476fe554` 通过 160 项 Rust 测试，忽略 33 项。ZIP 为 67,194,253 字节，SHA-256 为 `e28f11c4b12c6f91357bffb34a747ee9e5636ecc6bc9064ca654a7ff8210e201`。676 项解包清单的 SHA-256 为 `b3cb94e848e292e2705df8fdc98d339f5612ea192744aefdb0169190a8478f45`，六个 PE 均为 `NotSigned`。接受了 29 项夹具标志，覆盖原产品 PNG、Release 桌面、意图置信度、Grok 和 Oh My Pi 各 12 轮。此前两次 Grok 试跑分别只记录了未筛选的标记和 `new` 标记。续跑同一包时完整通过 `new`、`load`、未筛选钩子和原生历史检查。验后远端 `main` 为 `19477340ec38eb64497d600683fd98e9604e09f8`，包含该构建提交。 [Run 46110378934](https://github.com/blueokanna/Crabforge/actions/runs/46110378934) · [Artifact 15250260787](https://github.com/blueokanna/Crabforge/actions/runs/46110378934/artifacts/15250260787)

Run `46110378934` passed 160 Rust tests with 33 ignored for `f4977507bb2f5cad4123af39cfd6c227476fe554`. The 67,194,253-byte ZIP has SHA-256 `e28f11c4b12c6f91357bffb34a747ee9e5636ecc6bc9064ca654a7ff8210e201`. The 676-entry extraction manifest has SHA-256 `b3cb94e848e292e2705df8fdc98d339f5612ea192744aefdb0169190a8478f45`, and all six PE files reported `NotSigned`. Acceptance covered 29 fixture flags, including the original product PNG, the Release desktop, intent confidence, and 12 turns each in Grok and Oh My Pi. Two earlier Grok attempts recorded only the unfiltered marker and only the `new` marker, respectively. The resumed run on the same package passed the `new`, `load`, and unfiltered hook checks and verified the native history. Remote `main` at acceptance was `19477340ec38eb64497d600683fd98e9604e09f8` and contained the build commit. [Run 46110378934](https://github.com/blueokanna/Crabforge/actions/runs/46110378934) · [Artifact 15250260787](https://github.com/blueokanna/Crabforge/actions/runs/46110378934/artifacts/15250260787)

## 方舟构建及兼容性验收 / Ark build and compatibility acceptance

构建 `19477340ec38eb64497d600683fd98e9604e09f8` 的下载包通过 37 项夹具标志，覆盖 Release 桌面、原 PNG、两个 CLI、备份迁移和四种决策接口。Windows CI 通过 169 项 Rust 测试，忽略 30 项。ZIP 为 67,154,550 字节，SHA-256 为 `a272b2e357d4b63472b3b50213c5b5b304e311810008ad690edcb65a37a4dea8`。556 项解包清单的 SHA-256 为 `36327645f8025c4b4b044db5d24b4f123e56f7c66f658827032cbb93e9fd8fae`，六个 PE 均报告 `NotSigned`。验后远端 `main` 精确匹配该构建。 [Run 46114171055](https://github.com/blueokanna/Crabforge/actions/runs/46114171055) · [Artifact 15251431680](https://github.com/blueokanna/Crabforge/actions/runs/46114171055/artifacts/15251431680)

The downloaded `19477340ec38eb64497d600683fd98e9604e09f8` package passed 37 fixture flags covering the Release desktop, original PNG, both CLIs, backup migration, and four decision adapters. Windows CI passed 169 Rust tests with 30 ignored. The 67,154,550-byte ZIP has SHA-256 `a272b2e357d4b63472b3b50213c5b5b304e311810008ad690edcb65a37a4dea8`. The 556-entry extraction manifest has SHA-256 `36327645f8025c4b4b044db5d24b4f123e56f7c66f658827032cbb93e9fd8fae`, and all six PE files reported `NotSigned`. Remote `main` matched the full build SHA after acceptance. [Run 46114171055](https://github.com/blueokanna/Crabforge/actions/runs/46114171055) · [Artifact 15251431680](https://github.com/blueokanna/Crabforge/actions/runs/46114171055/artifacts/15251431680)

以下历史包各验证 676 项文件、原 PNG、Release 桌面与两版 CLI。每包的六个 PE 均报告 `NotSigned`。验后提交已确认包含对应构建。

Each historical package below verified 676 files, the original PNG, the Release desktop, and both CLIs. All six PE files in each package reported `NotSigned`. The verified remote commits contained the corresponding build commits.

| 完整构建 SHA / Full build SHA | Rust 通过／忽略 / Passed / ignored | 夹具 / Fixture flags | ZIP 字节数及 SHA-256 / ZIP bytes and SHA-256 | 解包清单 SHA-256 / Extraction manifest SHA-256 | 验后远端 / Verified remote `main` | 构建和制品 / Build and artifact |
| --- | ---: | ---: | --- | --- | --- | --- |
| `e3716f226bdb342399a346c0d42ff9d8cfe8446d` | 161 / 33 | 30 | 67,263,926 · `fc19181fbc16fadfc9fb62a324c469fe180919951fd6874238f293bbce6b9826` | `9f1056079097bdf2045d29e92cc82e8575cc1bbcac446ab7670bb9faa3ac0b40` | `19477340ec38eb64497d600683fd98e9604e09f8` | [Run 46110387122](https://github.com/blueokanna/Crabforge/actions/runs/46110387122) · [Artifact 15249289711](https://github.com/blueokanna/Crabforge/actions/runs/46110387122/artifacts/15249289711) |
| `601ccf1fcba99da98895a9963ccc49c20699359d` | 162 / 33 | 31 | 67,328,800 · `f10137392667066160b4c02805111483c25896ab64b4f74e9e90553ed2fa71be` | `6880b3f30f2d8b2ba4d2356c1d80ab7bdf474d9872d84af508920558f8bd26c4` | `cd80a5365ea85d50d3366147c9482ec50d7e8cbe` | [Run 46110394891](https://github.com/blueokanna/Crabforge/actions/runs/46110394891) · [Artifact 15249764814](https://github.com/blueokanna/Crabforge/actions/runs/46110394891/artifacts/15249764814) |
| `c660af3e14138e826bdc287876a2ff3f94fb7647` | 164 / 33 | 32 | 67,508,059 · `b982f485877c0e78a2eed0fe174d10977a9e36ecab14fa5c8a28cbabb9be5c07` | `ae85481bd8cbeb1a69455480e0fe3be8fd62b3c9edb839a8f23ea233d4e84b6c` | `ec5e935b1bf0c75a5e2d3770ecdc9a4e99c71411` | [Run 46110403373](https://github.com/blueokanna/Crabforge/actions/runs/46110403373) · [Artifact 15249833732](https://github.com/blueokanna/Crabforge/actions/runs/46110403373/artifacts/15249833732) |
| `3ff81cfc68425df17a1e74e49d11b9d92d61cea1` | 165 / 33 | 33 | 67,551,982 · `8740560ace6476c7a6526d086717506d411c0cdc53857c5e9d05c5b61be3c8fd` | `5ddcca3c89637e8618e22d20d8eb5c69ff317c749e56df60214d43cdfa8a4aa9` | `ec5e935b1bf0c75a5e2d3770ecdc9a4e99c71411` | [Run 46110411671](https://github.com/blueokanna/Crabforge/actions/runs/46110411671) · [Artifact 15250600534](https://github.com/blueokanna/Crabforge/actions/runs/46110411671/artifacts/15250600534) |
| `9782a0ff31c321e16d75612e1d82bd1be31b5afd` | 165 / 33 | 34 | 67,568,822 · `388ae95ce806cfe3b728fdea1d520e4f891179b4aedb35f870a24a9211e4b170` | `71b2e981457f9009adbe53c0d795fd6d83a5b82494ec4c330713e4547d190a00` | `ec5e935b1bf0c75a5e2d3770ecdc9a4e99c71411` | [Run 46110420225](https://github.com/blueokanna/Crabforge/actions/runs/46110420225) · [Artifact 15250591042](https://github.com/blueokanna/Crabforge/actions/runs/46110420225/artifacts/15250591042) |
| `96ca6e3917ec40474f13a30f6bf02413506210e9` | 165 / 33 | 35 | 67,600,960 · `bd4c68fe5f1a3dd8b35f20b4e6bbfde2bc5746fab4d69437dcad170b57800ad3` | `4bb143173d796b9c2bcaa9649422c5bdd8115bfee86af4adba4ff1b03357d22c` | `ec5e935b1bf0c75a5e2d3770ecdc9a4e99c71411` | [Run 46110430070](https://github.com/blueokanna/Crabforge/actions/runs/46110430070) · [Artifact 15249119468](https://github.com/blueokanna/Crabforge/actions/runs/46110430070/artifacts/15249119468) |

## 记忆提炼构建 / Memory distillation build

构建 `0a759eaec3daa0fbe19f7bc3157f65eb5dd552a3` 的下载包通过 38 项夹具标志。双 CLI 验证手动及自动提炼、基于用户原话的候选记忆和重启保留。Windows CI 通过 170 项 Rust 测试，忽略 30 项。67,315,604 字节的 ZIP SHA-256 为 `51cfd22ba1fe4be7dfafac3420cbb889f63d16f3b35df66c7fe7d2b7ab79aaf5`，556 项解包清单 SHA-256 为 `11bcebda33b7980a9cd40286dd7cd28b1f2b6d6e6b388af1b1c86d060381c322`。六个 PE 均报告 `NotSigned`。验后远端 `main` 为 `ec5e935b1bf0c75a5e2d3770ecdc9a4e99c71411`，包含该构建。 [Run 46115874366](https://github.com/blueokanna/Crabforge/actions/runs/46115874366) · [Artifact 15252028183](https://github.com/blueokanna/Crabforge/actions/runs/46115874366/artifacts/15252028183)

The downloaded `0a759eaec3daa0fbe19f7bc3157f65eb5dd552a3` build passed 38 fixture flags. Both CLIs verified manual and automatic memory distillation, candidates backed by users' original words, and persistence across restart. Windows CI passed 170 Rust tests with 30 ignored. The 67,315,604-byte ZIP has SHA-256 `51cfd22ba1fe4be7dfafac3420cbb889f63d16f3b35df66c7fe7d2b7ab79aaf5`. The 556-entry extraction manifest has SHA-256 `11bcebda33b7980a9cd40286dd7cd28b1f2b6d6e6b388af1b1c86d060381c322`. All six PE files reported `NotSigned`. Remote `main` was `ec5e935b1bf0c75a5e2d3770ecdc9a4e99c71411` after acceptance and contained the build commit. [Run 46115874366](https://github.com/blueokanna/Crabforge/actions/runs/46115874366) · [Artifact 15252028183](https://github.com/blueokanna/Crabforge/actions/runs/46115874366/artifacts/15252028183)

后续构建 `cd80a5365ea85d50d3366147c9482ec50d7e8cbe` 复跑了 38 项下载包夹具。Windows CI 通过 170 项 Rust 测试，忽略 30 项。67,315,609 字节的 ZIP SHA-256 为 `38acf59c52d9e1a312e1ff0bd705ca9cbdae22e2bbbd7cb8b66f8d869aaf1db5`，556 项解包清单 SHA-256 为 `849fb876bebfca09d933f1ce596c80f6eb7d24d5148f8d61c89c9b6c74a3efcf`。六个 PE 均报告 `NotSigned`。验后远端 `main` 为 `ec5e935b1bf0c75a5e2d3770ecdc9a4e99c71411`，包含该构建。 [Run 46116259096](https://github.com/blueokanna/Crabforge/actions/runs/46116259096) · [Artifact 15252297977](https://github.com/blueokanna/Crabforge/actions/runs/46116259096/artifacts/15252297977)

The follow-up build `cd80a5365ea85d50d3366147c9482ec50d7e8cbe` passed all 38 downloaded-package fixture flags. Windows CI passed 170 Rust tests with 30 ignored. The 67,315,609-byte ZIP has SHA-256 `38acf59c52d9e1a312e1ff0bd705ca9cbdae22e2bbbd7cb8b66f8d869aaf1db5`. The 556-entry extraction manifest has SHA-256 `849fb876bebfca09d933f1ce596c80f6eb7d24d5148f8d61c89c9b6c74a3efcf`. All six PE files reported `NotSigned`. Remote `main` was `ec5e935b1bf0c75a5e2d3770ecdc9a4e99c71411` after acceptance and contained the build commit. [Run 46116259096](https://github.com/blueokanna/Crabforge/actions/runs/46116259096) · [Artifact 15252297977](https://github.com/blueokanna/Crabforge/actions/runs/46116259096/artifacts/15252297977)

## 动态目录构建 / Dynamic directory builds

`b71dba3cb007f0f9dbaf86e0de54bc738337683f` 的 multipart 夹具将字节数组传给字符串请求接口，目录刷新还触发未读赋值。`ec5e935b1bf0c75a5e2d3770ecdc9a4e99c71411` 在目录刷新分支触发 Clippy。两次 Windows CI 均在 Rust 检查阶段失败，制品数均为零。 [Run 46116339104](https://github.com/blueokanna/Crabforge/actions/runs/46116339104) · [Run 46116558964](https://github.com/blueokanna/Crabforge/actions/runs/46116558964)

The `b71dba3cb007f0f9dbaf86e0de54bc738337683f` multipart fixture passed bytes to a string request API, and its directory refresh triggered an unread-assignment diagnostic. In `ec5e935b1bf0c75a5e2d3770ecdc9a4e99c71411`, Clippy rejected a directory refresh branch. Both Windows runs failed during the Rust checks and produced zero artifacts. [Run 46116339104](https://github.com/blueokanna/Crabforge/actions/runs/46116339104) · [Run 46116558964](https://github.com/blueokanna/Crabforge/actions/runs/46116558964)

构建 `89f1d5369fb90f3192a3f14c114da9d02e36d27e` 通过 39 项下载包夹具标志、173 项 Windows CI Rust 测试，忽略 30 项。两个 CLI 均验证远端目录鉴权、规则纳入、下架处理、网关运行时刷新和重启读回。67,688,327 字节的 ZIP SHA-256 为 `98be74992c98e3f9ff4f82038d1b5f17c1360cccaf0d4b8e706d964f1fdfee4b`，556 项解包清单 SHA-256 为 `a6fff643e2b728a28ee1e42564b22be7a38400efa5635698bdd3a6172c20a6c3`。六个 PE 均报告 `NotSigned`。产品 PNG 和 Release 桌面冒烟测试通过，验后 `main` 与构建 SHA 完全一致。对同一 SHA 的干净检出另跑 Windows Debug 集成测试，验证 1200／560 像素布局、模型目录、九语与字体重启读回。 [Run 46119257571](https://github.com/blueokanna/Crabforge/actions/runs/46119257571) · [Artifact 15254211022](https://github.com/blueokanna/Crabforge/actions/runs/46119257571/artifacts/15254211022)

Build `89f1d5369fb90f3192a3f14c114da9d02e36d27e` passed 39 downloaded-package fixture flags and 173 Windows CI Rust tests, with 30 ignored. Both CLIs verified remote directory authentication, inclusion rules, delisting, live refresh, and persistence across restart. The 67,688,327-byte ZIP has SHA-256 `98be74992c98e3f9ff4f82038d1b5f17c1360cccaf0d4b8e706d964f1fdfee4b`. The 556-entry extraction manifest has SHA-256 `a6fff643e2b728a28ee1e42564b22be7a38400efa5635698bdd3a6172c20a6c3`. All six PE files reported `NotSigned`. The product PNG and Release desktop smoke check passed. Remote `main` matched the build SHA exactly after acceptance. A separate Windows Debug integration run from a clean checkout of the same SHA verified the 1200- and 560-pixel layouts, the model directory, all nine languages, and font persistence across restart. [Run 46119257571](https://github.com/blueokanna/Crabforge/actions/runs/46119257571) · [Artifact 15254211022](https://github.com/blueokanna/Crabforge/actions/runs/46119257571/artifacts/15254211022)

## 每日发布记忆 / Daily memory publication

在 Claude Code 或 Codex 的「用户指令」中启用「每日发布记忆」，或在双 CLI 的 `memory agent-instructions <claude|codex>` 子命令中使用 `--auto-publish`。网关启动时及每小时检查到期目标，启用满 24 小时后首次自动发布。关闭开关或使用 `--stop-auto-publish` 会停止自动更新，已写入的摘要仍可从用户指令中查看。更换文件或编辑托管摘要后，可重新加载并重新启用以确认目标。 [调度实现](../../../crates/core/src/project_instructions/publication.rs)

Enable **Publish memory daily** in **User instructions** for Claude Code or Codex, or use `--auto-publish` with `memory agent-instructions <claude|codex>` in either CLI. The gateway checks for due targets at startup and hourly. The first automatic publication is due 24 hours after opt-in. Turning the switch off or using `--stop-auto-publish` stops updates and keeps the published digest in the instruction file. After switching files or editing the managed digest, reload and re-enable the setting to approve the target. [Scheduler](../../../crates/core/src/project_instructions/publication.rs)

Windows Debug 桌面复验在 1200 像素启用 Claude Code 的每日发布，在 560 像素重新打开对话框确认状态后关闭。双 CLI 夹具使用隔离的 Claude Code 和 Codex 目录，验证到期后重启网关即发布、两种 Agent 独立设置、手动发布仍可用、候选和其他 Agent 的记忆排除、BOM 与 CRLF 保留。Rust 回归覆盖外部编辑、目标切换和中断恢复。 [桌面集成测试](../../../gui/integration_test/memory_publication_test.dart)

The Windows Debug desktop test enabled daily publication for Claude Code at 1200 pixels, reopened the dialog at 560 pixels to confirm persistence, and then disabled it. Fixtures for both CLIs used isolated Claude Code and Codex directories. They verified due publication at gateway startup, independent opt-in, manual publication, exclusion of candidates and other agents' notes, and preservation of BOM and CRLF. The Rust regression test covered external edits, target changes, and interrupted writes. [Desktop integration test](../../../gui/integration_test/memory_publication_test.dart)

每日发布提交 `ad2dee070aadd2acbbdbe00c97b481cfe2e5cf04` 的干净检出通过两次 Windows 桌面测试。第一轮在宽布局启用并在窄布局重新确认，第二轮启动新的进程，在窄布局读回并关闭。两轮均逐字节核对原指令文件。 [桌面集成测试](../../../gui/integration_test/memory_publication_test.dart)

Both Windows desktop runs passed from a clean checkout of `ad2dee070aadd2acbbdbe00c97b481cfe2e5cf04`. The first enabled daily publication in the wide layout and confirmed it in the narrow layout. The second launched a new process, confirmed the persisted setting in the narrow layout, then disabled it. Both runs compared the instruction file byte for byte. [Desktop integration test](../../../gui/integration_test/memory_publication_test.dart)

提交 `ad2dee070aadd2acbbdbe00c97b481cfe2e5cf04` 的 Windows 下载包通过双 CLI、Release 桌面和 `--memory-publication` 在内的 40 项夹具。验包后远端 `main` 精确匹配该提交。Rust 174 项通过、30 项忽略。Artifact `15254507568` 的 ZIP 为 67,784,560 字节，SHA-256 为 `5ccd918f2589040eb87477461c00eb9391979badc8288a3eacaaf5742a5e38f2`。验包生成的 556 项文件清单 SHA-256 为 `0b48049eff9b7c9c1585002518d522ac26ada682fdb500f26f49c9983af3d091`。六个 PE 的 Authenticode 状态均为 `NotSigned`。 [Windows Run 46122083209](https://github.com/blueokanna/Crabforge/actions/runs/46122083209)

The Windows package for `ad2dee070aadd2acbbdbe00c97b481cfe2e5cf04` passed checks on both CLIs and the Release desktop, including 40 fixture flags with `--memory-publication`. Remote `main` matched the build SHA exactly after acceptance. Rust reported 174 passed and 30 ignored. Artifact `15254507568` was 67,784,560 bytes with ZIP SHA-256 `5ccd918f2589040eb87477461c00eb9391979badc8288a3eacaaf5742a5e38f2`. The 556-entry manifest generated during acceptance had SHA-256 `0b48049eff9b7c9c1585002518d522ac26ada682fdb500f26f49c9983af3d091`. All six PE files reported `NotSigned` for Authenticode. [Windows run 46122083209](https://github.com/blueokanna/Crabforge/actions/runs/46122083209)

## 夜间整理记忆 / Nightly memory tidy

Windows 本机通过 176 项 Rust 测试，忽略 30 项，全目标严格 Clippy 通过。Rust 回归覆盖纽约夏令时切换日的本地 03:00 边界、整理写入中断后的恢复，以及撤销时保留外部编辑。两个 Debug CLI 均通过回环模型夹具，验证网关启动时补跑到期任务、顺序组路由、模型失败保留原件、同作用域合并和来源保留、保护手写和置顶项、过期候选归档及逐字节撤销。Windows Debug 桌面在 1200 像素启用并立即整理，在 560 像素重开后撤销并停用，核对原文件的 BOM 和 CRLF。 [Rust 回归](../../../crates/core/src/memory/tidy.rs) · [桌面集成测试](../../../gui/integration_test/memory_tidy_test.dart)

On Windows, 176 Rust tests passed, 30 were ignored, and strict Clippy passed for all targets. Rust regressions covered the local 3:00 AM boundary on New York's daylight saving transition, recovery after an interrupted tidy, and preservation of later edits during undo. Both Debug CLIs used a loopback model fixture to verify startup catch-up, group routing, preservation of originals on model failure, same-scope merging and provenance, protection for handwritten and pinned notes, candidate expiry, and byte-exact undo. The Windows Debug desktop enabled and ran the tidy at 1200 pixels, then reopened the dialog, undid the tidy, and disabled it at 560 pixels. The test verified the original file's BOM and CRLF. [Rust regressions](../../../crates/core/src/memory/tidy.rs) · [Desktop integration test](../../../gui/integration_test/memory_tidy_test.dart)

## WebDAV 记忆同步 / WebDAV memory sync

两套 Windows Debug CLI 通过 Magpie `0.1.4936` 加密备份双向互读、预览、条件写、逐字节读回、凭据隔离、删除记录、保护记忆和首次创建夹具。Rust 回归验证保留 `mergedFrom`，并忽略超过 180 天的删除记录。Windows Debug 桌面在 1200 像素预览，在 560 像素应用，夹具核对 Basic 鉴权、条件写和本地记忆。 [Rust 回归](../../../crates/core/src/memory/sync.rs) · [桌面集成测试](../../../gui/integration_test/memory_sync_test.dart)

Both Windows Debug CLIs passed fixtures for two-way reads of a Magpie `0.1.4936` encrypted backup, preview, conditional writes, byte-for-byte readback, credential isolation, tombstones, protection for notes, and first sync. The Rust regression verified that WebDAV retains `mergedFrom` and ignores tombstones older than 180 days. The Windows Debug desktop previewed at 1200 pixels and applied at 560 pixels. Its fixture checked Basic authentication, the conditional write, and the local memory library. [Rust regression](../../../crates/core/src/memory/sync.rs) · [Desktop integration test](../../../gui/integration_test/memory_sync_test.dart)

## 签名与许可 / Signing and licenses

本地隔离测试使用临时自签名证书。SignTool 添加了 RFC 3161 时间戳，随后因证书链不受信任拒绝该文件。签名脚本清除了临时密钥，发布目录未创建。正式发布需要受信任的发布者密钥。

An isolated local test used a temporary self-signed certificate. SignTool applied an RFC 3161 timestamp and then rejected the binary because the certificate chain was untrusted. The signing script removed the temporary private key and created no publishing directory. A trusted publisher credential is required for the public release. [Signing guide](../../windows-release.en.md)

GitHub 发布试跑以 `ba09164c4a2265462ef736d64722f1b20b085206` 为源。`identify` 和 `build` 成功，`sign` 下载并校验 CI 包后报告三项签名设置缺失，`publish` 跳过。仓库 Release 列表为零条。

The GitHub release trial used source `ba09164c4a2265462ef736d64722f1b20b085206`. `identify` and `build` passed. `sign` downloaded and verified the CI archive, then reported that the three signing settings were missing. `publish` was skipped, and the repository had zero release entries. [Release trial 45952988583](https://github.com/blueokanna/Crabforge/actions/runs/45952988583) · [Signing job 140361568621](https://github.com/blueokanna/Crabforge/actions/runs/45952988583/job/140361568621)

公开 `v0.1.0` 的标签和 GitHub 构建来源证明均对应 `cf76ad373c90a14abfc32a3803ec970767404d8c`。通过发布页下载的 ZIP 为 67,369,055 字节，SHA-256 为 `0715b23e9e77b95e92f5b0b4ce24b5cdcf299b5f99905ac7c7207bd69ea3d604`。ZIP 和 `.sha256` 附件均与 GitHub 记录的摘要一致。包内 `SHA256SUMS` 的 SHA-256 为 `3962e8daa3d60ade364188c83ca43b8759ff35837786539c74cc3dff76956554`，记录的 556 个文件逐一通过校验。两个 CLI 均报告 `0.1.0`，产品 PNG 与源提交一致，Release 桌面可响应并正常退出。六个 PE 均报告 `NotSigned`。验收时远端 `main` 为 `19477340ec38eb64497d600683fd98e9604e09f8`，包含该标签提交。 [公开发布](https://github.com/blueokanna/Crabforge/releases/tag/v0.1.0) · [发布 Run 46111898874](https://github.com/blueokanna/Crabforge/actions/runs/46111898874)

The public `v0.1.0` tag and the GitHub build attestation both identify `cf76ad373c90a14abfc32a3803ec970767404d8c`. The ZIP downloaded from the release is 67,369,055 bytes with SHA-256 `0715b23e9e77b95e92f5b0b4ce24b5cdcf299b5f99905ac7c7207bd69ea3d604`. Both the ZIP and its `.sha256` attachment match GitHub's recorded digests. The internal `SHA256SUMS` has SHA-256 `3962e8daa3d60ade364188c83ca43b8759ff35837786539c74cc3dff76956554`, and all 556 file hashes verified. Both CLIs reported `0.1.0`, the product PNG matched the source commit, and the Release desktop responded and closed cleanly. All six PE files reported `NotSigned`. Remote `main` was `19477340ec38eb64497d600683fd98e9604e09f8` at acceptance and contained the tag commit. [Public release](https://github.com/blueokanna/Crabforge/releases/tag/v0.1.0) · [Release run 46111898874](https://github.com/blueokanna/Crabforge/actions/runs/46111898874)

双语发布指南的 PowerShell 示例通过了本地验证。有效时间戳签名和 UTF-8 路径可正确核对，空清单、缺失清单、篡改文件及未签名的 CI 二进制均被拒绝。

The bilingual release guides' PowerShell example verified a timestamped signature and a UTF-8 file path. It rejected missing and empty manifests, a modified file and an unsigned CI executable. [中文发布指南](../../windows-release.md) · [English release guide](../../windows-release.en.md)

从 `26acbe6` 到 `17cae79` 的 CI 归档含 311 个 Rust 许可目录，`1faa92f` 和 `95298d2` 的归档含 313 个。打包脚本为 28 个 crate 加入对应上游提交的许可正文。`base256emoji`、`base45`、`futures-bounded`、`if-watch`、`match-lookup`、`no-proxy` 和 `yaml-edit` 在对应提交的包目录至仓库根目录中只有 Cargo 许可声明可供归档。

The checked artifacts from `26acbe6` through `17cae79` have 311 Rust license directories. The `1faa92f` and `95298d2` packages each have 313. Packaging adds license texts from the corresponding publication commits for 28 crates. For `base256emoji`, `base45`, `futures-bounded`, `if-watch`, `match-lookup`, `no-proxy`, and `yaml-edit`, the audit retained the Cargo license declaration after checking the package directory and its ancestors at the publication commit. [Supplemental license texts and source revisions](../../../licenses/rust-upstream.txt)
