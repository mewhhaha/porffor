# Checkpoint publication

The JSON snapshot is the authority for resuming a Test262 family. The text
summary describes it but cannot admit or replace execution state. Existing
compiler, suite and checkpoint-identity admission still runs before replacement.

Each replacement is written to a uniquely named file beside its destination.
The writer owns that temporary only after `create_new` succeeds. It writes all
bytes and synchronizes the file before closing it and renaming it over the
destination. A failed write or synchronization leaves the previous destination
intact. Normal error paths close and remove their own temporary. A terminated
process may leave a temporary, whose name cannot enter the JSON snapshot census.
If a later process reuses that PID, it selects another unused temporary name.

JSON and text publish separately. An interruption between their renames can
leave an older text summary beside valid newer JSON; resume uses the JSON.
This provides atomic replacement for process interruption and failed writes,
without claiming a transaction across both files or power-loss durability for
the directory entry. Snapshot schema, provenance, deadlines and checkpoint
frequency do not change.

The regression injects a failure after writing replacement bytes for each file.
It checks the previous JSON and text byte for byte, admits the old JSON through
the native checkpoint decoder and case contract, checks temporary cleanup, and
then completes the same family through the production writer.
