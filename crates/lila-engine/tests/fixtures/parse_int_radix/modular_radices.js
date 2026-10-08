function check(value, label) { if (!value) throw new Error(label); }
var parsers = [parseInt, Number.parseInt];
var multiple = 4294967296 * 4096;
for (var p = 0; p < parsers.length; p++) {
  var parse = parsers[p];
  check(parse('10', 4294967312.75) === 16, 'positive fractional residue');
  check(parse('10', -4294967280.75) === 16, 'negative fractional residue');
  check(parse('10', multiple + 16.75) === 16, 'large positive fractional residue');
  check(parse('10', -multiple + 16.75) === 17, 'negative truncation toward zero');
  check(parse('10', 72057594037927950) === 16, 'finite radix above exact integer range');
  check(parse('10', -4294967292) === 4, 'negative wrapped valid radix');
  check(parse('10', 4294967298.9) === 2, 'binary wrapped radix');
  check(parse('z', 36) === 35, 'maximum valid radix');
  check(parse('0x10', 16) === 16 && parse('0x10', 8) === 0, 'explicit prefix policy');
  var invalid = [1, 37, -1, -multiple - 16.75, 2147483648, 4294967295,
                 4294967297.9, -4294967297.9];
  for (var r = 0; r < invalid.length; r++) {
    check(Number.isNaN(parse('10', invalid[r])), 'invalid signed radix');
  }
}
