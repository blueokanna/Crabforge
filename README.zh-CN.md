# Crabforge

[English](README.md) · [操作手册](CRABFORGE.md) · [Windows 发布](docs/windows-release.md) · [兼容性与验收记录](docs/crabforge-validation.md)

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

可在路由组编辑器设置「首段内容超时」，或在 `group set` 中传入 `--first-content <秒>`。`0` 继承外层组或「网关 → 网络设置」中的默认值，`-1` 表示单独指定等待。若成员在时限内没有流式输出文本、推理或工具调用，且还有兼容的候选，网关会为本次尝试记录 HTTP 504 并切到下一成员。修改后重启网关。

OpenAI Chat Completions、Responses 和 Anthropic Messages 的 SSE 响应开启后，若首批字节未到或上一个事件已完整结束，网关在静默 15 秒时发送保活注释，单次静默最多保活五分钟。客户端如需在等待首次响应期间保活，可在 `stream: true` 的请求中加上 `X-Crabforge-Keepalive: true`。网关等待三秒后开启 SSE 响应，随后每五秒发送一条注释，期间继续执行回退和繁忙重试。发出首条注释后，最终错误通过 SSE 事件报告，载荷保留原 HTTP 状态及收到的 `Retry-After` 值。

路由组编辑器的「繁忙重试时限」或 `group set --patience <秒>` 适用于上游临时过载和限流。首次轮询候选结束后，网关会逐步延长等待时间，并在成员的时限内按 `Retry-After` 重试繁忙成员。填 `0` 继承，填 `-1` 关闭重试，或填 `1` 至 `240` 秒。网关默认值可在「网关 → 网络设置」或 `group default-patience <秒>` 中设置，此处 `0` 代表 60 秒。现有配置默认关闭重试。修改后重启网关。

针对直接发往供应商的请求，可在供应商编辑器设置「繁忙重试时限」，或执行 `carbforge-cli provider set primary patience=15`。填 `0` 使用 60 秒，填 `-1` 关闭。直接发往路由组的请求使用组的时限，嵌套组明确设置的时限优先。导入 Magpie 备份时会保留供应商明确设置的 `patience`。保存后重启网关。

要按条件转发生成请求，可在路由组编辑器修改「路由规则（JSON）」，也可执行 `carbforge-cli group rules work rules.json`。执行 `carbforge-cli group rules work` 可查看当前规则，保存 `[]` 可清空。`use` 和可选的 `more` 填写组内成员。命中的规则按顺序将成员提到前面，其余成员沿用组的路由策略。命中 `pause` 时跳过指定成员，手动选定优先。规则可随 Magpie 分享及加密路由组备份迁移。保存后重启网关。

规则可用 `useEffort` 将已请求推理的请求调整为 `low`、`medium`、`high`、`xhigh` 或 `max`。网关在原生转发 OpenAI Chat Completions、Responses、Anthropic Messages 或 Gemini 请求时，对每次尝试分别适配目标模型。Gemini 2.5 使用推理 token 预算，受支持的 Gemini 3 和 4 模型采用上游声明或官方文档列出的等级。目标模型能力未知或客户端已显式关闭推理时，沿用客户端设置。在供应商编辑器使用「发现模型」，或执行 `carbforge-cli models primary --sync` 读取上游声明的等级。可执行 `carbforge-cli model spec primary/model --reasoning low,medium,high` 设置自定义等级，用 `--reasoning auto` 清除。Magpie 导入保留组内的 `useEffort`，并通过 `--settings` 或 `--settings-only` 保留 `modelEfforts`。

网关在这些协议之间转译请求时，会将请求的推理设置适配到目标模型支持的参数。显式关闭推理需要目标模型支持关闭。请求推理摘要、显示思考过程或携带带签名的思考历史时，需要使用原生端点。

四种协议之间转译用户消息时，可以携带内联 Base64 图片。Chat、Responses 和 Messages 也可将 HTTP 图片 URL 直接传给所选供应商。转译到 Gemini 时请使用内联图片。Chat 与 Gemini 之间支持 WAV 和 MP3 音频输入，Chat 工具返回的录音也可转发到 Gemini。目标模型须支持相应的输入模态。供应商专用文件 ID 以及目标协议无法表达的图片细节设置需要兼容端点。

在路由组 JSON 的 `classifier` 同级加入 `"effort":"auto"`，由分类器为已请求推理的请求评估等级。网关可在一次调用中判定意图与等级，并在同一命名会话的工具续轮中复用结果、传给嵌套组。命中规则的 `useEffort` 优先。路由组分享和加密备份均保留该设置。

配置 `intent` 规则时，在同一编辑器或 CLI 命令中传入 JSON 对象。`classifier` 和 `classifierMore` 接受已配置的模型和路由组 ID，`classifyPrompt` 可选。

```json
{
  "classifier": "openai/gpt-4.1-mini",
  "classifierMore": ["group/decision-backup"],
  "rules": [{"intent": "审查改动", "use": "openai/gpt-4.1"}]
}
```

