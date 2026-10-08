var nested = [[[7]]], middle = nested[0], leaf = middle[0], source = [nested, 8];
var zeroDepths = [-0, -1, -Infinity, NaN, 0.9, null];
for (var i = 0; i < zeroDepths.length; i++) {
  var result = source.flat(zeroDepths[i]);
  print(result.length === 2 && result[0] === nested && result[1] === 8);
}
print(source.flat()[0] === middle);
print(source.flat(undefined)[0] === middle);
print(source.flat(true)[0] === middle);
print(source.flat('2.9')[0] === leaf);
print(source.flat(Infinity)[0]);
print(source.flat(1e300)[0]);
var ordinary = { valueOf: function () { print('valueOf'); return {}; }, toString: function () { print('toString'); return '2.9'; } };
print(source.flat(ordinary)[0] === leaf);
var exotic = { [Symbol.toPrimitive]: function (hint) { print(hint); return 1; } };
print(source.flat(exotic)[0] === middle);
try { source.flat(Symbol()); } catch (error) { print(error instanceof TypeError); }
try { source.flat(1n); } catch (error) { print(error instanceof TypeError); }
var numbers = [[-0, NaN, Infinity]].flat(Infinity);
print(Object.is(numbers[0], -0) + ':' + Number.isNaN(numbers[1]) + ':' + (numbers[2] === Infinity));
