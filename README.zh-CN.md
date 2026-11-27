# Crabforge

[English](README.md) · [操作手册](CRABFORGE.md) · [兼容性与验收记录](docs/crabforge-validation.md)

Crabforge 在本机提供带鉴权的模型网关。桌面控制台和命令行共用 Rust 核心，可以为不同模型配置供应商、协议和回退顺序。Agent 使用稳定的本地地址和网关密钥，供应商密钥由网关进程从环境变量读取。

## 第一次运行

1. 在启动 Crabforge 的环境中设置供应商的 API 密钥，例如 `DEEPSEEK_API_KEY`。打开桌面应用的「供应商」，选择预设，填写环境变量名，读取模型目录或手动填写模型 ID，然后保存。
2. 在「网关」启动服务，复制 API 地址和网关密钥。默认的 OpenAI 兼容地址为 `http://127.0.0.1:3425/v1`。从「模型」复制一个完整 ID，例如 `deepseek/deepseek-chat`，填入客户端。
3. 需要多供应商回退时，在「路由组」添加成员并选择策略。客户端可以一直请求 `group/work`。在「路由」查看每次尝试实际使用的供应商和 HTTP 状态。

两个 CLI 可执行文件 `carbforge-cli` 和 `crabforge-cli` 提供相同命令。以下示例使用 PowerShell，模型名应替换为已配置的 ID。`serve` 占用当前终端，启动 Agent 时另开终端。

```powershell
carbforge-cli catalog
carbforge-cli models deepseek
carbforge-cli group set work primary/model backup/model
carbforge-cli serve
```

```powershell
carbforge-cli agent claude group/work
```

Windows 默认将配置、网关密钥和 P2P 身份保存在 `%LOCALAPPDATA%\com.blue.crabforge`。设置绝对路径 `CRABFORGE_HOME` 或传入 `--data-dir` 可以使用独立配置。客户端访问网关时使用 `carbforge-cli key` 输出的密钥。

在「客户端连接」中选定 Agent，打开「项目指令」并选择项目文件夹。Crabforge 在 Agent 的项目文件里维护与 Magpie 兼容的 Markdown 区块。「恢复上一版本」会先检查当前文件是否与备份对应，再恢复文件。

对于 Gemini CLI、Qwen Code、Qoder 和 Qoder CN，Crabforge 依次检查项目设置、适用的 Crabforge profile 和 Agent 用户设置中的 `context.fileName`。配置为列表时取第一个非空名称。文件必须位于项目目录内。

```powershell
carbforge-cli project-instructions D:\Projects\app codex
carbforge-cli project-instructions D:\Projects\app codex --from instructions.md
carbforge-cli project-instructions D:\Projects\app codex --restore
```

打开「客户端连接 → 记忆」可以添加、编辑、置顶和归档 Markdown 记忆。桌面支持文本搜索，CLI 还可按范围、类型和标签筛选。每条记忆以带 YAML 元数据的 Markdown 文件保存在 Crabforge 数据目录的 `memory/` 下。迁移 Magpie 的 Markdown 记忆时，将各条 `.md` 文件复制到该目录。

```powershell
carbforge-cli memory add preference.md --kind preference --tag workflows
carbforge-cli memory search --kind preference --status active
carbforge-cli memory edit <id> --status archived
```

通过记忆「范围」字段的文件夹按钮，将记忆关联到项目。Crabforge 依据 Git 远端识别同一仓库的克隆和工作树。没有网络远端时使用本地路径。在「项目指令 → 项目记忆」中可预览有效的项目记忆，并将摘要发布到选定 Agent 的指令文件。能访问文件的人都能读取摘要。归档记忆后再次发布即可从摘要移除该条记忆。「恢复上一版本」可以撤销上次发布。

```powershell
carbforge-cli memory project-key D:\Projects\app
carbforge-cli memory digest D:\Projects\app
carbforge-cli memory publish D:\Projects\app codex
```