请求进入需要判断意图的路由组时，分类器会收到最近一条用户消息和上一轮对话的节选。遇到错误、拒绝、无效答案或超时，网关按顺序用同一分类提示询问候补。分类器回答 `0` 时继续使用普通路由规则。嵌套组共享同一截止时间，可在「网关 → 网络设置」或用 `carbforge-cli group classify-timeout SECONDS` 修改。`0` 表示默认的 15 秒，可显式设置 1 至 60 秒。

客户端送来会话 ID 和该轮用户消息历史时，并发请求可共用意图判断。工具续轮沿用轮初选定的规则和自动推理等级，每次仍检查暂停和手选。请求达到当前模型已知上下文容量的 95% 后，可切换到命中的规则，其模型须拥有更大容量且可容纳当前请求。在 `settings.json` 的供应商 `contexts` 对象中以模型 ID 设置容量，`*` 表示该供应商的默认容量。导入 Magpie 供应商备份时保留这些值。分类结果和路由决策分别在内存保留最多 256 条，每条最长 30 分钟。

```json
[
  {
    "use": "local/review",
    "where": [{"field": "message", "op": "starts", "value": "review"}]
  },
  {
    "use": "backup/general",
    "pause": true,
    "time": {"from": "22:00", "to": "06:00", "days": ["fri"], "zone": "Asia/Shanghai"}
  }
]
```

同一条规则的条件需全部满足。规则可检查估算 `tokens`、`images`、推理 `effort`、`agents`、`compact`、`folders`、`exceptFolders` 和 `time`。`where` 数组支持 `tokens` 和 `effort` 比较、`agent` 和 `subagent` 匹配，以及对 `message`、`system` 或 `last` 使用 `contains`、`excludes`、`starts` 或 `matches`。跨午夜的时间窗按所填 IANA 时区的起始日计算。

## 从 Magpie 导入供应商

使用 `magpie backup` 生成的 `.magpie-backup` 文件。CLI 会提示输入备份口令，输入内容不可见。先运行第一条命令预览，再加 `--apply` 保存。省略 `--provider` 可选择全部供应商，重复此选项可选择多个。

```powershell
carbforge-cli provider import-magpie .\magpie.magpie-backup --provider deepseek --key-env deepseek=DEEPSEEK_API_KEY
carbforge-cli provider import-magpie .\magpie.magpie-backup --provider deepseek --key-env deepseek=DEEPSEEK_API_KEY --apply
```

导入支持包含明确模型 ID 和标准 HTTP API 端点的供应商，并保留已支持的路由设置。供应商 ID 和旧 ID 必须唯一。启动 Crabforge 前，在指定环境变量中设置 API 密钥。结果中的 `other_sections` 列出备份的其他区段。备份超过 64 MiB 时，可用 `magpie backup --no-usage --no-library` 生成较小的文件。

配置供应商后，可以从同一备份预览和应用路由组。重复使用 `--group` 选取要导入的组，包括目标中尚不存在的子组。省略此选项会选择备份中保存的所有组。

```powershell
carbforge-cli group import-magpie .\magpie.magpie-backup --group work --group fallback
carbforge-cli group import-magpie .\magpie.magpie-backup --group work --group fallback --apply
```

导入保留组 ID、旧 ID、显示顺序和已支持的路由设置。遇到 ID 冲突、成员不可用或分类器等尚未支持的设置时，整批导入会被拒绝。通过 `magpie group share <id> --text` 导出的便携 JSON 也可用 `carbforge-cli group import <file>` 导入。

要同时恢复供应商和路由组，请使用顶层 `import-magpie` 命令。供应商的回退目标可以指向同一备份中的路由组。省略 `--provider` 或 `--group` 会选择对应区段中的全部条目。预览列出选中的 ID 和备份中的其他区段。加上 `--apply` 后，所有引用通过校验才会一次写入两类设置。

```powershell
carbforge-cli import-magpie .\magpie.magpie-backup --key-env deepseek=DEEPSEEK_API_KEY
carbforge-cli import-magpie .\magpie.magpie-backup --key-env deepseek=DEEPSEEK_API_KEY --apply
```

加 `--settings` 可同时导入网关端口、允许的浏览器来源、等价模型名、路由组的默认首段内容超时和繁忙重试时限。仅迁移这些设置时使用 `--settings-only`。预览在 `settings` 中显示导入值，并在 `settings.skipped_fields` 中列出跳过的字段名。重启网关后会在现有回环地址上监听导入的端口。备份中的来源列表替换现有列表，等价模型名合并到现有映射。同一模型的等价名称发生冲突时，整批导入会被拒绝。

```powershell
carbforge-cli import-magpie .\magpie.magpie-backup --settings-only
carbforge-cli import-magpie .\magpie.magpie-backup --settings-only --apply
```

## 管理客户端资源

在「客户端连接」中选定 Agent，打开「项目指令」并选择项目文件夹。Crabforge 在 Agent 的项目文件里维护与 Magpie 兼容的 Markdown 区块。「恢复上一版本」会先检查当前文件是否与备份对应，再恢复文件。

