#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Start the released Hune and a ClickHouse in containers with no network, and run the series U against Hune. See ../README.md.
# Input: BENCH_BIN, BENCH_SRC, BENCH_TOOLS, BENCH_KEYS, BENCH_OUT.
set -euo pipefail

scratch="$(mktemp -d)"
hune_name="bench-hune-$$"
ch_name="bench-hune-ch-$$"
cleanup() {
    docker rm -f -v "$ch_name" "$hune_name" >/dev/null 2>&1 || true
    rm -rf "$scratch"
}
trap cleanup EXIT INT TERM

# The tag of Hune pins the ClickHouse image in tests/images.lock. Read the line "clickhouse-lts".
ch_image="$(awk '$1 == "clickhouse-lts" { print $2; exit }' "$BENCH_SRC/tests/images.lock")"
if [ -z "$ch_image" ]; then
    echo "ERROR: no line clickhouse-lts in tests/images.lock." >&2
    exit 2
fi

# The volume of the fixture, the keys, the profile and the configuration.
# The credential of the PDP comes from Hune, because config.hcl and profile.json name this file.
conf="$BENCH_SRC/tests/conformance"
work="$scratch/work"
mkdir -p "$work/data/dashboards"
cp -r "$conf/fixture/conf" "$work/data/dashboards/conf"
printf '{\n  "format_version": 1,\n  "title": "Conformance"\n}\n' >"$work/data/dashboards/conf/_folder.json"
for f in config.hcl jwks.json pdp-credential.txt clickhouse_password profile.json; do
    cp "$conf/$f" "$work/$f"
done
cp "$BENCH_KEYS/test-edge-ed25519.pem" "$BENCH_KEYS/test-edge-es256.pem" "$work/"
cp "$BENCH_BIN" "$scratch/hune"
# Hune runs as the owner of these files. It refuses a volume that others can write, so do not give the write bit to others.
chmod 0755 "$scratch"
chmod -R u+rwX,go+rX,go-w "$work"

# ClickHouse: no network. It owns the network namespace, so Hune, ClickHouse and the runner all use 127.0.0.1.
# ClickHouse starts first: Hune then finds its source at once and does not wait for the next probe (30 s).
docker run --detach --name "$ch_name" --network none \
    --volume "$BENCH_SRC/tests/fixtures/clickhouse/init.sql:/docker-entrypoint-initdb.d/01-init.sql:ro" \
    --volume "$conf/init-conf.sql:/docker-entrypoint-initdb.d/02-conf.sql:ro" \
    "$ch_image" >/dev/null

# Wait until ClickHouse has the table of the fixture. The start has 90 tries of 1 second.
ready=0
for _ in $(seq 1 90); do
    if docker exec "$ch_name" clickhouse-client -q 'SELECT count() FROM analytics.conf_rows' >/dev/null 2>&1; then
        ready=1
        break
    fi
    sleep 1
done
if [ "$ready" -ne 1 ]; then
    docker logs --tail 40 "$ch_name" >&2 || true
    echo "ERROR: ClickHouse did not start." >&2
    exit 2
fi

# Hune: in the network namespace of ClickHouse, a read-only root file system. The binary and the volume are the only mounts.
docker run --detach --name "$hune_name" --network "container:$ch_name" --read-only --user "$(id -u):$(id -g)" \
    --volume "$scratch/hune:/hune:ro" --volume "$work:/conformance:ro" --volume "$work/data:/data" \
    alpine:3.22 /hune serve --config /conformance/config.hcl >/dev/null

# Wait until Hune is ready and its source is up. The check runs in the container of ClickHouse: it has wget.
ready=0
for _ in $(seq 1 60); do
    if [ "$(docker inspect -f '{{.State.Running}}' "$hune_name")" != true ]; then
        break
    fi
    if docker exec "$ch_name" wget -qO- http://127.0.0.1:9090/readyz 2>/dev/null | grep -q '"ready":true.*"conf":"up"'; then
        ready=1
        break
    fi
    sleep 1
done
if [ "$ready" -ne 1 ]; then
    docker logs --tail 40 "$hune_name" >&2 || true
    echo "ERROR: Hune did not become ready." >&2
    exit 2
fi

# The runner runs in the network namespace of Hune: it acts as the edge and as the PDP.
set +e
docker run --rm --init --network "container:$ch_name" --user "$(id -u):$(id -g)" -w /work \
    --volume "$work:/work:ro" --volume "$BENCH_TOOLS:/tools:ro" debian:stable-slim \
    timeout 600 /tools/access-conformance --role pep --series U --profile profile.json --url http://127.0.0.1:8080 --format json \
    >"$BENCH_OUT/U.json" 2>"$BENCH_OUT/U.err"
code=$?
set -e
# The runner gives 0 or 1 for a run that ended. Another code is a problem of the set-up.
if [ "$code" -ne 0 ] && [ "$code" -ne 1 ]; then
    tail -n 20 "$BENCH_OUT/U.err" >&2
    exit 2
fi
