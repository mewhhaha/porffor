#!/usr/bin/env python3
"""Serial exact-byte robustness seeds under the existing verification cap.

Run through limited_verification.py. This driver never builds the selected CLI,
alters worker deadlines on retry, resumes partial evidence or claims conformance.
"""

import argparse
from dataclasses import dataclass
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

from limited_verification import DEFAULT_MEMORY_MIB, MIB, require_cloud_budget, require_kernel_budget


REPORT_LIMIT = 1024 * 1024
SEED_LIMIT = 4 * 1024 * 1024
# Same explicit budget as the existing actual-worker robustness controls.
# Generated differential grammars retain their separate, unchanged deadlines.
ATTEMPT_TIMEOUT_MS = 300_000
TIERS = {"pr-fast": 2, "nightly": 64}


def json_bytes(value):
    return (json.dumps(value, sort_keys=True, ensure_ascii=True) + "\n").encode()


def read_bounded(path, limit):
    with path.open("rb") as stream:
        data = stream.read(limit + 1)
    if len(data) > limit:
        raise ValueError(f"{path} exceeds its {limit}-byte input bound")
    return data


def file_digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


@dataclass(frozen=True)
class Seed:
    target: str
    data: bytes
    native: bool
    first_seed: int = 0

    def wire_target(self):
        if self.target in ("script", "module"):
            return {"kind": "compiler", "goal": self.target}
        if self.target in ("json", "regexp", "number", "bigint", "temporal-date",
                           "encode-uri", "encode-uri-component", "decode-uri", "decode-uri-component"):
            parser = {"regexp": "reg_exp", "bigint": "big_int"}.get(self.target, self.target.replace("-", "_"))
            return {"kind": "builtin", "parser": parser}
        if self.target in ("ir-admission", "prelude", "filesystem-resolver", "frontmatter",
                           "snapshot", "corpus", "module-graph", "report-observation"):
            return {"kind": self.target.replace("-", "_")}
        raise ValueError(f"unknown retained robustness seed target: {self.target}")


def inventory(repository, tier):
    corpus = repository / "crates/lila-test262/tests/differential"
    snapshot = repository / "crates/lila-test262/tests/fixtures/fake_test262/snapshots/-case-1027921027793165688-15918031624465637722.json"
    # The legacy snapshot is an explicit decoder seed, never current conformance.
    entries = [
        Seed("script", b"let total = 0; for (let i = 0; i < 3; i++) total += i; total;\n", False),
        Seed("module", b"export const value = 3; export default value;\n", False),
        Seed("ir-admission", json_bytes({"schema_version": 1, "body": [
            {"op": "define_property", "target": {"kind": "object"}, "key": "answer",
             "value": {"kind": "number", "bits": "4045000000000000"}}
        ]}), False),
        Seed("frontmatter", b"/*---\nflags: [onlyStrict]\n---*/\n1;\n", True),
        Seed("snapshot", read_bounded(snapshot, SEED_LIMIT), True),
        Seed("corpus", read_bounded(corpus / "v3/t25-foundation-primitive-number-and-print.json", SEED_LIMIT), True),
        Seed("module-graph", read_bounded(corpus / "v4/t25-module-cycles-and-metadata.json", SEED_LIMIT), True),
        Seed("report-observation", json_bytes({"kind": "terminal", "print_count": 0,
            "execution": {"disposition": "engine_error", "phase": "parse", "message": "native parser seed"}}), True),
        Seed("prelude", json_bytes({"schema_version": 1,
            "source": "/*---\nincludes: [helper.js]\n---*/\n1;",
            "execution_mode": "strict-script", "harness_profile": "none", "merged_harness": None,
            "files": [{"name": "assert.js", "contents": "/* assertion seed */\n"},
                      {"name": "helper.js", "contents": "/* include seed */\n"}], "overrides": []}), True),
        Seed("filesystem-resolver", json_bytes({"schema_version": 1, "operation": "resolve_and_load",
            "referrer": "entry", "layout": "plain", "specifier": "./dep.js", "attributes": []}), True),
        Seed("json", b'{"array":[1,null,true],"text":"\\uD800"}', False),
        Seed("regexp", b"(?<word>a+)(?:b|c)*\\k<word>", False),
        Seed("number", b"-1.25e+3", False),
        Seed("bigint", b"123456789012345678901234567890", False),
        Seed("temporal-date", b"2024-02-29", False),
    ]
    for target in ("encode-uri", "encode-uri-component", "decode-uri", "decode-uri-component"):
        entries.append(Seed(target, json_bytes({"schema_version": 1, "units": [37, 50, 70]}), False))
    selected = [Seed(entry.target, entry.data, entry.native, ordinal * 128)
                for ordinal, entry in enumerate(entries) if tier == "nightly" or entry.native]
    if len({entry.target for entry in selected}) != len(selected):
        raise ValueError("robustness seed inventory contains duplicate targets")
    for entry in selected:
        entry.wire_target()
        if len(entry.data) > SEED_LIMIT:
            raise ValueError("robustness seed exceeds the original native byte bound")
    return selected


def persist(output, report):
    temporary = output / "tier.tmp"
    with temporary.open("xb") as stream:
        stream.write(json_bytes(report))
        stream.flush()
        os.fsync(stream.fileno())
    temporary.replace(output / "tier.json")
    directory = os.open(output, os.O_RDONLY)
    try:
        os.fsync(directory)
    finally:
        os.close(directory)


