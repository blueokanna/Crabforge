# Crabforge

Crabforge 在本机提供模型网关。Flutter 桌面应用和 Rust CLI 共用供应商配置、路由、客户端密钥和 libp2p 身份。Windows 应用标识为 `com.blue.crabforge`，当前版本为 `0.1.0`。

## 启动

Windows 桌面版本的入口是 `gui/build/windows/x64/runner/Release/crabforge.exe`。分发时保留整个 `Release` 目录，其中包含 Flutter 运行时、Rust 桥接库和应用资源。

1. 在启动 Crabforge 的环境中设置供应商密钥，例如 `DEEPSEEK_API_KEY`。
2. 在「供应商」页选择预设或填写 API 基址，填入密钥的环境变量名，获取或手动填写模型 ID，然后保存。
3. 在「网关」页启动服务，复制客户端密钥与 API 基址。客户端请求使用此密钥，模型可填写 `供应商ID/模型ID`。默认基址为 `http://127.0.0.1:3425/v1`。
4. 在「路由组」页添加模型或子组，选择按列表顺序、轮询或加权轮询，让客户端请求 `group/组名`。父组把每个子组算作一个成员，子组按自己的策略选择模型。加权轮询只计算支持当前协议的成员。权重默认是 1，最大为 1000。加权首选成员返回可重试错误时，网关按列表顺序尝试其余成员。在「路由」页可查看每次尝试的子组路径、供应商、状态和 Agent。

至少两家供应商配置同一模型时，「路由组」页默认按供应商配置顺序生成路由组。例如 `vendor/claude-opus-5.5@1m` 和 `claude-opus-5-5` 可以通过 `auto/claude-opus-5.5` 或 `group/auto-claude-opus-5-5` 调用。如果供应商 ID 本身为 `auto`，其模型继续占用 `auto/` 前缀。编辑自动组会保存为自定义组，关闭自动分组后仍可使用。可以在配置中用 `"auto_group_skip":["供应商ID"]` 排除供应商，用 `"model_same_as":{"供应商ID/别名":"模型ID"}` 指定两个名称指向同一模型。

自定义供应商可在编辑器填写 models.dev 目录 ID（配置字段 `catalog`），多个目录用逗号或空格隔开。网关按填写顺序查找模型省略的厂商前缀。例如在 `kimi-code-plan-cn` 目录中，`k3.1-pro` 能与其他供应商的 `kimi-k3.1-pro` 合并。已有组优先拥有自己的旧式和公开地址。

Bedrock Runtime 和 Mantle 支持 API key 或 IAM 签名。使用 IAM 时，将供应商的「API 密钥环境变量」留空，在启动 Crabforge 的环境中设置 `AWS_ACCESS_KEY_ID`、`AWS_SECRET_ACCESS_KEY`，临时凭证还需设置 `AWS_SESSION_TOKEN`。Crabforge 从 AWS 端点识别区域和服务名，签署推理请求及 Mantle 的在线目录请求。Runtime 使用内置模型清单。

第三方 Anthropic 接口可在编辑器中选择密钥请求头，或配置 `endpoints.auth_header` 为 `x-api-key` 或 `bearer`。默认发送两个请求头。当 HTTP 401 明确指出两个请求头冲突时，网关或 CLI 目录查询会依次尝试单头，成功后保存选择。编辑器读取尚未保存的供应商目录时，协商只在本次查询内有效。

Google Vertex AI 预设可使用 Express API key。要使用 Google Cloud 项目，在编辑器中打开「Vertex AI 项目设置」，填写项目 ID 和区域，再读取该区域的内置模型清单。使用应用默认凭证（ADC）时把 API 密钥环境变量留空。Crabforge 先读取指定的 ADC 文件，再按 `GOOGLE_APPLICATION_CREDENTIALS`、gcloud 默认文件的顺序查找；指定路径无效时会报告错误。支持用户刷新令牌、服务账号 RSA 密钥及服务账号模拟。也可为代理填入 API 密钥环境变量与资源基址。对应 CLI 配置示例为 `"endpoints":{"vertex":{"project":"example-project","location":"global"}}`，可选字段为 `credentials`、`impersonate` 和 `base_url`。