要让 Agent 搜索和保存记忆，在客户端添加本地 stdio MCP 服务，命令使用 `carbforge-cli.exe` 的完整路径，参数为 `mcp memory --agent <Agent ID> --project <文件夹>`。也可将 `--project <文件夹>` 换成 `--project-key <项目键>`。使用自定义数据目录时，为客户端设置相同的 `CRABFORGE_HOME`。`memory_search` 搜索可用范围内的有效记忆，`memory_save` 创建记录或更新 Agent 写入的记忆，`memory_forget` 归档可见记录，`memory_context` 显示适用范围。直接运行 `mcp memory` 时仅开放全局记忆。

打开 **客户端连接 → 本地会话**可浏览最近最多 50 个 Claude Code 和 Codex 会话、按 Agent 筛选并复制会话 ID。运行 `carbforge-cli sessions --limit 50` 可输出 JSON，使用 `--agent claude` 或 `--agent codex` 筛选。Crabforge 读取原生 JSONL 和 Codex 的 `.jsonl.zst`，显示会话 ID、提示预览、时间、工作目录和所属配置目录。主要配置目录来自 `CLAUDE_CONFIG_DIR` 和 `CODEX_HOME`，默认为用户主目录的 `.claude` 和 `.codex`。Crabforge 也会在主要配置的同级和用户主目录内查找含会话的 Claude 或 Codex 命名目录，并检查下一层配置子目录。

在会话详情中选择**提炼记忆**，检查用户消息并选择已配置的模型路由。点击**发送给模型**会通过该路由提交预览。Crabforge 核对引用原文后，保存最多五条候选记忆。在**记忆**中审核，将状态设为**有效**后即可纳入对应范围的摘要和 MCP 搜索。

CLI 中先运行 `carbforge-cli memory distill claude <会话-ID>` 获取消息和预览修订，再运行 `carbforge-cli memory distill claude <会话-ID> --model <路由-ID> --revision <修订>`。如果会话位于其他已发现的配置目录，两次命令均加上 `--profile <配置目录>`。Codex 会话使用 `codex`。运行 `carbforge-cli memory search --status candidate` 可查看待审核记忆。

在**客户端连接**选中 Claude Code 或 Codex，再打开**用户指令**，即可编辑受管理的 Markdown 或预览并发布**用户记忆**。摘要包含有效的全局记忆及所选 Agent 的记忆。Crabforge 将摘要写入 `CLAUDE.md` 或 Codex 用户目录的 `AGENTS.md`，存在 `AGENTS.override.md` 时优先使用该文件。客户端会在新会话中读取该文件。使用**恢复上一版本**可撤销 Crabforge 最近一次对该文件的编辑。

`carbforge-cli memory context --agent codex` 预览同一摘要。`carbforge-cli memory agent-instructions codex` 显示当前指令文件及修订。附加 `--publish` 发布，使用 `--preview <UTF-8 文件>` 要求摘要与已保存的预览一致，或使用 `--restore` 恢复。`--revision <修订>` 核对指令文件是否仍是已查看的版本。

在 **客户端连接** 的 **规则** 中保存 Markdown 指令，并分配给 Claude Code、Cline 或 Kiro CLI。Claude Code 和 Kiro CLI 支持可选的路径模式，Cline 规则适用于每次会话。规则库显示原生文件的位置和状态。编辑、改名或删除规则时，Crabforge 会更新自己管理且内容未变化的副本。既有文件和本地修改会保留并显示冲突。点击 **应用规则** 可重试当前 Agent 配置中尚未完成的更改。

```powershell
carbforge-cli rules save diagnostics --from diagnostics.md --agent claude --agent kiro-cli --path 'src/**/*.rs'
carbforge-cli rules list
```

使用 `--old <名称>` 编辑现有规则或改名，可加 `--revision <修订>` 核对 `rules list` 中读取的版本。`rules remove <名称>` 删除规则及内容未变化的托管副本。

在 **客户端连接** 的 **技能** 中导入本地文件夹，包括 `SKILL.md` 和附属文件。选择 Claude Code、Codex、Gemini CLI、OpenCode 或 Pi，可将副本发布到当前配置。选择 Gemini CLI 或 Pi 时，Crabforge 也会向 `carbforge-cli agent` 使用的配置发布副本。**重新导入文件** 会再次读取来源。**应用技能** 可重试待处理的更改。原生目录已有同名技能，或已发布的副本在本地被修改时，Crabforge 会保留目录并显示冲突。