def completed_report(output, seed, first_seed, cases, executable_digest):
    path = output / "robustness.json"
    data = read_bounded(path, REPORT_LIMIT)
    report = json.loads(data)
    if not isinstance(report, dict):
        raise ValueError("robustness aggregate is not an object")
    identity = report.get("controller_identity")
    if not isinstance(identity, dict) or identity.get("executable_sha256") != executable_digest:
        raise ValueError("robustness aggregate is not bound to the selected CLI image")
    expected = {"schema_version": 1, "target": seed.wire_target(), "seed": first_seed,
                "total": cases, "completed": cases, "state": "finished",
                "semantic_equivalence": "not_established"}
    if any(report.get(key) != value for key, value in expected.items()):
        raise ValueError("robustness aggregate is incomplete, failed or names a different request")
    records = report.get("cases")
    if not isinstance(records, list) or len(records) != cases:
        raise ValueError("robustness aggregate does not retain every requested case")
    for ordinal, record in enumerate(records):
        if not isinstance(record, dict) or record.get("ordinal") != ordinal \
                or record.get("seed") != first_seed + ordinal \
                or record.get("completed_without_failure") is not True \
                or record.get("disposition") not in ("accepted", "rejected", "executed"):
            raise ValueError("robustness case is incomplete or outside the completed target domain")
        for key, suffix in (("request", "input"), ("observation", "observation")):
            filename = f"case-{ordinal:03}.{suffix}.json"
            if record.get(key) != filename or not (output / filename).is_file():
                raise ValueError("robustness case lost its original input or observation")
    original = json.loads(read_bounded(output / "base.input.json", 2 * SEED_LIMIT + 1024))
    if original != {"schema_version": 1, "id": "robustness/input", "target": seed.wire_target(),
                    "timeout_ms": ATTEMPT_TIMEOUT_MS, "bytes_hex": seed.data.hex()} \
            or read_bounded(output / "base.bin", SEED_LIMIT) != seed.data:
        raise ValueError("robustness campaign changed its original bytes, target or deadline")
    return {"path": str(path), "sha256": hashlib.sha256(data).hexdigest(), "controller_identity": identity}


def run_tier(executable, output, tier, seeds):
    cases = TIERS[tier]
    digest = file_digest(executable)
    output.mkdir()
    (output / "inputs").mkdir()
    (output / "logs").mkdir()
    report = {"version": 1, "tier": tier, "state": "incomplete",
              "scope": "bounded-native-robustness;not-semantic-equivalence;not-conformance",
              "executable": str(executable), "executable_sha256": digest,
              "cases_per_target": cases, "timeout_ms": ATTEMPT_TIMEOUT_MS,
              "targets": []}
    for seed in seeds:
        source = output / "inputs" / f"{seed.target}.bin"
        with source.open("xb") as stream:
            stream.write(seed.data)
        report["targets"].append({"target": seed.target, "seed": seed.first_seed,
            "input": str(source), "input_sha256": hashlib.sha256(seed.data).hexdigest(), "state": "pending"})
    persist(output, report)
    for seed, record in zip(seeds, report["targets"]):
        record["state"] = "running"
        persist(output, report)
        command = [str(executable), "--jobs", "1", "differential", "robustness",
                   "--input", record["input"], "--output-dir", str(output / seed.target),
                   "--target", seed.target, "--seed", str(record["seed"]), "--cases", str(cases),
                   "--timeout-ms", str(ATTEMPT_TIMEOUT_MS), "--oracle", "spec-exec"]
        record["command"] = command
        try:
            print(f"robustness tier: {seed.target}, {cases} exact-byte cases", flush=True)
            with (output / "logs" / f"{seed.target}.stdout").open("xb") as stdout, \
                    (output / "logs" / f"{seed.target}.stderr").open("xb") as stderr:
                completed = subprocess.run(command, stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr, check=False)
            record["exit_code"] = completed.returncode
            if completed.returncode != 0:
                raise ValueError(f"selected CLI exited {completed.returncode}; retained its original report and logs")
            record["aggregate"] = completed_report(output / seed.target, seed, record["seed"], cases, digest)
            record["state"] = "completed"
        except (OSError, ValueError) as error:
            record["state"] = "failed"
            record["error"] = str(error)
        except KeyboardInterrupt:
            record["state"] = "interrupted"
            persist(output, report)
            raise
        persist(output, report)
    if file_digest(executable) != digest:
        report["state"] = "failed"
        report["error"] = "selected CLI image changed during the tier"
    elif report["targets"] and all(record["state"] == "completed" for record in report["targets"]):
        report["state"] = "completed"
    else:
        report["state"] = "failed"
    persist(output, report)
    return 0 if report["state"] == "completed" else 1


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("cli", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("tier", choices=TIERS)
    parser.add_argument("--cloud", action="store_true",
                        help="use the managed machine's inherited finite memory cap")
    args = parser.parse_args(argv)
    try:
        if args.cloud:
            require_cloud_budget()
        else:
            require_kernel_budget(DEFAULT_MEMORY_MIB * MIB)
        executable = args.cli.resolve(strict=True)
        if not executable.is_file() or not os.access(executable, os.X_OK):
            raise ValueError("the selected feature-enabled CLI executable is required")
        repository = Path(__file__).resolve().parents[1]
        return run_tier(executable, args.output.absolute(), args.tier, inventory(repository, args.tier))
    except (OSError, ValueError) as error:
        print(f"robustness tier: {error}", file=sys.stderr)
        return 2
    except KeyboardInterrupt:
        return 130


if __name__ == "__main__":
    sys.exit(main())
