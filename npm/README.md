# OpenSwarmLayer npm installer

**Technical test beta — not ready for production.** This small, dependency-free
package installs the native Rust/Tauri model-sharing app. Node.js does not run
the model-sharing engine. Requires Node.js 20 or newer.

Once published to npm under the `beta` dist-tag:

```sh
npx openswarmlayer@beta install
```

Registry publication is pending authenticated maintainer access. Until then,
use the npm tarball on the GitHub beta release:

```sh
npx --package=https://github.com/HappyMonkeyAI/OpenSwarmLayer/releases/download/v0.1.1-beta.1/openswarmlayer-0.1.1-beta.2.tgz openswarmlayer install
```

Replace `install` with `download` to fetch without installing, or with `--help`
to inspect supported options. On Linux, add `--format appimage` after `download`
for the portable package. This URL points to a fixed wrapper version.

Windows x64 opens the normal unsigned installer. Debian/Ubuntu-family Linux x64
uses apt/sudo and requires glibc 2.39 or newer. Close the app before upgrading.
macOS, ARM and musl systems are unsupported. Full Linux desktop, tray and sharing
acceptance is incomplete.

Download without installing:

```sh
npx openswarmlayer@beta download
npx openswarmlayer@beta download --format appimage
```

AppImage download is Linux-only. Make the downloaded file executable with
`chmod +x <file>` and run it; use `--appimage-extract-and-run` if FUSE is unavailable.
No native application is installed by `npm install` or the help command. There
are no installation lifecycle hooks. Installation is an explicit CLI action.

Each npm version pins a native GitHub release, exact sizes and SHA-256 hashes.
Files are downloaded to a temporary directory and checked before an installer
can run. Hashes protect against mismatched release assets; trusting this npm
package and its publisher remains necessary. The wrapper does not configure
firewalls, model caches, signing keys, services or automatic application launch.

Current native release: `v0.1.1-beta.1`, source
`dec8e5099e782c9ddd6be75e8fecb3d65907989f`. The npm wrapper may be versioned
independently; it does not rebuild the native app. Publish beta builds with
`npm publish --tag beta`, keeping the stable `latest` channel unset.

See the [project README](https://github.com/HappyMonkeyAI/OpenSwarmLayer#readme)
for importing, downloading and re-sharing GGUF/Safetensors models. Software
license is MIT; model files retain their own licenses.