```powershell
carbforge-cli skills import .\diagnostics --agent claude --agent codex
carbforge-cli skills list
carbforge-cli skills assign diagnostics --agent codex --revision '<列表返回的修订>'
```

使用 `skills import` 加 `--revision` 更新同名技能。`skills remove <名称>` 删除库中的技能及内容未变化的托管副本。

导入公开 GitHub 技能时，将 `SKILL.md` 或其文件夹的链接填入来源栏，也可传给 `skills import`。仓库链接会选择根目录下的技能或仓库内唯一的技能。仓库有多个技能时，请指定要导入的文件夹。**重新导入文件** 会重新获取保存的引用和目录。可以用提交链接固定来源。分支名含 `/` 时，可在仓库链接后加 `?ref=feature%2Fdocs#path/to/skill`。

**已安装的技能** 扫描原生及网关客户端配置中的兼容文件夹。选择 **交由 Crabforge 管理** 后，Crabforge 校验扫描时的内容哈希，将文件保存为库快照并在原址管理该文件夹。取消分配会移除内容未变的文件夹。CLI 对应 `skills discover` 和 `skills adopt <Agent> <名称> <路径> --content <discover 返回的 content>`。

其他 Git 托管服务可使用 HTTPS 或 SSH 克隆链接，也支持 GitLab 和 Gitea 文件夹链接、`user@host:path` 形式的 SSH 地址和指向本地仓库的绝对 `file://` 链接。可加 `?ref=feature%2Fdocs#path/to/skill` 选择引用和技能目录。这些来源需要 `PATH` 上的 Git，并使用 Git 的凭据助手或 SSH Agent。**检查更新** 将来源文件与库快照比较，显示文件未变、有新内容或缺少 `SKILL.md`。**重新导入文件** 应用更新。CLI 对应 `skills check <名称> --revision <list 返回的 revision>`。

**浏览来源技能** 列出本地集合文件夹或选定 Git 引用中的技能。已入库或已忽略的候选不会在下次扫描中出现。选择技能和客户端后确认导入。`skills catalog <文件夹或 Git URL>` 返回候选及其导入来源，`skills dismiss <catalog 返回的 source>` 可忽略候选。仍可用该来源直接导入。

在 **客户端连接 → 环境变量** 保存变量后，可分配给 Claude Code、Codex 或 Pi。Crabforge 写入 Claude 的 `env`、Codex 的 `shell_environment_policy.set`，并为 Pi 的原生和网关配置发布受管理的扩展。Pi 保留从 Shell 继承的值。Crabforge 保留客户端配置中的已有值和外部修改。变量值存于本地库和客户端配置，桌面编辑器默认隐藏值。

在 **客户端连接 → 钩子** 保存生命周期命令，可按事件分配给 Claude Code、Codex、Gemini CLI 或 Cursor。Gemini 同时使用原生和 Crabforge 配置。库可原址接管兼容的命令，并显示冲突及外部修改。客户端按自身的钩子和信任设置运行命令。`carbforge-cli hooks list` 可查看修订和已有条目。`hooks save` 从标准输入读取 UTF-8 JSON 对象，字段为 `name`、`event`、`command` 和 `agents`，可选字段为 `description`、`matcher` 和以秒为单位的 `timeout`。通用事件为 `turn-end`、`after-tool`、`before-tool`、`prompt` 和 `session-start`。使用 `claude:SessionStart` 等原生事件可指定客户端专用匹配条件。`hooks adopt <key> <name>` 接管未改动的原生条目，`hooks remove <name> --revision <revision>` 撤销受管理的副本，`hooks sync` 重试发布。

在 **客户端连接 → 子代理** 向 Claude Code、Codex、OpenCode、Gemini CLI、Qwen Code、Factory Droid、Grok Build、Cursor 和 Copilot CLI 发布原生 Markdown 或 TOML。也可以在同一窗口选择项目文件夹，管理该项目的子代理。项目配置另支持 Antigravity。Claude Code 支持技能引用和推理级别，Claude Code 和 Factory Droid 支持 MCP 服务器名称，Codex 支持模型 ID 和推理级别。

