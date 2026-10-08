function assert(value, message) { if (!value) throw new Error(message); }
var trace = '', left = { valueOf() { trace += 'old'; return 1; } };
var right = { valueOf() { trace += 'R'; return 5; } };
function evaluateLeft() { trace += 'l'; return left; }
function evaluateRight(value) { trace += 'r'; return value; }
function* add() { return evaluateLeft() + evaluateRight(yield 'rhs'); }
var adding = add();
assert(adding.next().value === 'rhs' && trace === 'l', 'LHS GetValue before RHS yield without coercion');
left.valueOf = function () { trace += 'L'; return 4; };
gc();
var sum = adding.next(right);
assert(sum.done && sum.value === 9 && trace === 'lrLR', 'actual object hook mutation and both evaluations before conversion');

trace = '';
function* multiply() { return (yield 'left') * (yield 'right'); }
var multiplying = multiply();
assert(multiplying.next().value === 'left', 'first operand');
assert(multiplying.next(left).value === 'right' && trace === '', 'whole left operand stays uncoerced');
gc();
assert(multiplying.next(right).value === 20 && trace === 'LR', 'both retained whole values survive GC');
var marker = {}; marker.self = marker;
left.valueOf = function () { throw marker; };
var abrupt = multiply(); abrupt.next(); abrupt.next(left);
var caught;
try { abrupt.next(right); } catch (error) { caught = error; }
assert(caught === marker && caught.self === marker, 'original coercion Throw identity');
trace = '';
var returned = multiply(); returned.next(); returned.next(left);
assert(returned.return(marker).value === marker && trace === '', 'injected Return skips pending operator');
var thrown = multiply(); thrown.next(); thrown.next(left); caught = null;
try { thrown.throw(marker); } catch (error) { caught = error; }
assert(caught === marker && trace === '', 'injected Throw skips pending operator');

function* shift() { return (yield 'a') << (yield 'b'); }
var shifting = shift(); shifting.next(); shifting.next(6n);
assert(shifting.next(1n).value === 12n, 'BigInt bitwise operation');
function* power() { return (yield 'a') ** (yield 'b'); }
var powering = power(); powering.next(); powering.next(3n);
assert(powering.next(4n).value === 81n, 'BigInt exponentiation');
function* unsigned() { return (yield 'a') >>> (yield 'b'); }
var unsignedIterator = unsigned(); unsignedIterator.next();
assert(unsignedIterator.next(1n).value === 'b', 'BigInt refusal occurs after RHS evaluation');
caught = null; try { unsignedIterator.next(1n); } catch (error) { caught = error; }
assert(caught instanceof TypeError, 'BigInt unsigned shift keeps native error');

trace = '';
var compareLeft = { valueOf() { trace += 'L'; return 2; } };
var compareRight = { valueOf() { trace += 'R'; return 3; } };
function* greater() { return (yield 'a') > (yield 'b'); }
var comparing = greater(); comparing.next(); comparing.next(compareLeft);
assert(comparing.next(compareRight).value === false && trace === 'LR', 'relational reversal preserves original conversion order');
function* equality() { return (yield 'a') === (yield 'b'); }
var equal = equality(); equal.next(); equal.next(marker);
assert(equal.next(marker).value === true, 'strict equality compares retained whole objects without coercion');
function* membership() { return (yield 'key') in (yield 'object'); }
var member = membership(); member.next(); member.next('self');
assert(member.next(marker).value === true, 'in consumes the real selected object');
function* instance() { return (yield 'value') instanceof (yield 'constructor'); }
var instanceIterator = instance(); instanceIterator.next(); instanceIterator.next(marker);
assert(instanceIterator.next(Object).value === true, 'instanceof retains constructor identity');
function* complement() { return ~(yield 'operand'); }
var complemented = complement(); complemented.next();
assert(complemented.next(2n).value === -3n, 'unary BigInt operation');
function* negative() { return -(yield 'operand'); }
var negated = negative(); negated.next();
assert(negated.next(2n).value === -2n, 'unary numeric domain');
function* types() { return typeof (yield 'operand'); }
var typed = types(); typed.next();
assert(typed.next(Symbol('s')).value === 'symbol', 'typeof receives a whole Symbol');

var gets = 0, deleted = { get value() { gets++; throw marker; } };
var key = { toString() { trace += 'K'; return 'value'; } };
function* remove() { return delete (yield 'base')[yield 'key']; }
trace = '';
var removing = remove(); removing.next(); removing.next(deleted); gc();
assert(removing.next(key).value === true && gets === 0 && trace === 'K' && !('value' in deleted), 'delete retains a Reference without invoking Get');
var protectedObject = {};
Object.defineProperty(protectedObject, 'value', { value: 1, configurable: false });
var refusing = remove(); refusing.next(); refusing.next(protectedObject);
// The fixture inherits either Script strictness; DeleteProperty keeps it.
var strict = (function () { return this === undefined; })();
caught = null; var refused;
try { refused = refusing.next('value'); } catch (error) { caught = error; }
assert(strict ? caught instanceof TypeError : refused.value === false, 'delete preserves source strictness');

trace = '';
var stringPart = { [Symbol.toPrimitive](hint) { trace += hint; return 'X'; } };
function* template() { return `A${yield 'first'}B${yield 'second'}C`; }
var templating = template(); assert(templating.next().value === 'first', 'first template substitution');
assert(templating.next(stringPart).value === 'second' && trace === 'string', 'ToString finishes before later substitution Yield');
gc(); assert(templating.next('Y').value === 'AXBYC', 'template accumulator survives suspension');
var symbolTemplate = template(); symbolTemplate.next(); caught = null;
try { symbolTemplate.next(Symbol('bad')); } catch (error) { caught = error; }
assert(caught instanceof TypeError && symbolTemplate.next().done, 'Symbol ToString throws before later template Yield');
function* templateBinary() { return `n=${(yield 'a') + (yield 'b')}`; }
var composed = templateBinary(); composed.next(); composed.next(2);
assert(composed.next(3).value === 'n=5', 'template and eager operator compose their checked plans');
print('generator-eager-values:ok');
