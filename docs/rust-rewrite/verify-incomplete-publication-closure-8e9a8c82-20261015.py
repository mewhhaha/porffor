#!/usr/bin/env python3
"""Offline verification of an incomplete publication/history closure and retention IDs."""
from __future__ import annotations
import argparse
import base64
from collections import Counter
import gzip
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import sys

HEAD = "8e9a8c8210f99d8d14b4267461ff42079b603c00"
PRIOR_SHA = "ecfcd692f95747694ee7128e9a5e92604b2d1a3282529d765fe7218d4de4bcac"
PRIOR_READER_SHA = "eb96acf1501d66c1c4c38ce5d40f2ea7307e372065492a52d286c97ca697ea7d"
PRIOR_NAME = "guarded-publication-handoff-8e9a8c82-20261015.json"
PRIOR_READER_NAME = "verify-guarded-publication-handoff-8e9a8c82-20261015.py"
ROOT_SHA = "daa031ccb7eedacffba0925659be549d0bc0a401051eeae65bfd6effb1f1362d"
HISTORY_SHA = "163f18e4c39d9410f251bd8b280fb53620f23f169da5df8f21747393b1552145"
SOURCE_SHA = "86ee012d7a675c77cb0001eb950d1fa9816d416ee77be2dfe5c8e2ccc05a7e1b"


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def require(value, message):
    if not value:
        raise ValueError(message)


def exact(actual, expected, message):
    require(type(actual) is type(expected) and actual == expected, message)


def false_flags(doc, keys):
    for key in keys:
        require(doc.get(key) is False, "Incomplete evidence cannot claim " + key)


def unique(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "Duplicate JSON field: " + key)
        result[key] = value
    return result


def parsed(raw):
    return json.loads(raw, object_pairs_hook=unique)


def prior_module(path):
    raw = Path(path).read_bytes()
    exact(digest(raw), PRIOR_READER_SHA, "Prior published offline reader hash changed")
    spec = importlib.util.spec_from_file_location("partial_closure_prior", path)
    module = importlib.util.module_from_spec(spec)
    # This hash-pinned reader only uses stdlib and has no native/network calls.
    exec(compile(raw, str(path), "exec"), module.__dict__)
    return module


class Evidence:
    def __init__(self, value, prior):
        self.files = dict(prior.files)
        total = 0
        for path, row in value["files"].items():
            require(path.startswith(("/workspace/", "/tmp/")) and "/../" not in path, "Invalid original evidence path")
            exact(row["codec"], "gzip-base64", "Unknown evidence codec")
            packed = base64.b64decode(row["data"], validate=True)
            with gzip.GzipFile(fileobj=io.BytesIO(packed)) as stream:
                raw = stream.read(32 * 1024 * 1024 + 1)
            require(len(raw) <= 32 * 1024 * 1024, "Evidence file exceeds limit")
            exact(len(raw), row["bytes"], "Evidence size changed")
            exact(digest(raw), row["sha256"], "Evidence hash changed")
            require(not raw.startswith(b"\x7fELF"), "Native executable embedding forbidden")
            require(path not in self.files or self.files[path] == raw, "Prior immutable evidence was replaced")
            self.files[path] = raw
            total += len(raw)
        require(total <= 64 * 1024 * 1024, "Additional raw evidence exceeds limit")
        pending = dict(value["aliases"])
        while pending:
            advanced = False
            for path, row in list(pending.items()):
                source = row["source_path"]
                if source not in self.files:
                    continue
                raw = self.files[source]
                exact(digest(raw), row["sha256"], "Alias digest changed")
                exact(len(raw), row["bytes"], "Alias size changed")
                require(path not in self.files or self.files[path] == raw, "Alias rewrites prior immutable evidence")
                self.files[path] = raw
                del pending[path]
                advanced = True
            require(advanced, "Missing/cyclic evidence alias")

    def raw(self, ref):
        require(ref["path"] in self.files, "Required evidence absent: " + ref["path"])
        raw = self.files[ref["path"]]
        exact(digest(raw), ref["sha256"], "Bound raw digest changed")
        if "bytes" in ref:
            exact(len(raw), ref["bytes"], "Bound raw size changed")
        return raw

    def doc(self, ref):
        return parsed(self.raw(ref))

    def path(self, path, sha=None):
        require(path in self.files, "Required original path absent: " + path)
        raw = self.files[path]
        if sha is not None:
            exact(digest(raw), sha, "Original path digest changed")
        return raw