对于 Gemini CLI、Qwen Code、Qoder 和 Qoder CN，Crabforge 依次检查项目设置、适用的 Crabforge profile 和 Agent 用户设置中的 `context.fileName`。配置为列表时取第一个非空名称。文件必须位于项目目录内。

```powershell
carbforge-cli project-instructions D:\Projects\app codex
carbforge-cli project-instructions D:\Projects\app codex --from instructions.md
carbforge-cli project-instructions D:\Projects\app codex --restore
```

打开「客户端连接 → 记忆」可以添加、编辑、置顶和归档 Markdown 记忆。桌面支持文本搜索，CLI 还可按范围、类型和标签筛选。每条记忆以带 YAML 元数据的 Markdown 文件保存在 Crabforge 数据目录的 `memory/` 下。迁移 Magpie 的 Markdown 记忆时，将各条 `.md` 文件复制到该目录。

可用 `carbforge-cli memory import-magpie .\magpie.magpie-backup` 预览 Magpie 加密备份中的记忆。口令可在提示时输入，也可由重定向输入的首行提供。预览列出 ID、范围、状态，以及每条记忆将新建、文件相同还是 ID 冲突。加上 `--apply` 会创建 Markdown 文件，应用前须解决冲突。每个文件以原子方式写入，重复导入会跳过相同文件。导入保留记忆证据，发布仍通过现有审核及发布命令执行。预览按名称列出未支持的记忆元数据字段。

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

打开 **客户端连接 → 本地会话**可浏览最近最多 50 个 Claude Code、Codex、Gemini CLI、Grok Build、Oh My Pi 和 Pi 会话，并复制会话 ID。运行 `carbforge-cli sessions --limit 50` 可输出 JSON，使用 `--agent claude`、`--agent codex`、`--agent gemini`、`--agent grok`、`--agent omp` 或 `--agent pi` 筛选。Crabforge 读取原生 JSONL 和 Codex 的 `.jsonl.zst`，显示会话 ID、标题、时间、工作目录和所属配置目录。Claude Code 和 Codex 的主要配置目录来自 `CLAUDE_CONFIG_DIR` 和 `CODEX_HOME`，默认为用户主目录的 `.claude` 和 `.codex`。Crabforge 也会在主要配置的同级和用户主目录内查找含会话的 Claude 或 Codex 命名目录，并检查下一层配置子目录。

Gemini CLI 的 JSONL 和旧版 JSON 会话位于 `.gemini` 配置目录下的 `tmp/<项目>/chats`。Crabforge 从 `GEMINI_CLI_HOME` 或用户主目录查找 `.gemini`，也会查看 Crabforge 数据目录下的 `agents/gemini/.gemini`。若 Gemini 已在 `.project_root` 记录项目文件夹，详情中会显示该路径。记忆预览优先使用用户看到的提示文本，按撤回后的历史提取内容，并跳过已编辑的提示、生成的上下文及工具结果。Gemini 会话文件上限为 16 MiB。

Grok Build 的 `summary.json` 和 `updates.jsonl` 位于 `GROK_HOME/sessions` 或 `~/.grok/sessions`。记忆预览会拼接分段写入的用户提示，并跳过工具事件、助手回复、客户端内部操作和子代理会话。用于记忆预览的 Grok 更新日志上限为 16 MiB。

Crabforge 在 `~/.omp` 或 `PI_CONFIG_DIR` 对应目录下查找 Oh My Pi 的默认及命名配置，也会查看 Crabforge 数据目录中的 `agents/omp/sessions`。Linux 和 macOS 还会查看 `XDG_DATA_HOME` 下的 Oh My Pi 数据目录。使用与 Pi 共用的会话或配置目录覆盖项时，Crabforge 通过原生标题记录识别新版 Oh My Pi 日志。记忆预览沿保存的分支提取最近一次清空后的用户原话，单份日志上限为 16 MiB。

Pi 显示的配置目录为实际的会话存储目录。Crabforge 查看 `PI_CODING_AGENT_SESSION_DIR`、`PI_CODING_AGENT_DIR` 下的 `sessions` 目录（配置目录默认为 `~/.pi/agent`），以及 Crabforge 数据目录下的 `agents/pi/sessions`。用自定义 `--session-dir` 保存会话后，可设置 `PI_CODING_AGENT_SESSION_DIR` 让 Crabforge 找到它们。Pi 记忆预览从最后保存的分支提取用户原话，跳过受 Pi 上下文编辑影响的提示。记忆预览支持不超过 16 MiB 的 Pi 日志。

在会话详情中选择**提炼记忆**，检查用户消息并选择已配置的模型路由。点击**发送给模型**会通过该路由提交预览。Crabforge 核对引用原文后，保存最多五条候选记忆。在**记忆**中审核，将状态设为**有效**后即可纳入对应范围的摘要和 MCP 搜索。

CLI 中先运行 `carbforge-cli memory distill <Agent-ID> <会话-ID>` 获取消息和预览修订，再加上 `--model <路由-ID> --revision <修订>` 提交预览。如果会话位于其他已发现的配置目录，两次命令均加上 `--profile <配置目录>`。运行 `carbforge-cli memory search --status candidate` 可查看待审核记忆。

