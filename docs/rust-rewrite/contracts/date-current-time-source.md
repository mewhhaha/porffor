# Date current time

Status: source-authored for atomic T05 on 2026-10-04; verification is deferred
until the complete feature batch is written.

Date.now, zero-argument Date construction and the Date function call consume the
same injected Realm host-clock import. The native producer writes a typed I64
binary64-bits local. Date.now publishes a whole Normal Number completion;
construction applies TimeClip before the complete GC record is published.
Date function formatting uses immutable GC UTF-16 strings and the configured
system-zone snapshot. Date methods use branded captures rather than the clock.

Existing fixed-clock, default-zone and worker inheritance controls in
`aot_date_system_time_zone` remain required and unrun for this draft. The later
verification launcher must enforce a 4096 MiB cgroup cap across all descendants,
zero swap and serial workers before starting a payload.
