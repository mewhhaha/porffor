function computed(pattern, flags) {
  var units = [];
  for (var index = 0; index < pattern.length; index++) units.push(pattern.charCodeAt(index));
  return new RegExp(fromUnits(units), flags);
}

var spaces = fromUnits([9, 32, 160, 5760, 8192, 8202, 8232, 8233, 8239, 8287, 12288, 65279]);
var nonspaces = fromUnits([0, 133, 6158, 8203, 65535]);
var positive = [/^[\s]+$/, /^[\s]+$/u, /^[\s]+$/v];
var negative = [/[^\s]+/, /[^\s]+/u, /[^\s]+/v];
var reverse = [/(?<=[\s])x/, /(?<=[\s])x/u, /(?<=[\s])x/v];
var flags = ['', 'u', 'v'];
for (var mode = 0; mode < flags.length; mode++) {
  var dynamicPositive = computed('^[\\s]+$', flags[mode]);
  var dynamicNegative = computed('[^\\s]+', flags[mode]);
  var dynamicReverse = computed('(?<=[\\s])x', flags[mode]);
  require(positive[mode].test(spaces) && dynamicPositive.test(spaces), 'complete whitespace class');
  require(!positive[mode].test(nonspaces) && !dynamicPositive.test(nonspaces), 'non-whitespace exclusions');
  var text = spaces + nonspaces;
  var literalMatch = negative[mode].exec(text);
  var dynamicMatch = dynamicNegative.exec(text);
  require(literalMatch[0] === nonspaces && dynamicMatch[0] === nonspaces &&
          literalMatch.index === spaces.length && dynamicMatch.index === spaces.length,
          'class negation retains every whitespace member');
  require(reverse[mode].exec('\u00A0x').index === 1 &&
          dynamicReverse.exec('\u00A0x').index === 1, 'reverse range class');
}

var octals = [/^[\200]$/, /^[\377]$/, /^[\177-\377]+$/];
var patterns = ['^[\\200]$', '^[\\377]$', '^[\\177-\\377]+$'];
var members = ['\u0080', '\u00FF', '\u007F\u0080\u00FE\u00FF'];
for (var index = 0; index < octals.length; index++) {
  var dynamic = computed(patterns[index], '');
  require(octals[index].test(members[index]) && dynamic.test(members[index]), 'full octal byte domain');
  require(!octals[index].test('A') && !dynamic.test('A') &&
          !octals[index].test('\u0100') && !dynamic.test('\u0100'), 'octal class excludes aliased bits');
}
require(/^[^\200]+$/.test('A\u00FF') && !/^[^\200]+$/.test('\u0080') &&
        computed('^[^\\200]+$', '').test('A\u00FF') &&
        !computed('^[^\\200]+$', '').test('\u0080'), 'octal class complement');
require(/^[\s-z]+$/.test('\u00A0-z') && computed('^[\\s-z]+$', '').test('\u00A0-z'),
        'Annex B class-set range keeps its whitespace operand');
for (var pair of [['[\\200]', 'u'], ['[\\377]', 'v'], ['[\\377-\\200]', ''], ['[\\s-z]', 'u']]) {
  var rejected = false;
  try { computed(pair[0], pair[1]); } catch (error) { rejected = error instanceof SyntaxError; }
  require(rejected, 'class-domain selection preserves invalid grammar');
}
print('ok');
262;
