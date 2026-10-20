function expression(units, flags) { return new RegExp(fromUnits(units), fromUnits(flags)); }
function capture(name, body) { return [40, 63, 60].concat(name, [62], body, [41]); }
function reference(name) { return [92, 107, 60].concat(name, [62]); }
function anchored(body) { return [94].concat(body, [36]); }
var faceUnits = [0xd83d, 0xde00];
var face = fromUnits(faceUnits);
var capitalUnits = [0xd801, 0xdc00];
var capital = fromUnits(capitalUnits);
var small = fromUnits([0xd801, 0xdc28]);
var kelvin = fromUnits([0x212a]);
for (var mode = 117; mode <= 118; mode++) {
  var pairName = [112, 97, 105, 114];
  var repeated = expression(anchored(capture(pairName, faceUnits).concat(reference(pairName), [123, 50, 125])), [100, 103, mode]);
  var match = repeated.exec(face + face + face);
  require(match && match[0] === face + face + face && match.groups.pair === face, 'named astral backreference is one quantified atom');
  require(match.indices[0][0] === 0 && match.indices[0][1] === 6 && match.indices[1][0] === 0 && match.indices[1][1] === 2, 'Unicode comparison retains UTF-16 indices');
  require(match.indices.groups.pair === match.indices[1] && Object.getPrototypeOf(match.groups) === null, 'canonical named results retain capture identity and null prototype');
  require(repeated.lastIndex === 6 && repeated.exec(face + face + face) === null && repeated.lastIndex === 0, 'global named success and failure retain lastIndex policy');

  var later = [108, 97, 116, 101, 114];
  var forward = expression(anchored(reference(later).concat(capture(later, faceUnits))), [100, mode]).exec(face);
  require(forward && forward.groups.later === face && forward.indices.groups.later[1] === 2, 'forward named reference resolves after the complete capture census');
  var missing = [109, 105, 115, 115, 105, 110, 103];
  var optional = expression(anchored(capture(missing, faceUnits).concat([63], reference(missing), [98])), [100, mode]).exec('b');
  require(optional && optional.groups.missing === undefined && optional.indices.groups.missing === undefined, 'nonparticipating named capture consumes no input');

  var item = [105, 116, 101, 109];
  var other = [111, 116, 104, 101, 114];
  var duplicates = expression(anchored([40, 63, 58].concat(capture(item, faceUnits), [124], capture(other, [120]), [124], capture(item, [97]), [41], reference(item))), [100, mode]);
  var left = duplicates.exec(face + face);
  var right = duplicates.exec('aa');
  var middle = duplicates.exec('x');
  require(left && right && left.groups.item === face && right.groups.item === 'a', 'disjoint duplicate names select the participating capture');
  require(left.indices.groups.item === left.indices[1] && right.indices.groups.item === right.indices[3], 'Unicode named candidate table retains numbered identity');
  require(middle && middle.groups.item === undefined && middle.groups.other === 'x', 'all unmatched candidates retain the empty reference rule');

  var fold = [102, 111, 108, 100];
  var foldedPattern = anchored(capture(fold, capitalUnits).concat(reference(fold)));
  var folded = expression(foldedPattern, [100, 105, mode]).exec(capital + small);
  require(folded && folded.groups.fold === capital && folded.indices[0][1] === 4, 'Unicode named comparison folds full supplementary characters');
  require(!expression(foldedPattern, [mode]).test(capital + small), 'named comparison stays sensitive without i');
  require(expression(anchored(capture(fold, [0x212a]).concat(reference(fold))), [105, mode]).test(kelvin + 'k'), 'Unicode named comparison admits Kelvin ASCII equivalence');
  var scoped = expression(anchored([40, 63, 45, 105, 58].concat(capture(fold, capitalUnits), [41, 40, 63, 105, 58], reference(fold), [41, 40, 63, 45, 105, 58], reference(fold), [41])), [100, 105, mode]);
  require(scoped.test(capital + small + capital) && !scoped.test(capital + small + small) && !scoped.test(small + small + capital), 'reference-site scoped i is restored independently of global Unicode mode');

  var reverseName = [114, 101, 118, 101, 114, 115, 101];
  var reverse = expression([40, 63, 60, 61, 40, 63, 105, 58].concat(reference(reverseName), [41], capture(reverseName, capitalUnits), [41, 98]), [100, mode]).exec(small + capital + 'b');
  require(reverse && reverse.index === 4 && reverse[0] === 'b' && reverse.groups.reverse === capital, 'named Unicode comparison follows reverse lookbehind direction');
  require(reverse.indices.groups.reverse[0] === 2 && reverse.indices.groups.reverse[1] === 4, 'reverse capture publishes forward UTF-16 boundaries');

  var alpha = [0x03b1];
  var escapedAlpha = [92, 117, 48, 51, 66, 49];
  var greek = expression(anchored(capture(escapedAlpha, [97]).concat(reference(alpha))), [100, mode]).exec('aa');
  require(greek && greek.groups[fromUnits(alpha)] === 'a', 'fixed and direct Unicode names share canonical identity');
  var bracedName = [92, 117, 123, 49, 48, 52, 48, 48, 125];
  var fixedName = [92, 117, 68, 56, 48, 49, 92, 117, 68, 67, 48, 48];
  var astralName = expression(anchored(capture(bracedName, faceUnits).concat(reference(fixedName))), [100, mode]).exec(face + face);
  require(astralName && astralName.groups[capital] === face && astralName.indices.groups[capital] === astralName.indices[1], 'braced and paired fixed name escapes resolve one supplementary identifier');
  require(expression(anchored(capture(capitalUnits, [97]).concat(reference(bracedName))), [mode]).test('aa'), 'direct supplementary names resolve braced references');
  var escapedJoiner = [97, 92, 117, 50, 48, 48, 67];
  var directJoiner = [97, 0x200c];
  require(expression(anchored(capture(escapedJoiner, [97]).concat(reference(directJoiner))), [mode]).test('aa'), 'escaped and direct identifier continuation characters retain exact identity');
  var distinct = expression(anchored(capture([75], [97]).concat(capture([0x212a], [98]), reference([75]), reference([0x212a]))), [105, mode]).exec('abab');
  require(distinct && distinct.groups.K === 'a' && distinct.groups[kelvin] === 'b', 'input folding never folds capture names');

  var sticky = expression(capture(pairName, faceUnits).concat(reference(pairName)), [100, 121, mode]);
  sticky.lastIndex = 1;
  var stickyMatch = sticky.exec('x' + face + face);
  require(stickyMatch && stickyMatch.index === 1 && sticky.lastIndex === 5, 'sticky named comparison retains UTF-16 lastIndex');
  require(sticky.exec('x' + face + face) === null && sticky.lastIndex === 0, 'sticky named failure resets lastIndex');
  var retained = expression(anchored(capture([110], [97]).concat(reference([110]))), [mode]);
  var receiver = /old/g;
  receiver.lastIndex = 7;
  receiver.compile(fromUnits(capture(bracedName, faceUnits).concat(reference(fixedName))), fromUnits([100, mode]));
  var installed = receiver.exec(face + face);
  require(receiver.lastIndex === 0 && installed && installed.groups[capital] === face && installed.indices.groups[capital][1] === 2, 'successful compile installs a completed named program');
  require(retained.test('aa') && repeated.test(face + face + face), 'later publication leaves retained named descriptors usable');
}
require(expression(anchored(reference([110])), []).test('k<n>'), 'Legacy without named groups retains k identity');
require(expression([94, 92, 107, 36], []).test('k'), 'Legacy bare k remains an identity escape');
require(expression([94, 91, 92, 107, 45, 109, 93, 36], []).test('l'), 'Legacy class range retains k identity without named groups');
require(!expression(anchored(capture([110], [0x212a]).concat(reference([110]))), [105]).test(kelvin + 'k'), 'Legacy named comparison retains its distinct folding table');
require(expression(anchored(capture([92, 117, 123, 54, 49, 125], [97]).concat(reference([92, 117, 48, 48, 54, 49]))), []).test('aa'), 'Legacy named census still selects canonical named references');
print('ok');
262;
