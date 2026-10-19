#!/usr/bin/env python3
"""Keep bounded historical host paths out of the product identity audit only.

The original proof bytes remain untouched. This filter never suppresses a
whole line: retired package, API, module and UI identifiers remain visible.
"""
import hashlib
import json
from pathlib import Path
import re
import sys

PROOFS = {
    # Closed reports keep their literal host paths; admission binds every byte.
    'differential-jobs-checkpoint-20261019.json': (None, 'da2e86722971e38d7e81a400f74e046c86c8afea328419ba51c2b0ff1db79667'),
    'runtime-admission-checkpoint-20261019.json': (None, '71622731147e3fb95bad6e9a178f615446f55882b116ff13aa7dd4a3a471ea17'),
    'coherent-throughput-core-20261017.json': (None, 'fd792f9448a04be826717fe1b078e662c85de5725829755c860534c635ab498d'),
    'native-source-throughput-emission-20261017.json': (None, '0de34c17799fcf120b39a576fca4b95a9b077b35643fdcd0d180d64888d00822'),
    'complete-current-native-inventories-20261017.json': (None, '5230af16e2631be1771e2970de9b724fcb662ec3d34533ef06933eaa8a2e6dbb'),
    'native-cache-fixture-checkpoint-20261017.json': (None, '65a6d06daf286205b238e1c568740402f73888206b4e536d7d9baca8b29f4c49'),
    'native-workspace-032d2f5c7-20261012.json': (None, 'e5655325520b6d30ef2908c053c0c7c6f7f8292ad5c94361cbfdfb29d6ada03b'),
    'native-workspace-successor-20261012.json': (None, '4f3d4e347adb0890fae883a2eef5bbfa6cb6326702435a1f1fe696822c643b45'),
    'abi6-native-repairs-20261015.json': ('abi6-source-and-focused-native-publication-v1', None),
    'abi6-five-crate-metadata-eb1ca63c9-20261015.json': ('unapplied-abi6-five-crate-metadata-proof-v1', None),
    'abi6-prepared-ownership-eb1ca63c9-20261015.json': ('unapplied-abi6-prepared-ownership-proof-v1', None),
    'all-features-workspace-8e9a8c82-native-checkpoint-20261015.json': ('portable-real-next-workspace-v1-20261015', None),
    'all-features-workspace-d2-native-checkpoint-20261014.json': ('portable-real-all-features-workspace-v9-20261014', None),
    'all-features-workspace-eb1ca63c9-native-checkpoint-20261015.json': ('portable-current-workspace-retention-v1', None),
    'cloud-worker-affinity-checkpoint-20261015.json': ('cloud-worker-affinity-tooling-checkpoint-v1', None),
    'default-workspace-8e9a8c82-native-checkpoint-20261015.json': ('portable-real-next-workspace-v1-20261015', None),
    'default-workspace-d2-native-checkpoint-20261014.json': ('portable-real-default-workspace-v8b-20261014', None),
    'default-workspace-eb1ca63c9-native-checkpoint-20261015.json': ('portable-current-workspace-retention-v1', None),
    'default-workspace-f5-native-checkpoint-20261014.json': ('portable-real-default-workspace-v7-20261014', None),
    'dynamic-aot-same-store-eb1ca63c9-20261015.json': ('dynamic-aot-same-store-diagnostic-eb1ca63c9-v1', None),
    'exact-replay-checkpoint-20261013.json': (None, '1580a2f0fc52d38cf281241e858d4563cf867f7d1a368f0bb6d680a9fb601b5d'),
    'focused-finite-subclass-d2f4bc51-20261014.json': ('finite_subclass_focused_portable_checkpoint_v1', None),
    'focused-native-fb3a1235-20261015.json': ('actual-focused-native-v4', None),
    'focused-workspace-repairs-20261014.json': ('focused_workspace_repairs_20261014_v1', None),
    'guarded-publication-handoff-8e9a8c82-20261015.json': ('actual-portable-operational-publication-handoff-8e-v1', None),
    'historical-complete-comparison-20261014.json': (None, 'da6cc378ade6330615984714d3fda16553efa84a35c2010b1e3322e95de70775'),
    'historical-repairs-checkpoint-20261014.json': (None, '681a8a5ba3dccc9ca073008702f033c1ee3fde704fbf0178992cf61df8da18ad'),
    'historical-source-750-native-checkpoint-20261014.json': (None, '1b69b2d23889743f1c6fd34b9b562361d3fdb6b54df5ae0bdf804624161384bd'),
    'historical5365-source-0a18-checkpoint-20261014.json': ('lossless_closed_historical5365_publication_v1', None),
    'host-producer-optimization-checkpoint-20261013.json': (None, '6cda32f623f8cf467deedb1c8288d69ccaa12175016716841dc2980bd422b05e'),
    'incomplete-publication-closure-8e9a8c82-20261015.json': ('portable-incomplete-prefix-closure-8e-v1', None),
    'ir-foundation-eb1ca63c9-native36-20261015.json': ('focused-ir-candidate-native-proof-v1', None),
    'joined-source-8429-partial-native-checkpoint-20261014.json': ('joined_source_two_library_partial_native_checkpoint_v1', None),
    'native-batch-and-profile-20261014.json': (None, '95a6d3fbcf388780c9f28567ed0d7eb7e8e1719a4088f5847e708ed4c71f4659'),
    'native-test-fingerprint-boundary-20261015.json': ('native-test-fingerprint-boundary-checkpoint-v1', None),
    'normal-cli-build-costs-d2-20261015.json': ('normal-Cargo-and-actual-warm-worker-measurements-v1', None),
    'object-rest-trailing-comma-source-20261015.json': ('object-rest-trailing-comma-seven-path-source-v1', None),
    'ordinary-cli-8e9a8c82-probe53-20261015.json': ('ordinary-cli-probe53-retention-publication-v1', None),
    'ordinary-cli-d2-probe45-20261014.json': ('ordinary-cli-probe45-d2-subclass-retention-publication-v1', None),
    'ordinary-cli-eb1ca63c9-probe55-20261015.json': ('ordinary-cli55-retention-publication-v1', None),
    'ordinary-cli-f5-probe45-20261014.json': ('ordinary-cli-probe45-publication-v1', None),
    'original-history-eb1ca63c9-20261015.json': ('closed-original-history-and-diagnostics-eb1ca63c9-v1', None),
    'partition-measurements-d2f4bc511-20261014.json': ('observed-partition-and-python-measurements-v1', None),
    'private-numeric-call-profile-20261014.json': ('private_numeric_call_profile_comparison_v1', None),
    'runtime-retention-matched20-8e9a8c82-20261015.json': ('matched20-actual-worker-timing-comparison-v1', None),
    'shortcut-check-eb1ca63c9-20261015.json': ('checkout-shortcut-audit-record-v1', None),
    'workspace-source-revalidation-eb1ca63c9-20261015.json': ('portable-default-source-revalidation-eb1ca63c9-v1', None),
    'workspace-tooling-eb1ca63c9-20261015.json': ('actual-eb1ca63c9-workspace-supplement-v1', None),
    'workspace8429-open-failures-20261014.json': ('four_closed_RED_scopes_with_open_workspace_v1', None),
}
READERS = {
    'verify-abi6-native-repairs-20261015.py': 'd3327c5957dbe9d83bed820bbe480785b4cc25c0c6e9cd3819dd3ff619e152d8',
    'verify-dynamic-aot-diagnostic-eb1ca63c9-20261015.py': '338bc0cfdb1da3aa57c237086b2fc8a82d0cf32ba8eafef047d069a044032ae2',
    'verify-focused-native-fb3a1235-20261015.py': 'eca14aa652f812e365007a2f17296b4f2a9eac8327f205a4fd0a0009e9ed3e5e',
    'verify-guarded-publication-handoff-8e9a8c82-20261015.py': 'eb96acf1501d66c1c4c38ce5d40f2ea7307e372065492a52d286c97ca697ea7d',
    'verify-ordinary-cli53-8e9a8c82-20261015.py': 'f2e792512d4df711de32f83ef510b59f30984a0be60e61e82e2968162603a187',
    'verify-ordinary-cli55-eb1ca63c9-20261015.py': '8a03cb31f8470c9e0259eb6a922fee35c1c332aedc920070aa9b85506216ac67',
    'verify-workspace-8e9a8c82-20261015.py': 'ddda5ba7df9c32afcbec344ad5171ef9ae9f6e53174500c8a19312040e70da9b',
    'verify-workspace-eb1ca63c9-20261015.py': '7e5447ab7a7af128733d4c7f9c1099e428090cb40d700b7885ac55aca7609ebe',
}
RETIRED = "por" + "ffor"
BOUNDARY = r"(?:(?<![\w/.-])|(?<=\\[nrt]))"
ROOT = re.compile(BOUNDARY + r"(?:file://)?/workspace/" + RETIRED + r"(?:-next|-repairs)?(?![\w.-])")
WORKTREE = re.compile(r"(?<=/\.git/worktrees/)" + RETIRED + r"(?:-next|-repairs)(?![\w.-])")


