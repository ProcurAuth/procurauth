#!/usr/bin/env bash
# Build the programs as static Linux files with the musl target of the host architecture, inside Docker.
# The script accepts an x86_64 or aarch64 host. It stops on a different architecture.
# Use: scripts/static.sh            (build the seven programs)
#      scripts/static.sh --check    (build, then check that each file is static and runs)
# The files are in the target volume, in /cargo-target/<architecture>-unknown-linux-musl/release.
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/docker-env.sh"

case "$(uname -m)" in
    x86_64 | amd64) target=x86_64-unknown-linux-musl ;;
    aarch64 | arm64) target=aarch64-unknown-linux-musl ;;
    *) echo "unsupported architecture: $(uname -m) (use x86_64 or aarch64)" >&2; exit 2 ;;
esac

ensure_image

# The tag is the start of the SHA-256 of Dockerfile.dev and Dockerfile.static together.
# A change in one of the two recipes gives a new image.
static_image="procurauth-static:$(cat "$root/scripts/Dockerfile.dev" "$root/scripts/Dockerfile.static" | sha256sum | cut -c1-12)"
if ! docker image inspect "$static_image" >/dev/null 2>&1; then
    # Dockerfile.static starts FROM a development image.
    # The script replaces that line with the development image of this checkout.
    if ! grep -qE '^FROM procurauth-dev:' "$root/scripts/Dockerfile.static"; then
        echo "scripts/Dockerfile.static: no line 'FROM procurauth-dev:<tag>'" >&2
        exit 1
    fi
    sed -E "s|^FROM procurauth-dev:[^[:space:]]+|FROM $image|" "$root/scripts/Dockerfile.static" \
        | docker build --quiet --tag "$static_image" --file - "$root/scripts" >/dev/null
fi

# Same volumes as scripts/cargo.sh. Run `scripts/cargo.sh --version` one time to create them.
programs=(keycheck access-token access-lint fake-pdp access-conformance reference-service reference-edge)
flags=()
for program in "${programs[@]}"; do flags+=(-p "$program"); done

# The command in the container starts with `set -e`: a failed build or a failed check stops it
# with a status that is not zero. Old files in the target volume never hide a failed build.
check=""
if [ "${1:-}" = "--check" ]; then
    check="
cd /cargo-target/$target/release
for program in ${programs[*]}; do
    if ! file \"\$program\" | grep -q 'static'; then echo \"not static: \$program\" >&2; exit 1; fi
    if ldd \"\$program\" 2>&1 | grep -q '=>'; then echo \"dynamic: \$program\" >&2; exit 1; fi
    \"./\$program\" --version
done"
fi

exec docker run --rm \
    --user "$uid:$gid" \
    --volume "$root:/work" \
    --volume "$registry_volume:/cargo-home" \
    --volume "$target_volume:/cargo-target" \
    --workdir /work \
    --env CARGO_HOME=/cargo-home \
    --env CARGO_TARGET_DIR=/cargo-target \
    --env HOME=/cargo-home \
    --env CARGO_TERM_COLOR=never \
    "$static_image" bash -c "set -e; cargo build --release --locked --target $target ${flags[*]} $check"
