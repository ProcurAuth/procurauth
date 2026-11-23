# shellcheck shell=bash disable=SC2034
# Shared settings of the scripts that run Docker. Source this file. Do not run it.
# It sets: root, image, registry_volume, target_volume, uid and gid.
# The scripts that source this file use these variables, so SC2034 does not apply.

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# The tag is the start of the SHA-256 of Dockerfile.dev. A changed recipe gives a new image.
image="procurauth-dev:$(sha256sum "$root/scripts/Dockerfile.dev" | cut -c1-12)"
registry_volume="procurauth-cargo-registry"
# One target volume for each checkout: a worktree never shares build files with another.
target_volume="procurauth-target-$(printf %s "$root" | sha1sum | cut -c1-8)"
uid="$(id -u)"
gid="$(id -g)"

# Build the development image one time for each version of Dockerfile.dev: rust:1-bookworm with fmt and clippy.
ensure_image() {
    if ! docker image inspect "$image" >/dev/null 2>&1; then
        docker build --quiet --tag "$image" --file "$root/scripts/Dockerfile.dev" "$root/scripts" >/dev/null
    fi
}
