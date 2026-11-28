# Crabforge

[English](README.md) · [Windows 构建](https://github.com/blueokanna/Crabforge/actions/workflows/windows-build.yml) · [发行版](https://github.com/blueokanna/Crabforge/releases)

Crabforge 是一个带访问认证的本地模型网关。Flutter 桌面端与 CLI 共用 Rust 核心。客户端连上网关后，就可以在网关侧切换供应商、模型别名和路由策略。桌面端可管理供应商、读取模型目录、编辑路由组、查看请求记录，以及管理受支持的客户端配置。

## 在 Windows 上安装

从同一发行版下载 Windows x64 ZIP 和对应的 `.sha256` 文件。验证 SHA-256 和 GitHub 构建来源后解压，再运行 `desktop\crabforge.exe`。压缩包根目录的 `crabforge-cli.exe` 和 `carbforge-cli.exe` 提供相同的命令。 [验证步骤](docs/windows-release.md)

## 连接客户端

1. 在启动 Crabforge 的环境中设置供应商凭据。在「供应商」中选择预设或填写自定义端点，填入凭据的环境变量名，并保存至少一个模型。
2. 在「网关」中启动服务并复制密钥。兼容 OpenAI 的客户端默认使用 `http://127.0.0.1:3425/v1`。从「模型」页复制带供应商前缀的模型 ID。
3. 需要故障转移时，新建「路由组」，在客户端中选择该组发布的模型 ID。「路由」页会显示每次尝试的供应商和状态。

也可以在解压目录中运行以下命令。第二条会输出本地网关凭据。启动 `serve` 的终端需要能读取上游凭据。

```powershell
.\crabforge-cli.exe catalog
.\crabforge-cli.exe key
.\crabforge-cli.exe serve
```

Windows 默认将配置和网关密钥保存在 `%LOCALAPPDATA%\com.blue.crabforge`。可将 `CRABFORGE_HOME` 设置为绝对路径，或在 CLI 中使用 `--data-dir`，以使用独立配置。请妥善保管密钥。网关默认监听回环地址，浏览器客户端还需配置允许的来源。

## 协议与路由

- 网关提供 OpenAI Chat Completions、Responses、Anthropic Messages 和 Gemini Developer API 等接口。请求可按上游原生协议转发，受支持的生成请求也可跨协议转换。
- 路由组支持顺序回退、轮询、加权和手动选择，可设置请求匹配规则、首段内容超时和重试耐心值。
- 语音、图像和决策接口取决于所配置的供应商与模型。请求包含目标协议无法表达的字段时，请使用支持该字段的原生接口。
- CLI 可预览加密备份并按需导入。运行 `crabforge-cli import-backup --help` 查看选项。路由组可通过 `group share` 和 `group import` 在设备之间分享。

## Antigravity

启动网关并将 `agy` 加入 `PATH` 后，运行 `crabforge-cli agent antigravity <模型-ID>`。启动器使用 Antigravity 的 Gemini API 密钥认证模式，并将网关密钥和所选模型传入当前会话。在「客户端连接」中选择 **Antigravity CLI** 即可复制专用网关根地址。 [安装和启动步骤](docs/antigravity.md#简体中文)

Crabforge 可管理 Antigravity 的项目指令、MCP、钩子及子代理。全局技能和子代理写入 Antigravity 的共享配置目录。 [Antigravity 配置发现规则](https://antigravity.google/docs/customization-discovery)

## 从会话提炼记忆

自动提炼默认关闭。在「客户端连接」中打开「本地会话」和「自动提炼记忆」，选择模型或路由组及回溯天数，再保存。网关运行时，Crabforge 会把至少 30 分钟没有活动的会话中的用户消息发送给所选路由，每小时最多尝试 30 个会话。点击「立即读取」可处理最近列表中全部符合条件的会话，生成的候选记忆可在「记忆」中审核。

CLI 提供相同的操作。运行 `crabforge-cli memory distillation status` 查看状态，也可在 `memory distillation` 下使用 `enable <模型或组> --days 7`、`run` 和 `disable`。

## 桌面外观

桌面端使用 Material 3 Expressive 按钮形状、强调标题样式，以及 Google Material Symbols Rounded 图标。浅色、深色和高对比度的 Expressive 配色会随 Windows 强调色更新。工具栏中选择 Google Sans 时，拉丁文本使用 Google Sans，汉字使用 HarmonyOS Sans SC。选择 HarmonyOS Sans SC 时，中英文均使用该字体。 [内置字体来源和许可证](assets/fonts/manifest.json)

## 从源码构建

Windows CI 固定使用 Rust 1.99.1 和 Flutter 3.47.9。在 Windows 安装 Visual Studio 的 C++ 桌面开发工作负载和 Windows SDK 后，运行以下命令。

```powershell
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo build --locked --release --package crabforge-cli
Set-Location gui
flutter pub get --enforce-lockfile
flutter analyze --no-pub
flutter test --no-pub
flutter build windows --release --no-pub
```

[操作手册](CRABFORGE.md) · [发行工作流](.github/workflows/windows-release.yml) · [许可证](LICENSE)
