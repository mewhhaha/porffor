"""Narrow libtest roots while proving the ordinary workspace dependency graph."""

import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys


PACKAGE = "scriptw-verification-feature-roots"
TARGET = "feature_roots"


def verify_source_identities(output, environment):
    """Reject stale generated provenance before spending time on native builds."""
    command = [sys.executable, "-B", "scripts/intl_source_identities.py"]
    result = subprocess.run(command, env=environment, capture_output=True)
    path = Path(str(output) + ".intl-source-identities.log")
    path.write_bytes(result.stdout + result.stderr)
    if result.returncode:
        raise ValueError(
            "Stale Intl source identities; refresh with "
            "python3 -B scripts/intl_source_identities.py --write before verification"
        )
    return {
        "passed": True,
        "command": command,
        "path": str(path),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "exit_code": 0,
    }


def build_key(case, isolated, combined):
    if case["kind"] == "doc" or case["package"] in isolated:
        return None
    return (
        "feature-roots",
        case["package"],
        "combined" if combined else case["kind"],
        "*" if combined or case["kind"] == "lib" else case["target"],
    )


def cargo_selectors(cases, additional=()):
    """Stable union shared by build and reference graph commands."""
    selectors = {tuple(row["selector"]) for row in cases} | set(additional)
    return [word for selector in sorted(selectors) for word in selector]


def command(case, inventory, isolated, combined, profile_flags):
    key = build_key(case, isolated, combined)
    selected = [row for row in inventory if build_key(row, isolated, combined) == key]
    return [
        "cargo",
        "test",
        "--locked",
        "--offline",
        *profile_flags,
        "-p",
        case["package"],
        "-p",
        PACKAGE,
        "--features",
        PACKAGE + "/feature-roots",
        *cargo_selectors(selected),
        "--test",
        TARGET,
    ]


def closure_signature(graph, root):
    """Bind all unit fields and dependency-edge attributes, ignoring only IDs.

    Dependency IDs are array positions in Cargo's JSON, not stable identities.
    Every dependency is replaced by its complete recursive signature instead.
    Paths, package IDs, features, profiles and compiler modes stay exact.
    """
    memo = {}
    visiting = set()

    def visit(index):
        if index in visiting:
            raise ValueError("Cargo unit graph contains a dependency cycle")
        if index in memo:
            return memo[index]
        visiting.add(index)
        unit = dict(graph["units"][index])
        edges = []
        for dependency in unit.pop("dependencies"):
            edge = dict(dependency)
            edge["unit"] = visit(edge.pop("index"))
            edges.append(edge)
        unit["dependencies"] = sorted(
            edges, key=lambda edge: json.dumps(edge, sort_keys=True)
        )
        result = hashlib.sha256(json.dumps(unit, sort_keys=True).encode()).hexdigest()
        visiting.remove(index)
        memo[index] = result
        return result

    return visit(root)


def selected_root(graph, package_id, name, kind):
    matches = [
        index
        for index in graph["roots"]
        if graph["units"][index]["pkg_id"] == package_id
        and graph["units"][index]["target"]["name"] == name
        and graph["units"][index]["target"]["kind"] == [kind]
        and graph["units"][index]["mode"] == "test"
    ]
    if len(matches) != 1:
        raise ValueError("Missing or ambiguous selected Cargo unit root")
    return matches[0]


def prove_graph(
    candidate_command, cases, package_ids, isolated, profile_flags, output, environment
):
    """Plan only; RUSTC_BOOTSTRAP never enters the subsequent compiler process.

    Cargo's unit graph is an unstable inspection API. Fail closed if unavailable
    or changed. No build scripts, compiler jobs or native tests run in this step.
    """
    reference = [
        "cargo",
        "test",
        "--locked",
        "--offline",
        *profile_flags,
        "--workspace",
        *[word for package in sorted(isolated) for word in ("--exclude", package)],
        *cargo_selectors(cases, [("--lib",)]),
    ]
    environment = dict(environment, RUSTC_BOOTSTRAP="1")
    graphs = []
    evidence = []
    for name, base in [
        ("ordinary-workspace", reference),
        ("selected-feature-roots", candidate_command),
    ]:
        inspection = [*base, "--no-run", "--unit-graph", "-Z", "unstable-options"]
        result = subprocess.run(inspection, env=environment, capture_output=True)
        path = Path(str(output) + "." + name + ".unit-graph.json")
        path.write_bytes(result.stdout)
        path.with_suffix(".stderr").write_bytes(result.stderr)
        if result.returncode:
            raise ValueError("Cargo feature-root graph inspection failed: " + name)
        graph = json.loads(result.stdout)
        if graph["version"] != 1:
            raise ValueError("Unknown Cargo unit graph schema")
        graphs.append(graph)
        evidence.append(
            {
                "command": inspection,
                "path": str(path),
                "sha256": hashlib.sha256(result.stdout).hexdigest(),
                "units": len(graph["units"]),
                "roots": len(graph["roots"]),
            }
        )
    roots = []
    for case in cases:
        signatures = [
            closure_signature(
                graph,
                selected_root(
                    graph, package_ids[case["package"]], case["target"], case["kind"]
                ),
            )
            for graph in graphs
        ]
        if signatures[0] != signatures[1]:
            raise ValueError(
                "Selected Cargo dependency graph changed: "
                + case["package"]
                + ":"
                + case["target"]
            )
        roots.append(
            {
                "package": case["package"],
                "target": case["target"],
                "kind": case["kind"],
                "recursive_unit_sha256": signatures[0],
            }
        )
    return {
        "passed": True,
        "inspection_only": True,
        "selected_roots": roots,
        "graphs": evidence,
        "compiler_environment_unchanged": True,
    }


def verify_auxiliary(executable, output, environment):
    """An empty graph root is build evidence, never additional suite coverage."""
    executable = Path(executable)
    digest = hashlib.sha256(executable.read_bytes()).hexdigest()
    logs = []
    for name, arguments in [
        ("inventory", ["--list"]),
        ("execution", ["--test-threads=1"]),
    ]:
        result = subprocess.run(
            [str(executable), *arguments],
            env=environment,
            capture_output=True,
            timeout=30,
        )
        path = Path(str(output) + ".feature-roots-" + name + ".log")
        path.write_bytes(result.stdout + result.stderr)
        text = result.stdout.decode()
        if result.returncode:
            raise ValueError("Feature-root auxiliary failed: " + name)
        if name == "inventory":
            if text.strip() != "0 tests, 0 benchmarks":
                raise ValueError("Feature-root auxiliary must have no native IDs")
        else:
            summaries = re.findall(r"^test result: .+$", text, re.M)
            if len(summaries) != 1 or not re.fullmatch(
                r"test result: ok\. 0 passed; 0 failed; 0 ignored; "
                r"0 measured; 0 filtered out; finished in [0-9.]+s",
                summaries[0],
            ):
                raise ValueError(
                    "Feature-root auxiliary must execute exactly zero native IDs"
                )
        logs.append(
            {
                "kind": name,
                "path": str(path),
                "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                "exit_code": 0,
            }
        )
    if hashlib.sha256(executable.read_bytes()).hexdigest() != digest:
        raise ValueError("Feature-root auxiliary changed during verification")
    return {
        "executable": str(executable),
        "bytes": executable.stat().st_size,
        "sha256": digest,
        "native_ids": 0,
        "outcomes": {"passed": 0, "failed": 0, "ignored": 0},
        "included_in_suite_denominator": False,
        "retained_for_cargo_reuse": True,
        "logs": logs,
    }