def terminal_receipts(e, prefix, compiler):
    owner = e.doc(prefix["controller_after"])
    require(owner["complete"] is True, "Controller process did not close")
    exact(owner["exit_code"], 143, "Controller literal interruption lost")
    false_flags(owner, ("source_unchanged", "logical_publication_complete", "native_acceptance"))
    require("final_identity" not in owner, "Interrupted owner was assigned a fabricated final capture")
    exact(owner["error"], "GuardError: supervisor interrupted after native command", "Controller interruption reason changed")
    for name in ("outer-after-stop", "launcher-after-stop"):
        doc = e.doc(prefix["closed_receipts"][name])
        require(doc["complete"] is True and doc["exit_code"] == 143, "Actual outer/launcher143 missing")
        if name == "outer-after-stop":
            require(doc["source_unchanged"] is True, "Original outer source fence changed")
            for stream in ("stdout", "stderr"):
                ref = doc[stream]
                packed = e.path(ref["path"], ref["archive_sha256"])
                raw = gzip.decompress(packed)
                exact(digest(raw), ref["raw_sha256"], "Outer raw stream changed")
                exact(len(raw), ref["raw_bytes"], "Outer raw stream length changed")
    watcher = e.doc(prefix["closed_receipts"]["watcher-after-stop"])
    require(watcher["complete"] is True and watcher["exit_code"] == 0 and watcher["controller_exit_code"] == 143,
            "Owned affinity watcher did not close")
    false_flags(watcher, ("source_or_environment_changed", "cases_restarted"))
    exact([row["signal"] for row in prefix["signals"]], [19, 15, 18], "Controller STOP/TERM/CONT order changed")
    for row in prefix["signals"]:
        exact(row["pid"], 851296, "Wrong controller signal target")
        exact(row["start_ticks"], 47994345, "Wrong controller start ticks")
    proof = prefix["kernel_pending_term_proof"]
    require(any(int(value, 16) & (1 << 14) for value in proof["pending_fields"].values()), "Pending TERM bit missing")
    require(prefix["signals"][1]["unix"] <= proof["unix"] <= prefix["signals"][2]["unix"], "CONT preceded pending TERM proof")
    exact(prefix["live_owned_native_processes"], [], "Native writer survived closure")
    for item in prefix["terminal_process_observations"]:
        old, current = item["expected"], item["current"]
        require(current is None or current["start_ticks"] != old["start_ticks"] or current["state"] in ("Z", "X"),
                "A bound process remained live")
    return owner


