function check(condition, label) { if (!condition) throw new Error(label); }
var result = JSON.parse('[[0],[],[{"name":"A\\uD800B","name":"\\uDFFF"}],null,false,true,-0,1.25e+3,1e400,-1e400,1e-4000]', function (key, value, context) {
  if (key === 'name') check(context.source === '"\\uDFFF"', 'final exact source spelling');
  if (typeof value !== 'object' && context.source === undefined) throw new Error('lost primitive source');
  return value;
});
check(result.length === 11 && result[0].length === 1 && result[0][0] === 0 && result[1].length === 0, 'nested lengths');
check(result[2][0].name.length === 1 && result[2][0].name.charCodeAt(0) === 0xdfff, 'nested UTF16');
check(result[3] === null && result[4] === false && result[5] === true, 'nested primitive tags');
check(Object.is(result[6], -0) && result[7] === 1250 && result[8] === Infinity && result[9] === -Infinity && Object.is(result[10], 0), 'numeric bits');
print('fresh-array-nesting:ok');
