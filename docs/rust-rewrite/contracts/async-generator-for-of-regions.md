# Complete mixed async-generator iterator regions

The actual checked ForOf syntax owns distinct iterable, acquisition, advance,
per-key initialization, body and exit phases. ForAwait uses the original implicit
Next and Close suspension kinds. These points retain the saved parent chain;
explicit Await and Yield within the complete phases use the existing checked
invocation environment certificate. One source allocator records both kinds.

The initializer proof is minted only while lowering this exact source head
against its retained incoming cell. It owns the resulting region and the
original TDZ/per-iteration environment plan. Captured member bases and raw keys,
Identifier References and nested patterns use their original physical lowering
owners. The actual analysis token admits complete FunctionBody iteration while
unsupported heads and resource scopes retain their existing protocol owner.

Native iteration uses the original frame IteratorRecord table and iterator
methods, without a second object model. Acquisition occurs before the loop's
close scope. Initialization and body resume under that scope, step failures keep
the original DONE rule, matching Continue advances, and whole abrupt completions
close and retire the original record before dispatch.

IR and Wasm artifact fixtures cover synchronous caching, AsyncFromSync adoption,
head TDZ and distinct per-key captures, suspended targets/defaults, nested close
order, selected With References, rejected and injected whole completions, queued
requests and close-error precedence. They are authored but remain unrun in the
current dry-coding batch. Resource-head/suffix ownership remains separate.