选择带内置清单的预设后，桌面端会填入可编辑的模型 ID。`carbforge-cli catalog` 的 `models` 字段提供相同内容。`carbforge-cli models 供应商ID` 在使用内置清单时向标准错误输出提示，标准输出仍为每行一个 ID。模型能否调用取决于账户、套餐和区域。

对于提供多个在线目录的供应商，「读取模型目录」会合并各协议完整返回的模型 ID，并保留已填写的模型。如果供应商另有目录，可填写「模型目录完整地址」（配置字段 `endpoints.models_url`），例如 `https://relay.example.com/account/models`。读取目录会向此地址使用该供应商的鉴权配置，推理仍使用相应协议的基址。

使用 Gemini 原生接口的中转时，把服务地址填入 `endpoints.gemini`。可以填写 `https://relay.example.com`、带中转前缀的 `https://relay.example.com/relay/v1beta`，或完整的 `…/models/模型:generateContent` 方法地址。网关保留中转前缀，按页读取目录。目录明确列出支持的方法时，只收录支持 `generateContent` 的模型。

Ollama 客户端使用网关根地址（默认 `http://127.0.0.1:3425`）及 `Authorization: Bearer <网关密钥>`。可通过 `/api/version`、`/api/tags` 和 `/api/show` 查询版本与模型，通过 `/api/chat` 或 `/api/generate` 对话。默认逐行返回 NDJSON，设置 `"stream": false` 可获取单个 JSON 回复。支持 `group/组名` 和 `:latest`。模型详情中未知的上下文窗口按 32768 展示。

### 命令行

在「供应商」页选择「重命名供应商」，或运行 `carbforge-cli provider rename 旧ID 新ID`。下次启动网关时，原有会话仍可请求旧地址。旧 ID 被另一家供应商使用后，请求归当前供应商。改名同时更新路由组成员、固定供应商前缀的通配规则、权重和手选项，以及 Oh My Pi 各 profile 的角色、回退链、作用域、角色备份和模型目录。如果某个 profile 文件无法读取，修复文件后可重跑原命令。

`carbforge-cli` 和 `crabforge-cli` 提供同一组命令。以下示例使用 PowerShell。`config` 输出当前设置，`config <文件>` 校验并导入。`serve` 占用当前终端，Agent 命令在另一个终端中运行。

```powershell
carbforge-cli config
carbforge-cli catalog
carbforge-cli models deepseek
carbforge-cli group set daily primary/model secondary/model --weight primary/model=3
carbforge-cli group list
carbforge-cli group show daily
carbforge-cli group rename daily work
carbforge-cli group set pinned primary/model secondary/model --pick secondary/model
carbforge-cli group pick pinned primary/model
carbforge-cli group set pinned primary/model secondary/model --rotate
carbforge-cli group set pinned primary/model secondary/model --manual
carbforge-cli group copy work "Work backup"
carbforge-cli group order pinned work work-backup
carbforge-cli group set work-backup primary/model secondary/model --note "工作日备用"
carbforge-cli group member work-backup primary/model off
carbforge-cli group member work-backup primary/model on
carbforge-cli serve
```

`carbforge-cli group auto off` 关闭自动分组，`group auto on` 重新开启。`group remove auto-claude-opus-5-5` 隐藏单个组，`group restore auto-claude-opus-5-5` 恢复它。若其他组仍依赖被移除的组，保存时会指出失效的引用。模型后来只剩一家供应商、或用户关闭自动分组时，先前的自动组地址可继续请求同一模型。

选择「手动选择成员」后，可在组卡片上或用 `group pick` 切换目标。运行中的网关从下一次请求起使用新成员，已经发出的请求沿用原有选择。成员失败时该手动组返回错误，外层顺序组仍可使用其余成员。选定项已从配置中移除时，网关使用组内第一项。

临时改用其他策略后，再用 `group set … --manual` 或在桌面改回手动，仍会选中上次使用的成员。网关运行且组页面可见时，桌面每 3 秒读取外部 CLI 更新的选择。

桌面组卡片的上下箭头和 `group order` 可调整组的显示顺序。未点名的可见组保持相对顺序。下次启动网关时，OpenAI、Gemini 和 Ollama 的模型目录按保存的顺序优先列出这些组。`group copy` 创建独立副本，放在来源组后面，保留成员、权重、手动选择和备注。复制自动组会保存当前成员。名称冲突时按序号命名。备注合并空白，最多保留 200 个 Unicode 字符，`--note=` 可清除备注。

