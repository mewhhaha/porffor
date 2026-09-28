#!/usr/bin/env python3
"""Check exact coverage of the frozen Test262 failure backlog; optionally render it."""

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import sys


FAILURE_OUTCOMES = ("Bug", "NotImplemented", "Crash")
MODES = ("sloppy-script", "strict-script", "raw-script", "module", "raw-module")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def read_json(path):
    def unique_fields(pairs):
        fields = {}
        for key, value in pairs:
            require(key not in fields, f"{path}: duplicate JSON key {key}")
            fields[key] = value
        return fields

    return json.loads(path.read_text(), object_pairs_hook=unique_fields)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def local_path(root, relative):
    path = Path(relative)
    require(not path.is_absolute() and ".." not in path.parts,
            f"path must stay inside the backlog: {relative}")
    result = root / path
    require(result.resolve().is_relative_to(root.resolve()), f"path escapes backlog: {relative}")
    return result


def counts_for(failures):
    counts = Counter(failure["outcome"] for failure in failures)
    return {outcome: counts[outcome] for outcome in FAILURE_OUTCOMES}


def validate_data(root):
    registry = read_json(root / "registry.json")
    require(registry["schema_version"] == 1, "unsupported registry schema")
    baseline = read_json(local_path(root, registry["baseline"]))
    require(baseline["schema_version"] == 1, "unsupported baseline schema")
    for name, expected in baseline["artifacts"].items():
        require(digest(local_path(root, name)) == expected, f"baseline evidence changed: {name}")
    for name in ("aggregate.json", "failures.json", "provenance.json", "publication-session.json",
                 "published-status.json", "failures.executions"):
        require(f"evidence/{name}" in baseline["artifacts"], f"baseline lacks {name} digest")

    aggregate = read_json(root / "evidence/aggregate.json")
    provenance = read_json(root / "evidence/provenance.json")
    publication = read_json(root / "evidence/publication-session.json")
    status = read_json(root / "evidence/published-status.json")["real_suite"]
    failures = read_json(root / "evidence/failures.json")
    require(aggregate["producer"] == "lila" and aggregate["execution_backend"] == "wasm-aot"
            and aggregate["run_kind"] == "aggregate-matrix", "baseline is not a product matrix")
    require(baseline["execution_backend"] == "wasm-aot", "backlog must use the product backend")
    for key in ("total", "passed", "pinned_revisions", "manifest_hash", "counts_per_outcome"):
        require(baseline[key] == aggregate[key], f"baseline and aggregate disagree: {key}")
    for key in ("total", "passed", "failed", "pinned_revisions", "manifest_hash"):
        require(baseline[key] == status[key], f"baseline and publication disagree: {key}")
    require(baseline["compiler_sha256"] == publication["identity"]["executable_sha256"]
            and baseline["source_inputs_sha256"] == publication["identity"]["source_inputs_sha256"]
            and baseline["suite_sha256"] == publication["identity"]["suite_sha256"],
            "baseline and publication input identity disagree")
    require(baseline["completed_nodes"] == baseline["total_nodes"] > 0,
            "baseline must be a complete nonempty matrix")
    require(publication["progress"] == {"completed": baseline["completed_nodes"],
                                        "total": baseline["total_nodes"]},
            "publication did not verify a complete matrix")
    nodes = aggregate["completed_nodes"]
    entries = aggregate["aggregate_entries"]
    require(len(set(nodes)) == len(nodes) == baseline["completed_nodes"], "duplicate or missing matrix node")
    require(len(entries) == len(nodes) and {entry["node_id"] for entry in entries} == set(nodes),
            "matrix entries do not match completed nodes")
    require(sum(entry["total"] for entry in entries) == baseline["total"]
            and sum(entry["passed"] for entry in entries) == baseline["passed"],
            "matrix node totals disagree with baseline")
    require(provenance["aggregate"]["sha256"] == digest(root / "evidence/aggregate.json")
            and provenance["outputs"]["failures.json"]["sha256"] == digest(root / "evidence/failures.json"),
            "extraction provenance does not bind aggregate and failures")
    require(provenance["exclusion"]["execution_count"] == 0
            and provenance["collected"]["failed"] == provenance["observed"]["failed"] == baseline["failed"],
            "backlog extraction excluded failures")
    require(provenance["observed"]["total"] == baseline["total"]
            and provenance["observed"]["passed"] == baseline["passed"]
            and len(provenance["completed_leaves"]) == len(nodes), "extraction does not cover baseline")
    leaves = provenance["completed_leaves"]
    require({leaf["node_id"] for leaf in leaves} == set(nodes),
            "extraction contains duplicate or missing matrix nodes")
    entries_by_id = {entry["node_id"]: entry for entry in entries}
    for leaf in leaves:
        entry = entries_by_id[leaf["node_id"]]
        require(all(leaf[key] == entry[key] for key in ("total", "passed", "failed"))
                and leaf["outcomes"] == entry["counts_per_outcome"],
                f"extraction counts disagree for node {leaf['node_id']}")

    by_id = {}
    for failure in failures:
        identity = failure["test_id"]
        require(identity not in by_id, f"duplicate baseline failure: {identity}")
        require(re.fullmatch("(?:" + "|".join(MODES) + r"):[^\s]+\.js", identity),
                f"invalid execution identity: {identity}")
        require(identity.split(":", 1)[1] == failure["test_path"], f"path differs from execution: {identity}")
        require(failure["outcome"] in FAILURE_OUTCOMES, f"invalid failure outcome: {identity}")
        require(failure["detail"].strip(), f"missing observed diagnostic: {identity}")
        by_id[identity] = failure
    require(len(by_id) == baseline["failed"] == baseline["total"] - baseline["passed"],
            "failure inventory does not reconcile with baseline")
    require(counts_for(failures) == {key: baseline["counts_per_outcome"][key] for key in FAILURE_OUTCOMES},
            "failure outcomes differ from baseline")
    require((root / "evidence/failures.executions").read_text().splitlines() == sorted(by_id),
            "frozen execution list differs from failure evidence")

    task_ids, task_keys, assigned = set(), set(), set()
    tasks = registry["tasks"]
    require(tasks, "empty task registry")
    for task in tasks:
        identity, key = task["id"], task["key"]
        require(re.fullmatch(r"F[0-9]{3,}", identity) and identity not in task_ids,
                f"invalid or duplicate task ID: {identity}")
        require(re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", key) and key not in task_keys,
                f"invalid or duplicate task key: {key}")
        task_ids.add(identity)
        task_keys.add(key)
        require(task["status"] in ("open", "in-progress", "fixed", "blocked"), f"invalid status: {identity}")
        require(task["cause_status"] in ("confirmed", "suspected", "unresolved"),
                f"invalid cause status: {identity}")
        require(task["disposition"] in ("required", "aot-dynamic-boundary"), f"invalid disposition: {identity}")
        for field in ("title", "owner", "root_cause", "fix", "validation"):
            require(isinstance(task[field], str) and task[field].strip(), f"{identity}: missing {field}")
        require(task["evidence"], f"{identity}: no source evidence or investigation entry")
        for evidence in task["evidence"]:
            require(evidence["path"] and evidence["note"] and type(evidence["line"]) is int
                    and evidence["line"] > 0, f"{identity}: incomplete source evidence")
        if task["status"] == "fixed":
            require(task.get("resolution", "").strip(), f"{identity}: fixed task needs replay evidence")
        ids = task["test_ids"]
        require(ids and ids == sorted(set(ids)), f"{identity}: execution list must be sorted, unique and nonempty")
        require(not set(ids) - by_id.keys(), f"{identity}: task contains a non-baseline execution")
        require(not assigned.intersection(ids), f"{identity}: execution assigned to multiple tasks")
        assigned.update(ids)
        require(task["counts"] == counts_for(by_id[item] for item in ids), f"{identity}: counts differ from evidence")
        require(task["document"] == f"{identity}-{key}.md" and task["cases"] == f"cases/{identity}.executions",
                f"{identity}: document or execution-list name mismatch")
    require(assigned == by_id.keys(), f"unassigned failures: {sorted(by_id.keys() - assigned)[:5]}")
    return registry, baseline, by_id


