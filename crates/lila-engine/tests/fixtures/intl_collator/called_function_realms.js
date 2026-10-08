function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
var foreign = __lilaCreateRealm().global; var Other = foreign.Intl.Collator; var otherType = foreign.TypeError; var otherRange = foreign.RangeError;
var local = new Intl.Collator('en-US'); var remote = new Other('en-US');
same(Object.getPrototypeOf(remote), Other.prototype, 'foreign constructor prototype');
var getter = Object.getOwnPropertyDescriptor(Other.prototype, 'compare').get;
var cmp = getter.call(local); same(Object.getPrototypeOf(cmp), foreign.Function.prototype, 'called getter owns cached Function');
same(local.compare, cmp, 'later local getter shares cache');
same(Object.getPrototypeOf(Other.prototype.resolvedOptions.call(local)), foreign.Object.prototype, 'called method resolved object');
same(Object.getPrototypeOf(Other.supportedLocalesOf('en-US')), foreign.Array.prototype, 'called static array');
foreign.TypeError = function() { throw new Error('public TypeError observed'); }; foreign.RangeError = function() { throw new Error('public RangeError observed'); }; foreign.Intl.Collator = function() { throw new Error('public Collator observed'); };
throws(otherType, function() { getter.call({}); }, 'foreign brand intrinsic');
throws(otherRange, function() { new Other('en-US', {usage: 'invalid'}); }, 'foreign option intrinsic');
throws(otherType, function() { cmp(Symbol('x'), 'y'); }, 'cached compare defining Realm');
var x = {toString() { Intl.Collator.prototype.resolvedOptions.call(remote); return 'a'; }};
throws(otherType, function() { cmp(x, Symbol('y')); }, 'cached Realm after nested local call');
throws(otherType, function() { remote.compare(Symbol('x'), 'y'); }, 'foreign-created bound compare');
var freshRemote = new Other('en-US'); var localGetter = Object.getOwnPropertyDescriptor(Intl.Collator.prototype, 'compare').get; var localCmp = localGetter.call(freshRemote);
same(Object.getPrototypeOf(localCmp), Function.prototype, 'local borrowed getter owns first cache'); same(getter.call(freshRemote), localCmp, 'foreign getter reuses local cache');
throws(TypeError, function() { localCmp(Symbol('x'), 'y'); }, 'local cached function defining Realm');
print('ok called_function_realms');
262;