在组编辑器中停用成员，或运行 `group member 组名 成员 off`，可让顺序、轮询和加权轮询跳过该成员。这三种策略至少保留一个启用成员。重新启用会恢复原位置和权重。停用只在当前组生效，手动策略始终使用选中的成员。修改成员启停状态后重新启动网关。运行中的手动成员切换仍可直接使用 `group pick`。

`groups` 列出路由组。`group show 组名` 和 `group 组名` 可查看组的地址、备注、路由策略、成员状态及直接引用它的父组。

从加权轮询切换到其他策略时，组会记住现有成员的权重。桌面重新选择「加权轮询」或 CLI 使用 `group set 组名 成员列表 --weighted` 可恢复这些权重。显式传入 `--weight 成员=权重` 会按新列表设置权重。

组也可以随供应商的模型目录变化。`carbforge-cli group set free-models 'free:workbuddy/*'` 创建只包含目录明确标记为免费的模型组。`*` 可以跨越 `/`，`re:` 接受匹配整个 `供应商/模型` 的正则表达式，默认忽略大小写，支持 `(?-i)` 及前瞻。默认先尝试显式成员，再按目录顺序尝试匹配项。`group show 组名` 列出每条规则的命中数。`group place 组名 成员列表` 可把指定成员移到最前，省略成员则恢复默认顺序。桌面编辑器可逐行输入规则，预览匹配模型并调整优先级。

`carbforge-cli models 供应商 --sync` 用上游返回的模型 ID 和免费标记更新本机目录。如果会使显式成员失效，保存会报告哪个组引用了该模型。模式组可暂时匹配零项。下次启动网关时会使用更新后的目录。Magpie v1 分享携带 `match` 和 `placed`，收件人在自己的目录上重新匹配。

在桌面组卡片上选择「复制路由组以分享」，在另一台设备的「路由组 → 导入路由组」中粘贴。CLI 可用 `carbforge-cli group share work` 输出 Magpie v1 分享文本，用 `carbforge-cli group import work.json` 从 UTF-8 文件导入，省略文件名则读取标准输入。默认追加为新组；已有自定义组的 ID 与分享文本相同时，`--replace` 在原位置更新它并保留启停状态。导入报告列出缺失成员和本机采用的等价模型。当前可传递顺序、轮询、加权和手选策略、成员开关、目录可见性、备注与历史地址。导入器遇到尚未支持的策略或设置会报告错误。

用 `carbforge-cli group switch daily off` 或点击组卡片左侧的开关，可停用已保存的组。重启网关后，客户端目录会移除该组，向其当前或历史地址发送请求会收到 HTTP 404。顺序、轮询和加权父组会跳过它。手动父组选中它时，请选择另一个可用成员。`group switch daily on` 可恢复原有成员、策略和目录规则。停用期间仍可编辑组，复制得到的新组会启用。自动发现的组须先编辑保存才能切换。

例如，`group set daily local/m group/backup --listed=no` 会在客户端目录中列出 `group/daily`，隐藏其直接成员 `local/m`。子组 `group/backup` 仍控制自己的模型列表。可以继续按完整地址调用隐藏的成员。桌面组编辑器提供同一开关，网关重启和 Agent 下次启动时读取设置。使用 `--listed=yes` 可恢复列出成员，其他组设置的隐藏规则仍然有效。

```powershell
carbforge-cli agent codex deepseek/deepseek-chat -- exec "你好"
carbforge-cli agent copilot deepseek/deepseek-chat -- -p "你好"
carbforge-cli agent claude deepseek/deepseek-chat -- -p "你好"
carbforge-cli agent pi deepseek/deepseek-chat -- -p "你好"
carbforge-cli agent omp deepseek/deepseek-chat -- -p "你好"
carbforge-cli agent omp deepseek/deepseek-chat --profile work -- -p "你好"
carbforge-cli agent goose deepseek/deepseek-chat -- run -t "你好"
carbforge-cli agent qwen deepseek/deepseek-chat -- "你好"
carbforge-cli agent vibe deepseek/deepseek-chat -- -p "你好"
carbforge-cli agent crush deepseek/deepseek-chat -- run --quiet "你好"
carbforge-cli agent kimi deepseek/deepseek-chat -- -p "你好"
carbforge-cli agent nanobot deepseek/deepseek-chat -- -m "你好"
carbforge-cli agent omo deepseek/deepseek-chat -- -p "你好"
carbforge-cli agent mu deepseek/deepseek-chat -- -p "你好"
carbforge-cli agent prime-agent deepseek/deepseek-chat -- -p "你好"
carbforge-cli agent opencode deepseek/deepseek-chat -- run "你好"
carbforge-cli agent mimocode deepseek/deepseek-chat -- run "你好"
carbforge-cli agent deveco deepseek/deepseek-chat -- run "你好"
carbforge-cli agents
```

