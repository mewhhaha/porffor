# Suspended class evaluation cleanup

The native ClassDefinitionEvaluation owner uses one completion destination for
its complete suspended heritage and computed-key prefixes. An injected Return
and a Throw reach the same destination before an enclosing iterator close,
catch or finally. Ordinary Yield retains the prepared constructor and actual
class-name environment and exits before cleanup.

Only `compile_resumable_class_definition` owns the inclusive checked source
state gate. Skipped classes cannot dispatch an enclosing resumed completion.
Inside the active class, the existing finish Block is registered with both
throw and finally routing. Both routes are popped before the common tail;
branch-depth and environment unwinding remain owned by the existing native
control targets.

The tail restores the outer lexical and private environments and clears the
actual class environment capture. Any non-Normal Completion preserves its
kind, value and target; only Normal evaluation restores the preceding source
completion. The source state advances to the existing class exit and the frame
saves the restored lexical environment before the caller dispatches the whole
completion. The native layout, source plan and normal class element algorithm
are unchanged. This is a general class lifecycle fix, including classes used
as Array-pattern defaults.

The authored strict/sloppy Engine cohort covers named suspended heritage and
computed keys, normal/Return/Throw outcomes, the original class-name TDZ
closure, private environment parent capture, GC around suspension and close,
two nested retained Array iterators, close errors and a yielding outer finally.
It checks exact Return/Throw identity, close precedence, inner-to-outer order
and that exhausted prefixes and static elements do not replay after abrupt
completion. Source review and isolated formatting do not establish runtime
acceptance; compilation and tests remain deferred for the whole batch.
