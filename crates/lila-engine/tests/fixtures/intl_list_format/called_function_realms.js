function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label); }
function throws(C, body, label) { var caught = undefined; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
var foreign = __lilaCreateRealm().global;
var Other = foreign.Intl.ListFormat;
var foreignTypeError = foreign.TypeError;
var foreignRangeError = foreign.RangeError;
var local = new Intl.ListFormat('en-US');
var remote = new Other('en-US');
same(Object.getPrototypeOf(remote), Other.prototype, 'foreign family prototype');
same(Object.getPrototypeOf(Other), foreign.Function.prototype, 'foreign constructor');
var options = Other.prototype.resolvedOptions.call(local);
same(Object.getPrototypeOf(options), foreign.Object.prototype, 'resolved called-function Realm');
var parts = Other.prototype.formatToParts.call(local, ['A', 'B']);
same(Object.getPrototypeOf(parts), foreign.Array.prototype, 'parts called-function Realm');
for (var part of parts) same(Object.getPrototypeOf(part), foreign.Object.prototype, 'part called-function Realm');
same(Object.getPrototypeOf(Intl.ListFormat.prototype.formatToParts.call(remote, ['A'])), Array.prototype, 'borrowed local method owns array');
same(Object.getPrototypeOf(Other.supportedLocalesOf.call({}, ['en-US'])), foreign.Array.prototype, 'static called-function Realm');
var boxed = 0;
Object.defineProperty(foreign.Number.prototype, 'localeMatcher', {configurable: true, get() { boxed++; same(Object.getPrototypeOf(this), foreign.Number.prototype, 'foreign primitive boxing'); return 'lookup'; }});
same(Other.supportedLocalesOf('en-US', 7)[0], 'en-US', 'foreign boxed static options'); same(boxed, 1, 'one boxing Get');
foreign.TypeError = function PublicReplacement() { throw new Error('public constructor observed'); };
foreign.RangeError = function PublicReplacement() { throw new Error('public constructor observed'); };
foreign.Intl.ListFormat = function PublicReplacement() { throw new Error('public ListFormat observed'); };
throws(foreignTypeError, function() { Other('en-US'); }, 'foreign plain-call primordial TypeError');
throws(foreignTypeError, function() { new Other('en-US', 1); }, 'foreign strict options error');
throws(foreignRangeError, function() { new Other('en-US', {style: 'wrong'}); }, 'foreign option RangeError');
var gets = 0;
throws(foreignTypeError, function() { Other.prototype.format.call({}, new Proxy({}, {get() { gets++; throw new Error('brand'); }})); }, 'foreign brand error');
same(gets, 0, 'foreign brand before reads');
var closes = 0;
var iterator = {next() { var nested = Intl.ListFormat.prototype.resolvedOptions.call(remote); same(Object.getPrototypeOf(nested), Object.prototype, 'nested local call'); return {done: false, value: 1}; }, return() { closes++; return {}; }};
throws(foreignTypeError, function() { Other.prototype.format.call(local, {[Symbol.iterator]() { return iterator; }}); }, 'called-method error survives nested Realm switch');
same(closes, 1, 'foreign nonstring error closes');
throws(TypeError, function() { Intl.ListFormat.prototype.format.call(remote, [1]); }, 'local borrowed-method error');
print('ok called_function_realms');
262;