把示例中的模型替换为供应商实际提供的 ID，并安装相应客户端。启动器通过环境变量向子进程传递网关密钥。Gemini CLI 使用 Crabforge 数据目录下的 `agents/gemini` 配置。Pi 的 `agents/pi/models.json` 和 Goose 的 `agents/goose/config/custom_providers/crabforge.json` 包含裸模型名、供应商前缀名和路由组，启动器在每次运行时更新这两种目录中的对应文件。文件中只保存密钥变量名。Agent 请求的可用参数取决于上游协议，使用客户端的原生协议端点可保留其专用字段。`key` 用于手动配置客户端。

Qwen Code 使用 `agents/qwen` 作为自己的配置目录。启动器选择 OpenAI 认证和该 Agent 的 Chat Completions 地址，并通过 `OPENAI_API_KEY` 将网关密钥传给子进程。

Pi、mu、OmO 和 Prime Agent 各用 `agents/<客户端>/models.json`。启动器只更新其中的 `crabforge` 供应商，保留个人供应商条目，并为每个客户端写入它能解析的环境变量引用。

连接另一个 Crabforge 或 Magpie 网关时，用 `carbforge-cli models 供应商 --sync` 或桌面的「读取模型目录」再保存。目录中的 `native_endpoints` 表示模型能直接处理的协议，`magpie_prefers` 表示上游首选协议。启动器据此配置每个模型。Pi 和 mu 的 Gemini 模型默认经 Chat 转换，OmO 和 Prime Agent 可直接使用 Gemini。Responses 模型支持严格模式，Messages 模型使用网关根地址。

首次同步已有 Pi 格式的配置会建立模型协议记录。此后，在 `models.json` 中给某个模型指定一个此前未由 Crabforge 写入的 `api`，下次同步会保留它及同一条目中的 `baseUrl`、`compat`。删除模型的 `api` 可恢复自动选择。

Oh My Pi 的命令名是 `omp`。Windows 使用 npm 安装的 Oh My Pi 19.2.1 时，需将 Bun 1.4.4 或更高版本加入 `PATH`。启动器按 `models.yml`、`models.yaml`、`models.json` 的顺序读取 `agents/omp` 中的目录，并将旧 JSON 中的供应商带入 YAML。它保留个人供应商，将网关密钥通过子进程的 `CRABFORGE_GATEWAY_KEY` 提供。对话使用 `/agent/omp` 下的协议地址。

使用命名 profile 时，将 `--profile 名称` 放在传给 Oh My Pi 的 `--` 之前。省略它时依次读取 `OMP_PROFILE` 和 `PI_PROFILE`。空的 `OMP_PROFILE` 或名称 `default` 使用 Crabforge 的默认目录 `agents/omp`。命名 profile 使用已安装客户端通过 `config path` 报告的目录，通常是 `~/.omp/profiles/名称/agent`，其中的个人供应商继续可用。默认目录协议为 Chat Completions，支持原生 Responses、Messages 或 Gemini 的模型会在模型级别使用相应地址。组内所有可用成员都支持 Responses、且有成员首选它时，组也会首选 Responses。

`carbforge-cli omp-profiles` 列出本机默认和命名 profile、当前选择及各自的目录。Oh My Pi 19.2.1 将 `PI_CONFIG_DIR` 解释为相对用户主目录的路径，默认是 `.omp`。对于已有的原生调用，例如 `carbforge-cli agent omp local/m -- --offline --profile work -p "你好"`，启动器将目录写入客户端实际选择的 `work`。两侧都指定时，以 Oh My Pi 原生选项为准。

