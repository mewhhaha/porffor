function check(value, label) { if (!value) throw label; }
var whitespace = ['\u0009','\u000a','\u000b','\u000c','\u000d',' ','\u00a0','\u1680',
  '\u2000','\u2001','\u2002','\u2003','\u2004','\u2005','\u2006','\u2007','\u2008','\u2009','\u200a',
  '\u2028','\u2029','\u202f','\u205f','\u3000','\ufeff'];
for (var i = 0; i < whitespace.length; i++) {
  check(parseInt(whitespace[i] + '-0x10tail') === -16, 'parseInt exact leading whitespace');
  check(parseFloat(whitespace[i] + '-12.5e1tail') === -125, 'parseFloat exact leading whitespace');
}
for (var rejected of ['\u0000','\u0001','\u0008','\u000e','\u001f','\u0085','\u180e','\u200b','\ud800']) {
  check(Number.isNaN(parseInt(rejected + '1')), 'parseInt excludes non-whitespace');
  check(Number.isNaN(parseFloat(rejected + '1')), 'parseFloat excludes non-whitespace');
}
for (var parser of [parseInt, Number.parseInt]) {
  check(parser('101z', 2) === 5, 'radix digit prefix');
  check(parser('z!', 36) === 35, 'radix36');
  check(parser('0x10', 10) === 0, 'decimal does not strip hex');
  check(parser('0Xf', 16) === 15, 'explicit hex prefix');
  check(Number.isNaN(parser('1', -4294967295)), 'modular invalid radix1');
  check(parser('10', 4294967312.75) === 16, 'modular radix16');
  check(Object.is(parser('-0'), -0), 'negative integer zero');
}
for (var parser of [parseFloat, Number.parseFloat]) {
  check(parser('0x10') === 0, 'float decimal prefix');
  check(parser('1e+') === 1 && parser('1e-') === 1 && parser('.5e2x') === 50, 'longest exponent prefix');
  check(parser('Infinitysuffix') === Infinity && parser('-Infinity!') === -Infinity, 'signed infinity prefix');
  check(Number.isNaN(parser('+Infinity'.toLowerCase())) && Number.isNaN(parser('.')), 'invalid float tokens');
  check(Object.is(parser('-0.0suffix'), -0), 'negative float zero');
  check(parser('12\ud800tail') === 12, 'lone surrogate terminates prefix');
}
var trace = [], radix = {valueOf() { trace.push('radix'); return 16; }};
var input = {toString() { trace.push('string'); return '10'; }};
check(parseInt(input, radix) === 16 && trace.join('|') === 'string|radix', 'ToString before ToInt32');
var marker = Symbol('numeric host abrupt'), reached = 0, caught;
try { parseInt({toString() { throw marker; }}, {valueOf() { reached++; return 10; }}); } catch (error) { caught = error; }
check(caught === marker && reached === 0, 'whole first coercion abrupt');
print('native-host-numeric:ok');
262;