在**客户端连接**选中 Claude Code 或 Codex，再打开**用户指令**，即可编辑受管理的 Markdown 或预览并发布**用户记忆**。摘要包含有效的全局记忆及所选 Agent 的记忆。Crabforge 将摘要写入 `CLAUDE.md` 或 Codex 用户目录的 `AGENTS.md`，存在 `AGENTS.override.md` 时优先使用该文件。客户端会在新会话中读取该文件。使用**恢复上一版本**可撤销 Crabforge 最近一次对该文件的编辑。

`carbforge-cli import-magpie-instructions <备份> --agent claude` 预览 Magpie 加密备份中当前启用的共享指令集和该客户端的附加指令。Codex 使用 `--agent codex`，可用 `--set <ID>` 从预览中选择其他指令集。`--apply` 写入托管区块。已有托管文本时，沿用同一客户端和指令集并传入预览给出的 `--revision`。文件其余内容会保留。

`carbforge-cli memory context --agent codex` 预览同一摘要。`carbforge-cli memory agent-instructions codex` 显示当前指令文件及修订。附加 `--publish` 发布，使用 `--preview <UTF-8 文件>` 要求摘要与已保存的预览一致，或使用 `--restore` 恢复。`--revision <修订>` 核对指令文件是否仍是已查看的版本。

在 **客户端连接** 的 **规则** 中保存 Markdown 指令，并分配给 Claude Code、Cline 或 Kiro CLI。Claude Code 和 Kiro CLI 支持可选的路径模式，Cline 规则适用于每次会话。规则库显示原生文件的位置和状态。编辑、改名或删除规则时，Crabforge 会更新自己管理且内容未变化的副本。既有文件和本地修改会保留并显示冲突。点击 **应用规则** 可重试当前 Agent 配置中尚未完成的更改。

```powershell
carbforge-cli rules save diagnostics --from diagnostics.md --agent claude --agent kiro-cli --path 'src/**/*.rs'
carbforge-cli rules list
```

使用 `--old <名称>` 编辑现有规则或改名，可加 `--revision <修订>` 核对 `rules list` 中读取的版本。`rules remove <名称>` 删除规则及内容未变化的托管副本。

运行 `carbforge-cli rules import-magpie .\magpie.magpie-backup` 可预览 Magpie 加密备份中的规则。重复使用 `--name <名称>` 可选择多条规则。加上 `--apply` 会在一次写入中保存 Markdown、文件匹配模式和客户端分配。同名冲突会阻止整批写入，相同规则会跳过。用 `rules list` 审核，用 `rules sync` 发布。

在 **客户端连接** 的 **技能** 中导入本地文件夹，包括 `SKILL.md` 和附属文件。选择 Claude Code、Codex、Gemini CLI、OpenCode 或 Pi，可将副本发布到当前配置。选择 Gemini CLI 或 Pi 时，Crabforge 也会向 `carbforge-cli agent` 使用的配置发布副本。**重新导入文件** 会再次读取来源。**应用技能** 可重试待处理的更改。原生目录已有同名技能，或已发布的副本在本地被修改时，Crabforge 会保留目录并显示冲突。

```powershell
carbforge-cli skills import .\diagnostics --agent claude --agent codex
carbforge-cli skills list
carbforge-cli skills assign diagnostics --agent codex --revision '<列表返回的修订>'
```

使用 `skills import` 加 `--revision` 更新同名技能。`skills remove <名称>` 删除库中的技能及内容未变化的托管副本。

`carbforge-cli skills import-magpie <备份>` 预览 Magpie 加密备份中携带的完整技能快照。可重复传入 `--name` 选择技能。`--apply` 将原始文件字节及受支持的客户端分配保存到库中，再用 `skills sync` 发布。预览会列出跳过的元数据。备份缺少文件或要求远程获取时，需要提供完整的技能文件夹。

导入公开 GitHub 技能时，将 `SKILL.md` 或其文件夹的链接填入来源栏，也可传给 `skills import`。仓库链接会选择根目录下的技能或仓库内唯一的技能。仓库有多个技能时，请指定要导入的文件夹。**重新导入文件** 会重新获取保存的引用和目录。可以用提交链接固定来源。分支名含 `/` 时，可在仓库链接后加 `?ref=feature%2Fdocs#path/to/skill`。

**已安装的技能** 扫描原生及网关客户端配置中的兼容文件夹。选择 **交由 Crabforge 管理** 后，Crabforge 校验扫描时的内容哈希，将文件保存为库快照并在原址管理该文件夹。取消分配会移除内容未变的文件夹。CLI 对应 `skills discover` 和 `skills adopt <Agent> <名称> <路径> --content <discover 返回的 content>`。