在「客户端连接」选择 `omp` 后，打开「顾问设置」可编辑对应 profile 的审阅时机、等待阈值和恢复运行的最低问题级别。CLI 用 `carbforge-cli omp-advisor --profile work` 查看当前设置，加 `--review agent-end --wait 3 --wake blocker` 可设为运行结束时审阅、积压三项时等待、仅阻塞问题恢复运行。`--review off` 关闭顾问并保留上次审阅时机。三个选项均可用 `inherit` 清除该项覆盖。设置写入所选 profile 的 `config.yml`，已有 `config.yaml` 且没有 `config.yml` 时使用前者。Oh My Pi 有 `WATCHDOG.yml` 名单时，其中各顾问自己的审阅和等待设置优先。

在同一页打开「思考级别」可设定新会话的 `defaultThinkingLevel`，也可给已选单个模型的角色指定级别。例如 `carbforge-cli omp-thinking --profile work --session auto --role slow --level high` 会让会话自动选择思考级别，让 `slow` 角色使用 `high`。`--level inherit` 清除角色后缀，`--session inherit` 清除会话设置。模型名 `local/qwen3:8b` 中的版本标签会保留。多个模型组成的回退列表使用各自的级别。

「同步模型目录」和 `carbforge-cli omp-sync --profile work` 都能刷新所选 profile。`enabledModels` 直接列出的 Crabforge 模型若从目录消失，会暂时从列表移除。模型再次可用时，原条目连同思考级别会回到列表末尾。目录为空时保留当前作用域。项目专属列表、通配符和其他供应商的条目按原样保留。通过 `agent omp` 启动客户端时也会同步。

在「客户端连接」选择 `omp` 后，「网关协议」可为一个 profile 的对话模型指定 Chat Completions、Responses、Messages 或 Gemini。`carbforge-cli omp-protocol --profile work` 显示当前选择，命令末尾加 `openai-responses` 可切换到 Responses，加 `auto` 恢复各模型的原生协议。下一次启动 Oh My Pi 时生效。

在「客户端连接」选择 `omp`，打开「角色模型」可以分别给默认会话、子任务、顾问等十个角色选模型。`carbforge-cli omp-roles --profile work` 可查看角色及候选模型。例如，下列命令将新会话指向网关模型，随后恢复接管前的配置。原值可以是单个模型、逗号分隔的模型或 YAML 回退列表。

```powershell
carbforge-cli omp-roles --profile work --role default --model crabforge/local/m
carbforge-cli omp-roles --profile work --role default --reset
```

更换模型时会沿用角色原有的思考级别。给默认角色选模型时，已有的全局 `enabledModels` 会允许该模型。给顾问选模型时，若此前没有显式启停设置，Crabforge 会启用顾问，重置角色时会撤回这次启用。若直接运行 `omp --profile work`，先启动网关，再在该终端设置 `$env:CRABFORGE_GATEWAY_KEY = (carbforge-cli key)`。新启动的 Oh My Pi 会按已保存的默认角色选择模型。

「路由」页在每次尝试旁显示客户端提供的会话 ID。网关优先读取 `X-Crabforge-Session`，其次读取 `X-Magpie-Session`，最后读取客户端的原生会话头。带 ID 的已认证请求会在响应中收到 `X-Crabforge-Session` 和 `X-Magpie-Session`。Pi 和 Oh My Pi 16.0.6 起支持用 `x-session-affinity` 上报会话。路由页保留本网关进程最近 128 次尝试，重启会清空记录。

浏览器页面需要跨来源访问网关时，在「网关 → 网络设置」中每行填一个来源，例如 `https://app.example.com` 或 `http://localhost:3000`，保存并启动网关。对应配置为 `"cors_origins":["https://app.example.com"]`。支持 HTTP、HTTPS 和具有完整 ID 的 Chrome、Firefox、Safari 扩展来源。页面的实际请求需要网关密钥，获得授权后可用 `response.headers.get('X-Crabforge-Session')` 读取会话 ID。Obsidian 的 `app://obsidian.md` 窗口也可以使用网关密钥访问。

OpenCode V2 启动器在 `agents/opencode` 下生成单次运行的供应商配置，从子进程环境读取网关密钥。它通过 `--standalone` 启动私有服务，把本次选定的模型同时用于对话和会话标题，并在退出时清除临时配置。

