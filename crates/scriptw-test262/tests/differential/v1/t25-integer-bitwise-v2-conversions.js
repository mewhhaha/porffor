// Hand-authored semantic regression, not an executed generator artifact.
function check(actual, expected) {
  if (!Object.is(actual, expected)) throw 'integer bitwise conversion';
}
check(4294967296 | 5, 5);
check(-4294967295 & 5, 1);
check(-4294967297 ^ 0, -1);
check(9007199254740991 ^ -1, 0);
check(~4294967303, -8);
check(~9007199254740991, 0);
check(1 << 31, -2147483648);
check(1 << 32, 1);
check(1 << 33, 2);
check(1 << -1, -2147483648);
check(1 << -33, -2147483648);
check(1 << 4294967328, 1);
check(2147483648 >> 31, -1);
check(-1 >> 32, -1);
check(-1 >>> 1, 2147483647);
check(-1 >>> 32, 4294967295);
check(9007199254740991 >>> 0, 4294967295);
check((-1 >>> 0) + 1, 4294967296);
check(((-1 >>> 0) + 1) | 7, 7);
check(1 - 1, 0);
check(-1 + 1, 0);
true;
