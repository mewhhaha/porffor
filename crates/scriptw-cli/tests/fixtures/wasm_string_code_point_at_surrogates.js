function check(value, label) {
  if (!value) {
    throw "String codePointAt surrogate fixture failed: " + label;
  }
}

check("\uD800\uDBFF".codePointAt(0) === 0xD800, "lead followed by lead");
check("\uD800\uE000".codePointAt(0) === 0xD800, "lead followed by non-trail");
check("\uDC00\uAAAA".codePointAt(0) === 0xDC00, "trail at position");
check("123\uD800".codePointAt(3) === 0xD800, "lead at final position");
check("\uD800\uDC00".codePointAt(0) === 0x10000, "valid pair");
check("\uD800\uDC00".codePointAt(1) === 0xDC00, "second code unit");
check(String.fromCharCode(0xD800).codePointAt(0) === 0xD800, "fromCharCode lead");

function fromCharCodeUnit(value) {
  return String.fromCharCode(value).charCodeAt(0);
}

check(fromCharCodeUnit(Infinity) === 0, "fromCharCode positive infinity");
check(fromCharCodeUnit(-Infinity) === 0, "fromCharCode negative infinity");
check(fromCharCodeUnit(2 ** 63 + 4096) === 4096, "fromCharCode large positive residue");
check(fromCharCodeUnit(-(2 ** 63) - 4096) === 61440, "fromCharCode large negative residue");

true;