MiMo Code 和 DevEco Code 使用各自的 `MIMOCODE_CONFIG`、`DEVECO_CONFIG` 加载单次启动配置。主模型和小模型都使用选定的网关模型，密钥通过子进程环境提供。MiMo Code 的可执行文件名为 `mimo`，专属地址为 `/agent/mimocode/v1`。DevEco Code 0.2.6 要求用户在客户端确认产品协议后才能发起对话。

GitHub Copilot CLI 通过 `COPILOT_PROVIDER_*` 环境变量使用所选模型和 `/agent/copilot/v1` 的 Chat Completions 接口。Copilot 会话使用 `agents/copilot` 目录，启动器保留已有的 `providers.json`。

OpenCode Go 与 Zen 也可作为上游供应商。网关从官方地址识别套餐，读取相应的公开模型目录，使用随包 models.dev 数据选择原生协议。只填写一个官方基址即可补齐其他协议的缺省地址。访问 `/inference` 原生接口时使用 Bearer 密钥，并以 `crabforge/0.1.0` 标识对话请求。客户端可以用 `x-opencode-session` 明确区分会话。缺省时，网关优先采用客户端会话头，其次使用 `prompt_cache_key` 或首条用户消息派生本地标识。仅凭相同开场消息派生的两个会话会得到相同标识。[官方推理端点](https://opencode.ai/docs/console/api/inference) · [Go 客户端要求](https://opencode.ai/docs/console/go)

Mistral Vibe 使用 `agents/vibe/config.toml` 中的 `crabforge` 供应商及模型目录，启动参数 `--model crabforge/供应商ID/模型ID` 为本次会话选择模型。网关密钥由子进程环境提供。Vibe 程序化模式要求上游在请求 `stream_options.include_usage` 时提供 token 用量。首个进程使用 Vibe 配置的会话目录，并行进程默认使用 `agents/vibe/logs/parallel-1` 等独立目录。要恢复保存在其中的会话，可用 `VIBE_SESSION_LOGGING__SAVE_DIR` 指定对应的绝对路径。

Kimi Code 2.x 使用 Crabforge 数据目录下的 `agents/kimi/config.toml`。模型选择器列出当前网关模型，`--model` 为本次启动选择模型。启动器保留已有的 `default_model` 和模型窗口设置。新模型缺省使用 128,000 token 的上下文窗口，可在该文件中调整。`api_key_env` 指向子进程环境中的网关密钥。

nanobot 使用 `agents/nanobot` 保存工作区和会话。可以在该目录的 `config.json` 中设置自己的偏好。启动时生成单次运行用的配置副本，选定模型并从环境中读取网关密钥，退出后清除副本。

Crush 使用 `agents/crush/config/crush.json` 中的 `crabforge` 供应商。启动器为主模型与小模型选择同一网关地址，按所选模型分开保存 Crush 的选择记录。默认会话数据库位于项目的 `.crush/crabforge/` 下。显式设置 `CRUSH_DATA_DIR` 或传入 Crush 的 `--data-dir` 可以使用自己的会话目录。`run` 和 `r` 通过本次启动的本地 Crush 服务接收完整回复，结束后释放服务进程。Windows 上两端通过用户专属命名管道连接。

重命名路由组会同步更新父组成员及权重。旧组地址可供已有会话继续请求。其他组接管旧名后，该地址属于新组。自动组的公开拼写取自随包模型目录，Kimi Code 这类省略厂商前缀的模型也可归入同组。

Windows 数据目录默认为 `%LOCALAPPDATA%\com.blue.crabforge`。可通过绝对路径 `CRABFORGE_HOME` 或 CLI 的 `--data-dir` 选择独立目录。配置格式示例如下。

```json
{
  "listen": "127.0.0.1:3425",
  "providers": [{
    "id": "local",
    "endpoints": {"chat": "http://127.0.0.1:11434/v1"},
    "models": ["example-model"]
  }],
  "groups": {"default": ["local/example-model"]}
}
```

### 网页供应商

安装 Chrome 和匹配版本的 ChromeDriver，将驱动加入 `PATH` 或设置 `CRABFORGE_CHROMEDRIVER`。在「供应商」页添加 DeepSeek 网页或 ChatGLM 网页，点击该供应商，在弹出的浏览器中登录，再关闭登录窗口。CLI 也支持 `carbforge-cli web deepseek --login` 和 `carbforge-cli web chatglm --login`。

配置保存后，向 `/v1/chat/completions` 发送文本消息，模型分别为 `deepseek-web` 和 `chatglm-web`。每次请求打开该站点的独立浏览器配置，提交对话文本并等待回复完成。每个配置同时处理一个会话。页面布局或登录流程变化可能导致超时；目前的自动化验收使用本地测试页，线上登录回合待验证。

### P2P

客户端执行 `carbforge-cli peer-id`，把公钥标识交给服务端。服务端在「网关 → 网络设置」填写 QUIC 监听地址，例如 `/ip4/0.0.0.0/udp/3427/quic-v1`，将客户端 Peer ID 加入允许列表，然后启动网关。客户端用从其网络可达的服务端 IP、UDP 端口和服务端 Peer ID 建立隧道。

```powershell
carbforge-cli tunnel /ip4/192.168.1.10/udp/3427/quic-v1/p2p/服务端PeerID --listen 127.0.0.1:3426
```

应用随后访问 `http://127.0.0.1:3426/v1`，使用服务端的网关密钥。桌面端「远端网关隧道」提供相同操作。已验证同机独立进程间的直连，公网直连依赖服务端 UDP 地址可达。

## 构建与验证

Windows 构建需要 Rust MSVC 工具链、Flutter Windows 桌面工具链及 Visual Studio C++ 桌面开发组件。桥接生成文件已纳入工作区。修改桥接 API 后，在 `gui` 目录使用与 `gui/pubspec.yaml` 同版的 `flutter_rust_bridge_codegen generate`。

```powershell
cargo build --locked --release -p crabforge-cli
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cd gui
flutter pub get
flutter analyze --no-pub
flutter build windows --release --no-pub
```

Windows 集成测试使用一个新的 `CRABFORGE_HOME` 目录，目录名以 `crabforge-gui-test-` 开头。`CRABFORGE_TEST_CLI` 指向构建好的 CLI 绝对路径。在 `gui` 目录执行 `flutter test integration_test/console_test.dart -d windows --no-pub`。第一次通过后，复用该目录，设置 `CRABFORGE_RESTART_CHECK=1` 并再次执行，以验证新进程恢复中文以及跟随系统语言。

从仓库根目录可以运行以下专项验收。

- 将官方 Codex、Copilot、Claude Code、Gemini CLI、Pi、Oh My Pi、Goose、Qwen Code、Mistral Vibe、Crush、Kimi Code、nanobot、mu、OmO、Prime Agent、OpenCode V2、MiMo Code 和 DevEco Code 加入 `PATH`，执行 `cargo test --locked -p crabforge-cli --test agents -- --include-ignored --test-threads=1`。Windows 上的 Oh My Pi npm 入口还需要 Bun。
- 将 `MAGPIE_TEST_EXE` 设为 Windows Magpie 可执行文件路径，执行 `cargo test --locked -p crabforge-cli --test magpie quic_client_reaches -- --ignored`。同时设置 `LLAMA_TEST_SERVER` 和 `LLAMA_TEST_GGUF` 后，执行 `cargo test --locked -p crabforge-cli --test magpie -- --ignored --nocapture` 可再验证 QUIC 经两个网关调用真实模型。
- 将 `KUBO_TEST_EXE` 设为 Kubo `ipfs` 可执行文件路径，执行 `cargo test --locked -p crabforge-core --test kubo -- --ignored`。测试创建临时 Kubo 仓库，在回环端口上验证两个方向的 QUIC 转发和 HTTP 密钥校验。
- 将 `LLAMA_TEST_SERVER` 和 `LLAMA_TEST_GGUF` 分别设为 llama.cpp 服务程序与模型权重路径，执行 `cargo test --locked -p crabforge-core --test llama local_weights -- --ignored --nocapture`。再将 `OLLAMA_TEST_PYTHON` 指向已安装官方 `ollama` 包的 Python，执行 `cargo test --locked -p crabforge-core --test llama official_ollama -- --ignored --nocapture`，可复现原生客户端的端到端测试。
- 设置 `CRABFORGE_CHROMEDRIVER`，执行 `cargo test --locked -p crabforge-core browser_submits -- --ignored`。

已验证的协议、客户端、内存测量条件和剩余差异见[兼容性与内存记录](docs/crabforge-validation.md)。
