# Windows x64 releases

[简体中文](windows-release.md)

The Windows build workflow runs on `main` pushes and pull requests. It runs the Rust and Flutter checks, builds the CLI and desktop, and stages the portable distribution as `crabforge-windows-x64-ci`. The release workflow rebuilds the release commit, creates a ZIP with an internal `SHA256SUMS` manifest, and attests the ZIP through GitHub Actions. The publish job verifies the archive hash and provenance, uploads both release assets to a draft, checks the uploaded digests, and publishes the draft. [Build workflow](../.github/workflows/windows-build.yml) · [Release workflow](../.github/workflows/windows-release.yml)

## Verify and run

Download the ZIP and its `.sha256` sidecar from the same release. Install GitHub CLI and sign in with `gh auth login` before running the checks. The example uses `v0.1.0`. Set `$archive` and `$tag` to the ZIP filename and release tag you downloaded. The provenance check verifies that GitHub hosted the build and that the source matches the commit behind the tag. [GitHub Releases](https://github.com/blueokanna/Crabforge/releases) · [GitHub CLI](https://cli.github.com/)

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

Extract the verified archive into a new directory. Run the following commands from that directory to check each packaged file against `SHA256SUMS`.

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

Run `desktop\crabforge.exe` with the adjacent DLLs and `data` directory in place. The CLI binaries are at the archive root. Install the Microsoft Visual C++ x64 Redistributable if the PC lacks the runtime. [Microsoft Visual C++ Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist)

For builds with Authenticode, use `Get-AuthenticodeSignature` on the EXEs and DLLs to inspect the publisher and timestamp. GitHub build provenance applies to the entire ZIP and is required for every release.

## Publish a release

Keep the versions in `Cargo.toml`, `gui/pubspec.yaml`, and `assets/pubspec.yaml` aligned. Run **Windows release** from `main` or push the matching version tag. Flutter `0.1.0+1` maps to `v0.1.0`. Higher build numbers are included in the tag, such as `v0.1.0+2`. Select a new version or build number for subsequent releases.

To enable Authenticode, configure these settings in the `windows-release` GitHub environment. SignTool applies SHA-256 Authenticode signatures and RFC 3161 timestamps to EXE and DLL files and verifies them before packaging. If any signing setting is supplied, the full certificate configuration is required.

| Setting | Scope | Value |
| --- | --- | --- |
| `WINDOWS_SIGNING_PFX_BASE64` | Secret | Base64-encoded publisher PFX with its private key and intermediate certificates |
| `WINDOWS_SIGNING_PFX_PASSWORD` | Secret | PFX password |
| `WINDOWS_SIGNING_CERT_THUMBPRINT` | Variable | Publisher certificate’s 40-character SHA-1 thumbprint |
| `WINDOWS_SIGNING_REQUIRED` | Variable | `true` to require Authenticode on every release |

The certificate chain must validate under Windows Authenticode policy on the runner. The SHA-1 thumbprint identifies the certificate. File and timestamp digests use SHA-256.
