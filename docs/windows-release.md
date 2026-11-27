# Windows x64 发布

向 `main` 推送、提交拉取请求或手动运行时，[Windows 构建](../.github/workflows/windows-build.yml)会审计锁定的 Rust 依赖，执行 Rust 和 Flutter 检查，再编译桌面及两个 CLI。未签名的构建产物保存在该次运行的 `crabforge-windows-x64-ci` 中。

[签名发布工作流](../.github/workflows/windows-release.yml)从发行提交重新构建。签名任务独占发布密钥，使用 Windows SignTool 为 EXE 和 DLL 添加 SHA-256 Authenticode 签名以及 DigiCert RFC 3161 时间戳。全部验签后才生成包内清单和 ZIP 校验文件。发布任务先把两项附件传入 Release 草稿，上传完成后再公开。

## 配置签名

在仓库的 **Settings → Environments** 创建 `windows-release`，添加下表两项环境密钥和一项环境变量。部署来源应限制为默认分支和受信任的发行标签。

| 设置 | 类型 | 值 |
| --- | --- | --- |
| `WINDOWS_SIGNING_PFX_BASE64` | Secret | 发布者 Windows 代码签名 PFX 的 Base64，包含私钥和中间证书 |
| `WINDOWS_SIGNING_PFX_PASSWORD` | Secret | PFX 密码 |
| `WINDOWS_SIGNING_CERT_THUMBPRINT` | Variable | 发布者证书的 40 位 SHA-1 指纹，去掉空格 |

证书链须通过 GitHub Windows runner 上的 Authenticode 验证。指纹用于选择证书，文件摘要和时间戳都使用 SHA-256。签名任务需要可导出的 PFX。采用不可导出硬件密钥的证书须经其服务商的签名接口使用。

## 发布和核验

同步 `Cargo.toml`、`gui/pubspec.yaml` 和 `assets/pubspec.yaml` 中的产品版本。在 `main` 手动运行 **Signed Windows release**，或推送匹配标签。Flutter `0.1.0+1` 对应 `v0.1.0`，更高构建号会保留在标签中，例如 `v0.1.0+2`。已经公开的版本不可覆盖。

下载 ZIP 和 `.sha256`。使用 `Get-FileHash -Algorithm SHA256 <下载的ZIP>` 核对发布摘要。校验后解压，在解压根目录执行下列命令。签名输出列出各二进制文件的发布者和时间戳证书。[GitHub Releases](https://github.com/blueokanna/Crabforge/releases)

```powershell
$manifest = @(Get-Content -LiteralPath SHA256SUMS -Encoding UTF8 -ErrorAction Stop)
if (!$manifest.Count) { throw 'SHA256SUMS is empty' }
foreach ($entry in $manifest) {
    $expected, $file = $entry -split '  ', 2
    if ((Get-FileHash -LiteralPath $file -Algorithm SHA256 -ErrorAction Stop).Hash -ne $expected) {
        throw "SHA-256 mismatch for $file"
    }
}

$binaries = Get-ChildItem -Recurse -File -Force | Where-Object Extension -In '.exe', '.dll'
foreach ($binary in $binaries) {
    $signature = Get-AuthenticodeSignature -LiteralPath $binary.FullName
    if ($signature.Status -ne 'Valid' -or !$signature.TimeStamperCertificate) {
        throw "Invalid signature or missing timestamp for $($binary.FullName)"
    }
    $signature | Select-Object Path, Status, SignerCertificate, TimeStamperCertificate
}
```

运行 `desktop/crabforge.exe`，保持同目录的 DLL 和 `data` 目录完整。两个 CLI 位于解压根目录。

桌面和 CLI 需要 Microsoft Visual C++ x64 运行库。若 Windows 尚未安装，请安装当前版本。[Microsoft Visual C++ Redistributable](https://learn.microsoft.com/zh-cn/cpp/windows/latest-supported-vc-redist)

## 桌面外观

Windows DWM 强调色用于生成 Flutter Material 3 Expressive 的浅色、深色和高对比度配色。系统未提供强调色时使用内置铜色。应用操作和导航使用 Google 官方 Material Symbols Rounded。Crabforge 图标以及供应商和客户端标志保留各自品牌图案。

在顶栏的字体菜单选择 Google Sans 或 HarmonyOS Sans SC。Google Sans 模式下，拉丁字符使用 Google Sans，汉字使用 HarmonyOS Sans SC。HarmonyOS Sans SC 模式下，两类字符均直接使用该字体。韩文缺字由 Noto Sans KR 补齐。字体偏好保存在 Crabforge 数据目录，保存后立即生效。[字体来源与校验值](../assets/fonts/manifest.json)

[Windows 构建与桌面外观验收](validation/cf-windows-469e877/README.md)
