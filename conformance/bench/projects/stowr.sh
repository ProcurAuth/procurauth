#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Start the released Stowr in a container with no network, and run the series U against it. See ../README.md.
# Input: BENCH_BIN, BENCH_SRC, BENCH_TOOLS, BENCH_KEYS, BENCH_OUT.
set -euo pipefail

scratch="$(mktemp -d)"
name="bench-stowr-$$"
cleanup() {
    docker rm -f "$name" >/dev/null 2>&1 || true
    rm -rf "$scratch"
}
trap cleanup EXIT INT TERM

work="$scratch/work"
mkdir -p "$work/data/reports/2026"
cp "$BENCH_KEYS/test-edge-ed25519.pem" "$BENCH_KEYS/test-edge-es256.pem" "$BENCH_KEYS/edge-jwks.json" "$work/"
tr -d '\n' <"$BENCH_KEYS/pdp-credential.txt" >"$work/pdp-token"
cp "$BENCH_SRC/test/conformance/config.hcl" "$BENCH_SRC/test/conformance/profile.json" "$work/"
printf 'hello stowr\n' >"$work/data/reports/a.txt"
cp "$BENCH_BIN" "$scratch/stowr"
chmod 0755 "$scratch" "$work"
chmod -R a+rwX "$work"

docker run --detach --name "$name" --network none --read-only --user "$(id -u):$(id -g)" \
    --volume "$scratch/stowr:/stowr:ro" --volume "$work:/conformance:ro" --volume "$work/data:/data" \
    alpine:3.22 /stowr -c /conformance/config.hcl >/dev/null
sleep 1
if [ "$(docker inspect -f '{{.State.Running}}' "$name")" != true ]; then
    docker logs --tail 40 "$name" >&2 || true
    echo "ERROR: Stowr did not stay running." >&2
    exit 2
fi

# The runner runs in the network namespace of Stowr: it acts as the edge and as the PDP.
set +e
docker run --rm --init --network "container:$name" --user "$(id -u):$(id -g)" \
    --volume "$work:/work:ro" --volume "$BENCH_TOOLS:/tools:ro" debian:stable-slim \
    timeout 600 bash -c '
for i in $(seq 1 100); do (exec 3<>/dev/tcp/127.0.0.1/8080) 2>/dev/null && break; sleep 0.1; done
exec /tools/access-conformance --role pep --series U --profile /work/profile.json --url http://127.0.0.1:8080 --format json' \
    >"$BENCH_OUT/U.json" 2>"$BENCH_OUT/U.err"
code=$?
set -e
# The runner gives 0 or 1 for a run that ended. Another code is a problem of the set-up.
if [ "$code" -ne 0 ] && [ "$code" -ne 1 ]; then
    tail -n 20 "$BENCH_OUT/U.err" >&2
    exit 2
fi
