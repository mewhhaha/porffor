var expression = /a/g;
expression.lastIndex = 7;
var rejected = false;
try { expression.compile(fromUnits([97]).repeat(32769)); }
catch (error) { rejected = error instanceof RangeError; }
require(rejected, 'actual source instruction capacity retains its explicit RangeError');
require(expression.source === 'a' && expression.flags === 'g' && expression.lastIndex === 7,
        'resource failure must retain receiver state');
expression.lastIndex = 0;
require(expression.exec('a')[0] === 'a', 'old program survives resource failure');
print('ok');
262;