def verify_prefix(p, e, prior, value):
    prefix = e.doc(value["roots"]["prefix"])
    false_flags(prefix, ("full_suite_complete", "native_acceptance", "tasks_closed"))
    require(prefix["prefix_closed"] is True, "Incomplete prefix lacks real closure")
    transition = prior.doc(prior.value["transition"])
    identity = transition["identity"]
    final = e.doc(prefix["manifest_final"])
    exact(final["identity"], identity, "Final original family identity changed")
    exact(final["progress"], {"completed": 67, "total": 748}, "Actual incomplete matrix progress changed")
    exact(prefix["progress"], final["progress"], "Prefix high-water mismatch")
    old_audit = prior.doc(transition["artifact_audit"])
    exact(e.raw(prefix["matrix_cache"]), prior.raw(old_audit["matrix_cache"]), "Original full pinned partition cache changed")
    cache = e.doc(prefix["matrix_cache"])
    nodes = {node["node_id"]: node for node in cache["nodes"]}
    aggregate = e.doc(prefix["aggregate_final"])
    p.envelope(aggregate, identity["compiler_identity"])
    exact(aggregate["run_kind"], "aggregate-matrix", "Final aggregate kind changed")
    exact(len(aggregate["aggregate_entries"]), 67, "Closed-node denominator changed")
    exact(aggregate["completed_nodes"], [row["node_id"] for row in aggregate["aggregate_entries"]], "Aggregate node order changed")
    exact(aggregate["completed_test_ids"], [], "Aggregate invents case outcomes")
    exact(aggregate["failures"], [], "Aggregate invented failure rows")
    copies = {row["node_id"]: row for row in prefix["raw_node_copies"]}
    exact(len(copies), 67, "Duplicate/missing closed node copies")
    original_refs = {r["path"]: r for r in transition["previously_closed_snapshots"]}
    positive, negative, seen = set(), {}, set()
    kinds, outcomes = Counter(), Counter()
    reused = 0
    for entry in aggregate["aggregate_entries"]:
        node = nodes[entry["node_id"]]
        row = copies[node["node_id"]]
        for kind in ("json", "txt", "timing"):
            exact(e.raw(row[kind]["original"]), e.raw(row[kind]["immutable_copy"]), "Immutable node copy changed")
        raw = e.doc(row["json"]["immutable_copy"])
        result = p.audit_case(raw, node["case_ids"], identity["compiler_identity"])
        require("checkpoint_identity" not in raw, "Partial node passed as a closed partition")
        exact(raw["run_kind"], "matrix-" + {"FilterLeaf": "filter-leaf", "ChunkLeaf": "chunk-leaf"}[node["node_kind"]], "Wrong closed-node kind")
        exact(raw["manifest_hash"], p.node_hash(node), "Node manifest hash changed")
        exact(entry["manifest_hash"], raw["manifest_hash"], "Aggregate node hash differs")
        for key in ("filter", "matrix_path"):
            exact(entry[key], node[key], "Aggregate partition metadata changed")
        for key, expected in (("total", raw["total"]), ("passed", raw["passed"]), ("failed", len(result["failures"]))):
            exact(entry[key], expected, "Aggregate node count changed")
        for key, wire in (("outcomes", "counts_per_outcome"), ("kinds", "counts_per_kind"), ("origins", "counts_per_origin")):
            exact(entry[wire], result[key], "Aggregate taxonomy changed")
        exact(e.raw(row["txt"]["immutable_copy"]).decode(), p.render_summary(raw), "Node TXT differs from raw outcomes")
        timing = e.doc(row["timing"]["immutable_copy"])
        exact(timing["snapshot_sha256"], row["json"]["original"]["sha256"], "Timing lacks exact node binding")
        exact(timing["compiler"], identity["compiler_identity"], "Timing compiler changed")
        for key, expected in (("timeout_ms", 60000), ("configured_workers", 3), ("reuse_case_workers", True)):
            exact(timing["settings"][key], expected, "Native resource policy changed")
        current = set(raw["completed_test_ids"])
        require(not seen.intersection(current), "Duplicate execution across closed nodes")
        seen.update(current)
        positive.update(current - set(result["failures"]))
        negative.update({key: {"node_id": node["node_id"], **item} for key, item in result["failures"].items()})
        outcomes.update(result["outcomes"]); kinds.update(result["kinds"])
        if row["json"]["original"]["path"] in original_refs:
            exact(e.raw(row["json"]["original"]), prior.raw(original_refs[row["json"]["original"]["path"]]), "Previously published node bytes changed")
            reused += 1
    exact(reused, 28, "Not all original28 node proofs were retained")
    exact(len(seen), 10903, "Actual completed execution count changed")
    exact(len(positive), 10898, "Actual prefix passing count changed")
    exact(aggregate["total"], len(seen), "Aggregate completed count changed")
    exact(aggregate["passed"], len(positive), "Aggregate pass count changed")
    exact(aggregate["counts_per_outcome"], p.complete(outcomes, p.OUTCOMES), "Aggregate outcomes differ")
    exact(aggregate["counts_per_kind"], p.complete(kinds, p.KINDS), "Aggregate failure kinds differ")
    exact(p.complete(outcomes, p.OUTCOMES), {"Success": 10898, "NotImplemented": 4, "Crash": 0, "Bug": 1}, "Literal prefix outcomes changed")
    exact(e.doc(prefix["positive_ids"]), sorted(positive), "Portable future positive retention IDs changed")
    exact(e.doc(prefix["negative_ids_and_outcomes"]), [negative[key] for key in sorted(negative)], "Actual prefix red rows changed")
    prior_positive = set(prior.raw(transition["prior_success_ids"]).decode().splitlines())
    require(prior_positive <= positive, "Previously published passing IDs were lost")
    owner = terminal_receipts(e, prefix, identity["compiler_identity"])
    projection = e.doc(owner["effective_environment"])
    exact(set(projection), set(p.PUBLICATION_ENVIRONMENT | p.RUNTIME_ENVIRONMENT), "Environment dump/changed policy projection")
    expected_ops = ["progress-status"] + ["report-all", "progress-status"] * 38 + ["report-all"]
    exact([row["operation"] for row in owner["commands"]], expected_ops, "Native prefix command sequence changed")
    for i, row in enumerate(owner["commands"]):
        require(row["complete"] is True and row["exit_code"] == 0, "A native command failed")
        exact(row["sequence"], i, "Native sequence changed")
        exact(row["command"], p.native_command(projection, row["operation"]), "Native payload changed")
        exact(row["environment_sha256"], digest(p.canonical(projection)), "Native policy digest changed")
        for stream in ("stdout", "stderr"):
            e.raw(row[stream])
    exact(len(owner["observations"]), 39, "Native progress observations missing")
    for i, observation in enumerate(owner["observations"]):
        expected = {"completed": 28 + i, "total": 748}
        exact(observation["progress"], expected, "Actual native progress changed")
        exact(observation["command_sequence"], i * 2, "Progress command binding changed")
        exact(observation["after_report"], i > 0, "Progress after-report semantics changed")
        exact(p.progress(e.raw(owner["commands"][i * 2]["stdout"]).decode(), identity["compiler_identity"]), expected, "Raw native progress differs")
    observer = e.doc(prefix["progress_observer"])
    require(observer["exit_code"] == 0 and observer["after_report"] is True, "Final stock observer failed")
    exact(p.progress(e.raw(observer["stdout"]).decode(), identity["compiler_identity"]), final["progress"], "Final raw observer changed")
    exact(e.doc(observer["manifest_before"]["immutable_copy"])["progress"], {"completed": 66, "total": 748}, "Last naturally completed node mismatch")
    exact(e.doc(observer["manifest_after"]["immutable_copy"]), final, "Final manifest differs from stock observer")
    return prefix, identity, positive, set().union(*(set(node["case_ids"]) for node in nodes.values()))


