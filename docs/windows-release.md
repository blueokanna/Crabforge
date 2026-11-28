# Windows x64 发行版

[English](windows-release.en.md)

Windows 构建工作流在 `main` 分支更新和拉取请求时运行。它执行 Rust 和 Flutter 检查，构建 CLI 与桌面端，并将便携发行包暂存为 `crabforge-windows-x64-ci`。发行工作流从发行提交重新构建，生成包含 `SHA256SUMS` 清单的 ZIP，再通过 GitHub Actions 生成来源证明。发布任务验证压缩包哈希与构建来源，上传两个附件并核对上传后的摘要，然后公开发行版。 [构建工作流](../.github/workflows/windows-build.yml) · [发行工作流](../.github/workflows/windows-release.yml)

## 验证与运行

从同一发行版下载 ZIP 和对应的 `.sha256` 文件。安装 GitHub CLI 并用 `gh auth login` 登录后，再运行校验命令。以下命令以 `v0.1.0` 为例，请将 `$archive` 和 `$tag` 改成实际下载的 ZIP 文件名和发行标签。来源验证会核对构建是否使用 GitHub 托管运行器，以及构建来源是否与标签指向的提交一致。[GitHub Releases](https://github.com/blueokanna/Crabforge/releases) · [GitHub CLI](https://cli.github.com/)

```powershell
$repo = 'blueokanna/Crabforge'
$tag = 'v0.1.0'
$archive = '.\crabforge-0.1.0-windows-x64.zip'
$expected = (Get-Content -LiteralPath "$archive.sha256" -Raw).Trim().Split(' ')[0]
if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash -ne $expected) {
    throw 'Archive SHA-256 mismatch'
}
$commit = gh api "repos/$repo/commits/$tag" --jq .sha
if ($LASTEXITCODE -or $commit -notmatch '^[0-9a-f]{40}$') { throw 'Cannot resolve release commit' }
gh attestation verify $archive --repo $repo --signer-workflow "$repo/.github/workflows/windows-release.yml" --source-digest $commit --deny-self-hosted-runners
if ($LASTEXITCODE) { throw 'Build provenance verification failed' }
```

将验证通过的 ZIP 解压到新目录。在解压目录中执行以下命令，按 `SHA256SUMS` 清单逐个校验包内文件。

```powershell
foreach ($entry in Get-Content -LiteralPath '.\SHA256SUMS' -Encoding UTF8 -ErrorAction Stop) {
    if ($entry -notmatch '^([0-9a-f]{64})  (.+)$') { throw 'Invalid SHA256SUMS entry' }
    $digest = $Matches[1]
    $path = $Matches[2]
    if ((Get-FileHash -LiteralPath $path -Algorithm SHA256 -ErrorAction Stop).Hash -ne $digest) {
        throw "File SHA-256 mismatch for $path"
    }
}
```

启动 `desktop\crabforge.exe` 时，保留同目录的 DLL 和 `data` 目录。CLI 位于解压目录根部。如系统缺少运行库，请安装 Microsoft Visual C++ x64 Redistributable。 [Microsoft Visual C++ Redistributable](https://learn.microsoft.com/zh-cn/cpp/windows/latest-supported-vc-redist)

如果该版本使用了 Authenticode，可用 `Get-AuthenticodeSignature` 查看 EXE 和 DLL 的发布者与时间戳。GitHub 构建来源证明覆盖整个 ZIP，是每个发行版的必备验证项。

## 发布新版本

保持 `Cargo.toml`、`gui/pubspec.yaml` 和 `assets/pubspec.yaml` 中的产品版本一致。从 `main` 手动运行 **Windows release**，或推送匹配的版本标签。Flutter `0.1.0+1` 对应 `v0.1.0`。更高的构建号会写入标签，例如 `v0.1.0+2`。后续发行请使用新的版本号或构建号。

如需启用 Authenticode，在 GitHub 的 `windows-release` 环境中配置以下项目。SignTool 为 EXE 和 DLL 应用 SHA-256 签名和 RFC 3161 时间戳，并在打包前验证签名。提供任一签名项后，必须一并提供完整的证书配置。

| 配置 | 类型 | 值 |
| --- | --- | --- |
| `WINDOWS_SIGNING_PFX_BASE64` | Secret | 以 Base64 编码的发布者 PFX，包含私钥和中间证书 |
| `WINDOWS_SIGNING_PFX_PASSWORD` | Secret | PFX 密码 |
| `WINDOWS_SIGNING_CERT_THUMBPRINT` | Variable | 发布者证书的 40 字符 SHA-1 指纹 |
| `WINDOWS_SIGNING_REQUIRED` | Variable | 设置为 `true` 时，每个发行版均须通过 Authenticode 签名 |

证书链必须通过运行器的 Windows Authenticode 策略验证。SHA-1 指纹用于选择证书，文件和时间戳摘要均使用 SHA-256。