def table_cell(text):
    return text.replace("|", "\\|").replace("\n", " ")


def diagnostic_text(text):
    # Diagnostics can contain the actual CR/NUL/whitespace a RegExp was testing.
    # Escape controls in Markdown; preserve the original bytes in evidence JSON.
    escapes = {"\r": r"\r", "\t": r"\t"}
    displayed = "".join(escapes.get(character, f"\\u{ord(character):04x}")
                        if ord(character) < 32 and character != "\n" else character
                        for character in text)
    return re.sub(r" +$", lambda match: r"\u0020" * len(match[0]), displayed, flags=re.MULTILINE)


def task_markdown(task, by_id):
    failures = [by_id[identity] for identity in task["test_ids"]]
    physical = len({failure["test_path"] for failure in failures})
    lines = [f"# {task['id']}: {task['title']}", "",
             f"- **Status:** {task['status']}", f"- **Owner:** {task['owner']}",
             f"- **Cause assessment:** {task['cause_status']}",
             f"- **Disposition:** {task['disposition']}",
             f"- **Baseline:** {len(failures)} executions across {physical} physical files "
             f"(Bug {task['counts']['Bug']}, NotImplemented {task['counts']['NotImplemented']}, "
             f"Crash {task['counts']['Crash']})", "",
             "[Backlog](README.md) · [Exact execution list](" + task["cases"] + ") · "
             "[Unmodified diagnostics](evidence/failures.json)", "", "## Root cause", "", task["root_cause"], "",
             "## Source evidence", ""]
    for evidence in task["evidence"]:
        lines.append(f"- [{evidence['path']}:{evidence['line']}](../{evidence['path']}#L{evidence['line']}): {evidence['note']}")
    lines += ["", "## Work", "", task["fix"], "", "## Validation", "", task["validation"], "",
              "Replay all listed modes with a fresh compiler and retain the native snapshots:", "", "```sh",
              f"python3 scripts/replay-test262-executions.py tasks/{task['cases']} \\",
              "  --binary target/release/lila --suite-root test262/vendor/test262 \\",
              f"  --output-dir target/test262-scratch/{task['id']}-replay --workers 2", "```", "",
              "Use a new output directory after rebuilding; resume only with unchanged inputs. "
              "See the backlog resource-limit and closure rules.", "", "## Representative observations", ""]
    # Keep the task readable; the complete diagnostic inventory is preserved once.
    seen = set()
    for failure in failures:
        signature = re.sub(r"handle@\d+", "handle@<address>", failure["detail"])
        if signature in seen:
            continue
        seen.add(signature)
        lines += [f"- `{failure['test_id']}` — {failure['outcome']}", "", "```text",
                  diagnostic_text(failure["detail"]), "```", ""]
        if len(seen) == 3:
            break
    lines += ["## Closure", "",
              "Keep the frozen failure inventory and task ID. Record the fixing revision, "
              "fresh replay evidence for every listed execution, and adjacent-family results "
              "before marking fixed. A suspected cause needs confirmation from a reduced "
              "reproducer or an observed compiler path; a matching error string alone is insufficient.", ""]
    if task.get("resolution"):
        lines += ["## Resolution", "", task["resolution"], ""]
    return "\n".join(lines)