打开项目后选择 **共享子代理**，把用户库条目分配给该项目的客户端。用户库内的修改与重命名会同步到已分配的项目。取消选中所有客户端可撤销分配。CLI 用 `subagents --project <文件夹> list` 获取 `shared` 条目和修订，再用 `subagents --project <文件夹> assign <名称> --revision <修订> --agent claude` 分配。重复 `--agent` 可选多个客户端，省略它可撤销。撤销会移除未修改的受管理文件，并保留外部修改。

在 **客户端连接 → 斜杠命令** 向 Claude Code、OpenCode、Pi、Gemini CLI、Qwen Code 和 Factory Droid 发布可复用提示词。`$ARGUMENTS` 表示全部参数。Claude Code、OpenCode 和 Pi 也可用 `$1`、`$2` 等引用单个参数。Crabforge 按客户端格式转换占位符，并报告无法发布的分配。Codex 可用技能保存可复用提示词。

`carbforge-cli subagents` 和 `carbforge-cli commands` 均支持 `list`、`save`、`remove` 和 `sync`。用 `subagents --project <文件夹>` 选择项目库。`save` 从标准输入读取 UTF-8 JSON，字段包括 `name`、`prompt` 和 `agents`。子代理还要求 `description`，可选字段为 `model`、`effort`、`skills` 和 `mcp`。将以下内容保存为 `check.json`，在 Windows 执行 `cmd.exe /d /c "carbforge-cli commands save < check.json"` 即可发布。

```json
{"name":"check","description":"检查文件","prompt":"检查 $ARGUMENTS 是否正确。","agents":["claude","opencode","pi"]}
```

用 `save --old <名称> --revision <修订>` 编辑或改名，用 `remove <名称> --revision <修订>` 撤销。`list` 显示修订和每个文件的发布状态。已有同名文件和外部修改会保留。`sync` 可重试发布。

选择 **查找已有文件** 可扫描支持的客户端目录并原址接管未改动的原生 Markdown 或 TOML 文件，包括 OpenCode 的旧版 `command/`、`agent/` 目录和 Qwen 的 TOML 命令。CLI 的 `discover` 返回客户端 ID、路径和修订，`adopt <客户端> <路径> --revision <修订>` 用于接管。接管保留原始字节，直至编辑资源。撤销分配可能移除对应的未修改文件。

在用户子代理库选择 **导入子代理**，可扫描 Markdown 文件、本地文件夹或 Git 仓库。选择要复制的定义，并按需为新条目分配客户端。再次扫描同一来源可查看更新，已在本地修改的描述和提示词以及已保存的客户端分配会保留。导入会复制名称、描述和提示词。客户端专用选项可在编辑器中配置。

运行 `carbforge-cli subagents probe-source <路径或地址>` 获取候选的相对路径 `path` 与修订 `revision`。将选中的文件作为 UTF-8 JSON 传给 `carbforge-cli subagents import-source` 的标准输入。请求包含 `source` 字符串、由 `{ "path": "…", "revision": "…" }` 对象组成的 `files` 数组，以及可选的 `agents` 客户端 ID 数组，用于分配新条目。

`environment save <名称> --agent <claude|codex|pi>` 从标准输入读取原始 UTF-8 值。重复 `--agent` 可分配多个客户端。编辑时传入 `environment list` 返回的修订，改名另加 `--rename-from <旧名称>`。`environment remove <名称> --revision <修订>` 移除变量，`environment sync` 重试同步，`environment list --show-values` 显示变量值。

在 **客户端连接** 选择支持的客户端，打开 **项目 MCP** 并选择项目文件夹。添加服务器时使用客户端的原生 JSON 配置，Codex 使用 TOML 表正文。编辑器显示将写入的文件，保存和删除前会核对文件修订。客户端打开项目时读取配置并启动服务器。

```powershell
carbforge-cli project-mcp list . gemini
carbforge-cli project-mcp save . gemini diagnostics --from .\server.json
carbforge-cli project-mcp remove . gemini diagnostics --revision '<列表返回的修订>'
```

在 CLI 中编辑已有条目时，向 `project-mcp save` 传入 `--replace --revision <修订>`。

