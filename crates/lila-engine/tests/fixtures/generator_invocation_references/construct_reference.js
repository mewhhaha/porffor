function Original(first, second) { print('original:' + (first + second) + ':' + (new.target === Original)); this.value = first + second; }
function Replacement() { print('unexpected-constructor'); }
var holder = { get constructor() { print('get'); return Original; } };
function before() { print('before'); return 40; }
function* values() { return new holder[yield 'constructor?'](before(), yield 'argument?'); }
var iterator = values();
print(iterator.next().value);
print(iterator.next('constructor').value);
holder = { constructor: Replacement };
var prototype = {};
Original.prototype = prototype;
var result = iterator.next(2);
print(result.value.value + ':' + result.done);
print(Object.getPrototypeOf(result.value) === prototype);
function after() { print('after'); return 3; }
function* nonconstructor() { return new (1)(yield 'bad?', after()); }
iterator = nonconstructor();
print(iterator.next().value);
try { iterator.next(2); } catch (error) { print(error instanceof TypeError); }
