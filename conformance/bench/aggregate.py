#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Make the result and the badge of one project from the reports of the runner.

Use: aggregate.py PROJECT TAG SPEC_TAG OUT_DIR SRC_DIR PROJECT_JSON

OUT_DIR has one report of the runner for each series: U.json, D.json or E.json (the output of --format json).
SRC_DIR is the checkout of the project at TAG. It holds the files expected.txt.
The script writes OUT_DIR/result.json and OUT_DIR/badge.json. It exits with 0 when it wrote them.

A series is compliant when the failed cases are equal to the failed cases that the file expected.txt of the project
lists. A skipped case that the project does not list is a warning: it does not change the status.
"""
import json
import os
import re
import sys


def read_expected(path):
    """Return the two sets of case ids that the file expected.txt lists: failed and skipped."""
    failed, skipped = set(), set()
    if not os.path.isfile(path):
        return None
    for line in open(path, encoding="utf-8"):
        entry = line.split("#", 1)[0].strip()
        if not entry:
            continue
        words = entry.split()
        if not re.fullmatch(r"[UDE][0-9]{2,3}", words[0]):
            continue
        status = words[1] if len(words) > 1 else "fail"
        (skipped if status == "skip" else failed).add(words[0])
    return failed, skipped


def main():
    project, tag, spec_tag, out, src, project_json = sys.argv[1:7]
    entry = json.load(open(project_json))
    protocol = ".".join(spec_tag.lstrip("v").split(".")[:2])
    series_result = {}
    status = "compliant"
    runner_version = ""
    for item in entry["series"]:
        letter = item["series"]
        report_path = os.path.join(out, f"{letter}.json")
        expected = read_expected(os.path.join(src, item["expected"]))
        if not os.path.isfile(report_path) or expected is None:
            series_result[letter] = {"role": item["role"], "error": "no report or no expected.txt"}
            status = "error"
            continue
        report = json.load(open(report_path, encoding="utf-8"))
        runner_version = report.get("version", runner_version)
        cases = report["cases"]
        failed = {c["id"] for c in cases if c["status"] == "fail"}
        skipped = {c["id"] for c in cases if c["status"] == "skip"}
        summary = report["summary"]
        problems = {
            "failed_not_expected": sorted(failed - expected[0]),
            "failed_expected_but_passed": sorted(expected[0] - failed),
            "skipped_not_expected": sorted(skipped - expected[1]),
            "skipped_expected_but_run": sorted(expected[1] - skipped),
        }
        # A failed case that the project does not list is a failure. A skipped case that the project does not list is a
        # warning: the runner could not test it, and the result tells it.
        ok = not (problems["failed_not_expected"] or problems["failed_expected_but_passed"])
        series_result[letter] = {
            "role": item["role"],
            "cases": summary["cases"],
            "passed": summary["passed"],
            "failed": summary["failed"],
            "skipped": summary["skipped"],
            "warned": summary.get("warned", 0),
            "compliant": ok,
            "problems": {k: v for k, v in problems.items() if v},  # the keys skipped_* are warnings
        }
        if not ok and status != "error":
            status = "non-compliant"
    roles = "+".join(i["role"].upper() for i in entry["series"])
    total_failed = sum(s.get("failed", 0) for s in series_result.values())
    if status == "compliant":
        message, color = f"{protocol} · {roles}", "brightgreen"
    elif status == "non-compliant":
        message, color = f"{protocol} · {total_failed} failed" if total_failed else f"{protocol} · differs", "red"
    else:
        message, color = f"{protocol} · not tested", "lightgrey"
    result = {
        "project": project,
        "repo": entry["repo"],
        "tag": tag,
        "protocol": protocol,
        "procurauth_tag": spec_tag,
        "runner_version": runner_version,
        "date": os.environ.get("BENCH_DATE", ""),
        "seconds": int(os.environ.get("BENCH_SECONDS", "0")),
        "status": status,
        "series": series_result,
    }
    json.dump(result, open(os.path.join(out, "result.json"), "w"), indent=2, sort_keys=True)
    json.dump(
        {"schemaVersion": 1, "label": "ProcurAuth", "message": message, "color": color},
        open(os.path.join(out, "badge.json"), "w"),
    )
    print(f"{project} {tag}: {status} ({message})")


main()
