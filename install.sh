#!/usr/bin/env bash
# OpenSwarmLayer technical test beta installer: Debian/Ubuntu-family Linux x64.
set -euo pipefail

main() {
    local version='v0.1.1-beta.1' download_only=0
    while (($#)); do
        case "$1" in
            --version) [[ $# -ge 2 ]] || { echo 'Missing --version value.' >&2; return 1; }; version=$2; shift 2 ;;
            --download-only) download_only=1; shift ;;
            --help) echo 'Usage: bash install.sh [--version vX.Y.Z-beta.N] [--download-only]'; return 0 ;;
            *) echo "Unknown argument: $1" >&2; return 1 ;;
        esac
    done
    [[ $version =~ ^v[0-9]+\.[0-9]+\.[0-9]+-beta\.[0-9]+$ ]] || { echo 'Expected a beta tag such as v0.1.1-beta.1.' >&2; return 1; }
    [[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || { echo 'Only Linux x64 is supported.' >&2; return 1; }
    local tool
    for tool in curl sha256sum mktemp awk; do
        command -v "$tool" >/dev/null || { echo "Required command missing: $tool" >&2; return 1; }
    done
    command -v dpkg >/dev/null && command -v apt-get >/dev/null || { echo 'This installer requires a Debian/Ubuntu-family system with apt-get and dpkg.' >&2; return 1; }
    [[ $(dpkg --print-architecture) == amd64 ]] || { echo 'Only amd64 DEB packages are supported.' >&2; return 1; }
    if (( ! download_only && EUID != 0 )); then
        command -v sudo >/dev/null || { echo 'sudo is needed to install the DEB; use --download-only to download it.' >&2; return 1; }
    fi
    echo "OpenSwarmLayer $version - technical test beta; not ready for public release."
    echo 'Native Linux desktop acceptance is incomplete. Close the app before upgrading.'
    local asset="OpenSwarmLayer-$version-linux-amd64.deb"
    local base_url="https://github.com/SPhillips1337/OpenSwarmLayer/releases/download/$version"
    local download_dir expected actual count
    download_dir=$(mktemp -d -t OpenSwarmLayer.XXXXXXXX)
    # A subshell keeps cleanup traps and shell options out of the caller.
    (
        trap 'rm -f -- "$download_dir/SHA256SUMS" "$download_dir/$asset"; rmdir -- "$download_dir"' EXIT
        curl --proto '=https' --proto-redir '=https' --tlsv1.2 -fsSL --connect-timeout 20 --max-time 600 "$base_url/SHA256SUMS" -o "$download_dir/SHA256SUMS" || { echo 'Cannot download checksums: release missing, private or unreachable.' >&2; exit 1; }
        curl --proto '=https' --proto-redir '=https' --tlsv1.2 -fsSL --connect-timeout 20 --max-time 600 "$base_url/$asset" -o "$download_dir/$asset" || { echo 'Cannot download package: release missing, private or unreachable.' >&2; exit 1; }
        expected=$(awk -v name="$asset" 'length($1)==64 && $1 !~ /[^a-fA-F0-9]/ && $2==name && NF==2 {print tolower($1)}' "$download_dir/SHA256SUMS")
        count=$(printf '%s\n' "$expected" | awk 'NF {n++} END {print n+0}')
        [[ $count == 1 ]] || { echo 'Missing or duplicate package checksum.' >&2; exit 1; }
        actual=$(sha256sum "$download_dir/$asset"); actual=${actual%% *}
        [[ $actual == "$expected" ]] || { echo 'Package checksum mismatch; installation refused.' >&2; exit 1; }
        echo "SHA-256 verified: $asset"
        if (( download_only )); then
            rm -f -- "$download_dir/SHA256SUMS"
            trap - EXIT
            echo "Verified package saved to $download_dir/$asset"
        else
            # apt resolves the DEB's declared dependencies and asks before making changes.
            # Pass a readable file to apt's sandboxed downloader.
            chmod 755 "$download_dir"
            chmod 644 "$download_dir/$asset"
            if (( EUID == 0 )); then apt-get install "$download_dir/$asset";
            else sudo apt-get install "$download_dir/$asset"; fi
            echo 'Installation finished. Launch OpenSwarmLayer from your applications menu.'
        fi
    )
}

# Wrap all work: a truncated piped download cannot execute a partial main function.
main "$@"