其他 Git 托管服务可使用 HTTPS 或 SSH 克隆链接，也支持 GitLab 和 Gitea 文件夹链接、`user@host:path` 形式的 SSH 地址和指向本地仓库的绝对 `file://` 链接。可加 `?ref=feature%2Fdocs#path/to/skill` 选择引用和技能目录。这些来源需要 `PATH` 上的 Git，并使用 Git 的凭据助手或 SSH Agent。**检查更新** 将来源文件与库快照比较，显示文件未变、有新内容或缺少 `SKILL.md`。**重新导入文件** 应用更新。CLI 对应 `skills check <名称> --revision <list 返回的 revision>`。

**浏览来源技能** 列出本地集合文件夹或选定 Git 引用中的技能。已入库或已忽略的候选不会在下次扫描中出现。选择技能和客户端后确认导入。`skills catalog <文件夹或 Git URL>` 返回候选及其导入来源，`skills dismiss <catalog 返回的 source>` 可忽略候选。仍可用该来源直接导入。

在 **客户端连接 → 环境变量** 保存变量后，可分配给 Claude Code、Codex 或 Pi。Crabforge 写入 Claude 的 `env`、Codex 的 `shell_environment_policy.set`，并为 Pi 的原生和网关配置发布受管理的扩展。Pi 保留从 Shell 继承的值。Crabforge 保留客户端配置中的已有值和外部修改。变量值存于本地库和客户端配置，桌面编辑器默认隐藏值。

在 **客户端连接 → 钩子** 保存生命周期命令，可按事件分配给 Claude Code、Codex、Gemini CLI 或 Cursor。Gemini 同时使用原生和 Crabforge 配置。库可原址接管兼容的命令，并显示冲突及外部修改。客户端按自身的钩子和信任设置运行命令。`carbforge-cli hooks list` 可查看修订和已有条目。`hooks save` 从标准输入读取 UTF-8 JSON 对象，字段为 `name`、`event`、`command` 和 `agents`，可选字段为 `description`、`matcher` 和以秒为单位的 `timeout`。通用事件为 `turn-end`、`after-tool`、`before-tool`、`prompt` 和 `session-start`。使用 `claude:SessionStart` 等原生事件可指定客户端专用匹配条件。`hooks adopt <key> <name>` 接管未改动的原生条目，`hooks remove <name> --revision <revision>` 撤销受管理的副本，`hooks sync` 重试发布。

`carbforge-cli hooks import-magpie <备份>` 预览 Magpie 加密备份中的钩子名称、事件和客户端分配。可重复传入 `--name` 选择条目。`--apply` 将定义保存到共享库。可用 `hooks list` 审核命令，再用 `hooks sync` 发布。所选钩子的原事件及匹配条件须受分配到的每个客户端支持。

**移除已安装的钩子** 会移除所选事件条目，并把原客户端配置文件保存在 Crabforge 数据目录的 `hooks/backups` 下。一个分组条目可能包含多条命令。客户端文件仍与移除后完全一致时，可从 **钩子备份** 恢复。CLI 使用 `hooks remove-installed <key>` 和 `hooks restore <id>`，其中键和 ID 分别来自 `hooks list` 的 `installed` 和 `backups` 字段。

在 **钩子 → 打开项目** 中，将共享钩子分配给兼容客户端。Claude Code 读取 `.claude/settings.local.json`，Codex 读取 `.codex/hooks.json`，Antigravity 读取 `.agents/hooks.json`，Grok Build 读取 `.grok/hooks/crabforge.json`。项目分配使用独立修订，并随共享定义更新。取消选择所有客户端即可撤销。在 CLI 中，用 `hooks --project <目录> list` 获取 `shared` 下的分配修订，再执行 `hooks --project <目录> assign <名称> --revision <修订> --agent claude`。其他客户端使用 `codex`、`agy` 或 `grok`，撤销时省略 `--agent`。原生发现、移除和恢复也可使用同一个项目选项。新增或修改 Codex 钩子后，按客户端的信任设置在 `/hooks` 中确认。

用户分配写入 Antigravity 的 `~/.gemini/config/hooks.json` 和 Grok Build 的 `$GROK_HOME/hooks/crabforge.json`。Grok 的默认用户目录为 `~/.grok`。Antigravity 将条目发布到名为 `crabforge` 的分组，并支持 `agy:PreInvocation` 和 `agy:PostInvocation` 原生事件。Grok 发现功能也会读取用户或项目钩子目录中的其他 `.json` 文件，接管后的用户钩子保留在原文件。Antigravity 和 Grok 都按各自的工作区信任设置启用项目钩子。

DeepSeek Harness 的用户分配覆盖 `$DSH_HOME/profiles`（默认为 `~/.dsh/profiles`）下已初始化的配置。确保 `dsh` 和 `pnpm` 在 `PATH` 中，Crabforge 会通过 `dsh plugin --profile` 安装与 `dsh` 版本一致的官方钩子插件和协议包，并添加引用 `crabforge-hooks.json` 的配置项。已有配置会保留，若用户已经配置该插件，则显示冲突。撤销最后一个钩子后会移除 Crabforge 的配置项。同步后启动新的 Harness 进程即可加载更新。

