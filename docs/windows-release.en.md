# Windows x64 releases

The [Windows build](../.github/workflows/windows-build.yml) runs on pushes to `main`, pull requests, and manual dispatch. It checks the locked Rust dependency graph, runs Rust and Flutter checks, and builds the two CLI entry points and the Flutter desktop. The run stores its unsigned distribution in `crabforge-windows-x64-ci`.

The [signed release workflow](../.github/workflows/windows-release.yml) builds the candidate again from its release commit. The signing job is the only job with access to signing material. Windows SignTool applies SHA-256 Authenticode signatures and DigiCert RFC 3161 timestamps to the EXEs and DLLs. The job verifies every signature before writing the package manifest and ZIP checksum. The publishing job attaches the ZIP and its checksum to a draft Release, then publishes it once both uploads succeed.

## Signing setup

In the repository's **Settings → Environments**, create `windows-release`. Add the two environment secrets and one environment variable below. Limit deployments to the default branch and trusted release tags.

| Setting | Type | Value |
| --- | --- | --- |
| `WINDOWS_SIGNING_PFX_BASE64` | Secret | Base64 of the publisher's Windows code-signing PFX, including its private key and intermediate certificates |
| `WINDOWS_SIGNING_PFX_PASSWORD` | Secret | PFX password |
| `WINDOWS_SIGNING_CERT_THUMBPRINT` | Variable | Publisher certificate's 40-character SHA-1 thumbprint, without spaces |

Use a code-signing certificate whose chain validates under the Windows Authenticode policy on the GitHub-hosted runner. The thumbprint selects the certificate. The file digest and timestamp both use SHA-256. The signing job requires exportable PFX material. A hardware-backed, non-exportable key must be used through its provider's signing service.

## Publish and verify

Keep `Cargo.toml`, `gui/pubspec.yaml`, and `assets/pubspec.yaml` on the same product version. Run **Signed Windows release** from `main` or push the matching tag. Flutter `0.1.0+1` maps to `v0.1.0`. A higher Flutter build number is included in the tag, as in `v0.1.0+2`. A published release is write-once.

Download the ZIP and its `.sha256` file from [GitHub Releases](https://github.com/blueokanna/Crabforge/releases). Compare the result of `Get-FileHash -Algorithm SHA256 <downloaded.zip>` with the published hash. Extract the ZIP and launch `desktop/crabforge.exe`. Keep its neighboring DLLs and `data` directory together. The two CLI executables are at the archive root.

The desktop and CLI require the Microsoft Visual C++ x64 runtime. Install the current redistributable on a Windows PC that does not already have it. [Microsoft Visual C++ Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist)

## Desktop appearance

Windows' DWM accent seeds Flutter's Material 3 Expressive light, dark, and high-contrast schemes. A bundled copper seed is used when Windows supplies no accent. Crabforge uses Google's Material Symbols Rounded for its UI actions and navigation. The application mark and provider and client logos retain their branding.

Use the app bar's font menu to choose Google Sans or HarmonyOS Sans SC. With Google Sans selected, Latin text uses Google Sans and Han characters use HarmonyOS Sans SC. HarmonyOS Sans SC handles both directly when selected. Noto Sans KR provides Hangul fallback. The preference is saved in the Crabforge data directory and applies as soon as the choice is saved. [Font provenance and checksums](../assets/fonts/manifest.json)

[Windows build and desktop appearance validation](validation/cf-windows-469e877/README.md)
