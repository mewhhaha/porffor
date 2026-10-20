# Native partition cost estimates

`scripts/workspace_test_partitions.py` assigns every qualified native test ID
exactly once using deterministic longest-estimated-work-first placement. Cost
estimates affect placement only. Each child still has one CPU, one libtest
thread, the original watcher allowance, and the inherited cache limits. Test
bodies, CLI worker arguments, assertions and deadlines stay unchanged.

The complete 855-ID CLI run at source `78593daf2` on 2026-10-20 passed in three
partitions. Their watcher completion times were 870, 1605 and 1080 seconds,
despite equal old weight totals of 1226. The old generic Intl weight of 25 and
frontend weight of 1 put several expensive projection and matrix tests in the
same partition. The subset fixture alone took approximately 498 seconds;
DateTime, Duration, Number and Segmenter projection intervals were approximately
180, 143, 130 and 159 seconds.

The estimates retain the existing fallback classes and assign three weight
units per observed second to 22 completed outliers of at least 30 seconds.
These times come from changes to the watcher's last-seen test, rounded to whole
seconds; they include polling uncertainty and can change with cache state and
contention. Exact IDs prevent a new test from inheriting a measured cost merely
because its name shares a prefix. Renamed and new tests remain included through
the fallback classes.

A read-only placement of all 855 original IDs yields 284, 285 and 286 tests,
with estimated loads of 3389, 3388 and 3388 units. This is a scheduling estimate;
an elapsed-time improvement requires a subsequent complete native run. The
observation and input log/inventory byte counts and hashes are retained in
`target/cli-partition-cost-observation-20261020-run1/receipt.json`.

Do not alter a frozen run's helper or source to apply updated costs. Apply
them in the next verification batch, and retain the same complete-inventory
and exact disjoint-union admission checks before any native body starts.
