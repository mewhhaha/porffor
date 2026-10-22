#!/usr/bin/env python3
"""Run source-only builtin guards before building the compiler dependency graph.

This is an additive diagnostic. The full Cargo structure_builtins target still
owns all typed IR/Wasm checks and remains required for native acceptance.
"""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time
import tomllib

from native_test_protocol import parse_inventory, result_summaries

# Keep these complete modules in the authoritative Cargo target. Do not extract
# only the text assertions from modules that also exercise typed/compiler APIs.
CARGO_MODULES = frozenset({
    "array_species_create_operation_evidence_structure",
    "to_property_descriptor_operation_evidence_structure",
    "regexp_modifier_override_structure",
    "regexp_nullable_quantifier_progress_structure",
})


def run(root, output):
    config = root / ".cargo/config.toml"
    if not config.is_file():
        raise ValueError(
            "missing tracked .cargo/config.toml; restore it before Cargo builds "
            "so a sparse checkout does not fork dependency fingerprints"
        )
    crate = root / "crates/scriptw-aot-wasm"
    entry = crate / "tests/structure_builtins/main.rs"
    modules = []
    for line in entry.read_text().splitlines():
        if not line.strip() or line.lstrip().startswith("//"):
            continue
        match = re.fullmatch(r"mod ([a-z_][a-z_0-9]*);", line)
        if not match:
            raise ValueError(f"unclassified structure target declaration: {line}")
        modules.append(match[1])
    if len(modules) != len(set(modules)) or not CARGO_MODULES <= set(modules):
        raise ValueError("duplicate or missing explicitly classified guard module")
    selected = [name for name in modules if name not in CARGO_MODULES]
    if not selected:
        raise ValueError("source-only guard selection is empty")
    manifest = dict(
        kind="source-only diagnostic; not full native or Wasm/IR acceptance",
        source_modules=selected,
        full_cargo_only_modules=sorted(CARGO_MODULES),
        cargo_config=str(config),
    )
    (output / "coverage.json").write_text(json.dumps(manifest, indent=2) + "\n")
    harness = output / "source_guards.rs"
    harness.write_text("\n".join(
        f"#[path = {json.dumps(str(entry.parent / (name + '.rs')), ensure_ascii=False)}]\nmod {name};"
        for name in selected
    ) + "\n")
    edition = str(tomllib.loads((root / "Cargo.toml").read_text())["workspace"]["package"]["edition"])
    environment = dict(os.environ, CARGO_MANIFEST_DIR=str(crate))
    binary = output / "source-guards"
    command = [os.environ.get("RUSTC", "rustc"), "--edition", edition,
               "--test", "-C", "opt-level=1", "-C", "debuginfo=0",
               str(harness), "-o", str(binary)]
    start = time.monotonic_ns()
    with (output / "build.stdout").open("w") as stdout, (output / "build.stderr").open("w") as stderr:
        build = subprocess.run(command, env=environment, stdout=stdout, stderr=stderr)
    build_ns = time.monotonic_ns() - start
    code = build.returncode
    inventory = []
    summaries = []
    if code == 0:
        listed = subprocess.run([str(binary), "--list"],
                                env=environment, cwd=root, capture_output=True, text=True)
        (output / "inventory.stdout").write_text(listed.stdout)
        (output / "inventory.stderr").write_text(listed.stderr)
        if listed.returncode != 0:
            raise ValueError("source guard inventory command failed")
        inventory = parse_inventory(listed.stdout)
        if {name.split("::", 1)[0] for name in inventory} != set(selected):
            raise ValueError("source guard inventory does not cover every selected module")
        start = time.monotonic_ns()
        with (output / "tests.stdout").open("w") as stdout, (output / "tests.stderr").open("w") as stderr:
            tests = subprocess.run([str(binary), "--test-threads=1", "--include-ignored"],
                                   env=environment, cwd=root, stdout=stdout, stderr=stderr)
        code = tests.returncode
        run_ns = time.monotonic_ns() - start
        summaries = result_summaries((output / "tests.stdout").read_text())
        if code == 0 and summaries != [("ok", str(len(inventory)), "0", "0", "0", "0")]:
            raise ValueError("successful source guard process did not execute its exact nonempty inventory")
    else:
        run_ns = None
    receipt = dict(**manifest, finished=True, passed=code == 0,
                   compile_ns=build_ns, execute_ns=run_ns, exit_code=code,
                   rustc_command=command, test_inventory=inventory,
                   result_summaries=summaries)
    (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    for name in ["build.stderr", "tests.stdout", "tests.stderr"]:
        path = output / name
        if not path.exists():
            continue
        raw = path.read_text()
        if code != 0:
            print(raw, end="")
        elif name == "tests.stdout":
            print("\n".join(line for line in raw.splitlines() if line.startswith("test result:")))
    print(f"Source-only guards: {len(selected)}/{len(modules)} modules; "
          f"compile {build_ns / 1e9:.3f}s; full Cargo target still required.")
    return code


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--output", type=Path, help="fresh evidence directory; default: temporary")
    args = parser.parse_args()
    if args.output:
        args.output.mkdir(parents=True, exist_ok=False)
        return run(args.root.resolve(), args.output.resolve())
    with tempfile.TemporaryDirectory(prefix="scriptw-source-guards-") as directory:
        return run(args.root.resolve(), Path(directory))


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError) as error:
        raise SystemExit(f"source-guards: {error}")
