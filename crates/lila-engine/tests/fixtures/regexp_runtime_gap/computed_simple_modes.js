for (var i = 0; i < 2; i++) {
  var flags = fromUnits([i === 0 ? 117 : 118]);
  var expression = new RegExp(fromUnits([97]), flags);
  require(expression.test('a') && !expression.test('b'), 'clean computed u/v pattern');
}
print('ok');
262;
