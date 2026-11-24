#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Test the released binary of one project with the runner access-conformance, and write the result and the badge.
# Use: conformance/bench/run.sh PROJECT [TAG]
#
# PROJECT is the name of a file projects/PROJECT.json (the data of the project) with its script projects/PROJECT.sh. Without TAG, the script takes the latest release of the project.
# The script needs: bash, gh, git, python3, docker. It builds nothing.
#
# Input (environment):
#   BENCH_TOOLS_DIR  The directory of a procurauth-tools archive: it holds access-conformance. Required.
#   BENCH_SPEC_TAG   The tag of the ProcurAuth release that the archive comes from, for example v1.1.1. Required.
#   BENCH_OUT        The directory for the result. The default is bench-out/PROJECT.
# Output: BENCH_OUT/result.json, BENCH_OUT/badge.json and one report of the runner for each series.
# The script exits with 0 when the run ended, also when a case failed: the result tells. It exits with 2 for a problem
# of the set-up, for example a missing asset.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$here/../.." && pwd)"
project="${1:?Use: run.sh PROJECT [TAG]}"
tag="${2:-}"
tools="${BENCH_TOOLS_DIR:?BENCH_TOOLS_DIR is required}"
spec_tag="${BENCH_SPEC_TAG:?BENCH_SPEC_TAG is required}"
out="${BENCH_OUT:-bench-out/$project}"
die() { echo "ERROR: $*" >&2; exit 2; }

[ -x "$tools/access-conformance" ] || die "no runner in $tools"
[ -f "$here/projects/$project.sh" ] && [ -f "$here/projects/$project.json" ] || die "no files projects/$project.sh and projects/$project.json"
field() { python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1]))[sys.argv[2]])' "$here/projects/$project.json" "$1"; }
repo="$(field repo)"
asset_pattern="$(field asset)"
binary="$(field binary)"

[ -n "$tag" ] || tag="$(gh release view --repo "$repo" --json tagName --jq .tagName)"
version="${tag#v}"
asset="${asset_pattern//\{version\}/$version}"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$out" "$work/dl" "$work/bin"
echo "== $project $tag: download $asset" >&2
gh release download "$tag" --repo "$repo" --pattern "$asset" --dir "$work/dl" || die "no asset $asset in $repo $tag"
case "$asset" in
    *.tar.gz | *.tgz) tar -xzf "$work/dl/$asset" -C "$work/bin" ;;
    *) die "unknown archive type $asset" ;;
esac
bin="$(find "$work/bin" -type f -name "$binary" | head -n 1)"
[ -n "$bin" ] || die "the archive has no file $binary"
chmod 0755 "$bin"
echo "== $project $tag: check out the files of the tag" >&2
git clone --quiet --depth 1 --branch "$tag" "https://github.com/$repo.git" "$work/src"

start="$(date +%s)"
echo "== $project $tag: run" >&2
BENCH_BIN="$bin" BENCH_SRC="$work/src" BENCH_TOOLS="$tools" BENCH_KEYS="$repo_root/crates/access-conformance/examples" \
    BENCH_OUT="$out" bash "$here/projects/$project.sh"
end="$(date +%s)"

BENCH_SECONDS="$((end - start))" BENCH_DATE="${BENCH_DATE:-$(date -u +%Y-%m-%dT%H:%M:%SZ)}" \
    python3 -I "$here/aggregate.py" "$project" "$tag" "$spec_tag" "$out" "$work/src" "$here/projects/$project.json"
