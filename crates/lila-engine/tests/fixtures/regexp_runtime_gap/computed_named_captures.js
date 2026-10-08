// Reconstruct every pattern through runtime UTF-16 units to avoid a finite
// literal RegExp table entry answering before the emitted compiler is reached.
function computedPattern(source) {
  var units = [];
  for (var i = 0; i < source.length; i++) units.push(source.charCodeAt(i));
  return fromUnits(units);
}
function computedRegExp(source, flags) { return new RegExp(computedPattern(source), flags); }
function rejects(source) {
  var error;
  try { computedRegExp(source, ''); } catch (caught) { error = caught; }
  require(error instanceof SyntaxError, 'invalid name syntax must throw SyntaxError: ' + source);
}
// Exercise workspace padding with both ASCII source-byte parities, including
// unnamed programs: temporary descriptor validation precedes heap compaction.
var boundaryPatterns = ['', 'a', 'ab', 'abc', '[a]', '[ab]'];
var boundaryInputs = ['', 'a', 'ab', 'abc', 'a', 'b'];
var alignmentReceiver = /old/;
for (var boundary = 0; boundary < boundaryPatterns.length; boundary++) {
  var boundarySource = computedPattern(boundaryPatterns[boundary]);
  var boundaryMatch = new RegExp(boundarySource, '').exec(boundaryInputs[boundary]);
  require(boundaryMatch[0] === boundaryInputs[boundary], 'unnamed source-byte alignment boundary');
  alignmentReceiver.compile(boundarySource, '');
  require(alignmentReceiver.exec(boundaryInputs[boundary])[0] === boundaryInputs[boundary], 'repeated workspace/heap alignment boundary');
}
var namedBoundaryPatterns = ['(?<n>a)', '(?<nn>a)'];
var namedBoundaryKeys = ['n', 'nn'];
for (var namedBoundary = 0; namedBoundary < namedBoundaryPatterns.length; namedBoundary++) {
  var namedBoundaryMatch = computedRegExp(namedBoundaryPatterns[namedBoundary], 'd').exec('a');
  require(namedBoundaryMatch.groups[namedBoundaryKeys[namedBoundary]] === 'a', 'named source-byte alignment boundary');
  require(namedBoundaryMatch.indices.groups[namedBoundaryKeys[namedBoundary]] === namedBoundaryMatch.indices[1], 'aligned named table remains canonical');
}
require(computedRegExp('(?<n>a)\\k<n>', '').exec('aa')[0] === 'aa', 'even named-reference byte boundary');
require(computedRegExp('(?<n>ab)\\k<n>', '').exec('abab')[0] === 'abab', 'odd named-reference byte boundary');
var expression = computedRegExp('^(z)(?<pair>ab)\\k<pair>$', 'dg');
var match = expression.exec('zabab');
require(match[0] === 'zabab' && match[1] === 'z' && match[2] === 'ab', 'capture numbering');
require(match.groups.pair === 'ab' && Object.getPrototypeOf(match.groups) === null, 'null prototype groups');
require(match.indices.groups.pair === match.indices[2] && match.indices[2][0] === 1 && match.indices[2][1] === 3, 'indices use capture identity');
require(expression.lastIndex === 5, 'global lastIndex advances');
require(expression.exec('zabab') === null && expression.lastIndex === 0, 'global failure resets');
var forward = computedRegExp('^\\k<later>(?<later>a)$', 'd').exec('a');
require(forward[0] === 'a' && forward.groups.later === 'a', 'forward reference matches empty before capture');
var optional = computedRegExp('^(?<missing>a)?\\k<missing>b$', 'd').exec('b');
require(optional[1] === undefined && optional.groups.missing === undefined, 'nonparticipating capture remains undefined');
require(optional.indices[1] === undefined && optional.indices.groups.missing === undefined, 'nonparticipating indices remain undefined');
var folded = computedRegExp('^(?<fold>aB)\\k<fold>$', 'i').exec('aBAb');
require(folded[0] === 'aBAb' && folded.groups.fold === 'aB', 'backreference uses scoped folding');
var duplicates = computedRegExp('^(?:(?<item>a)|(?<other>x)|(?<item>b))\\k<item>$', 'd');
var left = duplicates.exec('aa');
var right = duplicates.exec('bb');
var middle = duplicates.exec('x');
require(left.groups.item === 'a' && right.groups.item === 'b', 'duplicate names on distinct alternatives resolve participating capture');
require(left.indices.groups.item === left.indices[1] && right.indices.groups.item === right.indices[3], 'duplicate named indices share candidate identity');
require(left[3] === undefined && right[1] === undefined, 'inactive alternative captures are undefined');
require(middle.groups.item === undefined && middle.groups.other === 'x' && middle.indices.groups.other === middle.indices[2], 'distinct names and nonadjacent candidates retain canonical records');
// Escaped and direct spellings must resolve to one canonical Unicode name.
var greek = computedRegExp('^(?<\\u03B1>a)\\k<α>$', 'd').exec('aa');
require(greek.groups['α'] === 'a', 'fixed escaped Unicode name');
var astral = computedRegExp('^(?<\\u{10400}>a)\\k<\\uD801\\uDC00>$', '').exec('aa');
require(astral.groups['𐐀'] === 'a', 'braced and paired escaped astral names');
var direct = computedRegExp('^(?<𐐀>a)\\k<\\u{10400}>$', '').exec('aa');
require(direct.groups['𐐀'] === 'a', 'direct astral name');
var joiner = computedRegExp('^(?<a\\u200C>a)\\k<a‌>$', '').exec('aa');
require(joiner.groups['a‌'] === 'a', 'joiner is a continuation');
require(computedRegExp('^\\k<identity>$', '').test('k<identity>'), 'legacy identity escape without named captures');
require(computedRegExp('^[\\k]$', '').test('k'), 'class identity escape without named captures');
require(computedRegExp('^[\\k-m]$', '').test('l'), 'class range start identity without named captures');
require(computedRegExp('^[i-\\k]$', '').test('j'), 'class range end identity without named captures');
require(computedRegExp('(?<a>a)[k]', '').test('ak'), 'ordinary class k remains valid with named captures');
rejects('(?<a>a)[\\k]');
rejects('[\\k](?<a>a)');
rejects('(?<a>a)[\\k-m]');
rejects('[i-\\k](?<a>a)');
rejects('(?<>a)');
rejects('(?<1bad>a)');
rejects('(?<\\u{D800}>a)');
rejects('(?<\\uD801>a)');
rejects('(?<\\u{110000}>a)');
rejects('(?<\\u{}>a)');
rejects('(?<\\x61>a)');
rejects('(?<same>a)(?<same>b)');
rejects('(?<a>x)(?<\\u0061>y)');
rejects('(?:(?<same>a)|b)(?<same>c)');
rejects('(?<a>x)\\k<missing>');
rejects('(?<a>x)\\k');
var receiver = /old/g;
receiver.lastIndex = 3;
var failed = false;
try { receiver.compile(computedPattern('(?<same>a)(?<same>b)'), 'g'); }
catch (error) { failed = error instanceof SyntaxError; }
require(failed && receiver.source === 'old' && receiver.lastIndex === 3, 'invalid recompile is transactional');
receiver.compile(computedPattern('(?<fresh>x)\\k<fresh>'), 'dg');
var fresh = receiver.exec('xx');
require(fresh.groups.fresh === 'x' && fresh.indices.groups.fresh === fresh.indices[1], 'valid recompile publishes canonical names');
var sticky = computedRegExp('(?<single>x)\\k<single>', 'dy');
sticky.lastIndex = 1;
require(sticky.exec('axx').index === 1 && sticky.lastIndex === 3, 'sticky named matching uses lastIndex');
require(sticky.exec('axx') === null && sticky.lastIndex === 0, 'sticky named failure resets lastIndex');
print('ok');
262;
