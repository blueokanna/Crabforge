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

## 协议与路由

- 支持 OpenAI Chat Completions、OpenAI Responses、Anthropic Messages、Gemini 和 Ollama 客户端。目录、对话流、工具调用、媒体和鉴权的验证范围见验收记录。不同供应商及套餐开放的模型会变化。
- 路由组支持顺序回退、轮询、加权轮询、手选和嵌套。`carbforge-cli provider fallback primary group/work` 为直接请求 `primary` 的模型设置回退目标。`carbforge-cli provider set backup fallback-for-others=no` 限制其他普通供应商失败后将请求转给 `backup`。
- 桌面端支持简体中文、英语、法语、德语、西班牙语、葡萄牙语、印地语、日语和韩语。默认跟随系统语言。
- 六个控制台页面均可通过视图按钮切换列表或网格。选择会应用到其他页面，并在下次启动时恢复。

## 连接另一台网关

在客户端运行 `carbforge-cli peer-id`，将其输出加入服务端的受信 Peer ID。服务端在网络设置中填写监听地址，然后启动网关。对于直连，使用可达的 IPv4 或 IPv6 地址和 UDP 端口。通过 Circuit Relay v2 连接时，先准备双方均可访问的中继，并在服务端填写中继预约地址，例如 `/ip6/RELAY_IP/udp/PORT/quic-v1/p2p/RELAY_ID/p2p-circuit`。

在客户端将网关显示的完整 P2P 地址传给 `carbforge-cli tunnel <地址>`。同时拥有直连和中继地址时，使用 `carbforge-cli tunnel <直连地址> --relay-fallback <中继电路地址>`。两者须以同一网关 Peer ID 结尾。桌面的连接窗口也提供可选中继地址。本地入口默认为 `http://127.0.0.1:3426/v1`，使用远端网关密钥。同机已验证 Kubo 双向 QUIC、IPv6 中继回退，以及 DCUtR 升级为直连后在中继下线时发起新的鉴权请求。独立公网主机之间的直连及 NAT 打洞仍待验证。

## 构建 Windows x64 版本

安装 Rust 的 MSVC 工具链、Flutter 和 Visual Studio 的「使用 C++ 的桌面开发」工作负载。已记录的构建环境为 Rust 1.99.1、Flutter 3.47.9 和 Dart 3.13.7。

```powershell
cargo build --release --locked -p crabforge-cli
cd gui
flutter pub get
flutter build windows --release
```

双 CLI 位于 `target/release`。分发桌面应用时复制整个 `gui/build/windows/x64/runner/Release` 目录，其中包含 `crabforge.exe`、Flutter 运行库、Rust 桥接库及 `data`。项目仍在移植 Magpie 功能，各次冻结版本的实测结果和限制可在验收记录中查询。