def verify_history(p, e, value, identity):
    history = e.doc(value["roots"]["history"])
    exact(value["roots"]["history"]["sha256"], HISTORY_SHA, "Actual historical closure binding changed")
    false_flags(history, ("full_coverage_verified", "full_passing_retention_verified", "conformance_passed", "native_acceptance", "tasks_closed", "cases_restarted", "journal_in_flight_is_success"))
    owner = e.doc(history["closed_historical_owner"])
    false_flags(owner, ("complete", "coverage_verified", "passing_retention_verified", "conformance_passed"))
    require(owner["source_unchanged"] is True and owner["protected_images_unchanged"] is True, "Partial history source/producer changed")
    exact(owner["native_exit_code"], -2, "Actual replay interruption exit changed")
    require("FileNotFoundError" in owner["controller_error"] and "summary.json" in owner["controller_error"], "Literal partial controller error missing")
    artifacts = {item["original"]["path"]: item for item in history["artifacts"]}
    for row in artifacts.values():
        exact(e.raw(row["original"]), e.raw(row["immutable_copy"]), "Historical immutable raw copy changed")
    runs = [row for path, row in artifacts.items() if path.endswith("/replay/run.json")]
    snapshots = [row for path, row in artifacts.items() if "/snapshots/" in path and path.endswith(".json")]
    journals = [row for path, row in artifacts.items() if path.endswith(".attempts")]
    exact((len(runs), len(snapshots), len(journals)), (1, 1, 1), "Historical raw checkpoint inventory changed")
    run, checkpoint, journal = (e.doc(x["immutable_copy"]) for x in (runs[0], snapshots[0], journals[0]))
    exact(run["compiler_identity"], identity["compiler_identity"], "History compiler differs")
    exact(run["batch_threads"], 1, "History worker count changed")
    selected = p.ids(run["batch_executions"], "Historical selected native IDs")
    absent = p.ids(run["absent_executions"], "Historical absent IDs")
    exact((len(selected), len(absent)), (5363, 2), "Historical native/preflight denominator changed")
    require(not selected.intersection(absent), "Absent and native historical IDs overlap")
    exact(run["execution_list_sha256"], "95d46445f4c3fda986c7cdf390dccdddfb8ab370b8a831e3f77ea7f387a1a43e", "Historical input changed")
    ids = p.ids(checkpoint["completed_test_ids"], "Historical completed checkpoint IDs")
    require(ids < selected, "Historical checkpoint is not incomplete")
    exact(checkpoint["run_kind"], "resume-case-checkpoint", "Incomplete historical checkpoint kind changed")
    result = p.audit_case(checkpoint, ids, identity["compiler_identity"])
    positive = ids - set(result["failures"])
    exact(len(ids), 1600, "Actual observed history count changed")
    exact(result["outcomes"], {"Success": 1436, "NotImplemented": 161, "Crash": 0, "Bug": 3}, "Literal historical outcomes changed")
    exact(set(journal["selected_test_ids"]), selected, "Journal selected IDs changed")
    exact(journal["manifest_hash"], checkpoint["manifest_hash"], "Journal is not bound to historical checkpoint")
    exact(journal["in_flight"], history["literal_journal_in_flight"], "Literal pending journal changed")
    require(all(row["test_id"] in selected for row in journal["in_flight"]), "Unknown pending historical ID")
    exact(history["pending_native_ids"], sorted(selected - ids), "Pending cases were promoted/removed")
    exact(history["success_ids"], sorted(positive), "Observed historical positives changed")
    observed = e.doc(history["observed_outcomes"])
    actual = {row["execution_id"]: row for row in observed}
    exact(len(actual), len(observed), "Duplicate observed outcome ID")
    exact(set(actual), ids | absent, "Observed native/preflight IDs changed")
    for name in ids:
        exact(actual[name]["outcome"], result["failures"].get(name, {"outcome": "Success"})["outcome"], "Historical raw outcome differs")
        require(actual[name]["native_case_executed"] is True, "Historical native origin changed")
    for name in absent:
        exact(actual[name]["outcome"], "InfrastructureError", "Absent preflight was promoted")
        require(actual[name]["native_case_executed"] is False, "Missing input became a native execution")
        raw = e.doc(actual[name]["raw"])
        require(raw["execution_id"] == name and raw["exit_code"] != 0 and raw.get("infrastructure_error") and "outcome" not in raw,
                "Original missing-input evidence changed")
    baseline = e.doc(value["roots"]["historical_baseline"])
    exact(value["roots"]["historical_baseline"]["sha256"], "427a50b760853ce07906b69cf2fbcde8045689097c1eb1d7306001203e436ca3", "Prior accepted baseline changed")
    require(baseline["complete"] is True and baseline["coverage_verified"] is True, "Prior baseline lacked full historical coverage")
    summary = e.doc(value["roots"]["historical_baseline_summary"])
    exact(value["roots"]["historical_baseline_summary"]["sha256"], baseline["summary"]["sha256"], "Prior baseline summary hash changed")
    prior = {row["execution_id"] for row in summary["results"] if row.get("outcome") == "Success"}
    exact(len(prior), 5142, "Prior historical passing denominator changed")
    exact(history["prior5142_retained_so_far"], sorted(prior & positive), "Observed prior retention subset differs")
    exact(len(prior & positive), 1405, "Observed prior retained count changed")
    exact(history["prior5142_observed_nonpasses"], sorted(prior & ids - positive), "Observed prior regressions changed")
    exact(history["prior5142_not_observed_yet"], sorted(prior - ids), "Unobserved prior IDs changed")
    long = e.doc(history["closed_long_owner"])
    require(long["complete"] is True and long["source_unchanged"] is True, "Original LONG not closed/source-bound")
    require(long["error"].endswith("ValueError: Actual historical closure/source/ordinary worker differs\n"), "Original literal LONG red changed")
    exact({row["label"]: row["exit_code"] for row in long["commands"]}, {"historical5365": 1, "fullpin102994": 143}, "Original literal child exits changed")
    launcher = e.doc(history["original_long_launcher"])
    require(launcher["complete"] is True and launcher["exit_code"] == 1 and launcher["pid"] == long["pid"], "Original root launcher exit changed")
    failed = e.doc(value["roots"]["failed_v2"])
    exact(failed["prefix_signals"], [], "Failed V2 attempt signalled the prefix")
    exact(failed["message"], "original long must preserve its actual incomplete-history error", "Literal failed gate changed")
    exact([(row["pid"], row["start_ticks"], row["signal"]) for row in history["signals"]], [(813653, 47732314, 15)], "Historical termination target/order changed")
    exact(history["live_historical_processes"], [], "Historical writer survived closure")
    return positive, prior


