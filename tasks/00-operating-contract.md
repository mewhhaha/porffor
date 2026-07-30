# T00 — Operating contract and contribution protocol

**Status:** Complete as of 2026-07-30

**Parallel group:** Bootstrap  
**Depends on:** None  
**Blocks:** None directly; reduces coordination failures across every task

## Current repository state

The repository now has the task-plan validator, pull-request template, generated
README status guard, shortcut audit, host-ABI audit, module-boundary audit and
interpreter-quarantine audit wired into CI. `./scripts/check-task-plan.sh`
passes in the current working tree. Keep this task as the record for those
repository contracts; new compiler work belongs to its semantic owner task.

## Objective

Turn the rules in `AGENTS.md` and `tasks/README.md` into lightweight, enforceable repository workflow so agents cannot accidentally report fake-suite success as real conformance, hide failures, or merge overlapping giant-file edits without coordination.

## Scope

- Add a small task-plan validator, preferably `scripts/check-task-plan.sh` or a Rust test under `porffor-cli`, that verifies:
  - every `TNN` file has a unique ID;
  - every dependency points to an existing task;
  - every link in `tasks/README.md` resolves;
  - task files contain status, dependencies, objective, acceptance criteria, and test instructions.
- Add or update a pull-request template with required fields:
  - task ID;
  - exact baseline Test262 command/count;
  - exact post-change command/count;
  - new semantic invariant;
  - files/modules owned;
  - test-specific materializations added/removed;
  - remaining failure hashes and follow-up task IDs.
- Add a short contributor note explaining the one-owner rule for the monolithic IR/Wasm files until `T02` lands.
- Add a CI check that fails when README status markers are edited without a corresponding generated status artifact change, unless the edit is explicitly documentation-only outside the generated block.
- Document which commands are smoke tests and which commands are evidence for real-suite progress.
- Document the backend policy: only `wasm-aot` results are Lila conformance evidence; `spec-exec` (Boa) output is internal oracle/differential diagnostics and must never appear as product status.

## Out of scope

- Compiler semantics.
- Changing published conformance counts.
- Adding CODEOWNERS that prevents maintainers from merging; use advisory ownership unless the repository explicitly wants enforced review rules.

## Implementation notes

Keep the validator dependency-free and deterministic. It must run on Linux and should finish in under one second. Avoid a general Markdown parser; stable headings and simple line parsing are sufficient. The PR template must not demand a full matrix for a focused semantic PR, but it must demand at least one real pinned Test262 case or filter when conformance behavior changes.

## Acceptance criteria

- A malformed task dependency and a broken README link both fail the validator with actionable messages.
- A normal checkout passes the validator.
- CI invokes the validator.
- The PR template clearly distinguishes fake fixture counts from pinned real Test262 counts.
- The workflow states that `Unsupported`, timeout, crash, and bug are all non-passing outcomes.
- No runtime/compiler behavior changes.

## Required tests

```sh
./scripts/check-task-plan.sh
cargo test -p porffor-cli --quiet
```

Also run the validator against a temporary intentionally broken copy to prove both error paths before submitting the PR.
