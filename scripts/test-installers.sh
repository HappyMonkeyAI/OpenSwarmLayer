#!/usr/bin/env bash
# Offline tests with fake transport/package-manager commands; no packages installed.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
fixture=$(mktemp -d)
trap 'rm -rf -- "$fixture"' EXIT
mkdir "$fixture/bin" "$fixture/downloads"
export INSTALL_TEST_FIXTURE=$fixture
export TMPDIR="$fixture/downloads"
printf 'offline installer fixture' > "$fixture/payload"
cat > "$fixture/bin/curl" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
[[ $INSTALL_TEST_CASE != unavailable ]] || exit 22
output=${!#}
asset=OpenSwarmLayer-v0.1.1-beta.1-linux-amd64.deb
if [[ $output == */SHA256SUMS ]]; then
    hash=$(sha256sum "$INSTALL_TEST_FIXTURE/payload"); hash=${hash%% *}
    [[ $INSTALL_TEST_CASE != corrupt ]] || hash=$(printf '%064d' 0)
    [[ $INSTALL_TEST_CASE != missing ]] || asset=other.deb
    printf '%s  %s\n' "$hash" "$asset" > "$output"
    [[ $INSTALL_TEST_CASE != duplicate ]] || printf '%s  %s\n' "$hash" "$asset" >> "$output"
else cp "$INSTALL_TEST_FIXTURE/payload" "$output"; fi
MOCK
cat > "$fixture/bin/dpkg" <<'MOCK'
#!/usr/bin/env bash
echo amd64
MOCK
cat > "$fixture/bin/apt-get" <<'MOCK'
#!/usr/bin/env bash
touch "$INSTALL_TEST_FIXTURE/installed"
[[ $INSTALL_TEST_CASE != installer-failure ]]
MOCK
cat > "$fixture/bin/sudo" <<'MOCK'
#!/usr/bin/env bash
exec "$@"
MOCK
cat > "$fixture/bin/uname" <<'MOCK'
#!/usr/bin/env bash
if [[ $1 == -s ]]; then echo Linux;
elif [[ $INSTALL_TEST_CASE == unsupported ]]; then echo aarch64;
else echo x86_64; fi
MOCK
chmod +x "$fixture/bin/"*
export PATH="$fixture/bin:$PATH"
for scenario in success corrupt missing duplicate unavailable installer-failure download-only unsupported; do
    export INSTALL_TEST_CASE=$scenario
    rm -f "$fixture/installed"
    args=()
    [[ $scenario != download-only ]] || args+=(--download-only)
    status=0
    bash "$root/install.sh" "${args[@]}" > "$fixture/result" 2>&1 || status=$?
    if [[ $scenario == success || $scenario == download-only ]]; then
        [[ $status == 0 ]] || { cat "$fixture/result"; exit 1; }
    else [[ $status != 0 ]] || { echo "Expected failure: $scenario"; exit 1; }; fi
    if [[ $scenario == success || $scenario == installer-failure ]]; then
        [[ -f $fixture/installed ]] || { echo "Installer not called: $scenario"; exit 1; }
    else [[ ! -f $fixture/installed ]] || { echo "Unsafe installation: $scenario"; exit 1; }; fi
    if [[ $scenario != download-only && $scenario != unsupported ]]; then
        [[ -z $(find "$fixture/downloads" -type f -print -quit) ]] || { echo "Unverified package retained: $scenario"; exit 1; }
    fi
    echo "PASS: $scenario"
done
if bash "$root/install.sh" --version '../../escape' > "$fixture/result" 2>&1; then
    echo 'Invalid version accepted.'; exit 1
fi
echo 'PASS: invalid version'
