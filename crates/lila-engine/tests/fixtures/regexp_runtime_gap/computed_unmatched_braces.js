var patterns = [
  [123], [123, 125], [123, 49], [123, 49, 44], [123, 49, 44, 120, 125],
  [123, 49, 44, 50], [97, 123], [97, 123, 49], [97, 123, 49, 44],
  [97, 123, 49, 44, 120, 125], [97, 123, 49, 44, 50]
];
for (var units of patterns) {
  var pattern = fromUnits(units);
  var legacy = new RegExp(pattern);
  require(legacy.test(pattern), 'Annex B retains unmatched braces as literals');
  var legacyRecompiled = /old/;
  legacyRecompiled.compile(pattern);
  require(legacyRecompiled.test(pattern), 'legacy recompilation retains literal grammar');
  for (var mode = 117; mode <= 118; mode++) {
    var flags = fromUnits([mode]);
    var caught = false;
    try { new RegExp(pattern, flags); }
    catch (error) { caught = error instanceof SyntaxError; }
    require(caught, 'Unicode constructors reject unmatched braces');
    var retained = /test262/ig;
    retained.lastIndex = 2;
    caught = false;
    try { retained.compile(pattern, flags); }
    catch (error) { caught = error instanceof SyntaxError; }
    require(caught, 'Unicode recompilation rejects unmatched braces');
    require(retained.source === 'test262' && retained.flags === 'gi' && retained.lastIndex === 2,
      'failed recompilation preserves source, flags and lastIndex');
    var match = retained.exec('xxTeSt262');
    require(match && match.index === 2 && match[0] === 'TeSt262',
      'failed recompilation preserves the old matcher');
  }
}
for (var mode = 117; mode <= 118; mode++) {
  var flags = fromUnits([mode]);
  var quantified = new RegExp(fromUnits([94, 97, 123, 49, 44, 50, 125, 36]), flags);
  require(quantified.test('a') && quantified.test('aa') && !quantified.test('aaa'),
    'complete Unicode quantifiers keep their bounds');
  var escaped = new RegExp(fromUnits([92, 123, 49, 92, 125]), flags);
  require(escaped.test('{1}'), 'escaped Unicode braces remain literal');
  var caught = false;
  try { new RegExp(fromUnits([92, 50]), flags); }
  catch (error) { caught = error instanceof SyntaxError; }
  require(caught, 'unbound Unicode numeric backreferences still reject');
}
print('ok');
262;
