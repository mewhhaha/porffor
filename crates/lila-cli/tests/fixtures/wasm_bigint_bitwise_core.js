function fail(label) {
  throw "bigint bitwise fixture failed: " + label;
}

function check(actual, expected, label) {
  if (actual !== expected) {
    fail(label + ": " + actual + " !== " + expected);
  }
}

let a = 0x123456789abcdef0fedcba9876543210n;
let b = 0xffff0000ffff0000ffff0000ffff0000n;
check(a & b, 0x123400009abc0000fedc000076540000n, "multi-limb and");
check(a | b, 0xffff5678ffffdef0ffffba98ffff3210n, "multi-limb or");
check(a ^ b, 0xedcb56786543def00123ba9889ab3210n, "multi-limb xor");
check(-1n & a, a, "negative sign extension and");
check(-1n | a, -1n, "negative sign extension or");
check(-1n ^ a, -a - 1n, "negative sign extension xor");
check(Object(a) & Object(b), 0x123400009abc0000fedc000076540000n, "boxed operands");

check(~0n, -1n, "zero complement");
check(~(-1n), 0n, "negative one complement");
check(~a, -a - 1n, "multi-limb positive complement");
check(~(-a), a - 1n, "multi-limb negative complement");
check(~Object(a), -a - 1n, "boxed multi-limb complement");
check(~0, -1, "number zero complement");
check(~(-1), 0, "number negative one complement");
check(~Infinity, -1, "number infinity complement");
check(~(2 ** 63), -1, "number 2^63 complement residue");
check(~1e300, -1, "number huge finite complement residue");
check(~Object(2), -3, "boxed number complement");

let complementTrace = "";
let complementOperand = {
  valueOf: function () {
    complementTrace += "2";
    return a;
  },
};
check(
  (complementTrace += "1", ~complementOperand),
  -a - 1n,
  "object complement"
);
check(complementTrace, "12", "complement evaluates and coerces once");

let complementThrown = {};
let complementCaught;
let complementThrowTrace = "";
let throwingComplement = {
  valueOf: function () {
    complementThrowTrace += "2";
    throw complementThrown;
  },
};
try {
  complementThrowTrace += "1";
  ~throwingComplement;
} catch (error) {
  complementCaught = error;
}
check(complementThrowTrace, "12", "complement preserves ToNumeric throw order");
check(complementCaught, complementThrown, "complement preserves thrown identity");
__lilaAssertThrows(TypeError, function () {
  ~Symbol("complement");
});

check(0x123456789abcdefn << 32n, 0x123456789abcdef00000000n, "left word shift");
check(0x123456789abcdef00000000n >> 32n, 0x123456789abcdefn, "right word shift");
check(1n << 129n, 2n ** 129n, "left multi-limb boundary");
check((2n ** 129n) >> 129n, 1n, "right multi-limb boundary");
check(-5n >> 1n, -3n, "negative arithmetic right rounding");
check(8n << -2n, 2n, "negative left count reverses direction");
check(8n >> -2n, 32n, "negative right count reverses direction");
check(-5n << -1n, -3n, "negative count preserves arithmetic rounding");
check(-5n >> -1n, -10n, "negative count reverses to left shift");
check(5n >> 0x10000000000000000n, 0n, "huge right count saturates positive");
check(-5n >> 0x10000000000000000n, -1n, "huge right count saturates negative");
check(0n << 0x10000000000000000n, 0n, "huge zero left shift needs no allocation");
check(0x10000000000000000n >> 2n, 0x4000000000000000n, "heap result returns inline");

__lilaAssertThrows(RangeError, function () {
  1n << 0x10000000000000000n;
});
__lilaAssertThrows(TypeError, function () {
  1n & 1;
});
__lilaAssertThrows(TypeError, function () {
  1 | 1n;
});
__lilaAssertThrows(TypeError, function () {
  1n >>> 0n;
});
__lilaAssertThrows(TypeError, function () {
  Object(1n) >>> Object(0n);
});

let trace = "";
let orderedLeft = {
  valueOf: function () {
    trace += "3";
    return a;
  },
};
let orderedRight = {
  valueOf: function () {
    trace += "4";
    return b;
  },
};
check(
  (trace += "1", orderedLeft) & (trace += "2", orderedRight),
  0x123400009abc0000fedc000076540000n,
  "ordered object bitwise and"
);
check(trace, "1234", "both expressions precede ordered ToNumeric");

let throwTrace = "";
let throwingLeft = {
  valueOf: function () {
    throwTrace += "3";
    throw "left coercion";
  },
};
let skippedRight = {
  valueOf: function () {
    throwTrace += "4";
    return 1n;
  },
};
try {
  (throwTrace += "1", throwingLeft) & (throwTrace += "2", skippedRight);
} catch (e) {}
check(throwTrace, "123", "left ToNumeric throw skips right ToNumeric");

let unsignedTrace = "";
let unsignedLeft = {
  valueOf: function () {
    unsignedTrace += "3";
    return 1n;
  },
};
let unsignedRight = {
  valueOf: function () {
    unsignedTrace += "4";
    return 0n;
  },
};
try {
  (unsignedTrace += "1", unsignedLeft) >>> (unsignedTrace += "2", unsignedRight);
} catch (e) {}
check(unsignedTrace, "1234", "BigInt unsigned shift throws after both conversions");

let updateTrace = "";
let target = {
  get value() {
    updateTrace += "g";
    return 1n;
  },
  set value(next) {
    updateTrace += "s";
  },
};
try {
  target.value >>>= (updateTrace += "r", 0n);
} catch (e) {}
check(updateTrace, "gr", "throwing compound shift suppresses PutValue");

check(Infinity | 0, 0, "number positive infinity ToInt32");
check(Infinity >>> 0, 0, "number positive infinity ToUint32");
check(2 ** 63 | 0, 0, "number 2^63 residue");
check(1e300 | 0, 0, "number huge finite residue");
check(1 << Infinity, 1, "number infinite shift count");
check(1 << 2 ** 63, 1, "number huge finite shift count");
check(-1 >>> 0, 4294967295, "number unsigned reading");

true;