def index_markdown(tasks, baseline, repository_checks=None):
    assessments = Counter(task["cause_status"] for task in tasks)
    boundary = sum(len(task["test_ids"]) for task in tasks if task["disposition"] == "aot-dynamic-boundary")
    lines = ["# Test262 failure backlog", "",
             f"The completed **{baseline['refresh_date']}** Wasm-AOT run recorded "
             f"**{baseline['passed']:,}/{baseline['total']:,} passing executions** across "
             f"**{baseline['completed_nodes']}/{baseline['total_nodes']} matrix sections**. "
             f"All **{baseline['failed']:,} failures** are assigned exactly once to the "
             f"**{len(tasks)} tasks** below: {baseline['counts_per_outcome']['Bug']:,} Bug, "
             f"{baseline['counts_per_outcome']['NotImplemented']:,} NotImplemented and "
             f"{baseline['counts_per_outcome']['Crash']:,} Crash.", "",
             f"Root-cause assessments: **{assessments['confirmed']} confirmed**, "
             f"**{assessments['suspected']} suspected**, **{assessments['unresolved']} unresolved**. "
             "A suspected or unresolved task is an investigation with source pointers, "
             "not a claim that its mechanism has been proven. Large groups can contain "
             "secondary failures exposed after the first cause is repaired.", "",
             f"{boundary} executions are assigned to explicit dynamic-source AOT-boundary tasks. "
             "They remain failures in the denominator. Review finite-source specialization "
             "opportunities within those tasks; never turn this classification into a skip list.", "",
             "## Working order", "",
             "1. Claim one task by setting its registry status to `in-progress` and recording an assignee if useful.",
             "2. Reproduce its exact modes, confirm the cause, and fix the general compiler or runtime operation.",
             "3. Replay the whole task and adjacent families. Record a revision and native evidence in `resolution`, then mark `fixed`.",
             "4. Keep all original IDs and diagnostics. Refresh conformance totals only with another complete published matrix.", "",
             "Tasks with crashes come first, followed by required failure groups ordered by size, "
             "then dynamic-source boundary work. This is a triage order, not a dependency claim. "
             "Split a task when a minimal reproducer reveals independent mechanisms; move each "
             "execution to exactly one new owner and preserve existing task IDs.", "",
             "| Task | Executions | Bug | NotImplemented | Crash | Cause | Status |",
             "|---|---:|---:|---:|---:|---|---|"]
    for task in tasks:
        counts = task["counts"]
        lines.append(f"| [{task['id']}: {table_cell(task['title'])}]({task['document']}) | "
                     f"{len(task['test_ids'])} | {counts['Bug']} | {counts['NotImplemented']} | "
                     f"{counts['Crash']} | {task['cause_status']} | {task['status']} |")
    if repository_checks:
        lines += ["", "## Repository check follow-up", "",
                  f"[Repository check findings]({repository_checks}) record nine inherited "
                  "Rust assertion failures and the existing module-size, formatting and "
                  "identity-check failures found during this cleanup. They have Rxxx IDs, "
                  "owners, source evidence and next steps. These are separate from the "
                  "Test262 baseline above; the failing assertions remain enabled."]
    lines += ["", "## Evidence and maintenance", "",
              "[registry.json](registry.json) is the editable task registry. "
              "The Markdown and per-task execution lists are generated from it. "
              "Edit the registry, then render and validate:", "", "```sh",
              "python3 scripts/check-failure-backlog.py --write", "./scripts/check-task-plan.sh", "```", "",
              "The check rejects missing, duplicated or invented executions, changed frozen "
              "evidence, mismatched outcome totals, and drifted task documents or replay lists. "
              "Marking a task fixed does not delete its original failure membership.", "",
              "- [Baseline and hashes](evidence/baseline.json): pins, compiler and source identity, exact totals.",
              "- [Original diagnostics](evidence/failures.json): every outcome, path, mode, detail and duration.",
              "- [Extraction provenance](evidence/provenance.json): hashes and counts from all native matrix leaves, with zero exclusions.",
              "- [Aggregate](evidence/aggregate.json) and [publisher output](evidence/published-status.json): frozen full-run evidence.",
              "- [Publication session](evidence/publication-session.json): observed inputs and verified matrix completion.", "",
              "The compiler was built from a dirty worktree. Its observed source-input and "
              "binary hashes identify the tested build; the recorded Git commit alone does "
              "not reproduce it. Source line references describe the triage checkout and may "
              "move as repairs land. This backlog stores the complete failed-execution evidence; "
              "the native passed-case inventories remain in the source snapshot directory "
              "recorded by the baseline. Per-leaf hashes are retained for auditing.", "",
              "The old T00–T29 plans have been removed. Their IDs remain domain taxonomy in "
              "[conformance ownership](../docs/rust-rewrite/conformance-ownership.md) for compiler-owned "
              "reports and the CLI ledger; F001 and later IDs identify the actionable tasks here.", "",
              "## Replay limits", "",
              "Each task gives an exact-list replay command using two case workers and one "
              "compiler job per case. Run expensive verification within the current limit of "
              "10 CPUs and half the machine's physical RAM, including all child processes. "
              "For Linux with a user systemd manager:", "", "```sh",
              "# Set the aggregate limit from this machine's physical memory.",
              "test_ram_limit=$(awk '/^MemTotal:/ {printf \"%.0f\", $2 * 1024 / 2}' /proc/meminfo)",
              "systemd-run --user --scope -p AllowedCPUs=0-9 -p CPUQuota=1000% \\",
              '  -p "MemoryMax=$test_ram_limit" -p MemorySwapMax=0 \\',
              "  python3 scripts/replay-test262-executions.py tasks/cases/F001.executions \\",
              "    --binary target/release/lila --suite-root test262/vendor/test262 \\",
              "    --output-dir target/test262-scratch/F001-replay --workers 2", "```", "",
              "Choose up to ten available CPUs if this host has a different affinity. "
              "The replay script freezes the compiler and execution list. Use `--resume` "
              "only for the same binary and inputs; use a fresh directory after a fix. "
              "No full suite was rerun while creating this backlog.", ""]
    return "\n".join(lines)


