# Primitive numeric locale formatting through GC

The actual Number and BigInt prototype native callers validate their receiver
before entering the shared formatter. It borrows their retained whole primitive
through locales/options hooks, invokes the recorded Intl.NumberFormat
constructor and intrinsic format getter, then calls the returned format
callable with a private one-element GC argument vector.

All constructor/getter/call results are whole completions. A Throw prevents
later stages and retains the original reference. Temporary roots clear on one
cleanup edge. Neither the mutable public Intl.NumberFormat property nor the
public prototype format accessor is read by this internal operation.

This leaf is dry-authored and uncompiled. It consumes the actual typed direct
call/whole completion API. Remaining NativeIntl constructor, configuration and
rendering producers must be ported in the same atomic T05 batch before execution;
there is no raw-ABI bridge or partial MAIN integration. The new Engine control
for hooks, exact BigInt formatting and intrinsic identity is unrun. Final
verification requires the confirmed 4096 MiB aggregate kernel memory cap.
