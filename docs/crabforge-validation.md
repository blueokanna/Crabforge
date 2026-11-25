# Crabforge 0.1.0 验收记录

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