在 **客户端连接 → 子代理** 向 Claude Code、Codex、OpenCode、Gemini CLI、Qwen Code、Factory Droid、Grok Build、Cursor 和 Copilot CLI 发布原生 Markdown 或 TOML。也可以在同一窗口选择项目文件夹，管理该项目的子代理。项目配置另支持 Antigravity。Claude Code 支持技能引用和推理级别，Claude Code 和 Factory Droid 支持 MCP 服务器名称，Codex 支持模型 ID 和推理级别。

打开项目后选择 **共享子代理**，把用户库条目分配给该项目的客户端。用户库内的修改与重命名会同步到已分配的项目。取消选中所有客户端可撤销分配。CLI 用 `subagents --project <文件夹> list` 获取 `shared` 条目和修订，再用 `subagents --project <文件夹> assign <名称> --revision <修订> --agent claude` 分配。重复 `--agent` 可选多个客户端，省略它可撤销。撤销会移除未修改的受管理文件，并保留外部修改。

在 **客户端连接 → 斜杠命令** 向 Claude Code、OpenCode、Pi、Gemini CLI、Qwen Code 和 Factory Droid 发布可复用提示词。`$ARGUMENTS` 表示全部参数。Claude Code、OpenCode 和 Pi 也可用 `$1`、`$2` 等引用单个参数。Crabforge 按客户端格式转换占位符，并报告无法发布的分配。Codex 可用技能保存可复用提示词。

选择 **导入斜杠命令** 可从文件、本地文件夹或 Git URL 预览外部 Markdown 或 TOML 命令。导入将提示词和描述复制到库，并保留来源供后续检查更新。文件布局可识别格式时，Crabforge 会转换 Claude、Pi、Gemini 和 Qwen 的占位符。CLI 使用 `commands probe-source <来源>` 和 `commands import-source`，JSON 请求格式与子代理相同。

运行 `carbforge-cli commands import-magpie .\magpie.magpie-backup` 或 `carbforge-cli subagents import-magpie .\magpie.magpie-backup` 可从 Magpie 加密备份恢复资源定义。重复使用 `--name <名称>` 可筛选条目。预览列出客户端分配、名称冲突和跳过的元数据。加上 `--apply` 后，Crabforge 检查冲突并一次性写入所选区段，相同定义会跳过。使用 `list` 审核已保存的定义，再通过 `sync` 发布到所选客户端。

在 **客户端连接 → Claude Code 插件** 管理 Claude Code 用户配置中的插件和市场。将 Claude Code 加入 `PATH` 后，可以添加本地目录或远程市场，浏览市场目录、安装和更新插件，或卸载插件并保留持久数据。CLI 提供 `claude-plugins list`、`available`、`add-marketplace`、`update-marketplace`、`remove-marketplace`、`install`、`update` 和 `uninstall`。用 `list` 返回的 `--revision` 配合 `enable` 或 `disable` 修改用户配置。

选择 **打开项目**，把已安装插件启用到该目录下的 `.claude/settings.local.json`。CLI 用 `claude-plugins --project <目录> list` 获取修订，再使用同一项目选项执行 `enable <名称@市场> --revision <修订>`。仅当插件键由 Crabforge 写入且值仍为 `true` 时，撤销分配才会移除该键。配置编辑会保留其他键和已有的项目选择。

插件菜单支持以 JSON 导出和导入设置。CLI 可保存 `claude-plugins export-settings` 的输出，再用 `claude-plugins import-settings <文件>` 导入。JSON 携带 GitHub 或普通 HTTPS 市场声明及已启用的插件 ID，也包括官方市场中已启用的插件。本机路径和含凭据、请求头或 URL 查询参数的来源保留在原配置中。导入补充缺失的声明和启用项，保留本地选择，由 Claude Code 负责安装。

`carbforge-cli claude-plugins import-magpie <备份>` 预览 Magpie 加密备份中的便携市场和已启用插件声明。加 `--apply` 可将这些声明补充到用户配置。导入会保留本地选择，预览会列出冲突市场和受影响的插件。Claude Code 根据声明负责安装。

在 **客户端连接 → 共享插件库** 中，从目录、清单或 Git URL 预览 Claude Code、Codex、Antigravity 或 Pi 插件。保存时核对来源仍与预览一致，并记录包含附件的完整快照。更新操作重新预览来源，再按库中的当前修订保存。

对于兼容 Claude 的市场，输入目录、`.claude-plugin/marketplace.json` 或 Git URL，再选择 **浏览插件**。CLI 对应命令为 `plugin-bundles marketplace <来源>`。选择列表中的来源，预览清单和文件后即可保存。

打开已保存的插件包并选择资源文件，即可审核并复制单个技能、子代理、斜杠命令、MCP 服务或命令钩子。客户端和项目分配、更新与撤销由各资源库管理。复制后，资源独立于插件包存在。审核界面会标明定义对原生插件环境的依赖。Pi 包中声明的 `SKILL.md` 和 Markdown 提示模板可复制到资源库，声明的扩展和主题随快照携带，可在 Pi 中使用。

