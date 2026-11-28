# Connect Antigravity CLI

Install Antigravity CLI and put `agy` on `PATH`. Start the Crabforge gateway and copy a model ID from **Models**. From your project directory, launch the client with the model ID you copied. [Antigravity installation](https://antigravity.google/docs/getting-started?tab=cli#download-and-install)

```powershell
crabforge-cli agent antigravity '<model-id>'
```

Crabforge uses `agy --settings` to configure Gemini API key mode and select the model for this session. It passes the gateway key and client root through `GEMINI_API_KEY` and `GOOGLE_GEMINI_BASE_URL`. You can pass additional Antigravity flags after `--`. [Gemini API key mode](https://antigravity.google/docs/install)

Project instructions use `AGENTS.md`. MCP, hooks, and subagents use Antigravity’s `.agents` workspace structure. Global hooks, skills, and subagents use `~/.gemini/config`. Supported MCP transports are stdio and Streamable HTTP. [Configuration discovery](https://antigravity.google/docs/customization-discovery) · [MCP documentation](https://antigravity.google/docs/mcp)

With Antigravity CLI 1.5.18, a session launched through Crabforge loaded `AGENTS.md`, initialized a project Streamable HTTP MCP server with its configured header, and listed the server’s tools. The test used local MCP and Gemini-compatible servers. The `agy mcp list` subcommand did not show the workspace assignment in that test.

## 简体中文

安装 Antigravity CLI 并将 `agy` 加入 `PATH`。启动 Crabforge 网关，从「模型」中复制模型 ID，在项目目录中运行上方命令。 [Antigravity 安装步骤](https://antigravity.google/docs/getting-started?tab=cli#download-and-install)

Crabforge 使用 `agy --settings` 为当前会话启用 Gemini API 密钥模式并选择模型，通过 `GEMINI_API_KEY` 和 `GOOGLE_GEMINI_BASE_URL` 传入网关密钥和客户端根地址。可在 `--` 后追加 Antigravity 参数。 [API 密钥认证](https://antigravity.google/docs/install)

项目指令使用 `AGENTS.md`。项目 MCP、钩子和子代理使用 `.agents` 配置目录。全局钩子、技能和子代理使用 `~/.gemini/config`。MCP 支持 stdio 和 Streamable HTTP。 [配置发现规则](https://antigravity.google/docs/customization-discovery) · [MCP 文档](https://antigravity.google/docs/mcp)

使用 Antigravity CLI 1.5.18 测试时，从 Crabforge 启动的会话加载了 `AGENTS.md`，携带配置的请求头初始化了项目的 Streamable HTTP MCP 服务器，并获取了工具列表。测试使用本地 MCP 服务器和兼容 Gemini API 的本地服务。该次测试中，`agy mcp list` 子命令未显示工作区配置的服务器。