def verify_source(e, value, prefix, identity):
    exact(value["roots"]["source_proof"]["sha256"], SOURCE_SHA, "Actual source-byte proof changed")
    source = e.doc(value["roots"]["source_proof"])
    exact(source["identity"], identity, "Final captured source identity differs from original")
    require(source["source_unchanged"] is True and source["in_memory_guard_only"] is True, "Full byte proof lacks original guard")
    exact(source["native_cases_executed"], 0, "Source capture executed cases")
    exact(len(source["full_captures"]), 1, "Final full capture missing/duplicated")
    capture = source["full_captures"][0]
    exact(capture["identity"], identity, "Full captured bytes differ")
    require(capture["seconds"] > 0, "Invalid capture time")
    for check in prefix["source_guard_final_checks"]:
        exact(check["guard_sha256"], capture["guard_sha256"], "Final fresh input inventory changed")
    exact(len(source["native_queries"]), 1, "Unexpected native identity query count")
    query = source["native_queries"][0]
    exact(query["command"], [identity["executable"], "compiler-identity"], "Source native query changed")
    exact(query["exit_code"], 0, "Native identity query failed")
    exact(query["affinity"], [0, 1, 2], "Source query affinity changed")
    exact(e.doc(query["stdout"]), identity["compiler_identity"], "Raw executing compiler identity changed")
    e.raw(source["original_helper"]); e.raw(source["guard_driver"])