CLI 用 `carbforge-cli plugin-bundles preview <来源>` 获取内容哈希，再用 `plugin-bundles save <来源> --content <哈希>` 保存。更新时附上 `plugin-bundles list` 返回的 `--revision <修订>`。用 `plugin-bundles resources <名称> <类型> <路径> --revision <修订>` 查看单个资源文件，再附上 `--copy <键>` 将选中的定义复制到对应的库。

要将已保存的快照带到另一个 Crabforge 配置，可选择 **导出插件归档**，或执行 `plugin-bundles export <名称> <文件.tar> --revision <修订>`。在目标配置的导入窗口打开 TAR，或用 `plugin-bundles preview` 和 `save` 导入。归档包含原插件文件和附件，分配和资源库中的修改由目标配置管理。

`carbforge-cli plugin-bundles import-magpie <备份>` 预览备份内嵌的完整插件包。用 `--name <名称>` 选取条目，再用 `--apply` 保存快照。预览会显示来源中的客户端分配和启用状态。使用项目分配命令启用已保存的插件包。导入需要完整文件树，并要求清单中的组件均保留。

在共享插件库中打开 Pi、Claude Code、Codex 或 Antigravity 对应的项目包或项目插件页面，选择项目文件夹并分配已审核的快照。库中保存新版后，项目继续使用固定版本，选择 **同步分配** 才会切换。撤销时保留在 Crabforge 外部做出的修改。项目的信任状态由各客户端管理。

- Pi 通过 `.pi/settings.json` 引用快照，并由原生加载器发现技能、提示模板、扩展和主题。
- Claude Code 需要位于 `PATH`。Crabforge 调用官方 CLI，以 `local` 作用域安装快照，撤销时使用 `--keep-data` 保留插件数据。
- Codex 从受信任项目的 `.codex/config.toml` 读取市场和启用项。Crabforge 将固定包放入当前 Codex 配置的插件缓存。撤销会移除仍属 Crabforge 的项目条目，磁盘上的快照继续保留。
- Antigravity 通过 `.agents/plugins.json` 引用放在工作区内的快照。撤销会移除仍属 Crabforge 的发现条目，磁盘上的快照继续保留。

在 `carbforge-cli plugin-bundles` 下选择 `pi-project`、`claude-project`、`codex-project` 或 `antigravity-project`。执行 `<命令> <文件夹> list` 获取项目 `revision` 和各包的 `bundle_revision`。用 `assign <名称> --project-revision <修订> --bundle-revision <包修订>` 分配或同步，用 `withdraw <名称> --project-revision <修订>` 撤销。

跨项目携带时，在源项目选择 **导出分配**，再为每个已分配包选择 **导出插件归档**，保存项目正在使用的版本。在目标库导入 TAR，打开同一客户端的目标项目，再选择 **应用分配**。JSON 保存包名及内容哈希，目标端核对哈希后建立本机路径和所有权记录。CLI 对应 `export --project-revision <修订>`、`archive <名称> <文件.tar> --project-revision <修订>` 和 `apply <分配.json> --project-revision <修订>`。

`carbforge-cli subagents` 和 `carbforge-cli commands` 均支持 `list`、`save`、`remove` 和 `sync`。用 `subagents --project <文件夹>` 选择项目库。`save` 从标准输入读取 UTF-8 JSON，字段包括 `name`、`prompt` 和 `agents`。子代理还要求 `description`，可选字段为 `model`、`effort`、`skills` 和 `mcp`。将以下内容保存为 `check.json`，在 Windows 执行 `cmd.exe /d /c "carbforge-cli commands save < check.json"` 即可发布。

```json
{"name":"check","description":"检查文件","prompt":"检查 $ARGUMENTS 是否正确。","agents":["claude","opencode","pi"]}
```

用 `save --old <名称> --revision <修订>` 编辑或改名，用 `remove <名称> --revision <修订>` 撤销。`list` 显示修订和每个文件的发布状态。已有同名文件和外部修改会保留。`sync` 可重试发布。

选择 **查找已有文件** 可扫描支持的客户端目录并原址接管未改动的原生 Markdown 或 TOML 文件，包括 OpenCode 的旧版 `command/`、`agent/` 目录和 Qwen 的 TOML 命令。CLI 的 `discover` 返回客户端 ID、路径和修订，`adopt <客户端> <路径> --revision <修订>` 用于接管。接管保留原始字节，直至编辑资源。撤销分配可能移除对应的未修改文件。

在用户子代理库选择 **导入子代理**，可扫描 Markdown 文件、本地文件夹或 Git 仓库。选择要复制的定义，并按需为新条目分配客户端。再次扫描同一来源可查看更新，已在本地修改的描述和提示词以及已保存的客户端分配会保留。导入会复制名称、描述和提示词。客户端专用选项可在编辑器中配置。

运行 `carbforge-cli subagents probe-source <路径或地址>` 获取候选的相对路径 `path` 与修订 `revision`。将选中的文件作为 UTF-8 JSON 传给 `carbforge-cli subagents import-source` 的标准输入。请求包含 `source` 字符串、由 `{ "path": "…", "revision": "…" }` 对象组成的 `files` 数组，以及可选的 `agents` 客户端 ID 数组，用于分配新条目。