项目加载后，打开 **MCP 库**，可保存复用的服务器并分配给所选客户端。库中的定义用 `transport` 指定 `stdio`、`http` 或 `sse`。stdio 使用 `command`、`args` 和 `env`，远端服务器使用 `url` 和 `headers`。Crabforge 按各客户端的原生格式写入分配项。同步会保留已有同名条目和外部编辑过的副本，并显示冲突。

```powershell
carbforge-cli project-mcp library save diagnostics --from .\shared-server.json
carbforge-cli project-mcp library assign diagnostics . gemini --revision '<library list 返回的修订>'
carbforge-cli project-mcp library sync
```

## 协议与路由

- 支持 OpenAI Chat Completions、OpenAI Responses、Anthropic Messages、Gemini 和 Ollama 客户端。目录、对话流、工具调用、媒体和鉴权的验证范围见验收记录。不同供应商及套餐开放的模型会变化。
- 路由组支持顺序回退、轮询、加权轮询、手选和嵌套。`carbforge-cli provider fallback primary group/work` 为直接请求 `primary` 的模型设置回退目标。`carbforge-cli provider set backup fallback-for-others=no` 限制其他普通供应商失败后将请求转给 `backup`。
- 桌面端支持简体中文、英语、法语、德语、西班牙语、葡萄牙语、印地语、日语和韩语。默认跟随系统语言。
- 六个控制台页面、路由组成员编辑器及 Oh My Pi 的 profile、协议、角色和模型选择窗口都可以切换列表或网格。下次启动会恢复上次的布局。编辑路由组时切换布局，已输入的权重会保留。

## 连接另一台网关

在客户端运行 `carbforge-cli peer-id`，将其输出加入服务端的受信 Peer ID。服务端在网络设置中填写监听地址，然后启动网关。对于直连，使用可达的 IPv4 或 IPv6 地址和 UDP 端口。通过 Circuit Relay v2 连接时，先准备双方均可访问的中继，并在服务端填写中继预约地址，例如 `/ip6/RELAY_IP/udp/PORT/quic-v1/p2p/RELAY_ID/p2p-circuit`。

在客户端将网关显示的完整 P2P 地址传给 `carbforge-cli tunnel <地址>`。同时拥有直连和中继地址时，使用 `carbforge-cli tunnel <直连地址> --relay-fallback <中继电路地址>`。两者须以同一网关 Peer ID 结尾。桌面的连接窗口也提供可选中继地址。本地入口默认为 `http://127.0.0.1:3426/v1`，使用远端网关密钥。同机已验证 Kubo 双向 QUIC、IPv6 中继回退，以及 DCUtR 升级为直连后在中继下线时发起新的鉴权请求。独立公网主机之间的直连及 NAT 打洞仍待验证。

## 构建 Windows x64 版本

`9aeb7e5` 解压版网关的空目录工作集和私有字节中位数为 7.53 和 1.42 MiB，Chat 负载为 8.85 和 1.86 MiB。三次末帧延后 700 毫秒，客户端首个完整 SSE 事件分别在 6、2、2 毫秒到达。解压版桌面三次冷启动的工作集和私有字节中位数为 154.39 和 227.91 MiB。冻结源码通过 113 项 Rust 测试和 28 组依赖审计，Windows x64 与 Cargo 全目标的常规、构建和测试依赖树均未启用 Serde、Hyper 或 Reqwest。[依赖审计](docs/validation/cf-windows-9aeb7e5/dependency-audit.csv) · [Windows 验收记录](docs/crabforge-validation.md)

安装 Rust 的 MSVC 工具链、Flutter 和 Visual Studio 的「使用 C++ 的桌面开发」工作负载。已记录的构建环境为 Rust 1.99.1、Flutter 3.47.9 和 Dart 3.13.7。

```powershell
cargo build --release --locked -p crabforge-cli
cd gui
flutter pub get
flutter build windows --release
```

双 CLI 位于 `target/release`。分发桌面应用时复制整个 `gui/build/windows/x64/runner/Release` 目录，其中包含 `crabforge.exe`、Flutter 运行库、Rust 桥接库及 `data`。项目仍在移植 Magpie 功能，各次冻结版本的实测结果和限制可在验收记录中查询。
