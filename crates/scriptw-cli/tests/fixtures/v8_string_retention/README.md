These two workloads expose the cost of adding per-string metadata. Both retain
32,768 dynamically constructed two-unit strings. One reads only characters; the
other inserts every string into a Map once and reads only the values afterward.
They do not measure repeated-key cache wins.

The execution runner reports invocation RSS and GC capacity. Neither is a live
JavaScript heap measurement: capacity is coarse and RSS also includes code,
runtime metadata and allocator overhead. Use paired source comparisons alongside
the long, short and numeric workloads in `v8_repeated_keys`.
