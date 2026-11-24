#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Start the released ChDash and a ClickHouse server in a closed network, and run the series U against ChDash. See ../README.md.
# Input: BENCH_BIN, BENCH_SRC, BENCH_TOOLS, BENCH_KEYS, BENCH_OUT.
#
# The set-up is the one of tests/conformance/run.sh of ChDash, with the default profile and the default configuration. It uses the
# released binary. The network has no route to the outside. The runner is the edge, the PDP and the client.
set -euo pipefail

ch_image="${CLICKHOUSE_IMAGE:-clickhouse@sha256:d0e74e9970998f92c0490c23345259a539550eff24f551dbe385d3cd29481f74}"
image="debian:trixie-slim"
ch_password="conformance-test-password"
write_password="conformance-write-password"

scratch="$(mktemp -d)"
network="bench-chdash-$$"
ch_name="bench-chdash-ch-$$"
app_name="bench-chdash-app-$$"
cleanup() {
    docker rm -f "$ch_name" "$app_name" >/dev/null 2>&1 || true
    docker network rm "$network" >/dev/null 2>&1 || true
    rm -rf "$scratch"
}
trap cleanup EXIT INT TERM
die() { echo "ERROR: $*" >&2; exit 2; }

# The work directory has the keys, the profile, the configuration and the secrets. The containers mount it read-only.
work="$scratch/work"
mkdir -p "$work"
cp "$BENCH_KEYS/test-edge-ed25519.pem" "$BENCH_KEYS/test-edge-es256.pem" "$work/"
# The configuration of ChDash names the public keys of the test edge. The keys of ProcurAuth must be the keys of the repository.
cp "$BENCH_SRC/tests/conformance/identity-keys.json" "$work/identity-keys.json"
cmp -s "$work/identity-keys.json" "$BENCH_KEYS/edge-jwks.json" || die "identity-keys.json is not the public set of the test keys of ProcurAuth"
cp "$BENCH_SRC/tests/conformance/profile.json" "$BENCH_SRC/tests/config/access.conformance.hcl" "$work/"
mv "$work/access.conformance.hcl" "$work/config.hcl"
# ChDash and the runner share the credential of the PDP.
tr -d '\n' <"$BENCH_KEYS/pdp-credential.txt" >"$work/pdp-token"
printf '%s' "$ch_password" >"$work/ch-password"
printf '%s' "$write_password" >"$work/ch-write-password"
cp "$BENCH_BIN" "$scratch/chdash"
chmod 0755 "$scratch" "$work" "$scratch/chdash"
chmod 0644 "$work"/*

# The network is internal: no container has a route to the outside.
docker network create --internal "$network" >/dev/null
docker run --detach --name "$ch_name" --network "$network" --network-alias clickhouse-conformance \
    --env CLICKHOUSE_USER=chdash --env "CLICKHOUSE_PASSWORD=$ch_password" --env CLICKHOUSE_DEFAULT_ACCESS_MANAGEMENT=1 \
    "$ch_image" >/dev/null
ready=0
for _ in $(seq 1 90); do
    if docker exec "$ch_name" clickhouse-client --user chdash --password "$ch_password" -q 'SELECT 1' >/dev/null 2>&1; then
        ready=1
        break
    fi
    sleep 1
done
if [ "$ready" != 1 ]; then
    docker logs --tail 20 "$ch_name" >&2 || true
    die "ClickHouse did not start."
fi
# The write account of the tool run_write_query.
docker exec "$ch_name" clickhouse-client --user chdash --password "$ch_password" --multiquery -q \
    "CREATE USER chdash_write IDENTIFIED WITH plaintext_password BY '$write_password'; GRANT CREATE TABLE, SELECT ON default.* TO chdash_write" \
    >/dev/null || die "The write account of ClickHouse could not be made."

docker run --detach --name "$app_name" --network "$network" --network-alias chdash-conformance \
    --read-only --tmpfs /tmp:rw,mode=1777 --user "$(id -u):$(id -g)" \
    --volume "$scratch/chdash:/chdash:ro" --volume "$work:/conformance:ro" \
    "$image" /chdash --config /conformance/config.hcl >/dev/null
# The runner runs in the same network. It listens as the PDP on conformance-pdp:8181.
set +e
docker run --rm --init --network "$network" --network-alias conformance-pdp --user "$(id -u):$(id -g)" \
    --volume "$work:/work:ro" --volume "$BENCH_TOOLS:/tools:ro" --workdir /work "$image" \
    timeout 600 bash -c '
ready=0
for i in $(seq 1 120); do
    if (exec 3<>/dev/tcp/chdash-conformance/8080 && printf "GET /readyz HTTP/1.0\r\n\r\n" >&3 && head -n 1 <&3 | grep -q " 200 ") 2>/dev/null; then ready=1; break; fi
    sleep 0.5
done
[ "$ready" = 1 ] || { echo "ChDash did not become ready." >&2; exit 3; }
exec /tools/access-conformance --role pep --series U --profile /work/profile.json --url http://chdash-conformance:8080 --format json' \
    >"$BENCH_OUT/U.json" 2>"$BENCH_OUT/U.err"
code=$?
set -e
# The runner gives 0 or 1 for a run that ended. Another code is a problem of the set-up.
if [ "$code" -ne 0 ] && [ "$code" -ne 1 ]; then
    tail -n 20 "$BENCH_OUT/U.err" >&2
    docker logs --tail 20 "$app_name" 2>&1 | cut -c1-300 >&2 || true
    exit 2
fi
