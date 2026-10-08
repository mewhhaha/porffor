var child = [1, [2, 3]], root = [child, [4]];
Object.defineProperty(child, '0', { get: function () {
  print('enter');
  var nested = [[[[7]]]].flat(Infinity);
  print('inner:' + nested[0]);
  return [1];
} });
var result = root.flat(Infinity);
print(result.length + ':' + result[0] + ':' + result[1] + ':' + result[2] + ':' + result[3]);
