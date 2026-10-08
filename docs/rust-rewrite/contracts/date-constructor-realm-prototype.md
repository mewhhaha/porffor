# Date construction with a complete GC record

Status: source-authored for atomic T05 on 2026-10-04; no current compile or
execution receipt exists. No partial Date-family integration is authorized.

The actual constructor retains the whole NewTarget value, completes argument
conversion and TimeClip, then performs GetPrototypeFromConstructor with the
called intrinsic Date default. Its successful result supplies the complete
ordinary object header before the concrete GC Date record is published. Throws
from coercion and prototype lookup retain their original values and suppress the
remaining stages. Borrowed constructors use their defining-Realm intrinsic.

Zero arguments use the injected host clock. One real branded Date copies its
private time value without ToPrimitive or public property reads. Other single
arguments use Default ToPrimitive, then immutable UTF-16 parsing or ToNumber.
Multiple arguments and Date.UTC convert present arguments in order, distinguish
omission from explicit Undefined, apply the 0..99 full-year adjustment and use
the shared typed calendar kernels. The Date function call ignores its arguments
and formats the current host time in the immutable configured zone.

Existing Date system-zone and called-Realm Engine controls remain the executable
acceptance criteria. Source authorship and isolated formatting do not establish
prototype, completion, runtime or conformance success.