def verify(value, prior_value, p):
    exact(value["schema"], "portable-incomplete-prefix-closure-8e-v1", "Wrong closure schema")
    exact(value["head"], HEAD, "Wrong frozen source")
    exact(value["prior_handoff"]["sha256"], PRIOR_SHA, "Wrong prior capsule binding")
    exact(value["prior_handoff"]["reader_sha256"], PRIOR_READER_SHA, "Wrong prior reader binding")
    false_flags(value["claims"], ("native_acceptance", "full_pin_complete", "full_historical_coverage", "full_historical_passing_retention", "tasks_closed", "elf_embedded", "inherited_environment_embedded"))
    p.verify(prior_value)
    prior = p.Evidence(prior_value); prior.value = prior_value
    e = Evidence(value, prior)
    exact(value["roots"]["recovery"]["sha256"], ROOT_SHA, "Actual recovery owner changed")
    root = e.doc(value["roots"]["recovery"])
    require(root["complete"] is True and root["source_or_case_bytes_changed"] is False, "Recovery/source closure failed")
    false_flags(root, ("full_historical_coverage_verified", "full_pin_complete", "native_acceptance", "tasks_closed", "cases_restarted"))
    barrier = e.doc(root["source_barrier"])
    false_flags(barrier, ("full_historical_coverage_verified", "full_historical_passing_retention_verified", "full_pin_complete", "native_acceptance", "tasks_closed"))
    require(barrier["source_integration_barrier_closed"] is True, "Source barrier did not close")
    exact(barrier["strict_incomplete_full_prefix_closure"], value["roots"]["prefix"], "Prefix root mismatch")
    exact(barrier["intentional_historical_partial_closure"], value["roots"]["history"], "History root mismatch")
    prefix, identity, positive, universe = verify_prefix(p, e, prior, value)
    historical_positive, prior_positive = verify_history(p, e, value, identity)
    verify_source(e, value, prefix, identity)
    observed = positive | historical_positive
    required = observed | prior_positive
    require(required <= universe, "Required retained IDs are outside the full current pin")
    retention = e.doc(value["retention_requirement"])
    exact(retention["all_observed_8e_success_ids"], sorted(observed), "Observed success retention list differs")
    exact(retention["required_future_success_ids"], sorted(required), "Complete future positive retention list differs")
    return {"offline_evidence_verified": True, "head": HEAD, "closed_full_prefix_nodes": 67,
            "closed_full_prefix_executions": 10903, "full_prefix_outcomes": {"Success": 10898, "NotImplemented": 4, "Bug": 1, "Crash": 0},
            "historical_native_executions_observed": 1600, "historical_native_outcomes": {"Success": 1436, "NotImplemented": 161, "Bug": 3, "Crash": 0},
            "historical_missing_preflight_ids": 2, "prior_historical_passes_required": 5142,
            "all_observed_8e_success_ids_required": len(observed), "required_future_success_ids": len(required),
            "source_capture_metadata_verified": True, "source_or_executable_presence_verified_offline": False,
            "full_pin_complete": False, "full_historical_coverage": False, "native_acceptance": False, "tasks_closed": False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("closure", type=Path)
    parser.add_argument("--prior", type=Path)
    parser.add_argument("--prior-reader", type=Path)
    args = parser.parse_args()
    value = parsed(args.closure.read_bytes())
    exact(digest(Path(__file__).read_bytes()), value["reader_sha256"], "Standalone closure reader hash changed")
    prior_path = args.prior or args.closure.parent / PRIOR_NAME
    prior_reader = args.prior_reader or args.closure.parent / PRIOR_READER_NAME
    prior_raw = prior_path.read_bytes()
    exact(digest(prior_raw), PRIOR_SHA, "Published prior handoff bytes changed")
    p = prior_module(prior_reader)
    print(json.dumps(verify(value, parsed(prior_raw), p), indent=2))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, TypeError, OSError) as error:
        print("Partial closure verification failed: " + str(error), file=sys.stderr)
        raise SystemExit(2)