`environment save <名称> --agent <claude|codex|pi>` 从标准输入读取原始 UTF-8 值。重复 `--agent` 可分配多个客户端。编辑时传入 `environment list` 返回的修订，改名另加 `--rename-from <旧名称>`。`environment remove <名称> --revision <修订>` 移除变量，`environment sync` 重试同步，`environment list --show-values` 显示变量值。

用 `carbforge-cli environment import-magpie .\magpie.magpie-backup` 预览变量名、客户端分配及备份是否含值。重复使用 `--name <名称>` 可选择变量。加 `--apply` 会在一次库写入中保存定义及备份中的值。已有不同内容的同名定义会阻止整批写入。审核后用 `environment sync` 发布。Magpie 用 `--no-keys` 创建的备份仅携带空值，可用 `environment save` 补填。

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

用 `carbforge-cli project-mcp library import-magpie .\magpie.magpie-backup` 预览可迁移的 stdio、HTTP 和 SSE 服务器定义。重复使用 `--name <名称>` 可选择服务器。加 `--apply` 后会一次写入整批定义，遇到同名冲突则停止写入。预览显示原客户端名称并隐藏连接详情。审核服务器后，用 `library assign` 选择本机项目和客户端。含禁用工具、OAuth 等尚未支持的访问设置的定义需要手动迁移。

## 协议与路由

- 支持 OpenAI Chat Completions、OpenAI Responses、Anthropic Messages、Gemini 和 Ollama 客户端。目录、对话流、工具调用、媒体和鉴权的验证范围见验收记录。不同供应商及套餐开放的模型会变化。
- 路由组支持顺序回退、轮询、加权轮询、手选和嵌套。`carbforge-cli provider fallback primary group/work` 为直接请求 `primary` 的模型设置回退目标。`carbforge-cli provider set backup fallback-for-others=no` 限制其他普通供应商失败后将请求转给 `backup`。
- 桌面端支持简体中文、英语、法语、德语、西班牙语、葡萄牙语、印地语、日语和韩语。默认跟随系统语言。
- 六个控制台页面、路由组成员编辑器及 Oh My Pi 的 profile、协议、角色和模型选择窗口都可以切换列表或网格。下次启动会恢复上次的布局。编辑路由组时切换布局，已输入的权重会保留。

## 连接另一台网关

在客户端运行 `carbforge-cli peer-id`，将其输出加入服务端的受信 Peer ID。服务端在网络设置中填写监听地址，然后启动网关。对于直连，使用可达的 IPv4 或 IPv6 地址和 UDP 端口。通过 Circuit Relay v2 连接时，先准备双方均可访问的中继，并在服务端填写中继预约地址，例如 `/ip6/RELAY_IP/udp/PORT/quic-v1/p2p/RELAY_ID/p2p-circuit`。

在客户端将网关显示的完整 P2P 地址传给 `carbforge-cli tunnel <地址>`。同时拥有直连和中继地址时，使用 `carbforge-cli tunnel <直连地址> --relay-fallback <中继电路地址>`。两者须以同一网关 Peer ID 结尾。桌面的连接窗口也提供可选中继地址。本地入口默认为 `http://127.0.0.1:3426/v1`，使用远端网关密钥。同机已验证 Kubo 双向 QUIC、IPv6 中继回退，以及 DCUtR 升级为直连后在中继下线时发起新的鉴权请求。独立公网主机之间的直连及 NAT 打洞仍待验证。

## 构建 Windows x64 版本

运行 Windows x64 版时，先完整解压 ZIP，再启动 `desktop\crabforge.exe`。两个 CLI 入口位于解压根目录。目标计算机需要安装 x64 版 Microsoft Visual C++ v14 可再发行组件。[微软运行库下载](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170)

GitHub Actions 构建 Windows x64 桌面和两个 CLI。成功的构建上传未签名的 `crabforge-windows-x64-ci.zip`。签名发布工作流从发行提交构建，并在通过 Authenticode 验证后公开发布包。[Windows 构建](https://github.com/blueokanna/Crabforge/actions/workflows/windows-build.yml) · [发布指南](docs/windows-release.md) · [构建与桌面验收](docs/validation/cf-windows-469e877/README.md)

安装 Rust 的 MSVC 工具链、Flutter 和 Visual Studio 的「使用 C++ 的桌面开发」工作负载。已记录的构建环境为 Rust 1.99.1、Flutter 3.47.9 和 Dart 3.13.7。

```powershell
cargo build --release --locked -p crabforge-cli
cd gui
flutter pub get
flutter build windows --release
```

双 CLI 位于 `target/release`。分发桌面应用时复制整个 `gui/build/windows/x64/runner/Release` 目录，其中包含 `crabforge.exe`、Flutter 运行库、文件选择器插件、Rust 桥接库及 `data`。项目仍在移植 Magpie 功能，各次冻结版本的实测结果和限制可在验收记录中查询。
