var expression = /a/g;
expression.lastIndex = 7;
var rejected = false;
try { expression.compile(fromUnits([92,65]), fromUnits([117])); }
catch (error) { rejected = error instanceof SyntaxError; }
require(rejected, 'invalid Unicode identity escape must be SyntaxError');
require(expression.source === 'a' && expression.flags === 'g' && expression.lastIndex === 7,
        'syntax failure must retain receiver state');
expression.lastIndex = 0;
require(expression.exec('a')[0] === 'a', 'old program survives syntax failure');
var invalidSetEscape = false;
try { new RegExp(fromUnits([91,92,113,93]), fromUnits([118])); }
catch (error) { invalidSetEscape = error instanceof SyntaxError; }
require(invalidSetEscape, 'v class string escape needs a brace opener');
print('ok');
262;