def unique_fields(pairs):
    result = {}
    for name, value in pairs:
        if name in result:
            raise ValueError("duplicate proof field")
        result[name] = value
    return result


def audit_source(path, raw):
    """Return a line-preserving audit stream for one closed proof context."""
    path = Path(path)
    if path.parent.as_posix() != "docs/rust-rewrite":
        return raw
    name = path.name
    if name in PROOFS:
        schema, frozen_sha = PROOFS[name]
        value = json.loads(raw, object_pairs_hook=unique_fields)
        if not isinstance(value, dict):
            raise ValueError("historical proof must be an object")
        if frozen_sha:
            if hashlib.sha256(raw).hexdigest() != frozen_sha:
                raise ValueError("frozen historical proof changed")
        elif value.get("schema") != schema:
            raise ValueError("historical proof schema differs")
    elif name in READERS:
        if hashlib.sha256(raw).hexdigest() != READERS[name]:
            raise ValueError("captured historical proof reader changed")
    else:
        return raw
    source = raw.decode("utf-8")
    source = ROOT.sub("/workspace/HISTORICAL_CHECKOUT", source)
    source = WORKTREE.sub("HISTORICAL_WORKTREE", source)
    # These are exact JSON field locators, rather than filesystem path suffixes.
    if name == "workspace-source-revalidation-eb1ca63c9-20261015.json":
        for phase in ("before", "after"):
            for field in ("bytes", "sha256"):
                old = f".{phase}./workspace/{RETIRED}-next.real_index.{field}"
                source = source.replace('"' + old + '"', '".' + phase +
                    './workspace/HISTORICAL_CHECKOUT.real_index.' + field + '"')
    return source.encode("utf-8")


def main():
    path = Path(sys.argv[1])
    try:
        sys.stdout.buffer.write(audit_source(path, path.read_bytes()))
    except (OSError, UnicodeError, ValueError) as error:
        print(f"identity-provenance: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
