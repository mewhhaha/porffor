# Resolution work for the four original failures — 2026-10-21

The local resolution package is prepared and verified. **The four original
failures remain open.** No compiler change, specification change or fixture
correction has been adopted into the pinned product/suite contract.

[Four reviewable patches and three upstream report drafts](../../test262/errata/proposals/README.md)
now cover the proposed resolutions. Nothing has been submitted upstream.

## What the investigation established

- **Arguments:** the accepted
  [2017 normative change](https://github.com/tc39/ecma262/commit/ea167900a06af090a1158a17b5ee04c7bcafb015)
  explicitly introduced the behavior Scriptw implements. V8 and SpiderMonkey
  also fail the outdated original test and replace the implicit binding.
  The Test262 correction is ready for review.
- **Labels:** the original staging test passes V8 and SpiderMonkey. Three
  standalone block/case probes also agree across V8, SpiderMonkey and QuickJS:
  chained declarations are initialized on entry and eligible declarations copy
  to the outer variable environment. Scriptw follows the conflicting literal
  pinned wording. The proposed ECMA-262 patch updates both lexical collectors
  and all three Annex B candidate loops; adopting it requires a normative
  resolution before changing the compiler under the current contract.
- **Timezone:** the proposed ECMA-402 branch explicitly handles historical
  backzone Zones, preserving geographic identity. The paired Test262 correction
  removes Johnston from the unconditional alias-equality group. Both remain
  proposals. Temporal is unavailable in the installed comparison engines, so
  no independent Temporal implementation result is claimed.

The installed diagnostic engines were Node 24.19.0 / V8 13.6.233.17-node.51,
PythonMonkey 1.4.1 with bundled SpiderMonkey `libmozjs-136a1.so`, and the QuickJS
Python binding 1.19.4. Seven [standalone probes](../../test262/errata/reproducers/README.md)
ran in fresh processes across all four engines, yielding 28 observations.
QuickJS differs on the arguments and top-level-hoisting controls; its raw errors
are retained. These observations are not extra Test262 passes.

## Native verification

| Selected Wasm-AOT run | Passed | Bug | Crash | Not implemented |
| --- | ---: | ---: | ---: | ---: |
| Four original failures plus ten original adjacent controls | 10 | 4 | 0 | 0 |
| Same four IDs with only the arguments/timezone test proposals | 3 | 1 | 0 | 0 |

The second run deliberately keeps the **original labelled-function test**.
Its remaining failure is not hidden by the existing literal-spec label-test
amendment. That amendment and the proposed label-specification change are
alternative resolutions and must not be adopted together.

Both native runs returned exit status 1. They used the unchanged, source-equivalent
optimized compiler, one reusable worker with fresh per-case state, one compiler
job and the original 60,000 ms deadline. The executable SHA-256 remains
`ef346e256752843179eceab239b6b995b3e43f1acb97c96b44cb05530a756666`;
its native identity remains an unversioned source archive, not a newly relabelled
Git build. Reusing it avoids an unnecessary release rebuild.

All four draft patches apply without fuzz in isolated copies and produce the
exact recorded output hashes. The two Test262 patches reproduce the previously
verified correction bytes. All 11 timezone-generation and seven errata-tooling
tests pass, with no skips.

[Machine-readable evidence](final-four-resolution-20261021.json) and the
[lossless archive](../../test262/snapshots/final-four-resolution-20261021.tar.gz)
retain the source specifications/history, patch checks, original fixtures,
native snapshots, exact ID lists, commands, process exits, oracle identities,
observations and transcripts. All 124 archive members were read back and
verified. The amended-suite preparation receipt explicitly forbids stock
Test262 publication.

## Remaining decisions and acceptance

1. Obtain upstream disposition of the arguments correction and the coordinated
   timezone test/algorithm proposals.
2. Resolve the labelled-function specification/compatibility disagreement.
   The current pinned specification remains authoritative. If its proposed
   normative change is adopted, implement the general shared collectors,
   initialization/copy rules and capture/eval storage planning; retain the
   original staging test and run the affected semantic controls.
3. After the authority and implementation agree, rerun the original failures
   and adjacent tests, then run the complete source-bound suite and publish
   its native artifacts/README graph.

No fresh full campaign was started while these prerequisites remain unresolved.
The existing complete pre-fix baseline and graph are unchanged. Local draft
successes cannot close the four original failures or establish full conformance.
