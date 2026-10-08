function check(value, label) { if (!value) throw new Error(label); }
check(parseInt === Number.parseInt, 'canonical parseInt alias');
var parsers = [parseInt, Number.parseInt];
var radices = [undefined, null, NaN, Infinity, -Infinity, 0, -0, 1e308, -1e308,
               9223372036854775808, -9223372036854775808,
               18446744073709551616, -18446744073709551616];
for (var p = 0; p < parsers.length; p++) {
  var parse = parsers[p];
  for (var r = 0; r < radices.length; r++) {
    check(parse('0x10', radices[r]) === 16, 'default hexadecimal prefix');
    check(parse('10', radices[r]) === 10, 'default decimal radix');
    check(parse(' -0X10tail', radices[r]) === -16, 'sign and prefix order');
    check(Object.is(parse('-0', radices[r]), -0), 'negative zero result');
  }
}
