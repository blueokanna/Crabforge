# Crabforge 0.1.0 验收记录

[English](crabforge-validation.en.md)

### 品牌资产和签名

`assets/brands` 中的 119 个文件与本地 Magpie 参考快照的 `internal/gui/assets/icons` 逐文件 SHA-256 相同。包内附带 Magpie 与 Lobe Icons 的 MIT 声明。OpenAI 的标志使用另受品牌指南及其授权条款约束，Anthropic 商业服务条款第 F 节将知识产权授权限于条款明确授予的范围。正式售卖所需的品牌授权核定仍在进行。当前 Windows 用户没有可访问私钥的代码签名证书，冻结包的 Authenticode 状态为 `NotSigned`。[第三方声明](../THIRD_PARTY_NOTICES) · [OpenAI 品牌指南](https://openai.com/brand/#usage-terms) · [Anthropic 商业条款](https://www.anthropic.com/legal/commercial-terms#f-intellectual-property)

### 子代理来源、项目分配与原生钩子恢复

外部子代理导入提交 `7b5d20afc69aff6347ec2bd5846d5e199f89ce67` 通过 Windows Debug 宽窄布局的本地导入和更新。CLI 从本机智能 HTTPS Git 仓库的 `feature/review` 导入，并重新打开保存的来源。公开 `msitarzewski/agency-agents` 工程目录返回 66 个候选。导入 `backend-architect` 后复查保存的 URL，状态为 `unchanged`。[本机 Git 夹具](validation/agent-library-followup/subagent-git-https.csv) · [公开 GitHub](validation/agent-library-followup/subagent-github.csv)

项目子代理分配提交 `92ba1938d645c295c968ba99f2b6ca6a71b5bcd0` 通过六项定向 Rust 测试、全目标 Clippy 和 Windows Debug 集成。CLI 将共享定义分别发布到两个项目的 Claude Code 和 Codex，更新两处提示词，撤销时保留外部编辑。桌面在宽窄窗口验证了项目分配与撤销，并保留用户配置副本。[CLI 结果](validation/agent-library-followup/shared-subagents.csv) · [Windows 集成](validation/agent-library-followup/shared-subagents.log)

原生钩子恢复提交 `18a92bf5098e7ecea83893577fe963e49331f8ac` 通过五项定向 Rust 测试、全目标 Clippy、Flutter 分析和 Windows Debug 集成。CLI 的 Codex 夹具验证了后续分组位置保留、过期移除和恢复被拒绝，以及 BOM、换行和注释逐字节恢复。桌面在网格中移除已有 Claude 钩子，并在列表中恢复。[CLI 结果](validation/agent-library-followup/hook-recovery.csv) · [Windows 集成](validation/agent-library-followup/hook-recovery.log)

斜杠命令来源导入通过七项定向 Rust 测试、全目标 Clippy 和 Windows Debug 集成。桌面在列表和网格中查看 TOML 命令，选中后将转换为 `$ARGUMENTS` 的提示词发布到 Claude Code。CLI 从本机智能 HTTPS Git 仓库的 `feature/review` 选择并导入 TOML 命令，复查保存的 URL 时状态为 `unchanged`。[Windows 集成](validation/agent-library-followup/command-import.log) · [本机 Git 夹具](validation/agent-library-followup/command-git-https.csv)

项目钩子分配提交 `9953bde0c9fbcd934574ca35a56f649fbfad1599` 通过七项定向 Rust 测试、全目标 Clippy、Flutter 分析和 560、1,280 像素布局的 Windows Debug 集成。测试覆盖项目隔离、过期修订、改名、保留外部编辑和逐字节恢复。Claude Code 2.1.332 和 Codex 0.172.0 执行了发布的 `SessionStart` 命令，并收到本机模型夹具的响应。Codex 在隔离配置中确认了项目和钩子哈希的信任状态。[原生客户端结果](validation/agent-library-followup/project-hooks-process.csv) · [Windows 集成](validation/agent-library-followup/project-hooks-windows.log)

项目钩子整合提交 `4975bdc5a667e706d2637afbf8cbd67826e289e9` 通过 150 项 Rust 测试，包含默认忽略的集成测试。审计 28 棵依赖树，均未启用 Serde、Hyper 或 Reqwest。在工作区扫描 60,534 个目录，`vendor` 目录为零。[Rust 回归](validation/agent-library-followup/project-hooks-rust.log) · [依赖审计](validation/agent-library-followup/project-hooks-dependencies.csv)

Antigravity、Grok Build 和 DeepSeek Harness 钩子整合提交 `18a3d431f284c021d9e99e72bcb65d801634550e` 通过十项定向 Rust 测试、全工作区全目标 Clippy、CLI 构建、Flutter 分析以及 560、1,280 像素布局的 Windows Debug 集成。桌面覆盖 Harness 用户钩子及 Antigravity、Grok 项目钩子的发布和撤销。[Rust 检查](validation/agent-library-followup/native-hooks-rust.log) · [Windows 集成](validation/agent-library-followup/native-hooks-windows.log)

Grok Build 1.0.92 在用户和项目作用域执行 `Stop` 钩子，写入本机标记并收到本机模型响应。通过 `inspect` 核对了项目信任和撤销。DeepSeek Harness 0.2.2-rc.5 安装两个同版本包并加载 Cordis 配置，原生会话日志记录了 `Stop` 退出码 0 与测试标记。撤销后配置逐字节恢复，新进程继续收到本机模型响应，日志中没有钩子调用。Antigravity CLI 1.5.18 的用户和项目隔离配置通过发布和撤销，用户钩子备份逐字节恢复。Antigravity 原生执行仍需 Google OAuth。[Grok 进程](validation/agent-library-followup/grok-hook-process.csv) · [Harness 进程](validation/agent-library-followup/dsh-hook-process.csv) · [Antigravity 配置](validation/agent-library-followup/antigravity-hook-config.csv)

冻结钩子版本 `18a3d431f284c021d9e99e72bcb65d801634550e` 在加载原生客户端夹具后通过全部 153 项 Rust 测试，包含默认忽略的集成测试。审计的 28 棵依赖树均未启用 Serde、Hyper 或 Reqwest。[回归日志](validation/agent-library-followup/native-hooks-regression.log) · [依赖审计](validation/agent-library-followup/native-hooks-dependencies.csv)

### 共享插件库

Windows Debug 集成在 560 像素列表中浏览本地市场并选择插件包，在 1,280 像素网格中审核并复制子代理，随后更新、移除并从 TAR 重新导入插件包。测试通过 Rust 桥接创建 TAR，再经桌面界面导入。[Windows 集成](validation/shared-plugin-library/plugin-bundles-windows.log)

Windows Release 的 Rust 运行器在 `688ea1a` 注册了 Flutter 文件选择器。独立桌面进程通过原生保存对话框导出 TAR，移除库中的快照，再通过原生打开对话框导回。导回后快照哈希一致，附件保留 `00 7F FF` 三个字节。夹具由 `3f09795` 的解压 CLI 建立和核对。[原生文件对话框](validation/shared-plugin-library/plugin-bundle-dialogs.csv)

本机智能 HTTPS Git 夹具使用 `feature/review` 浏览多个候选，拒绝越出市场目录的路径，并重新打开选中的来源。TAR 往返保留二进制附件，导入过程无需请求 Git。复制的子代理发布到 Claude Code 和 Codex 项目文件。移除插件包后仍可同步该子代理的提示词，撤销时保留了外部编辑的 Codex 文件。[Git 夹具](validation/shared-plugin-library/plugin-bundles-https.csv)

公开的 `anthropics/claude-plugins-official` 市场返回 317 个候选，其中 316 个有来源。CLI 选中并保存了含五个文件的 `frontend-design` 1.1.1。隔离的 Pi 1.6.0 进程在本地 Pi 包复制出的命令中展开位置参数和全部参数，并收到本机模型夹具响应。同一定义也生成了 Claude Code 命令文件。[公开市场](validation/shared-plugin-library/plugin-bundles-public.csv) · [Pi 进程](validation/shared-plugin-library/plugin-bundles-pi.csv)

共享插件合并提交 `340d95dc49f086dfea0451658897b4cddba5cea6` 通过 156 项 Rust 回归，包含默认忽略的真实客户端进程测试。该版的 Windows 本机会话用例切换到 Codex 过滤后超时。同一用例在 `688ea1a` 源码上通过。[完整 Rust 回归](validation/cf-windows-340d95d/regression.log) · [较早的会话尝试](validation/cf-windows-340d95d/agent-resources-attempt-1.log) · [冻结版会话结果](validation/cf-windows-688ea1a/agent-resources.log)

Rust 依赖审计覆盖 Windows x64 与 Cargo 全目标的常规、构建及测试依赖。`Cargo.lock` 中还有可选的 `hyper`、`hyper-util`、`serde`、`serde_core` 和 `serde_derive` 记录，当前启用的依赖树均未使用这些包。

Claude Code、Codex 和 Antigravity 的整包项目分配及携带待实现。隔离配置下，Claude Code 2.1.332 用原生 `local` 作用域登记市场并安装包，随后在本机模型回合执行了包内钩子。Codex 0.172.0 在用户配置中安装包，仅在打开了项目启用项的受信任项目内报告已启用。Antigravity 1.5.18 在隔离用户配置中验证、安装、列出并卸载了含一项技能的包。[原生客户端试验](validation/shared-plugin-library/native-project-probes.csv)

### Windows x64 Pi 项目包（`e7b7d65`）

从 `e7b7d659ce2f2440bcca2db8c82910cbfe559526` 离线构建的 Windows x64 ZIP 为 65,087,619 字节，SHA-256 为 `D9B49F8EBC6D34A6D95D83CEC1586B83D6A6A3D38C330C551E75AE6A3C24340B`。核对了 362 项构建输入、671 个包内及解压文件哈希、126 项资产和两个 CLI 各 97 项相同预设。三个入口的 Authenticode 状态均为 `NotSigned`。全目标 Clippy、Flutter 分析、两项组件测试及 28 棵启用依赖树审计均通过。[发布信息](validation/cf-windows-e7b7d65/snapshot.csv) · [构建输入](validation/cf-windows-e7b7d65/build-inputs.sha256) · [包文件清单](validation/cf-windows-e7b7d65/SHA256SUMS) · [签名检查](validation/cf-windows-e7b7d65/authenticode.csv) · [依赖审计](validation/cf-windows-e7b7d65/dependency-audit.csv)

解压后的 CLI 与 Pi 1.6.0 完成项目包分配、同步、撤销和跨项目携带。原生客户端发现夹具的技能和主题、展开提示模板参数，并运行快照中的 `session_start` 扩展。五轮均收到本机模型响应。库保存 2.0.0 后，原项目仍能使用和归档固定的 1.0.0 快照。显式同步才切换到 2.0.0，撤销后原项目设置逐字节恢复。独立 Crabforge 配置在缺少快照时拒绝应用，导入匹配的 TAR 后应用 JSON、运行 2.0.0 并撤销本地分配。[Pi 原生往返](validation/cf-windows-e7b7d65/pi-project-native.csv)

Windows Debug 集成在 560 和 1,280 像素的列表和网格布局下通过。解压版 Release 通过 Windows 原生对话框保存分配 JSON 和固定快照的 TAR，在第二个项目中打开并确认 JSON，随后撤销分配。125 项默认 Rust 测试通过，Cargo 默认跳过的用例有 33 项。工作区扫描 72,686 个目录，发现 0 个 `vendor` 目录和 0 个遍历错误。包中含 311 个 Rust 许可目录，九个 ARB 文件共享 331 个消息键，五个锁定依赖版本均一致。[Debug 集成](validation/cf-windows-e7b7d65/plugin-bundles-windows.log) · [Release 原生对话框](validation/cf-windows-e7b7d65/pi-project-dialogs.csv) · [默认 Rust 测试](validation/cf-windows-e7b7d65/workspace-tests.log) · [源码和许可检查](validation/cf-windows-e7b7d65/source-checks.csv)

冻结源码的 33 项默认跳过用例逐项通过，覆盖 MiMo Code、Oh My Pi 原生线协议、真实本地模型、官方 Ollama 客户端、浏览器夹具，以及 Kubo 和 Magpie 互通。较早的合并客户端回归记录一次 MiMo Code 失败，随后在 Oh My Pi 协议用例中达到整轮超时。MiMo Code 首次隔离重试也超时。用两个解压 CLI 实测 MiMo Code 0.1.17，各有两次请求到达本机上游。`e7b7d65` 的格式化输出未显示夹具响应，JSON 事件模式下两版均返回夹具响应。[逐项回归](validation/cf-windows-e7b7d65/ignored-tests.log) · [初次客户端回归](validation/cf-windows-e7b7d65/client-regression-initial.log) · [MiMo Code 首次重试](validation/cf-windows-e7b7d65/mimocode-isolated-initial.log) · [格式化输出](validation/cf-windows-e7b7d65/mimocode-release.csv) · [JSON 事件](validation/cf-windows-e7b7d65/mimocode-release-json.csv)

SSE 夹具把末事件延后 700 ms，三轮首个完整事件分别在 8、16 和 3 ms 到达，末事件分别在 716、727 和 717 ms 到达。每轮上游收到一次请求，代理收到零次 CONNECT。同一主机上的 IPv6 回环、网卡全局 IPv6 QUIC 直连和直连失败后的 Kubo Circuit Relay v2 回退均通过无效密钥 401 和有效密钥 200 检查。[SSE 样本](validation/cf-windows-e7b7d65/forward-h1.csv) · [IPv6 与回退](validation/cf-windows-e7b7d65/ipv6.csv)

三轮桌面采样的工作集与私有字节中位数为解压版 Crabforge 134.71 / 194.85 MiB，Magpie 主进程 78.11 / 94.96 MiB，Magpie 及七个 WebView2 子进程合计 427.60 / 382.32 MiB。Windows Graphics Capture 在 24 次页面切换中记录了 337 次局部画面变化，点击到首个变化为 19.445–52.986 ms，中位数为 31.861 ms。[桌面内存](validation/cf-windows-e7b7d65/desktop-comparison.csv) · [窗口采样](validation/cf-windows-e7b7d65/window-capture-frames.csv)

空目录负载下，Crabforge 与 Magpie 网关的工作集和私有字节中位数分别为 8.11 / 1.59 MiB 和 57.77 / 91.59 MiB。两者使用同一个本机 Chat 夹具时分别为 9.47 / 2.04 MiB 和 68.30 / 97.66 MiB，Chat 目录分别列出两项和一项。[空目录](validation/cf-windows-e7b7d65/gateway-catalog-memory.csv) · [Chat](validation/cf-windows-e7b7d65/gateway-chat-memory.csv)

### Windows x64 共享插件库（`688ea1a`）

从 `688ea1aa11811602d6e950fbc40291970d3ad6b1` 离线构建的 `crabforge-0.1.0-windows-x64.zip` 为 64,910,131 字节，SHA-256 为 `9D98775C7EB00B4585FF8920034B45F18F2451A915FFB5C007C2687A47349A61`。核对 360 项构建输入、671 项包内及解压文件哈希、126 项资产和两个 CLI 各 97 项相同预设。三个入口的 Authenticode 状态均为 `NotSigned`。[发布信息](validation/cf-windows-688ea1a/snapshot.csv) · [构建输入](validation/cf-windows-688ea1a/build-inputs.sha256) · [文件哈希](validation/cf-windows-688ea1a/SHA256SUMS) · [签名检查](validation/cf-windows-688ea1a/authenticode.csv)

打包源码 `688ea1a` 通过 157 项 Rust 全量回归，0 失败、0 忽略，包含真实客户端、Pi 包资源和同机 DCUtR 升级。全目标 Clippy、Flutter 分析、两项组件测试及 28 棵依赖树审计也已通过。[Rust 回归](validation/cf-windows-688ea1a/regression.log) · [Clippy](validation/cf-windows-688ea1a/clippy.log) · [Flutter 分析](validation/cf-windows-688ea1a/analyze.log) · [组件测试](validation/cf-windows-688ea1a/widget.log) · [依赖审计](validation/cf-windows-688ea1a/dependency-audit.csv)

冻结源码的 14 轮 Windows Debug 全部通过，覆盖控制台和 Oh My Pi 重启、控制台六页布局，以及客户端资源、规则、技能、项目 MCP、环境变量、Hooks、子代理和命令、Claude Code 插件、共享插件库。控制台在 560 与 1,200 像素、明暗主题及列表和网格组合下生成 48 张截图。Hooks 用例运行期间测试窗口最小化，手动恢复后继续并通过。控制台测试记录了一次点击未命中控件，随后用例通过。[Debug 日志](validation/cf-windows-688ea1a/) · [Hooks](validation/cf-windows-688ea1a/agent-hooks.log) · [浅色 560 像素](validation/cf-windows-688ea1a/light-560.jpg) · [深色 560 像素](validation/cf-windows-688ea1a/dark-560.jpg) · [浅色 1,200 像素](validation/cf-windows-688ea1a/light-1200.jpg) · [深色 1,200 像素](validation/cf-windows-688ea1a/dark-1200.jpg)

Flutter Profile 在布局用例中记录 1,343 个帧回调。UI 构建 P90/P99 为 1.421/7.443 ms，帧总跨度 P90/P99 为 2.988/8.264 ms，三帧超过 16.67 ms，最大值为 26.985 ms。解压后的 Release 用 Windows Graphics Capture 对窗口局部取样，四轮各点击六个页面导航项。24 次切换中，点击到首个局部画面变化为 18.911–45.903 ms，中位数为 27.734 ms，共记录 404 张局部画面变化样本。GPU 执行及物理显示时序待验。[Profile 日志](validation/cf-windows-688ea1a/profile.log) · [逐帧数据](validation/cf-windows-688ea1a/profile-frames.json) · [Windows Graphics Capture](validation/cf-windows-688ea1a/window-capture-frames.csv)

工作区复扫 69,280 个目录，发现 0 个 `vendor` 目录和 0 个遍历错误。冻结版的九个 ARB 文件各有 325 个消息键，键集合完全一致。锁文件核对通过，Courierust 为 1.1.3、NextJSON 为 0.1.5、recurse-x 为 0.2.6、RustBinary 为 0.1.10、libp2p 为 0.57.1。[依赖树](validation/cf-windows-688ea1a/dependency-audit.csv) · [构建输入](validation/cf-windows-688ea1a/build-inputs.sha256)

解压后的桌面通过原生保存和打开对话框完成 TAR 往返。解压 CLI 通过本机智能 HTTPS Git 夹具和 Pi 1.6.0 的本机模型回合。再次浏览公开 Claude 插件市场获得 317 个候选并保存 `frontend-design` 1.1.1。[原生文件对话框](validation/cf-windows-688ea1a/plugin-bundle-dialogs.csv) · [HTTPS Git](validation/cf-windows-688ea1a/plugin-bundles-https.csv) · [Pi](validation/cf-windows-688ea1a/plugin-bundles-pi.csv) · [公开市场](validation/cf-windows-688ea1a/plugin-bundles-public.csv)

解压 CLI 在同一 Windows 主机分别通过 IPv6 回环、网卡全局 IPv6 的 QUIC 直连，以及直连失败后的 Kubo Circuit Relay v2 回退。三条路径均在无效密钥时返回 401，在有效密钥时返回 200。独立公网主机之间的 IPv6、UDP 直连、打洞及回退待验。[IPv6 及回退样本](validation/cf-windows-688ea1a/ipv6.csv)

本机 SSE 上游将末事件延后 700 ms。首次三轮的首事件为 12、8、2 ms，末事件为 726、721、704 ms。复测三轮的首事件为 16、3、2 ms，末事件为 722、719、708 ms。每轮上游收到一个请求，设置的 HTTP 代理收到零个 CONNECT 请求。[首次流式样本](validation/cf-windows-688ea1a/forward-h1.csv) · [复测样本](validation/cf-windows-688ea1a/forward-h1-repeat.csv)

三个隔离网关进程样本的工作集与私有字节中位数，空目录负载下 Crabforge 为 8.09 / 1.54 MiB、Magpie 为 53.74 / 86.82 MiB。Chat 使用同一台主机上的模型夹具，Crabforge 为 9.45 / 2.07 MiB、Magpie 为 70.49 / 98.25 MiB。Chat 目录分别列出两项和一项。[空目录内存](validation/cf-windows-688ea1a/gateway-catalog-memory.csv) · [Chat 内存](validation/cf-windows-688ea1a/gateway-chat-memory.csv)

桌面内存取三轮的中位数。每轮将解压版 Crabforge 在独立数据目录内冷启动，窗口与 P2P 身份文件就绪三秒后采样，同时读取持续运行的 Magpie 参考窗口。Crabforge 的工作集与私有字节为 157.86 / 232.45 MiB，Magpie 主进程为 80.86 / 96.96 MiB，计入七个 WebView2 子进程后为 509.06 / 426.44 MiB。[桌面内存样本](validation/cf-windows-688ea1a/desktop-comparison.csv) · [主机与参考程序](validation/cf-windows-688ea1a/host.csv)

### Windows x64 Claude Code 插件（`ad2088c`）

从 `ad2088c9daefc5548683f629f8d53a3a412c88ed` 离线构建的 ZIP 为 64,547,641 字节，SHA-256 为 `D3DD84828CB96B773C6E669FB28488C43940C78FCE4A4877D5331B90FE90ADC1`。核验 353 项构建输入、671 项包内文件、126 项资产及两个 CLI 各 97 项相同预设。三个入口的 Authenticode 状态均为 `NotSigned`。冻结版通过 154 项 Rust 测试、全目标 Clippy、Flutter 分析和两项组件测试。审计的 28 棵依赖树均未启用 Serde、Hyper 或 Reqwest。[构建输入](validation/cf-windows-ad2088c/build-inputs.sha256) · [解压清单](validation/cf-windows-ad2088c/SHA256SUMS) · [回归日志](validation/cf-windows-ad2088c/regression.log) · [依赖审计](validation/cf-windows-ad2088c/dependency-audit.csv)

解压后的 CLI 调用 Claude Code 2.1.332 添加、浏览、刷新和移除本地市场，安装、禁用、更新插件至 1.1.0，再卸载并保留插件数据。隔离项目在用户配置禁用时成功启用插件。撤销逐字节保留外部写入的 `false`，过期修订被拒绝。CLI 完成带固定 ref 的 GitHub 市场声明及启用 ID 的导入导出。Windows 插件界面通过 560 和 1,280 像素的列表、网格、浏览安装、项目分配及设置导入导出测试。[Claude Code 进程](validation/cf-windows-ad2088c/claude-plugin-process.csv) · [Claude 插件桌面测试](validation/cf-windows-ad2088c/claude-plugins.log)

冻结版通过十三轮 Windows Debug 集成，覆盖控制台、Oh My Pi 重启、布局和客户端资源库。Hooks 重跑使用隔离的 Grok 和 DeepSeek Harness 配置。[Windows 集成日志](validation/cf-windows-ad2088c/) · [Hooks 重试](validation/cf-windows-ad2088c/agent-hooks.log)

Flutter Profile 采集 567 帧，UI 构建 P90/P99 为 2.387/14.796 ms，帧总跨度 P90/P99 为 3.958/15.606 ms。四帧总跨度超过 16.67 ms，最长 34.126 ms。GPU 与物理显示计时仍待实测。[Profile 日志](validation/cf-windows-ad2088c/profile.log) · [逐帧数据](validation/cf-windows-ad2088c/profile-frames.json)

三轮解压 Release 启动中，工作集／私有字节中位数为 Crabforge 133.44/195.92 MiB、Magpie 主进程 77.44/93.76 MiB、含七个 WebView2 子进程的 Magpie 441.67/390.72 MiB。上游等待 700 ms 时，本机 SSE 首事件为 24、5、3 ms，末事件为 734、712、704 ms。同机 IPv6 回环、网卡全局 IPv6 和 Kubo 回退均以无效密钥收到 HTTP 401，以有效密钥收到 HTTP 200。[桌面内存](validation/cf-windows-ad2088c/desktop-comparison.csv) · [SSE 延迟](validation/cf-windows-ad2088c/forward-h1.csv) · [IPv6 和回退](validation/cf-windows-ad2088c/ipv6.csv)

### Windows x64 子代理与斜杠命令（`5b905b3`）

从 `5b905b39d790a4eae9625b7995714373f72230ec` 离线构建的 ZIP 为 63,745,300 字节，SHA-256 为 `8C405A7762C8D1D8EFF01E75BDD83FBBF7754FB7622B9412725020A6975E4B7B`。核对 340 项构建输入、671 项解压清单、126 项资产及两个 CLI 各 97 项相同预设。28 组依赖树无启用的 Serde、Hyper 或 Reqwest。Clippy、Flutter 分析及两项组件测试通过。三个入口的 Authenticode 状态均为 `NotSigned`。[构建输入](validation/cf-windows-5b905b3/build-inputs.sha256) · [解压清单](validation/cf-windows-5b905b3/SHA256SUMS) · [依赖审计](validation/cf-windows-5b905b3/dependency-audit.csv)

解压 CLI 向原生及网关 profile 发布 12 个用户文件，向项目发布九个文件。Claude Code 和 OpenCode 2.4.1 将发布的子代理提示词发送给本机模型夹具。Claude Code、Pi 和 OpenCode 会话 API 展开位置参数与全部参数，Gemini CLI 和 Qwen Code 展开全部参数。原址接管保留 UTF-8 BOM 和 CRLF，撤销时保留外部修改的文件。[客户端进程验收](validation/cf-windows-5b905b3/agent-files-process.csv)

143 项 Rust 全量回归和十二轮 Windows Debug 集成全部通过。Flutter 帧回调采集 575 帧，UI 构建 P90/P99 为 2.139/13.132 ms，帧总跨度 P90/P99 为 3.487/14.363 ms。四帧总跨度超过 16.67 ms，最长 25.053 ms。GPU 与物理显示计时仍待实测。[Rust 回归](validation/cf-windows-5b905b3/regression.log) · [Windows 集成](validation/cf-windows-5b905b3/) · [Profile 日志](validation/cf-windows-5b905b3/profile.log) · [逐帧数据](validation/cf-windows-5b905b3/profile-frames.json)

三轮解压 Release 启动的工作集／私有字节中位数分别为 Crabforge 154.76/231.28 MiB、Magpie 主进程 79.55/97.30 MiB、含七个 WebView2 子进程的 Magpie 449.81/426.68 MiB。上游等待 700 ms 时，本机 SSE 首事件为 11、2、2 ms，末事件为 719、705、705 ms。同机 IPv6 回环、网卡全局 IPv6 地址及 Kubo 回退均以无效密钥收到 401，以有效密钥收到 200。[内存](validation/cf-windows-5b905b3/desktop-comparison.csv) · [SSE](validation/cf-windows-5b905b3/forward-h1.csv) · [IPv6](validation/cf-windows-5b905b3/ipv6.csv)

### Windows x64 Hooks（`1ef0c50`）

从 `1ef0c50d7c0c918283d2585b2821e4dd057e21ad` 离线构建的 ZIP 为 63,341,802 字节，SHA-256 为 `1437C6C0FAB40A80F6829B8CC91A950F314B4B6D2A7F790DB6FD4233C5B8DD2C`。核对 334 项构建输入、671 项解压清单、126 项资产，以及两个 CLI 各 97 项相同预设。28 组依赖树无启用的 Serde、Hyper 或 Reqwest。全目标 Clippy、Flutter 分析和两项组件测试通过。三个入口的 Authenticode 状态均为 `NotSigned`。[构建输入](validation/cf-windows-1ef0c50/build-inputs.sha256) · [解压清单](validation/cf-windows-1ef0c50/SHA256SUMS) · [依赖审计](validation/cf-windows-1ef0c50/dependency-audit.csv)

解压 CLI 发布的 `SessionStart` 由 Claude Code 2.1.332 执行，Gemini CLI 0.71.1 在原生及网关 profile 均执行，三个进程写入本机标记并收到了模型夹具响应。Codex 0.172.0 撤销首组后，下一组的原生 Hook 保留相同的键和已信任哈希，随后执行并完成本机模型回合。Cursor 核对了 `version: 1` 和扁平 `sessionStart` 配置，运行时执行仍待实测。解压 CLI 也通过了环境变量的格式和外部修改保护回归。[Hooks 客户端](validation/cf-windows-1ef0c50/hooks-process.csv) · [Codex 信任回归](validation/cf-windows-1ef0c50/hooks-trusted-followup.csv) · [环境变量](validation/cf-windows-1ef0c50/environment.csv)

三轮解压 Release 启动的工作集／私有字节中位数分别为 Crabforge 156.70/229.47 MiB、Magpie 主进程 80.98/96.92 MiB、含七个 WebView2 子进程的 Magpie 453.22/428.58 MiB。三轮本机 SSE 首事件为 25、17、29 ms，末事件为 741、732、732 ms，上游等待 700 ms。同机 IPv6 回环、网卡全局 IPv6 地址及 Kubo 回退均核对了 401/200。[内存](validation/cf-windows-1ef0c50/desktop-comparison.csv) · [SSE](validation/cf-windows-1ef0c50/forward-h1.csv) · [IPv6](validation/cf-windows-1ef0c50/ipv6.csv)

139 项 Rust 全量回归和十一轮 Windows Debug 集成通过，覆盖控制台和 Oh My Pi 重启、宽窄布局及客户端资源。Hooks 用例在两个布局中完成发布、撤销和外部修改保留。Profile 采集到 581 帧，UI 构建 P90/P99 为 1.150/6.632 ms，帧总跨度为 2.900/7.650 ms。计时来自 Flutter 帧回调。[Rust 回归](validation/cf-windows-1ef0c50/regression.log) · [Hooks 桌面用例](validation/cf-windows-1ef0c50/agent-hooks.log) · [Profile 日志](validation/cf-windows-1ef0c50/profile.log) · [逐帧数据](validation/cf-windows-1ef0c50/profile-frames.json)

### Windows x64 环境变量（`8064053`）

从 `806405399a164f9184fa1fa6530b416ab9aa1c48` 离线构建的 ZIP 为 62,982,543 字节，SHA-256 为 `A389FEADE3B16C4E9CFE31DE260896603F9E581D1663635DD2CE184B5D210027`。核对 328 项构建输入、671 项解压清单、126 项资产和双 CLI 各 97 项相同预设。28 组依赖树无启用的 Serde、Hyper 或 Reqwest。全目标 Clippy、Flutter 分析、两项组件测试和 136 项 Rust 全量回归通过。三个入口的 Authenticode 状态均为 `NotSigned`。[解压清单](validation/cf-windows-8064053/SHA256SUMS) · [依赖审计](validation/cf-windows-8064053/dependency-audit.csv) · [Rust 回归](validation/cf-windows-8064053/regression.log)

解压 CLI 通过 Claude 的 JSONC、Codex 内联 TOML 的 BOM、CRLF 和注释保留，以及过期修订、改名、撤销、外部值保护和 Pi 双配置发布。Claude Code 2.1.332 的 `SessionStart`、Codex 0.172.0 的 Shell 回合和 Pi 1.6.0 两套配置的 `session_start` 都读到了发布值。Pi 两套配置都保留了 Shell 已有值。宽窄窗口集成验证了默认遮盖、List/Grid、分配、外部修改和撤销。该用例来自仅修改测试的 `279434b39621da83308e4e60d657f48fd906be39`。[解压 CLI](validation/cf-windows-8064053/environment.csv) · [真实客户端](validation/cf-windows-8064053/environment-process.csv) · [Windows 集成](validation/cf-windows-8064053/agent-environment.log)

Profile 记录 577 帧，UI 构建 P90/P99 为 1.46/7.70 ms，帧总跨度为 2.95/9.78 ms。三轮解压 Release 启动的工作集／私有字节中位数为 Crabforge 155.14/229.99 MiB、Magpie 主进程 82.47/99.33 MiB、含七个 WebView2 子进程的 Magpie 454.73/419.18 MiB。三轮本机 SSE 首事件为 16、30、17 ms，末事件为 724、741、731 ms，上游等待 700 ms。同机 IPv6 回环、网卡全局 IPv6 地址及 Kubo 回退对无效和有效密钥分别返回 401 与 200。[Profile](validation/cf-windows-8064053/profile.json) · [内存采样](validation/cf-windows-8064053/desktop-comparison.csv) · [SSE](validation/cf-windows-8064053/forward-h1.csv) · [IPv6](validation/cf-windows-8064053/ipv6.csv)

### Windows x64 技能来源目录（`1d072e8`）

从 `1d072e8ac47aca7aeaa77798eb514101bdc971de` 离线构建的 ZIP 为 62,757,439 字节，SHA-256 为 `8EC82593C5F2098A2BCD840A9AD8AE5F9FB40AB8838D59636820A6196DCB8AB2`。核对 323 项构建输入、671 项解压清单、126 项资产和两个 CLI 各 97 项相同预设。28 组依赖树无启用的 Serde、Hyper 或 Reqwest。三个入口的 Authenticode 状态均为 `NotSigned`。[构建输入](validation/cf-windows-1d072e8/build-inputs.sha256) · [解压清单](validation/cf-windows-1d072e8/SHA256SUMS) · [依赖审计](validation/cf-windows-1d072e8/dependency-audit.csv)

解压 CLI 通过引用名含 `/` 的本地 Git 目录、元数据过滤、长文档选择、完整附件导入和本地集合持久忽略。直接导入被忽略技能仍然可用。公开 `anthropics/skills` 主分支返回 19 个候选。解压 CLI 也通过了 Git 技能更新及真实 Gemini CLI 的项目 MCP 回合。[来源目录](validation/cf-windows-1d072e8/git-catalog.csv) · [Git 技能](validation/cf-windows-1d072e8/git-skills.csv) · [共享 MCP](validation/cf-windows-1d072e8/project-mcp-library-process.csv)

Windows Debug 在宽窄窗口验证了来源目录的 List/Grid 与候选分配，确认 Codex 原生技能落盘、本地集合忽略保留，以及被忽略技能仍可直接导入 Claude Code。用例来自仅修改测试的 `2dfb3c6f8c358f51c67debdd82b01d6cd6de258b`。冻结提交的 134 项 Rust 全量回归通过。[Windows 集成](validation/cf-windows-1d072e8/agent-skills.log) · [Rust 回归](validation/cf-windows-1d072e8/regression.log)

Profile 记录 579 帧，UI 构建 P90/P99 为 1.66/7.64 ms，帧总跨度为 3.19/10.63 ms。三轮解压 Release 启动的工作集／私有字节中位数为 Crabforge 154.56/227.01 MiB、Magpie 主进程 84.08/100.48 MiB、含七个 WebView2 子进程的 Magpie 512.14/429.18 MiB。同机 IPv6 回环、网卡全局 IPv6 地址及 Kubo 回退通过 401/200 核对。本机 SSE 首事件为 32、13、16 ms，末事件为 744、728、717 ms。[Profile](validation/cf-windows-1d072e8/profile.json) · [内存采样](validation/cf-windows-1d072e8/desktop-comparison.csv) · [SSE](validation/cf-windows-1d072e8/forward-h1.csv) · [IPv6](validation/cf-windows-1d072e8/ipv6.csv)

### Windows x64 Git 技能（`876cd85`）

从 `876cd856c3dfbc2a513f93ac33d410cf94a14883` 离线构建的 ZIP 为 62,703,264 字节，SHA-256 为 `150ABDE2790DB7AC877D328E8505498CC0C54E840CD7B9738B8DD13709EDDF76`。核对 321 项构建输入、671 项解压清单、126 项资产、311 组 Rust 许可及两个 CLI 各 97 项相同预设。28 组依赖树无启用的 Serde、Hyper 或 Reqwest。全目标 Clippy、Flutter 分析和两项组件测试通过。解压桌面和两个 CLI 的 Authenticode 状态均为 `NotSigned`。[构建输入](validation/cf-windows-876cd85/build-inputs.sha256) · [解压清单](validation/cf-windows-876cd85/SHA256SUMS) · [依赖审计](validation/cf-windows-876cd85/dependency-audit.csv)

解压 CLI 从引用名含 `/` 的本地 Git 仓库导入技能，保留发布归档排除的附件，并保留受行尾和 ident 属性影响的 Git blob 原字节。无关提交不触发更新，检查更新不修改已安装文件，重新导入可应用更新，过期修订被拒绝，上游文档删除可检测。接管会拒绝发现后被改动的附件，刷新再接管可将原字节从 Claude 发布到 Codex。本机智能 HTTPS Git 夹具在开启证书校验的情况下通过引用发现、upload-pack 和导入。公开 GitHub 导入与 Gemini CLI 0.71.1 的项目 MCP 工具回合也通过。[本地 Git 和接管](validation/cf-windows-876cd85/git-skills.csv) · [HTTPS 夹具](validation/cf-windows-876cd85/git-https.csv) · [公开 GitHub 归档](validation/cf-windows-876cd85/github-skill-unpacked.csv) · [Gemini 工具回合](validation/cf-windows-876cd85/project-mcp-library-process.csv)

同机 IPv6 回环、网卡全局 IPv6 地址及 Kubo 回退对无效和有效密钥分别返回 401 与 200。上游等待 700 ms 时，三轮本机 SSE 首事件为 5、1、16 ms，末事件为 713、702、717 ms。[IPv6 和回退](validation/cf-windows-876cd85/ipv6.csv) · [SSE 采样](validation/cf-windows-876cd85/forward-h1.csv)

132 项 Rust 全量回归及九轮 Windows Debug 集成通过。集成覆盖控制台及 Oh My Pi 的独立启动和重启、宽窄布局、Agent 资源、规则、技能和项目 MCP。Profile 记录 579 帧，UI 构建 P90/P99 为 1.71/8.77 ms，帧总跨度 P90/P99 为 3.15/9.93 ms。三轮解压 Release 启动的工作集／私有字节中位数为 Crabforge 155.59/228.70 MiB、Magpie 主进程 78.46/93.47 MiB、含七个 WebView2 子进程的 Magpie 515.18/417.75 MiB。[Rust 回归](validation/cf-windows-876cd85/regression.log) · [集成记录](validation/cf-windows-876cd85/) · [Profile 帧数据](validation/cf-windows-876cd85/profile.json) · [内存采样](validation/cf-windows-876cd85/desktop-comparison.csv)

### Windows x64 共享项目 MCP（`b2feb6a`）

从 `b2feb6a6f0a350ee3f64b66b10e4c879a5ae351a` 离线构建的 ZIP 为 62,583,865 字节，SHA-256 为 `68DE35241A79629A88A9530D8AB33313B1F90E0114DB05BC447FBC0366DF9DED`。核对 319 项构建输入、671 项解压清单、126 项资产、311 组 Rust 许可及两个 CLI 各 97 项相同预设。28 组依赖树无启用的 Serde、Hyper 或 Reqwest。全目标 Clippy、Flutter 分析和两项组件测试通过。解压桌面及两个 CLI 的 Authenticode 状态均为 `NotSigned`。[构建输入](validation/cf-windows-b2feb6a/build-inputs.sha256) · [解压清单](validation/cf-windows-b2feb6a/SHA256SUMS) · [依赖审计](validation/cf-windows-b2feb6a/dependency-audit.csv)

解压 CLI 将分配的 `stdio` 服务器发布到 Gemini CLI 0.71.1 的原生项目配置。隔离客户端允许指定的记忆服务器，并向本机模型夹具返回工具结果。CLI 拒绝过期分配，取消分配后保留相邻的上下文设置。解压 CLI 也通过公开 GitHub 技能导入和附件检查。[Gemini 工具回合](validation/cf-windows-b2feb6a/project-mcp-library-process.csv) · [GitHub 技能导入](validation/cf-windows-b2feb6a/github-skill-unpacked.csv)

九轮 Windows 集成测试通过，覆盖控制台和 Oh My Pi 设置的独立启动与重启，以及控制台布局、Agent 资源、规则、技能和共享项目 MCP。项目 MCP 测试在 560 和 1,280 像素宽度下验收共享服务器的分配及移除。[集成记录](validation/cf-windows-b2feb6a/) · [Windows MCP 测试](validation/cf-windows-b2feb6a/project-mcp.log)

Rust 全量串行回归通过 132 项，包括标为忽略的客户端和服务夹具测试。Windows Profile 在控制台布局场景记录 573 帧，UI 构建 P90/P99 为 1.92/9.38 ms，帧总跨度 P90/P99 为 3.42/10.67 ms。[Rust 回归](validation/cf-windows-b2feb6a/regression.log) · [Profile 日志](validation/cf-windows-b2feb6a/profile.log) · [帧数据](validation/cf-windows-b2feb6a/profile.json)

IPv6 回环、同机全局 IPv6 和直连失败后的 Kubo 中继，对无效和有效密钥分别返回 401 与 200。本机 SSE 三轮首事件为 26、30、15 ms，末事件为 740、731、729 ms，上游等待 700 ms。三轮 Release 工作集／私有字节中位数，Crabforge 为 156.16/229.14 MiB，Magpie 主进程为 80.64/97.62 MiB，含七个 WebView2 子进程时为 450.44/421.48 MiB。[IPv6 与回退](validation/cf-windows-b2feb6a/ipv6.csv) · [SSE](validation/cf-windows-b2feb6a/forward-h1.csv) · [内存采样](validation/cf-windows-b2feb6a/desktop-comparison.csv)

### Windows x64 GitHub 技能（`04d3505`）

从 `04d3505b02eb039e71af83cba0ecbcf4f00bf7eb` 离线构建的 ZIP 为 62,309,657 字节，SHA-256 为 `6BCB0EFD7F8089DCC9B4B82E29F3CBBC8462B0BC8AC25C5674ECAAA46616C329`。核对 317 项构建输入、671 项解压清单、126 项资产、311 组 Rust 许可及两个 CLI 各 97 项相同预设。28 组依赖树无启用的 Serde、Hyper 或 Reqwest。全目标 Clippy、Flutter 分析和两项组件测试通过。解压桌面和两个 CLI 的 Authenticode 状态均为 `NotSigned`。[构建输入](validation/cf-windows-04d3505/build-inputs.sha256) · [解压清单](validation/cf-windows-04d3505/SHA256SUMS) · [依赖审计](validation/cf-windows-04d3505/dependency-audit.csv)

解压 CLI 从公开的 `anthropics/skills` 仓库导入 skill creator，将 11,345 字节的许可附件写入隔离的 Claude 配置，按保存的链接重新获取，并移除内容未变的发布副本。Gemini CLI 0.71.1 和 Pi 1.6.0 的原生及网关配置都收到相同附件，两个客户端发往本机模型夹具的首个请求均包含技能描述。取消分配后移除了两处副本并保留源文件。另实测 Gemini 完成原生项目 MCP 工具回合。[GitHub 技能及附件哈希](validation/cf-windows-04d3505/github-skill-unpacked.csv) · [Gemini 和 Pi 配置](validation/cf-windows-04d3505/gateway-skill-profiles.csv) · [项目 MCP](validation/cf-windows-04d3505/project-mcp-process.csv)

IPv6 回环、同机全局 IPv6 和直连失败后的 Kubo 中继，对无效密钥均返回 401，对有效密钥均返回 200。本机 SSE 三轮首事件为 8、1、16 ms，末事件为 713、713、731 ms，上游等待 700 ms。[IPv6 与回退](validation/cf-windows-04d3505/ipv6.csv) · [SSE](validation/cf-windows-04d3505/forward-h1.csv)

### Windows x64 技能库与项目 MCP（`5fb7c7f`）

从 `5fb7c7f119ce2f6cc5def7d2694de1aab056ca7d` 离线构建的 ZIP 为 61,947,901 字节，SHA-256 为 `7CC729D5C6630162720B25182797CD3E7B878FF6D66E07725BCC190F637DC0EA`。核对 316 项构建输入、667 项解压清单、126 项资产、309 组 Rust 许可及两个 CLI 各 97 项相同预设。28 组依赖树无启用的 Serde、Hyper 或 Reqwest。全目标 Clippy、Flutter 分析和两项组件测试通过。解压桌面和两个 CLI 的 Authenticode 状态均为 `NotSigned`。[构建输入](validation/cf-windows-5fb7c7f/build-inputs.sha256) · [解压清单](validation/cf-windows-5fb7c7f/SHA256SUMS) · [依赖审计](validation/cf-windows-5fb7c7f/dependency-audit.csv)

Claude Code 和 Codex 发往本机模型夹具的首个请求均包含已发布的技能描述。解压 CLI 保留二进制附件，拒绝过期分配，并移除内容未变的原生副本。Gemini CLI 0.71.1 从原生项目配置加载 stdio 服务器并向夹具返回工具结果。过期删除被拒绝，移除保留相邻设置。Windows Debug 验收覆盖宽窄窗口、列表和网格、技能发布与项目 MCP 编辑。解压版生产入口及控制台和 Oh My Pi 的独立重启通过。[Claude Code 和 Codex](validation/cf-windows-5fb7c7f/native-skill-process.csv) · [Gemini](validation/cf-windows-5fb7c7f/project-mcp-process.csv) · [技能桌面](validation/cf-windows-5fb7c7f/agent-skills.log) · [MCP 桌面](validation/cf-windows-5fb7c7f/project-mcp.log) · [控制台重启](validation/cf-windows-5fb7c7f/console-1.log) · [Oh My Pi 重启](validation/cf-windows-5fb7c7f/omp_settings-1.log)

构建与验收的合并任务在 Rust 回归阶段达到时限。对同一冻结源码单独重跑，127 项串行测试全部通过。[中断日志](validation/cf-windows-5fb7c7f/regression-interrupted.log) · [完整回归](validation/cf-windows-5fb7c7f/regression.log)

Profile 记录 584 帧，UI 构建 P90/P99 为 1.28/6.78 ms，总跨度 P90/P99 为 2.88/8.07 ms。三轮 Release 工作集／私有字节中位数，Crabforge 为 156.52/229.65 MiB，Magpie 主进程为 80.79/96.56 MiB，含七个 WebView2 子进程时为 462.48/427.54 MiB。本机 SSE 三轮首事件为 7、16、2 ms，末事件为 710、725、710 ms，上游等待 700 ms。IPv6 回环、同机全局 IPv6 和 Kubo 中继均通过无效和有效密钥校验。[Profile](validation/cf-windows-5fb7c7f/profile.json) · [内存采样](validation/cf-windows-5fb7c7f/desktop-comparison.csv) · [SSE](validation/cf-windows-5fb7c7f/forward-h1.csv) · [IPv6 与回退](validation/cf-windows-5fb7c7f/ipv6.csv)

### Windows x64 规则库（`8028b7a`）

从 `8028b7a25dd7371c25e6d2ff693184df08acbded` 离线构建的 ZIP 为 61,517,175 字节，SHA-256 为 `FF8CF197742AB5E11C8493F03F4074B4A6A3F3A220B94BA3F8BDEBF298AC0B35`。核对 306 项构建输入、667 项解压清单、126 项资产、309 组 Rust 许可及两个 CLI 各 97 项相同预设。123 项串行 Rust 回归、全目标 Clippy、Flutter 分析及两项组件测试通过。28 组依赖树无启用的 Serde、Hyper 或 Reqwest。解压桌面与 CLI 的 Authenticode 状态均为 `NotSigned`。[构建输入](validation/cf-windows-8028b7a/build-inputs.sha256) · [解压清单](validation/cf-windows-8028b7a/SHA256SUMS) · [Rust 回归](validation/cf-windows-8028b7a/regression.log) · [依赖审计](validation/cf-windows-8028b7a/dependency-audit.csv)

560 和 1280 像素的 Windows Debug 集成覆盖规则创建、路径范围、列表／网格、编辑、应用、外部修改保护和删除。解压 CLI 发布的规则分别出现在真实 Claude Code 和 Cline CLI 3.0.76 的首个本机模型请求。Cline 拒绝路径规则并移除先前由 Crabforge 写入的全局副本。Kiro CLI 3.6.2 的 MSI 签名有效，发布者为 Amazon Web Services, Inc.。解压 CLI 写入了 `inclusion: fileMatch` 和 `fileMatchPattern`。Kiro 在隔离环境中停在登录检查，原生读取和请求仍待实测。[规则桌面](validation/cf-windows-8028b7a/agent-rules.log) · [Claude Code](validation/cf-windows-8028b7a/native-rule-process.csv) · [Cline](validation/cf-windows-8028b7a/cline-native-rules.csv) · [Kiro 安装与规则](validation/cf-windows-8028b7a/kiro-rules-process.csv) · [Kiro 登录检查](validation/cf-windows-8028b7a/kiro-auth.csv)

解压版生产入口、控制台和 Oh My Pi 独立重启、宽窄布局均通过。会话桌面测试首轮未找到 Codex 夹具标题，用同一冻结源码和新隔离目录重跑通过。Profile 记录 584 帧，UI 构建 P90/P99 为 1.45/8.40 ms，总跨度 P90/P99 为 2.90/9.50 ms。解压 Release 桌面三轮工作集／私有字节中位数为 156.89/229.67 MiB，Magpie 主进程为 81.82/98.70 MiB，包含七个 WebView2 子进程时为 458.14/421.79 MiB。[首轮会话](validation/cf-windows-8028b7a/agent-resources.log) · [会话重跑](validation/cf-windows-8028b7a/agent-resources-retry.log) · [宽窄布局](validation/cf-windows-8028b7a/console-layout.log) · [Profile](validation/cf-windows-8028b7a/profile.json) · [内存采样](validation/cf-windows-8028b7a/desktop-comparison.csv)

解压 CLI 通过八项记忆检查、原生会话提炼以及 Gemini CLI 的本机 MCP 工具回合。IPv6 回环、同机全局 IPv6 和直连失败后的 Kubo 中继均通过 401、200 和目录校验。回环 SSE 三轮首事件为 12、54、32 ms，末事件为 716、769、744 ms，上游等待 700 ms。[记忆](validation/cf-windows-8028b7a/memory-process.csv) · [提炼](validation/cf-windows-8028b7a/memory-distill-process.csv) · [Gemini MCP](validation/cf-windows-8028b7a/mcp-client.csv) · [IPv6 与回退](validation/cf-windows-8028b7a/ipv6.csv) · [SSE](validation/cf-windows-8028b7a/forward-h1.csv)

### Windows x64 手动会话提炼（`d53252f`）

从 `d53252f1718f72454237782259dab20a4b0b6cbc` 离线构建的 ZIP 为 61,275,493 字节，SHA-256 为 `3741355AD8405F1C26D52D91B15589C1ACB7B6E8BEDC79914926BD39CAFF2E95`。核对 302 项构建输入、667 个清单文件、126 项资产、309 个 Rust 依赖许可及双 CLI 各 97 项相同预设。120 项串行 Rust 回归、全目标 Clippy、Flutter 分析和两项组件测试通过。28 组依赖树无启用的 Serde、Hyper 或 Reqwest。解压桌面和 CLI 均为 `NotSigned`。[构建输入](validation/cf-windows-d53252f/build-inputs.sha256) · [解压清单](validation/cf-windows-d53252f/SHA256SUMS) · [Rust 回归](validation/cf-windows-d53252f/regression.log) · [依赖审计](validation/cf-windows-d53252f/dependency-audit.csv)

真实 Claude Code 和 Codex 各运行一次本机模型夹具对话。解压 CLI 经配置的 `group/memory-fixture` 路由发送用户预览，拒绝过期预览，丢弃伪造引文并合并重复候选。审核之前候选未写入摘要，激活后纳入摘要。Codex 次级配置的压缩会话通过预览修订和配置限域检查。560 像素 Windows Debug 集成覆盖路由选择、发送、打开候选、查看引文及激活。另用真实 llama.cpp 和 SmolLM2 135M Q4_K_M 推理，模型响应的格式未通过数组校验，保存了零条候选。[提炼进程](validation/cf-windows-d53252f/memory-distill-process.csv) · [压缩会话限域](validation/cf-windows-d53252f/distill-profiles.csv) · [桌面审核](validation/cf-windows-d53252f/agent-resources.log) · [本机模型](validation/cf-windows-d53252f/distill-local-model.csv)

解压 CLI 在 IPv6 回环、同机全局 IPv6 和直连失败后的 Kubo 回退均通过 401、200 和目录校验。回环 SSE 三轮首事件为 7、2、2 ms，末事件为 715、710、707 ms，上游等待 700 ms。公网跨机与不同 NAT 的样本待采集。[IPv6 与中继](validation/cf-windows-d53252f/ipv6.csv) · [SSE](validation/cf-windows-d53252f/forward-h1.csv)

| 三轮中位数，工作集 / 私有字节（MiB） | Crabforge | Magpie |
| --- | ---: | ---: |
| 空目录网关 | 7.70 / 1.48 | 58.12 / 92.10 |
| Chat 网关 | 9.01 / 1.88 | 68.66 / 97.50 |
| 桌面主进程 | 155.59 / 231.46 | 76.60 / 92.12 |
| 桌面进程树 | 155.59 / 231.46 | 447.73 / 407.48 |

解压 Release 的桌面样本三轮采用独立的 Crabforge 配置，Magpie 复用一个窗口及七个 WebView2 子进程。两种 Chat 网关共用一个本机上游，分别公开两个和一个模型 ID。[空目录](validation/cf-windows-d53252f/catalog.csv) · [Chat](validation/cf-windows-d53252f/chat.csv) · [桌面](validation/cf-windows-d53252f/desktop-comparison.csv)

Windows Graphics Capture 在 24 次 Release 窗口切换中记录 272 次像素区域变化，点击至首次变化的中位数为 60.60 ms，变化间隔中位数为 16.67 ms、P90 为 33.31 ms、最大 33.39 ms。GPU 执行和物理屏幕输出待单独测量。[窗口采集](validation/cf-windows-d53252f/window-capture-frames.csv)

冻结 Windows Debug 通过控制台、Oh My Pi 设置及各自的独立重启、宽窄布局和会话提炼审核。Flutter Profile 记录 585 帧，UI 构建 P90/P99 为 1.36/5.64 ms，总跨度 P90/P99 为 2.90/7.44 ms，栅格时钟为 0–1 µs。[控制台](validation/cf-windows-d53252f/console-0.log) · [控制台重启](validation/cf-windows-d53252f/console-1.log) · [Oh My Pi](validation/cf-windows-d53252f/omp_settings-0.log) · [Oh My Pi 重启](validation/cf-windows-d53252f/omp_settings-1.log) · [布局](validation/cf-windows-d53252f/console-layout.log) · [Profile 日志](validation/cf-windows-d53252f/profile.log) · [逐帧数据](validation/cf-windows-d53252f/profile.json)

### Windows x64 多配置会话与压缩归档（`3f50294`）

从 `3f50294aabb76b7219c2acc261c7a79e763b3eb9` 离线构建的 ZIP 为 61,099,609 字节，SHA-256 为 `D19CBD38B3DDDD57C62773ED123DCC907FF81838FBEF6617D2CDB50B0F6038C2`。核对 301 项构建输入、667 个清单文件、126 项资产、309 个 Rust 依赖许可及两个 CLI 的 97 项相同预设。120 项全量 Rust 回归、全目标 Clippy、Flutter 分析和两项组件测试通过。28 组依赖树无启用的 Serde、Hyper 或 Reqwest。解压的桌面与 CLI 均为 `NotSigned`。[构建输入](validation/cf-windows-3f50294/build-inputs.sha256) · [解压清单](validation/cf-windows-3f50294/SHA256SUMS) · [Rust 回归](validation/cf-windows-3f50294/regression.log) · [依赖审计](validation/cf-windows-3f50294/dependency-audit.csv)

解压 CLI 从真实 Claude Code 和 Codex 的本机模型对话中读取原生会话。在嵌套的次级 Codex 配置放入同一会话的 `.jsonl.zst` 后，CLI 独立列出活动记录和归档，并报告一个截断文件。用户摘要的范围、预览修订、BOM、CRLF 和恢复通过。八项记忆进程检查及 Gemini CLI 0.71.1 的项目摘要与 stdio MCP 工具回合也通过。[会话](validation/cf-windows-3f50294/sessions-process.csv) · [多配置和归档](validation/cf-windows-3f50294/session-profiles.csv) · [用户摘要](validation/cf-windows-3f50294/agent-memory-process.csv) · [记忆](validation/cf-windows-3f50294/memory-process.csv) · [项目摘要](validation/cf-windows-3f50294/digest-publication.csv) · [Gemini MCP](validation/cf-windows-3f50294/mcp-client.csv)

| 三轮中位数，工作集 / 私有字节（MiB） | Crabforge | Magpie |
| --- | ---: | ---: |
| 空目录网关 | 7.65 / 1.51 | 57.92 / 91.27 |
| Chat 网关 | 9.00 / 1.91 | 68.33 / 97.11 |
| 桌面主进程 | 154.82 / 230.07 | 80.64 / 96.25 |
| 桌面进程树 | 154.82 / 230.07 | 451.56 / 421.29 |

桌面样本来自解压后的 Release。Crabforge 每轮使用独立配置，Magpie 复用同一窗口和七个 WebView2 子进程。两个 Chat 网关共用本机夹具，分别公开两个和一个模型 ID。[空目录](validation/cf-windows-3f50294/catalog.csv) · [Chat](validation/cf-windows-3f50294/chat.csv) · [桌面](validation/cf-windows-3f50294/desktop-comparison.csv)

回环 SSE 三轮首事件为 7、2、3 ms，末事件为 721、710、711 ms，上游等待 700 ms。IPv6 回环、同机网卡全局 IPv6 和直连失败后的 Kubo 中继均返回错误密钥 401、有效密钥 200 及预期目录。独立公网主机间及不同 NAT 的样本仍待采集。[SSE](validation/cf-windows-3f50294/forward-h1.csv) · [IPv6 与中继](validation/cf-windows-3f50294/ipv6.csv)

Windows Graphics Capture 在 24 次 Release 窗口切换期间记录 308 次像素区域变化。点击至首次变化的中位数为 43.23 ms，变化间隔中位数为 16.67 ms、P90 为 16.69 ms、最大为 33.39 ms。GPU 执行和物理屏幕输出待单独测量。[窗口采集](validation/cf-windows-3f50294/window-capture-frames.csv)

冻结 Windows Debug 控制台、Oh My Pi 配置及独立重启、宽窄窗口和会话浏览通过。Flutter Profile 共 583 帧，UI 构建 P90/P99 为 1.60/6.98 ms，总跨度 P90/P99 为 3.21/7.97 ms，栅格时钟为 0–1 µs。[控制台](validation/cf-windows-3f50294/console-0.log) · [控制台重启](validation/cf-windows-3f50294/console-1.log) · [Oh My Pi](validation/cf-windows-3f50294/omp_settings-0.log) · [Oh My Pi 重启](validation/cf-windows-3f50294/omp_settings-1.log) · [宽窄窗口](validation/cf-windows-3f50294/console-layout.log) · [会话集成](validation/cf-windows-3f50294/agent-resources.log) · [Profile 日志](validation/cf-windows-3f50294/profile.log) · [逐帧数据](validation/cf-windows-3f50294/profile.json)

### Windows x64 会话浏览与用户摘要（`45a935b`）

从 `45a935bf01d013027cbd83e821a078395a0ac0fb` 离线构建的 ZIP 为 60,925,796 字节，SHA-256 为 `BBC5029C89432867A163C5996C075DF7897A3C20D32755F3249CFCC204138B59`。解压校验覆盖 666 个文件、126 项资产及两个 CLI 的 97 项相同预设。[构建输入](validation/cf-windows-45a935b/build-inputs.sha256) · [解压清单](validation/cf-windows-45a935b/SHA256SUMS)

解压 CLI 通过原生会话发现与用户摘要验收。Claude Code 和 Codex 各自向本机模型夹具发出的首条请求包含全局及该 Agent 的摘要。过期预览和修订被拒绝，恢复后的字节保留原始 BOM 与 CRLF。冻结源码的 560 像素 Windows Debug 集成覆盖桌面会话和用户摘要发布及恢复。[会话](validation/cf-windows-45a935b/sessions-process.csv) · [用户摘要](validation/cf-windows-45a935b/agent-memory-process.csv) · [Windows 集成](validation/cf-windows-45a935b/agent-resources.log)

### Windows x64 项目摘要与本地会话（`85fd8fa`）

从 `85fd8fa556eb2448ed702ebbb59b0b38a4e426f8` 离线构建的 ZIP 为 60,726,090 字节，SHA-256 为 `42A5B7ED546586099C3708B4F184F8D13D96A8B59E85651D2C911B5FB20EEE45`。核对了 299 项构建输入、解压后的 666 个文件、126 项资产、308 个 Rust 依赖许可，以及两个 CLI 各自的 97 项相同预设。通过 118 项 Rust 串行回归、全目标 Clippy、Flutter 分析及两项组件测试。28 组 Windows x64 和 Cargo 全目标依赖树均无启用的 Serde、Hyper 或 Reqwest。首次 Release 构建的 Flutter assemble 停滞，结束对应子进程后，以相同源码重跑通过。[构建输入](validation/cf-windows-85fd8fa/build-inputs.sha256) · [解压清单](validation/cf-windows-85fd8fa/SHA256SUMS) · [Rust 回归](validation/cf-windows-85fd8fa/regression.log) · [依赖审计](validation/cf-windows-85fd8fa/dependency-audit.csv)

解压 CLI 通过八项记忆进程检查，验证了 Git 项目键、发布摘要与 CRLF、过期修订、归档后重发及两次恢复。Gemini CLI 0.71.1 的首条本机模型请求携带已发布摘要，后续请求携带 stdio MCP `memory_search` 的 `functionResponse`。真实 Claude Code 和 Codex 各发起一次本机模型对话，解压 CLI 从生成的原生 JSONL 读到会话 ID、首条提示、工作目录并通过 Agent 筛选。[记忆](validation/cf-windows-85fd8fa/memory-process.csv) · [摘要](validation/cf-windows-85fd8fa/digest-publication.csv) · [Gemini MCP](validation/cf-windows-85fd8fa/mcp-client.csv) · [原生会话](validation/cf-windows-85fd8fa/sessions-process.csv)

| 三轮中位数，工作集 / 私有字节（MiB） | Crabforge | Magpie |
| --- | ---: | ---: |
| 空目录网关 | 7.62 / 1.50 | 58.60 / 92.81 |
| Chat 网关 | 8.92 / 1.90 | 71.59 / 100.41 |
| 桌面主进程 | 151.39 / 225.62 | 78.84 / 95.56 |
| 桌面进程树 | 151.39 / 225.62 | 498.73 / 421.03 |

Crabforge 桌面每轮从解压的 Release 程序和独立配置启动。Magpie 复用同一可见窗口及七个 WebView2 子进程。两种 Chat 网关使用同一本机上游，分别公开两个和一个模型 ID。[空目录](validation/cf-windows-85fd8fa/catalog.csv) · [Chat](validation/cf-windows-85fd8fa/chat.csv) · [桌面](validation/cf-windows-85fd8fa/desktop-comparison.csv)

回环 SSE 首个完整事件分别为 5、29、15 ms，末帧分别为 713、740、724 ms，上游固定等待 700 ms。同机 IPv6 回环、网卡全局 IPv6 及直连失败后的 Kubo 回退均验证了错误密钥 401、有效密钥 200 和预期目录。独立公网主机以及不同 NAT 的连通性待实测。[SSE](validation/cf-windows-85fd8fa/forward-h1.csv) · [IPv6](validation/cf-windows-85fd8fa/ipv6.csv)

Windows Graphics Capture 从解压的 Release 桌面采得 24 次切换和 297 次像素区域变化。点击到首次变化中位数为 41.88 ms，同次切换变化间隔的中位数为 16.67 ms、P90 为 16.68 ms、最大值为 33.34 ms。GPU 逐帧执行及物理显示输出仍待实测。[窗口采集](validation/cf-windows-85fd8fa/window-capture-frames.csv)

冻结源码的 Windows Debug 测试通过了配置、网关与路由记录、独立重启、Oh My Pi 配置与重启及宽窄布局。Profile 测得 580 帧，UI 构建 P90 为 1.50 ms、P99 为 6.68 ms，总跨度 P90 为 2.96 ms、P99 为 7.58 ms，栅格时钟为 0–1 µs。[控制台](validation/cf-windows-85fd8fa/console-0.log) · [重启](validation/cf-windows-85fd8fa/console-1.log) · [Oh My Pi](validation/cf-windows-85fd8fa/omp_settings-0.log) · [Oh My Pi 重启](validation/cf-windows-85fd8fa/omp_settings-1.log) · [宽窄布局](validation/cf-windows-85fd8fa/console-layout.log) · [Profile 原始帧](validation/cf-windows-85fd8fa/profile.json)

### Windows x64 记忆管理（`c8eb96e`）

从 `c8eb96e79048add9d1f94680cc5fd44c0d9f372b` 离线构建的 ZIP 为 60,611,407 字节，SHA-256 为 `4E1CD367471290B5617351A3C1A41DDFF7FDB22773A0484C3C749679C4899AA1`。复核了 296 项构建输入、解压后的 666 个文件、126 项资产和两个 CLI 各自的 97 项相同预设。包内附 308 个 Rust 依赖的许可资料。全量 Rust 串行回归 115 项通过，包含通常忽略的测试。全目标 Clippy、Flutter 分析和两项组件测试通过。Windows x64 和 Cargo 全目标的 28 组启用依赖树均无 Serde、Hyper 或 Reqwest。经指定代理推送后核对了远端 `main` 的完整提交哈希。[构建输入](validation/cf-windows-c8eb96e/build-inputs.sha256) · [解压清单](validation/cf-windows-c8eb96e/SHA256SUMS) · [Rust 测试](validation/cf-windows-c8eb96e/regression.log) · [依赖审计](validation/cf-windows-c8eb96e/dependency-audit.csv)

解压版 CLI 通过八项进程检查，覆盖 Markdown 创建和搜索、拒绝过期修订，以及 MCP 初始化、工具、去重、作用域和归档。Gemini CLI 0.71.1 经 stdio 调用 `memory_search`，随后将返回的记忆作为 `functionResponse` 发送给本机模型夹具。解压版也复测了自定义项目指令。Qoder 和 Qoder CN 1.1.99 原生客户端以隔离配置启动，首条模型请求前均提示需要 `/login`。[进程结果](validation/cf-windows-c8eb96e/memory-process.csv) · [Gemini MCP](validation/cf-windows-c8eb96e/mcp-client.csv) · [自定义指令](validation/cf-windows-c8eb96e/custom-context.csv)

Windows Debug 控制台和 Oh My Pi 编辑器使用解压版 Release CLI，分别通过首次启动及独立进程重启。宽窄布局测试通过，覆盖记忆编辑及列表和网格切换。[控制台](validation/cf-windows-c8eb96e/console-0.log) · [控制台重启](validation/cf-windows-c8eb96e/console-1.log) · [Oh My Pi](validation/cf-windows-c8eb96e/omp_settings-0.log) · [Oh My Pi 重启](validation/cf-windows-c8eb96e/omp_settings-1.log) · [布局测试](validation/cf-windows-c8eb96e/console-layout.log)

| 三轮中位数，工作集 / 私有字节（MiB） | Crabforge | Magpie |
| --- | ---: | ---: |
| 空目录网关 | 7.60 / 1.41 | 60.75 / 93.76 |
| Chat 网关 | 8.88 / 1.84 | 69.30 / 97.93 |
| 桌面主进程 | 155.12 / 229.17 | 82.91 / 98.97 |
| 桌面进程树 | 155.12 / 229.17 | 455.67 / 431.39 |

Crabforge 每轮以独立数据目录启动解压的 Release 桌面。Magpie 三轮复用同一可见窗口及七个 WebView2 子进程。Chat 网关共用本机夹具，Crabforge 公开两个模型 ID，Magpie 公开一个。回环 SSE 首个完整事件分别耗时 15、19、2 ms，上游固定延迟 700 ms，末帧为 724、728、704 ms。解压 CLI 在 IPv6 回环、同机全局 IPv6 及 Kubo 中继回退路径均取得错误密钥 401、有效密钥 200 和预期目录。公网跨机样本仍待测。[空目录](validation/cf-windows-c8eb96e/catalog.csv) · [Chat](validation/cf-windows-c8eb96e/chat.csv) · [桌面](validation/cf-windows-c8eb96e/desktop-comparison.csv) · [SSE](validation/cf-windows-c8eb96e/forward-h1.csv) · [IPv6](validation/cf-windows-c8eb96e/ipv6.csv)

### Windows x64 自定义项目指令（`9aeb7e5`）

本地 ZIP 从 `9aeb7e58423929f0eabeac2b1be4f49fde085ccf` 离线构建，60,230,597 字节，SHA-256 为 `AB20EB4DEB65646DBBD174F8631B4FA9DFBDE4D266095DA7E5B59163F15318C9`。292 项构建输入前后一致，解压后逐一复核 666 个文件及 126 项应用资产，两个 CLI 各输出 97 项相同预设。包内附 308 个 Rust 依赖的许可资料。全量 Rust 串行回归 113 项通过、0 项失败、0 项跳过。全目标 Clippy、Flutter 分析和两项组件测试通过。Windows x64 和 Cargo 全目标的 28 组依赖树均无启用的 Serde、Hyper 或 Reqwest。[构建输入](validation/cf-windows-9aeb7e5/build-inputs.sha256) · [解压文件哈希](validation/cf-windows-9aeb7e5/SHA256SUMS) · [Rust 测试](validation/cf-windows-9aeb7e5/regression.log) · [依赖审计](validation/cf-windows-9aeb7e5/dependency-audit.csv)

| 三轮中位数，工作集 / 私有字节（MiB） | Crabforge | Magpie |
| --- | ---: | ---: |
| 空目录网关 | 7.53 / 1.42 | 57.33 / 91.20 |
| Chat 网关 | 8.85 / 1.86 | 68.91 / 97.56 |
| 桌面主进程 | 154.39 / 227.91 | 81.20 / 96.48 |
| 桌面进程树 | 154.39 / 227.91 | 453.75 / 429.78 |

Crabforge 桌面从解压目录以独立配置启动三次。Magpie 使用独立测试配置，三轮复用同一个可见窗口和七个 WebView2 子进程。Chat 网关共用本机上游夹具，Crabforge 公开两个模型 ID，Magpie 公开一个。[空目录](validation/cf-windows-9aeb7e5/catalog.csv) · [Chat](validation/cf-windows-9aeb7e5/chat.csv) · [桌面](validation/cf-windows-9aeb7e5/desktop-comparison.csv)

解压 CLI 在 IPv6 回环、同机网卡全局 IPv6 直连、直连失败后经 Kubo 0.43.1 回退三条路径上均取得错误密钥 401、正确密钥 200，并核对了目录正文。本机 SSE 夹具将末帧延后 700 ms，三轮首个完整事件为 6、2、2 ms，末帧为 719、710、713 ms。独立公网主机间的 IPv6 和不同 NAT 下的 UDP 连通、打洞与回退仍待验收。[IPv6](validation/cf-windows-9aeb7e5/ipv6.csv) · [SSE](validation/cf-windows-9aeb7e5/forward-h1.csv)

解压 CLI 针对 Gemini CLI、Qwen Code、Qoder 和 Qoder CN 均通过自定义 `context.fileName` 的 BOM 导入、CRLF 保留、过期修订拒绝及恢复。Gemini CLI 0.71.1 与 Qwen Code 0.28.4 的本机模型请求包含自定义文件原文。Gemini 同时提交了默认 `GEMINI.md`。Qoder 和 Qoder CN 的原生客户端尚待验证。[逐客户端结果](validation/cf-windows-9aeb7e5/custom-context.csv)

Windows 调试控制台与 Oh My Pi 编辑器使用包内 Release CLI，分别通过首次使用及独立进程重启。控制台首轮的隧道按钮在测试视口外。固定测试视口并精确定位按钮后，同一冻结生产源码通过。[初次运行](validation/cf-windows-9aeb7e5/console-0.log) · [控制台](validation/cf-windows-9aeb7e5/console-retry-0.log) · [控制台重启](validation/cf-windows-9aeb7e5/console-retry-1.log) · [Oh My Pi](validation/cf-windows-9aeb7e5/omp_settings-retry-0.log) · [Oh My Pi 重启](validation/cf-windows-9aeb7e5/omp_settings-retry-1.log)

冻结版通过 Windows 宽窄布局测试。Windows Graphics Capture 在解压 Release 的宽窗中采集 24 次页面切换，指定像素区域共出现 288 次变化。从投递点击到首次采集到画面变化的中位数为 45.59 ms。同一次切换内，相邻变化之间的中位数为 16.67 ms，P90 为 16.68 ms，最大 33.35 ms。该主机上的 PresentMon ETW 和 DXGI 输出复制未能采样，GPU 逐帧执行与物理显示输出仍待测。[布局测试](validation/cf-windows-9aeb7e5/console-layout.log) · [窗口采集时间](validation/cf-windows-9aeb7e5/window-capture-frames.csv)

### Windows x64 控制台与自有 P2P 地址编解码器（`a4a9e40`）

从 `a4a9e405a19b01e86957187ef3540644336f4aee` 归档构建。构建前后 292 项输入一致，解压后复核清单中 666 个文件和 126 项应用资产。两个 CLI 各列出 97 项相同的供应商预设。并行构建污染了桌面 Release 的 Flutter 中间产物，解压后启动的是集成测试入口。该 ZIP 的验收结论限于独立构建的 CLI 和网关。[构建输入](validation/cf-windows-a4a9e40/build-inputs.sha256) · [解压文件清单](validation/cf-windows-a4a9e40/SHA256SUMS)

`libp2p` 保持 `0.57.1`。Crabforge 自有 Multiaddr 包通过规范字节向量、Peer ID、组件替换和无效输入测试。锁定的 Courierust、NextJSON、Recurse-X 和 RustBinary 分别为 `1.1.3`、`0.1.5`、`0.2.6` 和 `0.1.10`。七个工作区包在 Windows x64 和 Cargo `--target all` 下共检查 28 组正常、构建及测试依赖树，均无启用的 Serde、Hyper 或 Reqwest 包。[地址包](../crates/peer-address/Cargo.toml) · [依赖审计](validation/cf-windows-a4a9e40/dependency-audit.csv)

冻结源码的 Rust 串行回归通过 112 项，包含通常忽略的客户端及本地模型测试。全目标 Clippy、Flutter 静态分析和两项组件测试通过。网关三轮采样的工作集及私有字节中位数分别为空目录 7.53 / 1.42 MiB、Chat 8.82 / 1.86 MiB。Magpie 同机采样为 60.88 / 94.78、68.57 / 97.10 MiB。[空目录](validation/cf-windows-a4a9e40/catalog.csv) · [Chat](validation/cf-windows-a4a9e40/chat.csv)

解压版 CLI 在同一台 Windows 主机通过 IPv6 回环、网卡全局 IPv6 直连，以及直连失败后经 Kubo 0.43.1 的中继回退。每条路径均校验错误密钥返回 401、正确密钥返回 200 和目录正文。回环 SSE 测试的首个完整事件分别耗时 25、4、11 ms，末帧为 732、712、725 ms。公网跨机与不同 NAT 的测试仍缺第二台独立主机。[IPv6](validation/cf-windows-a4a9e40/ipv6.csv) · [SSE](validation/cf-windows-a4a9e40/forward-h1.csv)

Windows 调试版截图覆盖六页、560 和 1200 像素宽度、明暗主题与列表和网格，共 48 个场景。解压版双 CLI 通过含 BOM 的 Markdown 导入、保留文件 CRLF、过期修订拒绝及共享 `AGENTS.md` 的跨 Agent 恢复。[浅色窄窗](validation/cf-windows-a4a9e40/light-560.jpg) · [浅色宽窗](validation/cf-windows-a4a9e40/light-1200.jpg) · [深色窄窗](validation/cf-windows-a4a9e40/dark-560.jpg) · [深色宽窗](validation/cf-windows-a4a9e40/dark-1200.jpg)

Windows Profile 版在 1200 × 800 窗口预热六页后，连续执行四轮导航，记录 309 帧。Flutter 报告的 UI 构建耗时平均 2.02 ms，P90 为 2.99 ms，P99 为 13.33 ms，最大值为 16.15 ms。栅格耗时大部分为零，无法据此评估 GPU 渲染和实际呈现帧率。[Profile 原始数据](validation/cf-windows-a4a9e40/profile.json)

### Windows x64 网关和依赖验收（`a4752de`）

冻结提交 `a4752de6991f8cdef947ecdde6d5c06c1e5b7d51` 的 Windows ZIP 共 61,159,262 字节，SHA-256 为 `C8155FAC6A7002F334AFDFF1F9E6EA103876D2AE5176F545DF5B475B9B8F5A3D`。构建前后 291 项输入一致。从 ZIP 解压后核对了清单中的 143 个文件、126 项资产和两个 CLI 各自的 97 项预设。Rust 串行回归包含通常忽略的测试，106 项通过、0 失败、0 跳过。Clippy、Flutter 分析与两项组件测试通过。使用解压版 CLI，Windows 调试桌面的控制台和 Oh My Pi 编辑器均通过首次使用与独立重启测试。[验收项目](validation/cf-windows-a4752de/checks.csv) · [构建输入](validation/cf-windows-a4752de/build-inputs.sha256) · [解压文件哈希](validation/cf-windows-a4752de/SHA256SUMS)

6 个工作区包分别检查 Windows x64 和 Cargo `--target all` 的 `normal,build`、`normal,build,dev`，24 组已启用依赖均无 Serde 系列、Hyper 或 Reqwest。`Cargo.lock` 仍记录可选包，例如 `igd-next` 关联的 Hyper，以及 `url` 和 `time` 关联的 Serde 包。Bedrock IAM 签名在本地匹配 AWS SDK 的目录查询向量和 botocore 的分段正文、临时令牌向量。AWS 账户签发和付费推理仍待实测。[逐包审计](validation/cf-windows-a4752de/dependency-audit.csv)

解压版 CLI 在设置 `HTTP_PROXY` 时直连回环上游，三次均为 HTTP 200，代理接收 0 个连接。夹具将末帧延后 700 毫秒，三次首个完整 SSE 事件为 31、16、2 毫秒，末帧为 746、731、709 毫秒。独立 Rust 测试在测试专用信任根下经 CONNECT 完成 TLS 握手并逐字核对正文，随后对同一非回环域名设置 `NO_PROXY`，验证直连成功且代理没有收到连接。[流式采样](validation/cf-windows-a4752de/forward-h1.csv)

| 三轮中位数，工作集 / 私有字节（MiB） | Crabforge | Magpie |
| --- | ---: | ---: |
| 空目录网关 | 7.47 / 1.39 | 58.16 / 91.33 |
| Chat 网关 | 8.81 / 1.85 | 69.73 / 98.61 |
| 桌面主进程 | 151.29 / 263.99 | 76.91 / 93.57 |
| 桌面进程树 | 151.29 / 263.99 | 440.50 / 424.93 |

网关按相同请求交替采样。桌面每轮用独立配置启动解压版 Crabforge，Magpie 的三轮采样复用同一可见窗口，进程树包括七个 WebView2 子进程。同机 IPv6 回环、网卡全局单播 IPv6、直连失败后的 Kubo 0.43.1 回退均验证错误密钥 401、正确密钥 200 和目录正文。独立公网主机和不同 NAT 的连通性仍待实测。[空目录](validation/cf-windows-a4752de/catalog.csv) · [Chat](validation/cf-windows-a4752de/chat.csv) · [桌面](validation/cf-windows-a4752de/desktop-comparison.csv) · [IPv6](validation/cf-windows-a4752de/ipv6.csv)

用与冻结包相同的界面代码，在 Windows 调试渲染器中拍摄英语深色主题的六个控制台页面，覆盖宽 560 和 1200、高 800 逻辑像素下的列表及网格，共 24 张原始截图。窄屏使用底部导航，宽屏使用左侧导航。截图中网关处于停止状态，路由页显示空历史。[窄屏列表](validation/cf-windows-a4752de/overview-560.png) · [宽屏列表](validation/cf-windows-a4752de/overview-1200.png) · [两种宽度的网格](validation/cf-windows-a4752de/grid-overview.png)

### Windows x64 上游流式转发（`076480f`）

`076480faa8f3fa03872fe560e3ff55ce3e2c7900` 的 ZIP 共 59,972,145 字节，SHA-256 为 `B04680E5B13E40D13CC02B33AE8A4B75C24C59E39556C47A42B26BEB1B527239`。构建前后 285 个输入的哈希一致。解压后逐项复核 143 个文件及校验清单，126 项随包资产与源码逐字节相同。两个 CLI 分别列出 97 项预设。[构建输入](validation/cf-forward-076480f/build-inputs.sha256) · [包内文件](validation/cf-forward-076480f/SHA256SUMS)

三轮回环 HTTP/1.1 测量均通过解压版 CLI。上游写出首个 SSE 事件后等待 700 毫秒。客户端分别在 8、1、12 毫秒收到首个完整事件，在 711、702、713 毫秒收到末帧。设置代理时，回环请求直连；另一次 HTTPS 请求抵达本机 CONNECT 代理，代理返回 407 后网关报告 502。十二个跨协议方向的首帧时机、断开连接后并发槽释放、Anthropic 鉴权重试、Bedrock 签名、二进制媒体均纳入 Rust 回归。[HTTP/1.1 采样](validation/cf-forward-076480f/forward-h1.csv)

Windows x64 核心、CLI 和桥接的生产及构建依赖树已移除 Reqwest 和 `hyper-rustls`，仍含 Axum、Hyper、`hyper-util` 与 `serde_core`。核心和 CLI 的开发树仍含 Reqwest、`serde_json` 和 `serde_path_to_error`。模型目录与令牌交换使用 Courierust 1.1.3，实时上游请求使用 ureq 3.5.2。[逐包依赖审计](validation/cf-forward-076480f/dependency-audit.csv)

冻结源码的 Rust 串行回归通过 102 项，0 项失败，0 项跳过。全目标 Clippy、Flutter 分析及两项组件测试通过。Windows 控制台的首次使用与冷启动均通过。Oh My Pi 首轮未配置原生 Bun 路径与隔离的 `PI_CONFIG_DIR`，编辑器未显示 `work` profile；补齐测试环境后，首次使用与独立冷启动均通过。窗口集成测试使用调试桌面和包内 Release CLI，桌面内存采样使用解压版 Release 程序。

| 三轮中位数（工作集 / 私有字节，MiB） | Crabforge | Magpie |
| --- | ---: | ---: |
| 空目录网关 | 7.67 / 1.39 | 61.42 / 94.38 |
| Chat 网关 | 8.98 / 1.88 | 69.88 / 97.41 |
| 桌面主进程 | 152.24 / 265.69 | 148.55 / 152.03 |
| 桌面进程树 | 152.24 / 265.69 | 585.77 / 688.68 |

网关使用相同本机负载，按轮交替运行。桌面每轮从新配置启动 Crabforge。Magpie 在采样前打开，三轮复用同一窗口，进程树包含七个 WebView2 子进程。[空目录](validation/cf-forward-076480f/catalog.csv) · [Chat](validation/cf-forward-076480f/chat.csv) · [桌面](validation/cf-forward-076480f/desktop-comparison.csv)

解压版 CLI 通过 IPv6 回环、同机网卡全局单播 IPv6、直连失败后经 Kubo 0.43.1 回退三条路径，逐一核对错误密钥的 401、正确密钥的 200 和目录内容。跨机公网 IPv6 及不同 NAT 后的直连、打洞与回退仍待实测。[P2P 采样](validation/cf-forward-076480f/ipv6.csv)

### Windows x64 Oh My Pi YAML 迁移版（`a86a49c`）

从 `a86a49c4beed124fb407ff3ed8c505bf828b1169` 构建的 ZIP 为 60,124,742 字节，SHA-256 为 `FA6893DB4D306C97D3B8FDF9E2C0E1D5990C2BC62B9A93FF3B2E09EC486D222A`。284 个输入在构建前后逐项一致。解压后核对了全部 144 个文件，126 项资产与源码一致，两个 CLI 各列出 97 项预设。[冻结输入](validation/cf-yaml-a86a49c/build-inputs.sha256) · [包内文件](validation/cf-yaml-a86a49c/SHA256SUMS)

全量 Rust 串行回归 102 项通过，0 项失败，0 项跳过，全目标 Clippy 通过。Flutter 分析及两项组件测试通过。Windows 控制台和 Oh My Pi 编辑器分别通过首次使用、独立进程冷启动，测试调用冻结源码的调试桌面与包内 Release CLI。内存测量使用解压后的 Release 桌面。

| 三轮中位数（工作集 / 私有字节，MiB） | Crabforge | Magpie |
| --- | ---: | ---: |
| 空目录网关 | 7.88 / 1.47 | 58.47 / 91.23 |
| Chat 网关 | 8.88 / 1.54 | 70.13 / 99.81 |
| 桌面主进程 | 153.70 / 267.39 | 115.32 / 218.82 |
| 桌面进程树 | 153.70 / 267.39 | 376.53 / 915.74 |

网关按轮交替运行相同负载。Chat 夹具含一个上游模型，Crabforge 同时公开了模型别名。桌面每轮重启 Crabforge 并使用新配置，Magpie 复用已有窗口和七个 WebView2 子进程。[空目录](validation/cf-yaml-a86a49c/catalog.csv) · [Chat](validation/cf-yaml-a86a49c/chat.csv) · [桌面](validation/cf-yaml-a86a49c/desktop-comparison.csv)

IPv6 回环、同机网卡全局单播 IPv6、直连失败后回退至 Kubo 0.43.1 的三次测试，均以错误密钥得到 401、以正确密钥得到 200，并核对了模型目录。跨机公网 IPv6 和不同 NAT 的打洞仍待独立主机验收。[P2P 采样](validation/cf-yaml-a86a49c/ipv6.csv)

Oh My Pi 配置和目录使用 Saphyr 读取 YAML，继续用 yaml-edit 保留原文件的注释与格式。Windows 生产及构建依赖树中，核心、CLI、桥接仍经 Axum 引入 `serde_core`，并含 Hyper、Reqwest。核心与 CLI 的测试树还含 `serde_json` 和 `serde_path_to_error`。`serde_yaml` 已移出锁文件。`libp2p`、Saphyr、验收工具和桌面启动器的生产、构建与测试树均通过禁用库审计。

在 Windows 回环 HTTP/1.1 夹具中，服务端发送首个 SSE 事件后等待 700 毫秒，再发送末帧并关闭分块响应。Courierust 1.1.3 `Client::execute` 在三轮中均于末帧到达后一并交付两个事件。该接口适合有大小限制的目录请求，实时转发需要增量读取。[首帧时序（毫秒）](validation/cf-yaml-a86a49c/courierust-h1.csv)

### Windows x64 四协议 NextJSON 版（`efb2db7`）

从 `efb2db789685080d9f4307e42b43dbf6163ec088` 的 283 个输入离线构建，构建前后逐文件校验一致。`dist/cf-protocol-efb2db7/crabforge-0.1.0-windows-x64.zip` 共 60,140,402 字节，SHA-256 为 `F6F48B059BB1946D7D866AD3EFBD647FBF283C6F256911D06A7E4D8E0878FC14`。复核了解压后的 143 项文件哈希及清单本身。126 项声明的资产与源码逐字节相等。两个 CLI 各读出 97 项预设。[冻结输入](validation/cf-protocol-efb2db7/build-inputs.sha256) · [包内文件](validation/cf-protocol-efb2db7/SHA256SUMS)

冻结源码全量 Rust 串行测试首轮 **100 项通过，1 项失败，0 项跳过**。OmO 的 Chat 对话在 150 秒内没有产生网关路由记录；同一冻结源码单独重跑该用例，其四种协议均通过。Flutter 静态分析、两项组件测试均通过。Windows 控制台和 Oh My Pi 设置各通过首次与独立冷启动，两套集成测试使用冻结源码构建的调试桌面及解压包内的 CLI。Release 桌面另从解压目录成功启动三次。

| 三轮中位数（工作集 / 私有字节，MiB） | Crabforge | Magpie |
| --- | ---: | ---: |
| 空目录网关 | 7.90 / 1.49 | 61.20 / 95.40 |
| Chat 网关 | 8.84 / 1.56 | 72.83 / 101.24 |
| 可见桌面 | 129.80 / 221.44 | 528.54 / 868.88（含七个 WebView2 子进程） |

Magpie 单独主进程的工作集和私有字节为 160.26 / 219.67 MiB。网关按轮交替运行同一夹具。桌面每轮重启 Crabforge 并使用新配置，Magpie 保留已有窗口和数据。[空目录采样](validation/cf-protocol-efb2db7/catalog.csv) · [Chat 采样](validation/cf-protocol-efb2db7/chat.csv) · [桌面采样](validation/cf-protocol-efb2db7/desktop.csv)

解压 CLI 在 IPv6 回环、同机网卡的全局单播 IPv6 直连、首选连接失败后回退至 Kubo 0.43.1 中继三条路径上都验证了错误密钥 401、正确密钥 200 和目录内容。跨公网主机和不同 NAT 的验收仍待第二台主机。[IPv6 采样](validation/cf-protocol-efb2db7/ipv6.csv)

打包后的源码进一步移除了核心清单对 `serde` 的直接依赖。配置、目录协议、枚举和 Pi 客户端模型只派生 NextJSON 编解码。工作区全量串行测试 101 项通过，0 项跳过，全目标 Clippy 通过。按 Windows x64 包分别核对 `normal,build` 及 `normal,build,dev` 两棵树，核心、CLI 和桥接的生产及构建树已无 `serde_derive` 或 `serde_json`，仍经 `serde_yaml` 引入 `serde`，并包含 `serde_core`、Axum、Hyper 和 Reqwest。核心与 CLI 的开发树还含 `serde_json` 和 `serde_path_to_error`。`libp2p`、`crabforge-verify` 和 `crabforge-runner` 的两棵树均未引入上述库。

### Windows 桌面同期采样（2026-11-26）

使用 `e3726ad` ZIP 解压出的桌面程序重测。Crabforge 每轮用新配置启动，Magpie 复用用户已经打开的可见窗口。每轮在同一台机器依次采样，三轮中位数如下。两个应用打开的页面和已有数据可能不同。[原始 CSV](validation/cf-nextjson-594fadc304/desktop-comparison.csv)

| 桌面进程 | 进程数 | 工作集 MiB | 私有字节 MiB |
| --- | ---: | ---: | ---: |
| Crabforge | 1 | 152.41 | 263.36 |
| Magpie 主进程 | 1 | 167.31 | 222.80 |
| Magpie 主进程及 WebView2 子进程 | 8 | 370.04 | 793.39 |

### Windows x64 配置与 ADC 迁移版（`cf-nextjson-594fadc304`）

从提交 `e3726ad969bed6ad216b4374904cd47b42f94672` 的 278 个输入构建。冻结清单、生成本地化文件后的清单，以及构建完成后的清单一致，SHA-256 为 `594FADC3041F9A76D307742BCDDD87091FB3A1344CEEDE3A47133E7B7408F208`。ZIP 位于 `dist/cf-nextjson-594fadc304/crabforge-0.1.0-windows-x64.zip`，共 62,254,055 字节，SHA-256 为 `F4D26E028532E7D85780AF80D416F15C24B42F64917F5E8E3F31A4B51A581D40`。解压后核对了 140 个文件的哈希和随包清单，126 项声明的品牌、图标、字体和许可证逐字节匹配。两个 CLI 分别读出了 97 条内置预设。[构建输入](validation/cf-nextjson-594fadc304/build-inputs.sha256) · [包内文件哈希](validation/cf-nextjson-594fadc304/SHA256SUMS)

冻结源码的 Rust 全量串行回归 **98 项通过，0 项跳过**。首次测试误用了 npm 中的 Bun 占位文件，修正 PATH，使 `@oven/bun-windows-x64/bin/bun.exe` 优先后重新运行了整套测试。Flutter 分析和两项组件测试通过。Windows 控制台集成测试使用解压出的 CLI，修正测试中先取 `.last` 后等待控件的竞态后，首次使用及独立进程冷启动均通过。

| 三轮中位数（工作集 / 私有字节，MiB） | Crabforge | Magpie 0.1.4744 |
| --- | ---: | ---: |
| 空目录网关 | 7.85 / 1.45 | 58.27 / 91.04 |
| Chat 网关 | 8.86 / 1.57 | 69.32 / 97.39 |
| 桌面 | 128.61 / 219.41 | 采样时无可见窗口 |

两个网关在独立数据目录下交替运行，Chat 使用同一上游夹具。桌面每轮从解压目录冷启动。[空目录 CSV](validation/cf-nextjson-594fadc304/catalog.csv) · [Chat CSV](validation/cf-nextjson-594fadc304/chat.csv) · [桌面 CSV](validation/cf-nextjson-594fadc304/desktop.csv)

发布 CLI 在同一台 Windows 主机上完成 IPv6 回环、网卡全局单播 IPv6 直连，以及直连失败后回退至 Kubo 0.43.1 中继的测试。三条路径均以错误和正确网关密钥分别取得 HTTP 401 和 200，并核对了目录内容。不同 NAT 和跨公网主机的连通性仍待实测。[IPv6 结果](validation/cf-nextjson-594fadc304/ipv6.csv)

### 源码依赖审计与控制台视图（2026-11-26）

CLI 和 Flutter 桥接层的直接运行依赖已切换到 NextJSON。配置修改、路由组分享和导入、状态记录、模型目录以及 Oh My Pi 角色快照、作用域恢复和协议历史均使用新编码。读入旧快照时按名称比较 JSON 对象字段，回退数组仍逐项按顺序比较。CLI 测试夹具和核心网关仍有 Serde 依赖。此批源码在上方 `e3726ad` 安装包之后。

Windows x64 使用 `cargo tree --offline --locked -p <包名> --edges normal,build --prefix none` 检查解析后的依赖，再将 `--edges` 换为 `normal,build,dev` 复核开发和测试依赖。两次检查中，`libp2p` 和 `crabforge-verify` 均未引入名称含 `serde` 的包、Hyper 系列、Reqwest 或 Axum。`crabforge-core`、`crabforge-cli` 和 `crabforge-bridge` 仍引入 `serde`、`serde_core`、`serde_derive`、`serde_json`、`serde_path_to_error`、`serde_yaml`、`hyper`、`hyper-rustls`、`hyper-util`、`reqwest`、`axum` 和 `axum-core`。

- 此次历史构建以修改过的 `multiaddr 0.19.1` 接入 `libp2p 0.57.1`。原 crates.io 压缩包 SHA-256 为 `fd707c05eadfb37577f01cbec206726feeecf0b9edea259aee6fc9bc541ee253`，上游提交为 `d5981da4b337d21807ee5869178fbaf3d5ec7e03`。同机重跑 DCUtR 升级、中继预约与回退、Kubo 双向鉴权共四项，全部通过。当前构建使用 Crabforge 自有地址编解码器。[当前清单](../crates/peer-address/Cargo.toml)
- 验收工具的 HTTP 客户端和测试服务使用 Courierust，模型目录和内存探针使用 NextJSON。使用 `cf-dcutr-1031922879` 的 `carbforge-cli` 与 Magpie 分别重跑一轮空目录及 Chat 负载，四次均完成。网页对话的 ChromeDriver 请求、模型输入和响应改用 NextJSON 后，两个真实 Chrome 本地夹具通过。
- 六个控制台页面共用列表或网格偏好，Rust 将选择原子写入数据目录的 `ui-layout`。Windows 桌面集成测试覆盖供应商、路由组、Agent、网关和模型的视图切换或沿用，另核对深色主题、系统剪贴板和独立进程重新启动后的布局。Flutter 静态分析和两个组件测试通过。
- 配置解析、快照、原子保存及内置预设输出使用 NextJSON。与旧版 CLI 对照，97 项预设的全部 JSON 字段相等。旧版快照测试覆盖 IPv6 监听地址、`noStandIn` 别名、显式空协议列表、匹配规则，以及过期快照、未知字段和重复字段的拒绝。15 项路由组专项和 Anthropic 中转鉴权专项通过。
- Vertex ADC 的凭据解析、JWT、IAM JSON 及 Claude 请求体使用 NextJSON，OAuth 表单和 IAM 请求通过 Courierust 发送，限制响应体为 64 KiB。本机实测并发刷新、凭据更改、两级 IAM 委派、提前续期和 JWT 签名校验。通过 Windows 系统根证书和 HTTP CONNECT 代理向 `oauth2.googleapis.com/token` 发送刻意无效的测试凭据，Google 返回 HTTP 401。真实账户的令牌签发和云端推理仍待验证。

在 `e3726ad` 之后，模型目录 GET 请求、分页、鉴权头协商和 Bedrock 目录签名改经 Courierust 发送。通过 CLI 从 OpenCode Go、Zen 公网目录分别收到 37、108 个模型 ID。本机测试覆盖多协议合并、显式目录、分页失败、鉴权冲突和 ADC。关闭 Axum 的表单和查询提取功能后，`serde_urlencoded` 已从锁文件及上述两类 Windows 依赖树中移除。

### Windows x64 直连升级与中继回退版（`cf-dcutr-1031922879`）

Windows 包 `dist/cf-dcutr-1031922879/crabforge-0.1.0-windows-x64.zip` 共 62,207,857 字节，SHA-256 为 `B674CA946F26730D462A4DAEDE387405A1AEDBC33C8D5312460A97C0457AD953`。解压到新目录后核对全部 141 个文件，分别运行双 CLI 并启动桌面三次。包内 126 项品牌图标、字体、应用图标和许可证与源码一致。[逐文件 SHA-256](validation/cf-dcutr-1031922879/SHA256SUMS)

原始 270 项源码清单的 SHA-256 为 `10319228792AC69D3AF1A18C8148ADAEEEAF2163B5079315EF7626D774C61ADB`。Flutter 在第一次构建中重新生成了 10 个本地化 Dart 文件。对生成后的实际输入重新取哈希并再次离线构建 Windows Release，构建前后 270 项完全一致，清单 SHA-256 为 `42A3BBE7441BA2F51074ED2F77A473C79842D0D9628A0DDDB62C1A356E4EFBC9`。[实际构建输入清单](validation/cf-dcutr-1031922879/build-inputs.sha256)

Rust 工作区一次串行跑完 **97 项，全部通过，0 跳过**，包含默认忽略的真实客户端用例。全目标 Clippy、Flutter 静态分析、两个组件测试，以及 Windows 控制台首次使用和独立进程冷启动均通过。

| 三轮中位数，工作集 / 私有字节（MiB） | Crabforge | Magpie 0.1.4744 |
| --- | ---: | ---: |
| 空目录网关 | 7.85 / 1.45 | 58.11 / 92.21 |
| Chat 网关 | 8.86 / 1.59 | 66.98 / 96.43 |
| 解压后桌面 | 129.26 / 220.52 | 本轮无可见窗口 |

网关在独立数据目录按轮交替启动。Chat 使用同一上游夹具，Crabforge 展示两个模型 ID，Magpie 展示一个。桌面各轮使用新配置，等待可见窗口和节点身份就绪后采样。[空目录](validation/cf-dcutr-1031922879/catalog.csv) · [Chat](validation/cf-dcutr-1031922879/chat.csv) · [桌面](validation/cf-dcutr-1031922879/desktop.csv)

- 三个 Rust libp2p 节点在 `::1` 上建立中继电路。客户端观察到 DCUtR 成功事件，核对其连接 ID 对应非中继 QUIC。关闭中继并观察到电路断开后，从新开的直连流读取受密钥保护的目录。中继以原身份、原端口重启后观察到网关再次预约成功。
- 两个发布版 CLI 通过 IPv6 回环和网卡全局单播地址分别完成直连。随后让首选地址指向仅占用 UDP 端口的本地套接字，客户端通过 `--relay-fallback` 自动经独立的 Kubo 0.43.1 进程建立隧道。三次均检查 401、200 和响应目录。Kubo 仅监听 `::1` 时返回的预约未包含可用地址，成功用例使用本机网卡分配的全局 IPv6。Rust 夹具另验证拒绝不同网关 Peer ID 的备用地址。[发布版请求结果](validation/cf-dcutr-1031922879/ipv6.csv)
- 上述试验全部在同一台 Windows 主机上完成。跨公网 UDP 可达性、不同 NAT 后的打洞与回退仍需第二台独立网络主机实测。

### Windows x64 回退链、中继与品牌图标版（`cf-relay-a78f732cf2`）

从 269 个构建输入生成的清单 SHA-256 为 `A78F732CF27C47E85C34DB4FB620F32D0DA1A0EF39D01F14DF4EBBE40FA70CE9`。从该目录离线构建了双 CLI 和 Flutter Windows Release。`dist/cf-relay-a78f732cf2/crabforge-0.1.0-windows-x64.zip` 的 SHA-256 为 `4D510F0CE7FFB42292CA546BF5F1B599207328654073B10448AC6276E7CEA431`。压缩包内有 140 个运行文件和一份逐文件 `SHA256SUMS`。已核对 119 个品牌资产及 7 个应用图标、字体和许可证文件。三轮桌面数据来自发布目录，另从新解压的 ZIP 复核了 140 项哈希，启动两个 CLI 及一次可见桌面窗口。

| 三轮中位数，工作集 / 私有字节（MiB） | Crabforge | Magpie 0.1.4744 |
| --- | ---: | ---: |
| 空目录网关 | 7.77 / 1.45 | 57.98 / 91.76 |
| Chat 网关 | 8.79 / 1.59 | 68.14 / 97.25 |
| 桌面 | 151.27 / 263.79 | 此轮没有可见窗口 |

网关每轮启动独立配置，交替运行同一工作负载。Chat 夹具使用同一上游模型；Crabforge 的公开目录含两个 ID，Magpie 含一个。[逐轮内存、IPv6 请求和发布文件哈希](validation/cf-relay-a78f732cf2/)

首轮含 ignored 用例的全量串行 Rust 回归为 93 项通过、2 项失败。Vibe 夹具的 Python 解释器路径失效，Ollama SDK 虚拟环境缺少 `python.exe`。恢复两处测试环境后，在同一冻结目录单独重跑这两个用例，均通过。随后整套串行重跑一次，**95 项全部通过，0 跳过**。冻结源码通过 `cargo fmt --check`、全工作区全目标 Clippy、Flutter 静态分析和两个组件测试。

- **网络。** 发布版 CLI 在两个独立进程间，通过 IPv6 回环地址和本机网卡分配的全局单播 IPv6 地址分别建立直连 QUIC 隧道。两次读取 `/v1/models` 均验证错误密钥 401、正确密钥 200 和目录内容。另在 `::1` 上以三个 libp2p 节点验证中继预约、两个指定 Peer ID 的 `CircuitReqAccepted` 和鉴权 HTTP 请求。Kubo 0.43.1 双向直连也已通过。以上均为同机试验。跨机公网 IPv6、不同 NAT 后的打洞和直连失败后的自动回退仍待实测。
- **桌面。** Windows 控制台实测过滤模型、切换列表和网格、编辑路由组、通过系统剪贴板读回分享文本，并在独立进程冷启动后读回 `no_stand_in`。Oh My Pi 的首轮测试缺少 `PI_CONFIG_DIR`，未能找到夹具的 `work` profile。将它指向独立配置并用 `omp --profile work config path` 核对后，保存和冷启动两轮均通过。组件测试覆盖窄、宽布局和品牌 SVG 解码。
- **依赖。** NextJSON 已用于编译期模型目录、运行时上游目录响应与请求级手选快照，三个生成的目录索引均与迁移前逐字节相同。Courierust 已用于 ChromeDriver 通信，两个真实 Chrome 本地用例通过。运行时仍直接依赖 Serde、Hyper 和 Reqwest。锁定的 libp2p 0.57.1 通过 `multiaddr` 0.19.1 无条件引入 Serde，零传递依赖目标尚未达成。

### Windows x64 供应商改名版（`cf-provider-6e4abd2e07`）

146 项冻结源码清单的 SHA-256 为 `6E4ABD2E07494110CB887F24701AEBEEE3F2235FF866FDADCC4C7DB48F6BA1BF`。首轮全量 Rust 回归 90 项通过、2 项失败；在同一源码上单独重跑 Crush 对话和 DevEco 配置两项均通过。DevEco 仅验证模型、鉴权端点和配置清理。桌面控制台和 Oh My Pi 设置均完成独立进程冷启动。

| 三轮中位数，工作集 / 私有字节（MiB） | Crabforge | Magpie 0.1.4744 |
| --- | ---: | ---: |
| 空目录网关 | 7.62 / 1.35 | 58.21 / 92.00 |
| Chat 网关 | 8.63 / 1.48 | 70.12 / 98.64 |
| 桌面 | 149.82 / 258.85 | 此轮没有可见窗口 |

五项发布文件的 SHA-256 均已复核。复核时本地发布副本缺少 Flutter 运行库和 ICU 数据，已从同工具链的 Release 目录补齐 `flutter_windows.dll` 和 `data/icudtl.dat`。这两个运行文件未列在原五项哈希内。[改名版原始 CSV 和五项哈希](validation/cf-provider-6e4abd2e07/)

### 早期协议与兼容性记录

本次在 Windows 10 x64（19045）、Intel Core i7-11850H、31.2 GiB 可见内存上使用 Rust 1.99.1、Flutter 3.47.9、Dart 3.13.7 及 flutter_rust_bridge 2.13.0，参考程序为 Magpie 0.1.4744。

成员目录开关版运行 `cargo test --locked --offline --workspace -- --include-ignored --test-threads=1` 通过 61 项测试，覆盖 16 种官方 Agent 客户端的对话、DevEco 生效配置、93 个专属地址、自动与手动分组、真实模型、Kubo、浏览器、Vertex 本地上游及跨网关联调。Clippy 的 `--workspace --all-targets -- -D warnings` 检查通过。[构建与验证](../CRABFORGE.md#构建与验证)

| 范围 | 已验证 | 仍有差异或需要验证 |
| --- | --- | --- |
| 供应商与模型 | 95 项含固定地址的预设，另有 Azure OpenAI 和 Remote Magpie 资源地址模板。Gemini、Anthropic 的分页经本地服务器验证。Bedrock SigV4 在固定时间、临时凭证和分段请求体下生成了与 botocore 相同的签名。Vertex 支持 Express 及项目模式，ADC 和项目模型路径已联调。 | 在线目录随时间变化。各预设未逐项完成真实推理。Bedrock、Vertex 待真实云账户验证，部分厂商专用鉴权待移植。 |
| 协议与流式 | Chat Completions、Responses、Anthropic Messages、Gemini 两两转换。本地上游覆盖 12 个方向的同步回复、仅返回 JSON 的上游和真正的增量 SSE。上游等待下游收取首段文字后继续，核对后续工具调用、用量及结束事件。 | 工具参数按调用聚合后下发。转换支持文本和函数工具。托管工具、推理块、图像内容、持久 Responses 状态和专用字段需要原生端点或进一步适配。 |
| 网关与媒体 | 原生 SSE 逐块转发，验证了回退后的首块。图片编辑、转写和翻译使用 multipart，验证二进制内容、文件名、重复文件字段及模型名称。 | 图片、语音、嵌入和重排的原生路径依赖供应商。专用媒体转换、异步视频任务及决策接口尚未覆盖。 |
| Ollama 客户端 | 支持 version、tags、show、chat、generate 五个接口。官方 Python SDK 0.6.3 经网关读取目录和模型详情，向 llama.cpp 请求同步 chat 和流式 generate，得到相同文字与 token 数。另验证 `:latest`、工具结果匹配、逐行 NDJSON、分帧工具参数、首帧错误回退，以及正文开始后上游截断时只提交一次。 | 模型详情使用网关提供的占位元数据。推理依赖上游能力，下载与管理权重需使用 Ollama 服务。 |
| 路由 | `供应商/模型`、裸模型 ID、顺序、轮询、加权轮询、最多八层嵌套、429 或服务故障回退。16 个并发请求按 3∶1 分为 12 和 4。子组在请求到达时才消耗自己的轮次。自动分组专项覆盖跨供应商拼写、同一供应商的窗口变体、公开与旧地址、隐藏及恢复、自定义组、组内回退和只剩一家供应商时延续请求。Windows 实测开关和派生组持久化边界。 | 按账户配额或价格排序、会话亲和、订阅优先、插件路由、用量历史和 Magpie 配置迁移尚未覆盖。 |
| Agent | 93 个专属地址均通过本地上游回复及日志归因检查。已通过 17 种官方客户端对话。包括 Codex CLI 0.172.0、Claude Code 2.1.332、Gemini CLI 0.71.1、Pi 1.6.0、Goose 1.61.0、Qwen Code 0.28.4、Mistral Vibe 3.1.5、Crush 0.102.0、Kimi Code 2.8.0、nanobot 0.4.2、mu 0.3.13、OmO 5.1.83、Prime Agent 0.10.5、OpenCode 2.4.1、MiMo Code 0.1.17、GitHub Copilot CLI 1.0.116 和 Oh My Pi 19.2.1。Pi 的第二轮请求带回分帧 `read` 工具调用的 ID 和文件内容。 | 其余 76 项的官方客户端对话待验收。DevEco Code 0.2.6 的 `debug config` 已读回主模型、小模型、网关地址和密钥，退出后临时配置已清除。对话需用户先确认 DevEco 产品协议。动态配置、长期会话和完整配置管理仍待移植。 |
| DeepSeek、ChatGLM 网页 | 网页供应商可通过 CLI 与 GUI 添加或登录。真实 Chrome 和 ChromeDriver 158 在本地站点夹具上验证两个适配器的输入、等待完成和读取回复。 | 线上账户、登录挑战和登录后的完整回合待验证。网页聊天支持纯文本，SSE 在网页显示完成后返回。浏览器进程不计入 API 网关基准。 |
| P2P | 同机桌面应用连接独立 CLI，读取远端目录。Kubo 0.43.1（go-libp2p）与 Crabforge 分别作为拨号方和监听方，在 `/crabforge/gateway/1.0.0` 上经 QUIC 读取目录，两次无密钥请求均返回 401。两端 Crabforge 另经 QUIC 调用 Magpie 0.1.4744，再由 Magpie 请求 llama.cpp，收到真实模型生成的 `The sky is cloudy.` 及 6 个输出 token。 | 所有节点位于同机。公网直连仍需第二台具有可达 UDP 地址的主机验证。 |
| 桌面与订阅 | Windows 桌面集成测试通过真实 Flutter ↔ Rust 绑定添加供应商，读取指定目录并保存新旧模型，启停网关、查询 Agent 路由、连接及断开隧道。 | 当前提供 Windows runner。Magpie 的订阅登录、令牌刷新、用量及社区插件运行时尚未移植。 |

原生 HTTP 请求体上限为 8 MiB，最多 16 个请求同时访问上游。跨协议转换接受最多 2 MiB 的 JSON 回复；SSE 单帧最多 2 MiB，单次流式回复累计最多 16 MiB 的 `data` 载荷。上游在已输出内容后断开时，客户端也会收到流中断。

故障测试覆盖首帧报错后改用组内下一上游、已输出正文后截断且只提交一次，以及达到 16 个在途请求后返回 429。客户端断开后，新请求可再次取得并发许可。上游给出的长度限制映射到 Responses 的 `response.incomplete` 及 `max_output_tokens`。

Windows 集成测试通过 Flutter 编辑器将一个组从 `desktop` 改名为 `renamed` 并配置权重 3。随后从父组向本地上游发送请求，在路由页读到 `group/parent → group/renamed → local`。Rust CLI 专项验证了自动组连续改名、父组权重迁移、旧地址调用、重用旧名和冲突回滚。

手动组专项让第一个请求等待上游，再从另一个 CLI 进程切换成员。新的请求立即使用新选择，等待中的请求收到 429 后继续使用原先选定的成员。已失效的选择实际转发到第一项。CLI 拒绝同时指定手动选择与权重或轮询。临时切换策略及子组改名均保留有效的选择。Windows 桌面在运行中观察到外部选择变化，拒绝用旧快照覆盖它，再由卡片切回目标并发出请求。独立进程冷启动恢复了所选成员。

成员开关专项经真实 CLI 保存并重新启动网关。停用权重为 3 的子组后，复制组的四次请求全部到达另一成员。重新启用源组中的该子组后，四次中有三次到达它。测试还覆盖子组改名、非手动组全停时磁盘内容保持原样，以及手动策略选中已停用成员。

在普通 API 地址，网关按 User-Agent 识别 Cursor、Pi-Desktop、Codex、DimCode、Dim、VS Code 和 Kimi，未知 SDK 保留产品名。93 个专属地址均以 `curl` 作为 User-Agent 实测，路由记录优先使用地址标识。Vibe 和 Crush 双进程各自选择 `local/m` 与 `local/n`，测试逐项核对两个模型不同的回复及两条归因。Crush 原有单进程嵌入模式曾在已退出时仅打印换行。改用本地服务的完成事件后，原有的快速响应夹具连续通过十轮双进程测试。Kimi 的用户默认模型、窗口值和 TOML 注释在启动后保留，nanobot 的个人配置、会话目录及临时文件清理也通过真实客户端检查。

已逐个比较发布包与源文件中 `crabforge.png`、五个字体文件及韩文字体许可证的 SHA-256，并核对 Windows 可执行文件中的所有图标图像和 `app_icon.ico`。九个 ARB 的译文字符均可在随包字体的 Unicode 字符映射表中找到。桌面集成另验证九语切换和独立进程重新启动。

### 在线模型目录

2026-11-25 使用与网关基准相同的发布版 `carbforge-cli.exe`（SHA-256 前 16 位 `946998EF8AA3F307`）逐项检查 97 个预设，使用独立临时配置，共用端点配置的别名复用同一次检查结果。9 项从远端公开目录读到模型，14 项返回内置清单，66 项在无密钥条件下未取得完整目录。另有 2 项需要填写资源地址，5 项需要本地服务，Bedrock Mantle 需要 AWS 凭证。每项的结果、模型数和错误见[目录实测明细](crabforge-provider-catalogs.csv)。

| 预设 | 模型 ID 数 | 预设 | 模型 ID 数 |
| --- | ---: | --- | ---: |
| Ollama Cloud | 25 | OpenRouter | 472 |
| OpenCode Go | 37 | ClinePass | 458 |
| OpenCode Zen | 108 | Command Code | 105 |
| NVIDIA | 30 | ModelScope | 38 |
| Vercel | 517 | | |

Google 预设的 Gemini 原生目录本次返回 HTTP 403。Kilo 早前曾返回 467 个 ID，本轮未取得完整目录。另用 Crabforge 向 OpenCode Zen 的 `deepseek-v4.1-flash-free` 发起一次无密钥对话，上游返回 HTTP 429 和 `FreeUsageLimitError`，未取得该模型的推理结果。

```powershell
$cli = 'gui/build/windows/x64/cargo/x86_64-pc-windows-msvc/release/carbforge-cli.exe'
cargo run --locked -q -p crabforge-verify -- catalogs $cli | Set-Content -Encoding utf8 catalog-results.csv
```

Bedrock Runtime、Vertex Express、Voyage AI、百度千帆、腾讯 TokenHub、火山方舟及华为云的相应套餐可使用内置清单。千帆按个人版和企业版区分，腾讯按中国与国际套餐区分。按量付费的方舟、千帆和腾讯仍读取远端目录。Fireworks Nexus 在选择预设时带入 `auto`。Bedrock 的协议选择按模型区分 Claude、闭源 OpenAI GPT 和其他模型，已用固定端点测试覆盖。

多协议目录夹具验证了 Chat 和 Responses 共用地址时只读取一次，合并 Chat 和 Gemini 重叠的 ID，以及 Anthropic 第二页损坏时丢弃该目录的不完整结果。显式指定 `endpoints.models_url` 会覆盖协议目录和内置清单。Google AI Studio 的双协议预设使用 Gemini 原生目录。

Gemini 本地验收覆盖裸主机、粘贴的 SDK 方法地址、带路径前缀的中转、`pageSize=1000` 的游标分页，以及按 `supportedGenerationMethods` 过滤模型。OpenCode 另用随包 models.dev 数据核对同名 `minimax-m2.7` 在 Go 上使用 Messages、在 Zen 上使用 Chat，验证了 Gemini 和 Responses 原生地址、显式中转地址、独立目录、会话头和鉴权头。OpenCode V2 官方 CLI `2.4.1` 已经通过本地网关调用上游并显示回复。

使用 SHA-256 前缀为 `932460AF6B580D63` 的 CLI 和隔离配置再次访问官方目录，Go 返回 37 个 ID，Zen 返回 108 个。另用本地 TLS 上游接收 Go 的 Chat 图像消息。测试核对 `/inference/go/openai/v1/chat/completions` 路径、替换模型名后的逐字节正文、会话头、User-Agent 和返回给客户端的回复。Go 与 Zen 的付费推理仍待账户验证。

第三方 Anthropic 鉴权专项通过真实 CLI 进程和本地上游验证。目录接口接受 `x-api-key` 后，第二次独立进程查询直接使用这个请求头。Messages 接口拒绝双头和单独的 `x-api-key` 后，`Authorization: Bearer` 成功，相同请求体重发，后续请求直接使用 Bearer。测试还检查了手工指定单头，以及普通 401 的 85,500 字节正文原样返回。Windows 编辑器已验证单头选择保存与重读。

### Vertex AI 项目模式

Rust 测试通过本地 OAuth 和 IAM 端点验证了用户刷新令牌、服务账号 JWT 换取令牌以及两级服务账号模拟。测试对 JWT 的 RS256 签名验签，检查授权范围和受众，确认 8 个并发请求仅交换一次令牌，临近过期和替换凭证文件后会再次交换。显式使用 ADC 的目录地址须属于已配置的 Google 项目和地区。

测试启动真实 CLI 网关，用 API key 访问模拟 Vertex 上游，核对 Gemini 流式路径、合作方 Chat Completions、Claude 同步、流式与 token 计数的路径和请求体。两个流式接口均在上游结束前读到首帧。另验证 API key 优先于无效 ADC 文件，以及首选供应商缺少 ADC 时回退。区域内置清单在 `global`、`eu`、`us`、`us-central1` 分别返回 23、9、9、3 个 ID。Windows 桌面测试修改区域并读取该区域清单，重读磁盘验证保存结果。Google Cloud 的实际权限、区域可用性及计费推理仍待持有相应权限的账号验证。

### 本地模型实测

使用 llama.cpp `b12722` 的 Windows x64 CPU 发布版加载 `SmolLM2-135M-Instruct-Q4_K_M.gguf`，仅向 Crabforge 配置该服务器的 Chat Completions 基址。Crabforge 从服务器发现 `smollm2` 后，分别经 Chat Completions、Responses、Messages、Gemini、Ollama chat 和 Ollama generate 的 HTTP 入口发送同一句提示，六次均得到 `The sky is cloudy.` 和 6 个输出 token。此测试使用真实模型权重，内存基准另用固定回复的上游，以便两端负载一致。

用于内存测量的 CLI 发布程序也单独启动过。Ollama SDK 0.6.3 使用网关密钥列出模型，向 `llama/m:latest` 发起流式 generate 请求，逐行合并后同样得到 `The sky is cloudy.` 和 6 个输出 token。

可将 `LLAMA_TEST_SERVER` 设为 `llama-server.exe` 绝对路径、`LLAMA_TEST_GGUF` 设为 GGUF 绝对路径，再运行 `cargo test --locked -p crabforge-core --test llama local_weights -- --ignored --nocapture`。模型文件 SHA-256 为 `2e8040ceae7815abe0dcb3540b9995eaa1fa0d2ca9e797d0a635ae4433c68c2d`。[llama.cpp b12722](https://github.com/ggml-org/llama.cpp/releases/tag/b12722) · [模型版本](https://huggingface.co/bartowski/SmolLM2-135M-Instruct-GGUF/tree/09816acd5d99df7be770d85ea30822623dab342c)

## 复现测量

Rust 验证工具为两个进程分别创建临时配置。每轮完成就绪探测和 100 次串行请求，等待两秒，再每隔 200 ms 读取一次 Windows 工作集与私有字节。取五次采样的中位数，再对三轮取中位数。启动顺序逐轮交替。

`Catalog` 使用空配置。`Chat` 使用相同的单模型本地 HTTP 上游，每个响应均检查文本。Crabforge 的目录列出 `bench/m` 和裸名 `m`，Magpie 列出一个名称；两端实际都请求 `bench/m`。脚本采样服务进程，HTTP 客户端、模拟上游、桌面及 Chrome 进程单独运行。此负载衡量请求结束后的驻留内存，未测高并发峰值。

```powershell
$cli = 'gui/build/windows/x64/cargo/x86_64-pc-windows-msvc/release/carbforge-cli.exe'
$magpie = "$env:LOCALAPPDATA\Programs\magpie\magpie.exe"
cargo run --locked -q -p crabforge-verify -- gateway $cli $magpie catalog --rounds 3
cargo run --locked -q -p crabforge-verify -- gateway $cli $magpie chat --rounds 3
cargo run --locked -q -p crabforge-verify -- desktop 'gui/build/windows/x64/runner/Release/crabforge.exe' --rounds 3
```

网关测量使用发布版且两端关闭网页会话与 P2P。双方启用的产品默认功能不同，测得的差异包含这部分成本。`desktop` 在全新 Crabforge 配置下打开窗口，网关保持初始停止状态。可用 `--magpie-pid <PID>` 同时采样已打开的 Magpie 窗口及其 WebView2 子进程。进程树工作集求和可能重复计算共享页。

### 网关实测

| 负载 | 网关 | 三轮工作集 MiB | 中位数 MiB | 三轮私有字节 MiB | 中位数 MiB |
| --- | --- | --- | ---: | --- | ---: |
| Catalog | Crabforge | 7.20、7.16、7.17 | 7.17 | 1.27、1.21、1.24 | 1.24 |
| Catalog | Magpie | 59.54、57.47、48.42 | 57.47 | 93.64、91.32、82.11 | 91.32 |
| Chat | Crabforge | 8.13、8.12、8.11 | 8.12 | 1.41、1.39、1.38 | 1.39 |
| Chat | Magpie | 67.00、67.51、71.42 | 67.51 | 95.57、96.20、100.20 | 96.20 |

100 次 Catalog 请求后，Crabforge 的工作集和私有字节中位数分别低 87.5% 和 98.6%。100 次 Chat 请求后分别低 88.0% 和 98.6%。

| 网关发布程序 | SHA-256 |
| --- | --- |
| carbforge-cli.exe | `946998EF8AA3F307319FCF57783FEF6D2BFA6394E725934D561C229AC8E37333` |
| crabforge-cli.exe | `051C32181693BDEC7AD891EE8B604A66253FAF06BB4B785C7032DEA4E2349091` |
| magpie.exe | `340C0F3349C684143926AE9CEDC7A803662428849A10283976B6E07E40A6E0BC` |

### 桌面实测

以下为三轮中位数。Crabforge 每轮启动新的空白窗口，使用 Flutter Impeller。Magpie 三轮采样同一个已运行窗口，其配置、网关状态和历史负载没有重置，因此桌面数据仅供观察。

| 采样对象 | 进程数 | 工作集 MiB | 私有字节 MiB |
| --- | ---: | ---: | ---: |
| Crabforge 空闲窗口 | 1 | 148.43 | 255.66 |
| Magpie 现有主进程 | 1 | 195.31 | 235.54 |
| Magpie 现有主进程及其 WebView2 子进程 | 8 | 877.96 | 1247.79 |

| 桌面采样对象 | 三轮工作集 MiB | 三轮私有字节 MiB |
| --- | --- | --- |
| Crabforge | 150.43、148.43、148.27 | 255.66、255.24、257.34 |
| Magpie 主进程 | 189.44、195.50、195.31 | 229.66、235.68、235.54 |
| Magpie 含 WebView2 | 871.16、878.46、877.96 | 1215.64、1277.42、1247.79 |

Crabforge 主进程的私有字节中位数比所观察的 Magpie 主进程高 20.12 MiB。当前数据无法给出同等桌面负载的节省比例。桌面采样包中 `crabforge.exe`、`crabforge_bridge.dll`、`data/app.so` 的 SHA-256 前 16 位分别为 `D0E1A6AA74767A58`、`70F5E8F4842B53D4`、`2FE85C6DD5D16E91`。

### 2026-11-26 复测

九客户端 CLI 发布版使用同一负载再跑三轮。目录巡检 CSV 对应上一轮发布版。

| 负载与进程 | 三轮工作集 MiB | 中位数 MiB | 三轮私有字节 MiB | 中位数 MiB |
| --- | --- | ---: | --- | ---: |
| Catalog · Crabforge | 7.27、7.26、7.29 | 7.27 | 1.28、1.26、1.30 | 1.28 |
| Catalog · Magpie | 57.46、58.50、57.27 | 57.46 | 90.88、91.50、91.22 | 91.22 |
| Chat · Crabforge | 8.21、8.20、8.21 | 8.21 | 1.41、1.41、1.41 | 1.41 |
| Chat · Magpie | 72.10、69.14、68.99 | 69.14 | 101.50、97.74、98.45 | 98.45 |
| Crabforge 空闲窗口 | 149.70、148.43、147.41 | 148.43 | 255.82、257.89、256.51 | 256.51 |
| Magpie 现有主进程 | 174.78、175.25、177.62 | 175.25 | 231.45、231.70、233.90 | 231.70 |
| Magpie 主进程及 WebView2（8 个进程） | 826.43、828.57、830.95 | 828.57 | 1335.61、1297.02、1344.68 | 1335.61 |

| 复测产物 | SHA-256 |
| --- | --- |
| carbforge-cli.exe | `5E72F5D463F5B8226A992955AF5CC2FF43615C37330A8992B88396C6215E0617` |
| crabforge-cli.exe | `D4BB3A335B6886C5D494478F9C15F7F462EE28D0F7680AB40FED39441E8A77DF` |
| crabforge.exe | `D0E1A6AA74767A58C0174ECE292779B73E5058CA49802BAA371B3FD2AFD9A81D` |
| crabforge_bridge.dll | `011F2401489425130ED9A1CF8ABCFD14D679D6442B9BF9458EBAEF5DF660C2BC` |
| data/app.so | `85E6960839E82CFA6D586D7B0A0C2DA6002B92777D73142F85A4D204EC81A547` |

### 手动路由版网关复测

| 负载 | 程序 | 三轮工作集 MiB | 中位数 | 三轮私有字节 MiB | 中位数 |
| --- | --- | --- | ---: | --- | ---: |
| Catalog | Crabforge | 7.29、7.29、7.28 | 7.29 | 1.27、1.27、1.26 | 1.27 |
| Catalog | Magpie | 57.60、58.36、61.79 | 58.36 | 91.82、92.52、96.10 | 92.52 |
| Chat | Crabforge | 8.18、8.18、8.18 | 8.18 | 1.36、1.37、1.35 | 1.36 |
| Chat | Magpie | 70.02、70.73、67.38 | 70.02 | 99.16、99.64、95.77 | 99.16 |

这一轮测量的 `carbforge-cli.exe` SHA-256 为 `1FE38B54C9654C091B7F0D41861060487A98322E3DBE0D58B6711351054EB41B`，同批 `crabforge-cli.exe` 为 `8E3FBCE52B3550FE1EBF69C74CB2D5DE91D8956ABB04BADF58D2A4A1F07412C0`。

### OpenCode 版复测

三轮中位数如下。桌面复用了现有 Magpie 窗口。

| 对象 | 工作集 MiB | 私有字节 MiB |
| --- | ---: | ---: |
| Catalog · Crabforge | 7.32 | 1.26 |
| Catalog · Magpie | 57.83 | 91.57 |
| Chat · Crabforge | 8.26 | 1.39 |
| Chat · Magpie | 68.46 | 97.17 |
| Crabforge 空闲窗口 | 148.37 | 257.68 |
| Magpie 现有主进程 | 182.37 | 235.52 |
| Magpie 主进程及 WebView2（8 个进程） | 920.74 | 1432.35 |

| 采样产物 | SHA-256 |
| --- | --- |
| carbforge-cli.exe | `932460AF6B580D633CDE0D2AD4EC4845977679EF697A4EF3A591AAD907908A66` |
| crabforge-cli.exe | `0B231F171C84D4CDD5C2D1E9432CC3A02ACB1E79A2EC465839A1EA051B5E9C8C` |
| crabforge.exe | `D0E1A6AA74767A58C0174ECE292779B73E5058CA49802BAA371B3FD2AFD9A81D` |
| crabforge_bridge.dll | `A0C74E5E72CA67E955581682CDFC04CA58E8686CBA78571E040F7EC4E108DC97` |
| data/app.so | `C74760CF31CCE729D1DDE1CFBFCA21B017C8398A15DF7E357E815F2211C343DB` |

### 路由组成员开关版

该版包含排序、复制、备注及成员启停。隔离配置复测官方目录，Go 返回 37 个模型，Zen 返回 108 个模型。网关测量使用同一批双 CLI，以下数值均为三轮中位数。

| 负载与进程 | 工作集 MiB | 私有字节 MiB |
| --- | ---: | ---: |
| Catalog · Crabforge | 7.36 | 1.29 |
| Catalog · Magpie | 57.88 | 91.58 |
| Chat · Crabforge | 8.30 | 1.42 |
| Chat · Magpie | 68.29 | 97.29 |

`carbforge-cli.exe` 的 SHA-256 为 `FD8BF7152E333BB8C607F32EC2BC8FA196811280651028A2BB85231A32B01B19`，`crabforge-cli.exe` 为 `2254A41FE18E0783EFD4B1D9329C649284E68EF8C3BB8A8A2648B3711EA07260`。

同批独立 CLI 与桌面集成通过成员停用、重新启用、手动覆盖、外部切换测试。独立进程冷启动恢复所选成员与界面语言。Windows Release 已构建，七个图标、字体和许可证资源的内容分别与源文件一致，九语译文使用的 720 种非空白字符在随包字体中均有字形。

| Windows 桌面采样对象 | 三轮工作集 MiB | 中位数 MiB | 三轮私有字节 MiB | 中位数 MiB |
| --- | --- | ---: | --- | ---: |
| Crabforge 空闲窗口 | 150.34、148.77、177.14 | 150.34 | 258.62、251.99、296.28 | 258.62 |
| Magpie 现有主进程 | 190.65、194.08、188.35 | 190.65 | 244.32、247.75、244.93 | 244.93 |
| Magpie 主进程及 WebView2（8 个进程） | 904.18、905.51、868.13 | 904.18 | 1472.08、1479.33、1467.77 | 1472.08 |

| 桌面采样产物 | SHA-256 |
| --- | --- |
| crabforge.exe | `D0E1A6AA74767A58C0174ECE292779B73E5058CA49802BAA371B3FD2AFD9A81D` |
| crabforge_bridge.dll | `23BFEDAD9C7A423FD61A03FC50DDAC8930E9526D805065C255E6176B991B3923` |
| data/app.so | `EEAA664086C3630DA173184E3DE103B705B5EE6DC72F2164CA2973CA239FBBC6` |

### MiMo Code 版网关复测

此版包含 `group show` 与 MiMo Code 启动器，构建时间早于 DevEco 参数修正及 Copilot 启动器。`carbforge-cli.exe` 的 SHA-256 为 `ABB10263500D9B23C1912298C9F3A90B9BEE37CD30C2174D198B463C71BD5584`，`crabforge-cli.exe` 为 `3813D098296DD7275E9C83A4BA5ABD1DBC479D8EDA2A2C3932180A40E713742B`。

| 负载与进程 | 三轮工作集中位数 MiB | 三轮私有字节中位数 MiB |
| --- | ---: | ---: |
| Catalog · Crabforge | 7.38 | 1.30 |
| Catalog · Magpie | 57.28 | 90.64 |
| Chat · Crabforge | 8.30 | 1.44 |
| Chat · Magpie | 69.84 | 98.33 |

### 成员目录开关版网关复测

双 CLI 的 `--version` 均返回 0.1.0，通过 `group set daily local/m --listed=no` 保存后，`group show daily` 均读回 `listed members` 为 `no`。OpenAI、Gemini、Ollama 三个目录和 Pi 格式客户端通过隐藏模型专项，直接请求隐藏模型及其组均有回复。

| 负载与进程 | 三轮工作集中位数 MiB | 三轮私有字节中位数 MiB |
| --- | ---: | ---: |
| Catalog · Crabforge | 7.42 | 1.29 |
| Catalog · Magpie | 56.95 | 90.79 |
| Chat · Crabforge | 8.29 | 1.39 |
| Chat · Magpie | 69.44 | 98.66 |

`carbforge-cli.exe` SHA-256 为 `11F5BE7839977B3F83DD3EA20E3954F5B2C9C32B9DD5C02B56CB2FDBE9EE2C75`，`crabforge-cli.exe` 为 `062777807B08B01E7F07233DAD78FE897A204EA95D026DCB65E6B2093FBF1CC1`。此轮采样对应加入整组启停之前的产物。

### 整组启停与 Oh My Pi 版

双 Release CLI 均读回 `state off` 和 `listed members no`，并拒绝使用已停用的组启动 Agent。Windows 桌面操作与独立进程冷启动通过。整组启停专项覆盖旧地址 404、恢复组、复制和手动父组。官方 Oh My Pi 19.2.1 经 Bun 1.4.5 在离线模式下收到本地上游的回复，目录保留个人供应商并过滤停用组。

| 负载与进程 | 三轮工作集 MiB | 中位数 MiB | 三轮私有字节 MiB | 中位数 MiB |
| --- | --- | ---: | --- | ---: |
| Catalog · Crabforge | 7.38、7.40、7.39 | 7.39 | 1.25、1.29、1.27 | 1.27 |
| Catalog · Magpie | 58.68、58.34、58.96 | 58.68 | 91.94、92.06、92.55 | 92.06 |
| Chat · Crabforge | 8.31、8.33、8.34 | 8.33 | 1.39、1.42、1.43 | 1.42 |
| Chat · Magpie | 66.88、68.42、84.69 | 68.42 | 94.90、96.86、113.30 | 96.86 |

本轮 `carbforge-cli.exe` SHA-256 为 `E3982DA8D249639B9411D53931083A43C3257F3B8DE57EDFD742FCDC343D89E1`，`crabforge-cli.exe` 为 `37BC3DA3577965871FE00AADA27A5C42D9D94F7AD0885AF2B88BF70875992BBB`。原始 CSV 随双 CLI 快照保存在 `%LOCALAPPDATA%\Temp\opencode\crabforge-group-switch-630e539a-db40-4011-86f2-93b23498b92f`。

该版 Windows Release 的七个随包资源和十张 EXE 图标图像已逐项核对，五个字体覆盖九种语言译文使用的 722 个非空白字符。桌面三轮中位数如下，Magpie 采样使用同一个已打开的窗口。

| 桌面进程 | 工作集 MiB | 私有字节 MiB |
| --- | ---: | ---: |
| Crabforge 空闲窗口 | 149.19 | 259.36 |
| Magpie 现有主进程 | 158.21 | 244.10 |
| Magpie 主进程及 WebView2（8 个进程） | 213.46 | 1402.43 |

| 桌面产物 | SHA-256 |
| --- | --- |
| crabforge.exe | `D0E1A6AA74767A58C0174ECE292779B73E5058CA49802BAA371B3FD2AFD9A81D` |
| crabforge_bridge.dll | `4C2AFC22AA80978681E1F192FB7066278D0514F729DF56032CBA8371264A0BD2` |
| data/app.so | `A0B9BB1B5759980929B76C6985170EAC412FE633A33878D86774EAF092B0F8DE` |

### 会话标识

官方 Pi 1.6.0 和 Oh My Pi 19.2.1 的本地对话均收到回复，路由记录读到了客户端发来的原生会话 ID。专项检查了 Magpie 请求头和 Crabforge 请求头、128 字节上限、请求级 ID 与会话 ID 的区别。Ollama 流式回退的两条尝试保留相同会话 ID，普通上游未收到网关会话头。跨进程两级网关及真实模型在串行复测中均通过。长期会话存储和重启后的会话浏览仍待移植。

Windows 桌面集成通过会话响应头、路由页会话译文、停用组和父组选择检查，随后用同一配置在独立进程中通过冷启动测试。会话版双 CLI 的网关中位数来自每种负载三轮隔离采样。

| 负载 | Crabforge 工作集 / 私有字节 MiB | Magpie 工作集 / 私有字节 MiB |
| --- | ---: | ---: |
| Catalog | 7.41 / 1.27 | 58.88 / 92.15 |
| Chat | 8.37 / 1.40 | 68.47 / 96.31 |

`carbforge-cli.exe` SHA-256 为 `8E4DF4FAC4F9CC66F8B7BD42C2965FA5A70E546F658DF12FE42107B74C4BC200`，`crabforge-cli.exe` 为 `8432832C99FA507E1FC28ED007E02B8D944EE4C82E26A6635412799B5F9FAA07`。双 CLI 和原始数据位于 `%LOCALAPPDATA%\Temp\opencode\crabforge-sessions-3f12147a-edee-49d0-81b0-3c061739c12f`。该快照的桌面 Release 构建读到了开发中的新译文，未完成归档。

### Oh My Pi 原生协议

在固定源码快照 `crabforge-native-src-0a247f95-a4d1-4015-adb3-c70dcbdf3413` 上执行含忽略项的全工作区串行回归，共 64 项通过，文档测试也已通过。Oh My Pi 19.2.1 的同一个验收用例依次将 `local/m` 交给四个独立的本地上游，分别使用 Chat Completions、Responses、Anthropic Messages 和 Gemini 并收到回复。网关模型目录和单模型查询另核对了原生端点、首选端点、组别名及停用成员。这批源码早于命名 profile 和浏览器来源设置。

### 浏览器来源与 Oh My Pi 命名 profile

冻结源码 `crabforge-cors-profile-src-5a018d3e-62c5-46b5-be93-7eee4b5c6781` 的 66 项 Rust 测试全部通过，包含所有忽略项，三个 crate 的文档测试也通过。真实 Chrome 在获准来源读到了两个会话头，用错误密钥得到 401，未获准来源无法读取响应。Oh My Pi 19.2.1 在四种协议的对话中验证了显式 profile、`OMP_PROFILE`、`PI_PROFILE` 的优先级和空环境变量的行为。四次运行均使用隔离配置和本地上游。

同批 Windows 桌面集成验证了来源编辑、保存时移除 HTTPS 默认端口和带 Origin 的网关请求，随后在独立进程中通过冷启动测试。网关三轮中位数与原始 CSV 对应 `%LOCALAPPDATA%\Temp\opencode\crabforge-browser-profiles-c6c6935c-e3d3-4036-98fd-630ff35ddce8` 中的双 CLI。

| 负载 | Crabforge 工作集 / 私有字节 MiB | Magpie 工作集 / 私有字节 MiB |
| --- | ---: | ---: |
| Catalog | 7.45 / 1.32 | 57.69 / 91.78 |
| Chat | 8.43 / 1.46 | 68.34 / 97.58 |

`carbforge-cli.exe` SHA-256 为 `ABC10B9B2F75A93CB957E0D05AFF633278E36AFB9A220D39D21C5158798433E3`，`crabforge-cli.exe` 为 `9D4BCB661C4492E2E6B994AE55DAC22C18D250179BAF44CF98BA6FA0A24F49DF`。

后续 profile 发现改动已通过原生参数边界单测，以及两个 Oh My Pi 真实客户端专项。后者覆盖自定义配置根目录、profile 列表及活动标记、透传 `--profile` 覆盖启动器选项、空值与 `default` 回到隔离目录、非法名称报错和四种协议的对话。该改动晚于上述 66 项快照。

此浏览器来源版 Windows Release 的 7 个随包资源逐一通过 SHA-256 比较，EXE 中的 10 张图标与 ICO 内容一致，五个字体覆盖该版九语译文使用的 732 个非空白字符。三轮桌面采样的工作集 / 私有字节中位数为 Crabforge `149.23 / 255.93` MiB、已打开的 Magpie 主进程 `170.39 / 242.28` MiB、含 WebView2 的 Magpie 进程树 `1006.33 / 1662.98` MiB。Magpie 窗口保留了原有会话，这组桌面数据只反映采样时的状态。

| 桌面产物 | SHA-256 |
| --- | --- |
| crabforge.exe | `D0E1A6AA74767A58C0174ECE292779B73E5058CA49802BAA371B3FD2AFD9A81D` |
| crabforge_bridge.dll | `9FCE55DAE8A83E511DCB4C29FA558C7CFEEB8C5C9C441BE0A8E10359E2DE25E5` |
| data/app.so | `EA3D502B93B0282809D211398656B0EF2F588EED5399255CD96B86202A3AC3BA` |

### 基础路由组分享版（`cf-share-301f0a9281`）

Windows 集成测试通过 Flutter 复制按钮、模拟平台剪贴板、`EditableTextState.pasteText` 和 Rust 桥接完成分享与导入。测试读取磁盘配置，确认 `renamed-2` 的成员和权重。独立进程用同一配置冷启动后再次核对成功。系统剪贴板与动态规则组属于后续验收范围。此版 7 个字体、许可证及图片资源的 SHA-256 与源码一致。

第二轮完整 Rust 回归通过 68/69 项，21 项 Agent 测试全部通过。唯一失败的测试在等待 Magpie 启动时超时。此后单跑两项 Magpie 联调，真实模型返回 6 个 token 和 `The sky is cloudy.`，另一项本地夹具在等待 Magpie 启动时超时。

| 负载 | Crabforge 工作集 / 私有字节 MiB | Magpie 工作集 / 私有字节 MiB |
| --- | ---: | ---: |
| Catalog，三轮中位数 | 7.48 / 1.31 | 58.46 / 92.04 |
| Chat，三轮中位数 | 8.47 / 1.41 | 70.67 / 99.18 |
| 桌面，三轮中位数 | 148.17 / 255.22 | 主进程 183.59 / 253.26，含 WebView2 925.80 / 1639.41 |

网关每轮使用隔离配置并交替启动。Chat 负载使用同一个本地上游模型，Crabforge 目录同时提供裸模型名和完整名称。桌面每轮重开 Crabforge，Magpie 复用已有窗口。逐轮数据在快照的 `release/catalog.csv`、`chat.csv`、`desktop.csv`。

| 发布文件 | SHA-256 |
| --- | --- |
| carbforge-cli.exe | `E409F1502CF248A62F8D4EF824428E2F047AC9DFA88781DB84CEE9E1CC8E9760` |
| crabforge-cli.exe | `FCB3931A67B5E873A0AAC3D19C5F82B3731DAB7E97F904C67442D86D10BF8FB0` |
| desktop/crabforge.exe | `21689707978C0F074CA19FC462198D75280AC8B2F8B30DBEFDE573BC9C344B8F` |
| desktop/crabforge_bridge.dll | `D11A864AB81F5301726D10A32C66806A194BC060E5C52947082AA524AFA88DB8` |
| desktop/data/app.so | `9C753A624B801CA0CE41E9841C5DA05798BF35628768E40AD358E658D86D375F` |
| magpie.exe | `340C0F3349C684143926AE9CEDC7A803662428849A10283976B6E07E40A6E0BC` |

### 动态路由组（`cf-pattern-01544a88b8`）

完整 Rust 回归通过 70/71 项。动态组匹配和分享、免费模型目录同步、两项 Magpie 联调、Chrome、Kubo 和本地模型生成均通过。OmO 在这次回归和随后一次单独运行中均超时。冻结后工作区的真实客户端专项中，OmO 在四种上游协议下均收到回复。两个版本及运行环境不同，超时原因尚未确定。

以下为该快照双 CLI 的三轮中位数。逐轮数据位于 `%LOCALAPPDATA%\Temp\opencode\cf-pattern-01544a88b8\release` 的 `catalog-r2.csv` 和 `chat-r2.csv`。首次采样在第三轮等待 Magpie 启动时超时。将就绪等待上限调整到 90 秒后，从第一轮重新采集，两项负载均完成。

| 负载 | Crabforge 工作集 / 私有字节 MiB | Magpie 工作集 / 私有字节 MiB |
| --- | ---: | ---: |
| Catalog | 7.48 / 1.29 | 61.26 / 94.96 |
| Chat | 8.49 / 1.43 | 72.68 / 101.06 |

| 采样文件 | SHA-256 |
| --- | --- |
| carbforge-cli.exe | `D873E8DE7D1EBE221907DC728DBEA6FCA4DCF8B6D7057EBBCBBC644C96A27E45` |
| crabforge-cli.exe | `CB091E5A88803E70F1730E3335AA7D49483B6787058EEE6E8BF6484C293664A3` |
| magpie.exe | `340C0F3349C684143926AE9CEDC7A803662428849A10283976B6E07E40A6E0BC` |

最初的 Windows 集成测试在保存无匹配规则的组时触发对话框返回类型断言。修复后的工作区通过 Windows 集成及同配置独立进程冷启动。重启后 `renamed-2` 保有显式成员和权重，`renamed-copy` 保有动态规则，`account-model` 保有模型专属的原生端点和首选协议。

### 两级中转与 Pi 模型协议（`cf-relay-ff0529bdd8`）

全工作区串行回归含忽略项，共通过 71/74 项。Pi 的模型协议覆盖、四协议对话与流式工具回合、两级中转的 `previous_response_id` 透传、动态路由组、真实 Chrome、Kubo 以及经 Magpie 访问本地模型全部通过。三个失败项分别为 Copilot CLI 在 90 秒内未退出，Crush 并发请求中的 `local/m` 只输出换行，OmO 在 Chat 对话中超时。

| 负载 | Crabforge 工作集 / 私有字节 MiB（三轮中位数） | Magpie 工作集 / 私有字节 MiB（三轮中位数） |
| --- | ---: | ---: |
| Catalog | 7.52 / 1.29 | 57.14 / 90.34 |
| Chat | 8.51 / 1.46 | 69.30 / 97.75 |
| 桌面 | 148.40 / 256.21 | 主进程 172.17 / 239.15，含 WebView2 的 8 个进程 966.68 / 1733.05 |

采样和发布目录为 `%LOCALAPPDATA%\Temp\opencode\cf-relay-ff0529bdd8\release`。`carbforge-cli.exe` 的 SHA-256 为 `A85407EF5AE95B9E0265A71261050D5D2D194604D9E0648166AE71D6CA387D63`，`crabforge-cli.exe` 为 `0F92B4B991F861AB32A1C586F0CBE1AB39A26EB7B8236D91C82304D8971D7685`。

Flutter 分析和 Windows Release 构建通过。发布包的图像、五份字体及字体许可证共七个文件，逐个与快照源文件比较，SHA-256 均相同。桌面采样沿用已打开的 Magpie 窗口，逐轮数据见 `desktop.csv`。

| 桌面发布文件 | SHA-256 |
| --- | --- |
| `desktop/crabforge.exe` | `611DC16A6C009910DA3D4B96B9A34A78B39D4D621AFE3B7108AD26A2F1A48A74` |
| `desktop/crabforge_bridge.dll` | `374657A85836627DA3A67A637329FED7BD452E65762B30BF3F94CBD314553ED1` |
| `desktop/data/app.so` | `97792CBAAB72E0D02912EFDEF2E7E2AF873971217757E66F4872B9EDCB190AD0` |

同一冻结测试程序单独复测 Copilot CLI 和 Crush 均通过。另用相同 PATH、隔离配置、同一 Chat 回复分别运行两个发布版的 OmO 5.1.83。动态组版在 Bun 下于网关启动后约 90.6 秒才向上游发出请求，上游在 22 毫秒内完成回复，客户端在期限内没有输出。该版在 Node 下 85.7 秒退出。两级中转版在 Bun 和 Node 下分别于 48.4 秒和 71.1 秒输出回复，均正常退出。数据见 `omo-runtime.ndjson`。这四次运行只定位了该次超时消耗时间的阶段。

### Oh My Pi 顾问设置（中转快照之后）

两项 Rust 专项验证了空文件、纯注释配置、已有 `config.yaml`、个人设置和注释保留、旧审阅时机保留、并发编辑冲突及无效输入原样保盘。在命名 profile 下，Oh My Pi 19.2.1 的 `config list --scope global --json` 三次读回 Crabforge CLI 写入的设置，确认启用、关闭和恢复默认三个阶段均生效，等待阈值 `"1"` 为字符串。

Windows 实际界面选择 `work`、保存、重开并检测并发写入的专项通过。工作区两项中转专项也已通过，覆盖首选原生协议没有配置入口时的实际 Responses 调用，以及目录刷新后删除旧模型协议记录。

在新增思考级别后，Windows 界面保存了命名 profile 的会话 `auto` 与 `slow` 角色的 `local/qwen3:8b:high`。独立进程重新启动后，界面读回这两项，以及 `agent-end` 审阅时机和 `blocker` 恢复级别。Oh My Pi 19.2.1 的真实 CLI 另读回 `auto`、角色 `off` 及清除两项覆盖后的配置。原来的双模型回退列表和 `:8b` 标签均保留。该工作区阶段的 Flutter 分析和全工作区 Clippy 通过。

### Oh My Pi 顾问、思考级别与目录同步（`cf-omp-50cc8fdfb2`）

冻结版含忽略项的全工作区串行 Rust 回归 **82/82 通过**，其中 25 项 Agent 用例全部通过。Oh My Pi 19.2.1 的 `config list` 在刷新模型目录后两次读回 `enabledModels`，验证模型消失后移除条目、恢复供应后带回原有 `:high`。Copilot、并发 Crush 及四协议 OmO 在本轮均成功。OmO 冷启动测试的时限为 150 秒。冻结版 Flutter 静态分析通过。工作区 Windows 测试在等待按钮重新可点击后完成了命名 profile 的目录同步与独立冷启动。

双 CLI、桌面包及三份原始 CSV 位于 `%LOCALAPPDATA%\Temp\opencode\cf-omp-50cc8fdfb2\release`。随包的 7 个资源与源文件字节一致，五个字体覆盖九语译文使用的 831 种非空白字符，EXE 中 10 张图标的图像内容与 ICO 一致。

| 三轮中位数 | Crabforge 工作集 / 私有字节 MiB | Magpie 工作集 / 私有字节 MiB |
| --- | ---: | ---: |
| 空目录网关 | 7.58 / 1.36 | 57.66 / 91.43 |
| 本地 Chat 网关 | 8.57 / 1.47 | 67.50 / 96.58 |
| 桌面 | 149.05 / 259.46 | 主进程 177.71 / 250.44，含 WebView2 的 8 个进程 1051.63 / 1777.57 |

网关交替启动并使用隔离配置。Chat 的同一上游模型在 Crabforge 目录有裸名和供应商限定名。桌面每轮重启 Crabforge，Magpie 复用同一窗口，桌面数据只反映本次采样状态。

| 发布文件 | SHA-256 |
| --- | --- |
| `carbforge-cli.exe` | `44B6FD77DCD70F50E8CD69ED267269B15C925946377392945A64E2F2EFEFED04` |
| `crabforge-cli.exe` | `5CA4EB6AA13FB6AE36F9A4C35C6356B19EC3EA91D2B2C8A8A77AFE8585AA7263` |
| `desktop/crabforge.exe` | `279FD4A6E2F36F8862AFFA422E1664D8C8C093DF85EF6098765C0D3D1D10FB11` |
| `desktop/crabforge_bridge.dll` | `382A69BB9553F351CC1D7CF904AADB375D2215D826B105A30CA33C694AD35AE5` |
| `desktop/data/app.so` | `4FC1728D7EEFB86DD6D427C1CD4FBC236241577C8253DFF12BD9024B696EF51E` |

### Oh My Pi 角色和原生目录（`cf-roles-f876457a8a`）

冻结源码清单的 142 个文件逐一通过 SHA-256 校验。含忽略项的串行 Rust 回归 **87/87 通过**，其中 Agent 26/26。Oh My Pi 19.2.1 在命名 profile 中按保存的默认角色获得本地上游回复，重置后读回原 YAML 回退列表。测试还检查了已安装客户端的 `openai-codex` 候选和新旧模型目录格式。冻结版 Flutter 静态分析通过。Windows x64 Release 的 7 个资源与源文件逐字节一致，五个打包字体覆盖九语译文的 840 种非空白字符，EXE 中的 10 张图标与 ICO 一致。

| 三轮中位数，工作集 / 私有字节（MiB） | Crabforge | Magpie |
| --- | ---: | ---: |
| 空目录网关 | 7.64 / 1.38 | 60.50 / 95.24 |
| 本地 Chat 网关 | 8.61 / 1.48 | 70.77 / 100.65 |
| 桌面 | 147.14 / 255.41 | 主进程 174.65 / 241.99，含 WebView2 的 8 个进程 1049.15 / 1747.36 |

网关使用隔离配置、相同请求和交替启动顺序。桌面每轮重启 Crabforge，Magpie 复用已有窗口。双 CLI、桌面包、哈希清单和三份采样 CSV 位于 `%LOCALAPPDATA%\Temp\opencode\cf-roles-f876457a8a\release`。

| 发布文件 | SHA-256 |
| --- | --- |
| `carbforge-cli.exe` | `BD5990A947E89BA5A29A62E6050FA6F8CAF4A97D84C3A917F6FAD85C00C078E1` |
| `crabforge-cli.exe` | `EAD1CE343EF5DA758322CD9FCF1E556AE0B33E08CCDCA169CB07BB716C2F9F88` |
| `desktop/crabforge.exe` | `2E60413D39ABE6BA542D425E3A40BBA32F81D3835E03B07B9A9AA367ED117BBE` |
| `desktop/crabforge_bridge.dll` | `8451920A5DDA337AAB84CCF39BF53CA61E20B3ED17A1D6E0878950C2939E0989` |
| `desktop/data/app.so` | `59A6F5871BF610E2EB2741DB70AA975E7F3078CB7664D77C0BF5B8994734D871` |
