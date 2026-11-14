#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0 OR MIT
"""Hold the Rust code to the suite's complexity ceilings.

Per function: cyclomatic <= 10, cognitive <= 15, Halstead difficulty
<= 30, <= 60 source lines. Per file: <= 500 lines. Every tracked .rs
file is measured, tests, examples, benches and fuzz targets included.
A function or file over a ceiling today would be recorded in
scripts/complexity-baseline.json; the gate fails on a new offender and
on a baselined one that gets worse. The baseline is empty: nothing is
over a ceiling. Run with --update only to record an improvement.

Metrics come from rust-code-analysis-cli 0.0.25 (pinned in CI).

Usage: scripts/complexity-gate.py [--update]
"""

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / "scripts" / "complexity-baseline.json"
CEILINGS = {"cyclomatic": 10, "cognitive": 15, "halstead": 30, "sloc": 60}
FILE_CEILING = 500


def metrics(node):
    """The four measured values of one function node."""
    m = node["metrics"]
    return {
        "cyclomatic": m["cyclomatic"]["sum"],
        "cognitive": m["cognitive"]["sum"],
        "halstead": round(m["halstead"].get("difficulty") or 0, 1),
        "sloc": m["loc"]["sloc"],
    }


def functions(node, prefix, out):
    """Every named function under node, keyed by file and nesting."""
    name = node.get("name") or "?"
    path = f"{prefix}::{name}"
    if node["kind"] == "function" and name != "<anonymous>":
        key = path
        while key in out:
            key += "'"
        out[key] = metrics(node)
    for child in node.get("spaces", []):
        functions(child, path if node["kind"] != "unit" else prefix, out)
    return out


def measure_file(rel):
    """(functions, line count) of one tracked file."""
    report = subprocess.run(
        ["rust-code-analysis-cli", "-m", "-O", "json", "-p", str(ROOT / rel)],
        capture_output=True, text=True, check=True,
    ).stdout
    lines = (ROOT / rel).read_text(encoding="utf-8").count("\n")
    return functions(json.loads(report), rel, {}), lines


def measure():
    """Offending functions and files over the ceilings, and the count."""
    tracked = subprocess.run(
        ["git", "ls-files", "*.rs"], cwd=ROOT,
        capture_output=True, text=True, check=True,
    ).stdout.split()
    offenders, files, total = {}, {}, 0
    for rel in tracked:
        funcs, lines = measure_file(rel)
        total += len(funcs)
        for key, values in funcs.items():
            if any(values[k] > c for k, c in CEILINGS.items()):
                offenders[key] = values
        if lines > FILE_CEILING:
            files[rel] = lines
    return offenders, files, total


def problems(offenders, files, base):
    """What is new or worse than the baseline."""
    found = []
    for key, values in offenders.items():
        old = base["functions"].get(key)
        limit = CEILINGS if old is None else old
        worse = [k for k in CEILINGS if values[k] > limit[k]]
        if worse:
            found.append(f"{key}: {values} (over or worse: {', '.join(worse)})")
    for rel, lines in files.items():
        if lines > base["files"].get(rel, FILE_CEILING):
            found.append(f"{rel}: {lines} lines")
    return found


def main():
    offenders, files, total = measure()
    if "--update" in sys.argv:
        BASELINE.write_text(json.dumps(
            {"ceilings": {**CEILINGS, "file_lines": FILE_CEILING},
             "functions": offenders, "files": files},
            indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(f"baseline: {len(offenders)} functions, {len(files)} files")
        return 0
    base = json.loads(BASELINE.read_text(encoding="utf-8"))
    found = problems(offenders, files, base)
    for line in found:
        print(f"FAIL {line}")
    print(f"{total} functions measured; {len(offenders)} over a ceiling "
          f"(baseline {len(base['functions'])}); {len(files)} files over "
          f"{FILE_CEILING} lines (baseline {len(base['files'])})")
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
