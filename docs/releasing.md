# Tagged technical beta releases

The first release is `v0.1.1-beta.1`, a **technical pre-release** with Windows
and experimental Linux x64 packages built from the same tagged source.
GitHub now reports the repository as public; release downloads are accessible
without signing in. Publication of these files does not establish production
readiness. The scripts do not handle authentication for private repositories.
Linux packages require glibc 2.39 or
newer and target an Ubuntu 24.04 baseline; they are not older-Deepin compatible.
DEB contents/dependencies and headless startup passed in WSL. AppImage headless
startup passed in WSL and native Mint, including normal FUSE mode on Mint.
Native Linux install/UI/tray/sharing acceptance remains open.

## Version and asset contract

Use an annotated Git tag such as `v0.1.1-beta.1`, then `v0.1.1-beta.2` for a new
test build. Create a GitHub **pre-release**, never label these builds stable.
The app currently reports `0.1.1`; the beta suffix identifies the release and
its exact source commit. Bump Cargo and Tauri versions together when that changes.
Do not move an existing tag or replace published package files; issue a new beta.

Each release carries these assets:

| Asset | Purpose |
| --- | --- |
| `OpenSwarmLayer-v0.1.1-beta.1-windows-x64-setup.exe` | Production-identity Windows NSIS installer |
| `OpenSwarmLayer-v0.1.1-beta.1-linux-amd64.deb` | Optional native Linux DEB, only when built and checked |
| `OpenSwarmLayer-v0.1.1-beta.1-linux-x86_64.AppImage` | Experimental portable Linux x64 app; still subject to host-library compatibility |
| `install.ps1` | Windows installer script from the tagged commit |
| `install.sh` | Debian/Ubuntu-family Linux installer script from the tagged commit |
| `SHA256SUMS` | ASCII SHA-256 hashes, two spaces, exact filename and Unix LF line endings |

The scripts accept only explicit `vX.Y.Z-beta.N` tags. They do not choose a
moving "latest" build. Each script's default version must match its release.
Do not advertise the Linux command when that release has no Linux package.
Other architectures, macOS and RPM-based Linux are not supported by these scripts.
The Linux script installs the DEB; AppImage is a manual download/run option.

## Prepare a release

1. Review and commit the intended changes, keeping models, credentials, build
   outputs and machine-local files out of Git. The current checkout contains
   uncommitted implementation and pre-existing work: review it before tagging.
   A tag on the old HEAD would not contain the current tested download flow.
2. Update the default version in both install scripts and the example commands
   for this beta. Run `scripts/test-installers.ps1` on Windows and
   `bash scripts/test-installers.sh` on Linux/WSL. Run the repository's Rust
   checks and applicable package/live acceptance.
3. Build from the reviewed commit using the default Tauri configuration:
   `cargo tauri build --bundles nsis` on Windows; `cargo tauri build --bundles deb`
   on compatible Linux, both from `desktop/src-tauri`. Do not upload the isolated
   acceptance package or its executable: it has a different name/profile.
4. Copy the resulting packages to an ignored release staging directory and rename
   them according to the table. Copy `install.ps1` and `install.sh` from the
   reviewed commit. Only include the Linux package if its evidence supports the
   claims in the notes. Record platform, package dependencies and unsigned status.
5. Generate `SHA256SUMS` over the package and script files. For example, in
   PowerShell, with `$releaseDir` set to the staging directory:

   ```powershell
   $lines = Get-ChildItem -LiteralPath $releaseDir -File |
       Where-Object Name -ne 'SHA256SUMS' |
       Sort-Object Name |
       ForEach-Object { '{0}  {1}' -f (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant(), $_.Name }
   [IO.File]::WriteAllText((Join-Path $releaseDir 'SHA256SUMS'), ($lines -join "`n") + "`n", [Text.Encoding]::ASCII)
   ```

6. Confirm `git status --short` is empty. Tag the reviewed commit and push the
   branch and tag when publishing is authorized:

   ```sh
   git tag -a v0.1.1-beta.1 -m "OpenSwarmLayer 0.1.1 technical test beta 1"
   git push origin master
   git push origin v0.1.1-beta.1
   ```

7. Create a draft GitHub pre-release using `gh release create v0.1.1-beta.1
   --verify-tag --draft --prerelease --title "0.1.1 beta 1 — technical testing"
   --notes-file <notes-file> <asset-files>`. Supply explicit asset paths.
   Review all assets/checksums, tag commit and notes before publishing the draft.
   A private release can be tested by invited users through authenticated GitHub
   downloads; these scripts do not handle private-repository authentication.
8. Once accessible, exercise the actual hosted scripts with download-only first,
   then install, launch and uninstall on a test machine. Fixture tests are not
   evidence of successful hosted installation. Preserve test results in PROGRESS.

Release notes must begin with **Technical test beta — not ready for public
release**, identify the source commit, list available platforms and checksums,
and describe current network setup and platform gaps. Link the beta guide.
Current-package acceptance uses an isolated Windows identity; default-profile
upgrade and native Linux desktop acceptance still have gaps.

## Install commands after publishing

These commands use the public `v0.1.1-beta.1` release assets.
They execute a remote script. Download and read the script first if you want to
inspect it before execution. The pinned tag makes the selected beta explicit.

Windows PowerShell:

```powershell
irm https://github.com/SPhillips1337/OpenSwarmLayer/releases/download/v0.1.1-beta.1/install.ps1 | iex
```

Debian/Ubuntu-family Linux x64, when the DEB is available:

```sh
curl --proto '=https' --proto-redir '=https' -fsSL https://github.com/SPhillips1337/OpenSwarmLayer/releases/download/v0.1.1-beta.1/install.sh | bash
```

For inspection and download-only testing, save the script to a new file, read it,
then run `./install.ps1 -DownloadOnly` or `bash install.sh --download-only`.
Use `-Version v0.1.1-beta.2` or `--version v0.1.1-beta.2` to select another beta.
Windows shows the normal installer; Linux asks through apt/sudo. No services,
firewall changes, signing keys or application launch are configured by the scripts.
Linux installation may install the package's declared dependencies.

Checksums detect mismatched/corrupt downloads; they are fetched from the same
GitHub release and are not an independent publisher signature. HTTPS and the
repository/release account remain trust boundaries. Windows packages are
currently unsigned; the scripts do not suppress operating-system warnings.

GitHub supports tagged release assets and pre-release status; see the
[official release documentation](https://docs.github.com/en/rest/releases/releases).
