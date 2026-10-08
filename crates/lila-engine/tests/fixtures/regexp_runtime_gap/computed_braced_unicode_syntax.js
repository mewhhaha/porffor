function syntax(units, flags) {
  var caught = false;
  try { new RegExp(fromUnits(units), fromUnits(flags)); }
  catch (error) { caught = error instanceof SyntaxError; }
  require(caught, 'malformed braced escape retains intrinsic SyntaxError');
}
for (var mode = 117; mode <= 118; mode++) {
  for (var units of [
    [92, 117, 123, 125],
    [92, 117, 123, 52, 49],
    [92, 117, 123, 71, 125],
    [92, 117, 123, 52, 95, 49, 125],
    [92, 117, 123, 45, 49, 125],
    [92, 117, 123, 49, 49, 48, 48, 48, 48, 125]
  ]) {
    syntax(units, [mode]);
  }
  syntax([91, 92, 117, 123, 52, 50, 125, 45, 92, 117, 123, 52, 49, 125, 93], [mode]);
  syntax([91, 92, 100, 45, 92, 117, 123, 52, 49, 125, 93], [mode]);
  syntax([40, 63, 60, 92, 117, 123, 48, 125, 62, 120, 41], [mode]);
  syntax([40, 63, 60, 92, 117, 123, 68, 56, 48, 48, 125, 62, 120, 41], [mode]);
  syntax([40, 63, 60, 92, 117, 123, 68, 70, 70, 70, 125, 62, 120, 41], [mode]);
  syntax([40, 63, 60, 97, 92, 117, 123, 50, 49, 125, 62, 120, 41], [mode]);
  var retained = new RegExp(fromUnits([120]), fromUnits([103]));
  retained.lastIndex = 7;
  var caught = false;
  try { retained.compile(fromUnits([92, 117, 123, 49, 49, 48, 48, 48, 48, 125]), fromUnits([mode])); }
  catch (error) { caught = error instanceof SyntaxError; }
  require(caught && retained.source === 'x' && retained.flags === 'g' && retained.lastIndex === 7, 'invalid braced recompile leaves receiver transaction intact');
  retained.compile(fromUnits([92, 117, 123, 54, 49, 125]), fromUnits([mode]));
  require(retained.lastIndex === 0 && retained.test('a'), 'valid braced recompile publishes the complete new program');
}
print('ok');
262;