def check(root, write=False):
    registry, baseline, by_id = validate_data(root)
    repository_checks = registry.get("repository_checks")
    if repository_checks:
        require(local_path(root, repository_checks).is_file(), "missing repository check findings")
    outputs = {"README.md": index_markdown(registry["tasks"], baseline, repository_checks)}
    for task in registry["tasks"]:
        outputs[task["document"]] = task_markdown(task, by_id)
        outputs[task["cases"]] = "\n".join(task["test_ids"]) + "\n"
    expected_docs = {name for name in outputs if name.endswith(".md")}
    extras = {path.name for path in root.glob("*.md")} - expected_docs
    require(not extras, f"unregistered task documents: {sorted(extras)}")
    expected_cases = {task["cases"] for task in registry["tasks"]}
    actual_cases = {path.relative_to(root).as_posix() for path in (root / "cases").glob("*")}
    require(not actual_cases - expected_cases, "unregistered replay list")
    for name, content in outputs.items():
        path = local_path(root, name)
        if write:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
        else:
            require(path.is_file() and path.read_text() == content,
                    f"generated backlog file drifted: {name}; edit registry.json and run --write")
    return len(registry["tasks"]), len(by_id)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tasks-dir", type=Path, default=Path("tasks"))
    parser.add_argument("--write", action="store_true", help="render task documents and replay lists after validating data")
    options = parser.parse_args()
    try:
        tasks, failures = check(options.tasks_dir, options.write)
    except (KeyError, TypeError, OSError, ValueError) as error:
        print(f"check-failure-backlog: {error}", file=sys.stderr)
        return 1
    print(f"check-failure-backlog: {tasks} tasks cover all {failures} failed executions exactly once")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
