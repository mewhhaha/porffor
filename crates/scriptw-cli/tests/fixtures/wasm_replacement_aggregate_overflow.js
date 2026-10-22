// Only run the new compiled path after its aggregate-length guard is present.
// Reusing one completed piece allows overflow to be rejected before huge allocation.
function check(ok, name) { if (!ok) throw new Error(name); }
const piece = 'b'.repeat(1 << 20);
const input = 'a'.repeat(4096);
for (let variant = 0; variant < 2; variant++) {
  let calls = 0;
  try {
    const replacer = function () { calls++; return piece; };
    if (variant === 0) input.replaceAll('a', replacer);
    else input.replace(/a/g, replacer);
    throw new Error('aggregate overflow was accepted');
  } catch (e) {
    check(e instanceof RangeError && calls > 0 && calls <= input.length, 'catchable aggregate overflow');
  }
}
print('replacement-aggregate-overflow-ok');
