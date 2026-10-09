var $262 = { createRealm: __lilaCreateRealm };
var foreign = $262.createRealm().global;
var LocalArray = Array;
var ForeignArray = foreign.Array;
var LocalObject = Object;
var SavedProxy = Proxy;
var localSplice = LocalArray.prototype.splice;
var foreignSplice = ForeignArray.prototype.splice;
var localArrayPrototype = LocalArray.prototype;
var foreignArrayPrototype = ForeignArray.prototype;
var localTypeErrorPrototype = TypeError.prototype;
var foreignTypeErrorPrototype = foreign.TypeError.prototype;
var foreignMarker = new foreign.Error('foreign splice marker');
function check(condition, name) { if (!condition) throw name; }
function array(ctor) {
  var result = new ctor();
  result[0] = 'a'; result[1] = 'b'; result[2] = 'c';
  return result;
}
function unreachable() { throw 'public constructor must not be consulted'; }
Array = unreachable;
foreign.Array = unreachable;
TypeError = unreachable;
foreign.TypeError = unreachable;

function realmDirection(ctor, method, resultPrototype, errorPrototype) {
  var source = array(ctor);
  source.splice = method;
  var coercions = [];
  var start = { valueOf: function () { coercions.push('start'); return -2; } };
  var count = { valueOf: function () { coercions.push('count'); return 2; } };
  var removed = source.splice(start, count, ...Object.keys({ inserted: true }));
  check(LocalObject.getPrototypeOf(removed) === resultPrototype, 'called splice result Realm');
  check(removed.length === 2 && removed[0] === 'b' && removed[1] === 'c', 'general deletion');
  check(source.length === 2 && source[0] === 'a' && source[1] === 'inserted', 'negative start');
  check(coercions.join(',') === 'start,count', 'coercion order');

  var target = { splice: method };
  LocalObject.defineProperty(target, 'length', { value: 0, writable: false });
  var caught;
  var prior = 17;
  var finalized = 0;
  try {
    prior = 18;
    target.splice(0, 0, ...Object.keys({}));
  } catch (error) { caught = error; }
  finally { finalized++; }
  check(caught !== undefined && LocalObject.getPrototypeOf(caught) === errorPrototype,
    'called splice native TypeError Realm');
  check(target.length === 0 && prior === 18 && finalized === 1, 'native error prior effects');
}
realmDirection(ForeignArray, localSplice, localArrayPrototype, localTypeErrorPrototype);
realmDirection(LocalArray, foreignSplice, foreignArrayPrototype, foreignTypeErrorPrototype);

function spreadFailure(ctor, method, stage, expected) {
  var source = array(ctor);
  source.splice = method;
  var trace = [];
  var speciesReads = 0;
  var closes = 0;
  LocalObject.defineProperty(source, 'constructor', { get: function () {
    speciesReads++;
    throw 'spread must finish before species';
  } });
  var iterator = {};
  LocalObject.defineProperty(iterator, 'return', { get: function () {
    closes++;
    throw 'argument spread must not close';
  } });
  var step = {};
  LocalObject.defineProperty(step, 'done', { get: function () {
    trace.push('done');
    if (stage === 'done') throw foreignMarker;
    return false;
  } });
  LocalObject.defineProperty(step, 'value', { get: function () {
    trace.push('value');
    throw foreignMarker;
  } });
  var next = new SavedProxy(function () { return step; }, { apply: function (target, receiver, args) {
    check(receiver === iterator && args.length === 0, 'next receiver and argc');
    trace.push('next');
    if (stage === 'next') throw foreignMarker;
    return step;
  } });
  LocalObject.defineProperty(iterator, 'next', { get: function () {
    trace.push('get-next');
    if (stage === 'get-next') throw foreignMarker;
    return next;
  } });
  var iterable = {};
  var open = new SavedProxy(function () { return iterator; }, { apply: function (target, receiver, args) {
    check(receiver === iterable && args.length === 0, 'iterator receiver and argc');
    trace.push('open');
    if (stage === 'open') throw foreignMarker;
    return iterator;
  } });
  LocalObject.defineProperty(iterable, Symbol.iterator, { get: function () {
    trace.push('get-iterator');
    if (stage === 'get-iterator') throw foreignMarker;
    return open;
  } });
  var Object = { keys: function () { trace.push('keys'); return iterable; } };
  var caught;
  var prior = 17;
  var finalized = 0;
  try {
    prior = 18;
    source.splice(0, 0, ...Object.keys({}));
  } catch (error) { caught = error; }
  finally { finalized++; trace.push('finally'); }
  check(caught === foreignMarker, 'original foreign spread abrupt');
  check(trace.join(',') === expected + ',finally', 'spread abrupt cutoff ' + stage);
  check(speciesReads === 0 && closes === 0, 'no species or close after spread abrupt');
  check(source.length === 3 && source[0] === 'a' && source[1] === 'b' && source[2] === 'c',
    'spread abrupt precedes source mutation');
  check(prior === 18 && finalized === 1, 'spread prior assignment and finally');
}
var stages = ['get-iterator', 'open', 'get-next', 'next', 'done', 'value'];
var cutoffs = [
  'keys,get-iterator',
  'keys,get-iterator,open',
  'keys,get-iterator,open,get-next',
  'keys,get-iterator,open,get-next,next',
  'keys,get-iterator,open,get-next,next,done',
  'keys,get-iterator,open,get-next,next,done,value'
];
for (var s = 0; s < stages.length; s++) {
  spreadFailure(ForeignArray, localSplice, stages[s], cutoffs[s]);
  spreadFailure(LocalArray, foreignSplice, stages[s], cutoffs[s]);
}

function primitiveStepFailure(ctor, method) {
  var source = array(ctor);
  source.splice = method;
  var closes = 0;
  var speciesReads = 0;
  LocalObject.defineProperty(source, 'constructor', { get: function () { speciesReads++; return {}; } });
  var iterator = { next: function () { return 1; } };
  LocalObject.defineProperty(iterator, 'return', { get: function () { closes++; return function () {}; } });
  var iterable = { [Symbol.iterator]: function () { return iterator; } };
  var Object = { keys: function () { return iterable; } };
  var caught;
  try { source.splice(0, 0, ...Object.keys({})); } catch (error) { caught = error; }
  check(caught !== undefined && LocalObject.getPrototypeOf(caught) === localTypeErrorPrototype,
    'argument iteration native errors belong to caller Realm');
  check(speciesReads === 0 && closes === 0 && source.length === 3, 'primitive step cutoff');
}
primitiveStepFailure(ForeignArray, localSplice);
primitiveStepFailure(LocalArray, foreignSplice);

function spliceFailure(ctor, method, stage, expected) {
  var source = array(ctor);
  source.splice = method;
  var trace = [];
  var species = {};
  LocalObject.defineProperty(species, Symbol.species, { get: function () {
    trace.push('species');
    if (stage === 'species') throw foreignMarker;
    return function Result(length) {
      trace.push('construct:' + length);
      if (stage === 'construct') throw foreignMarker;
      return {};
    };
  } });
  LocalObject.defineProperty(source, 'constructor', { get: function () {
    trace.push('constructor');
    return species;
  } });
  if (stage === 'deleted-get') {
    LocalObject.defineProperty(source, '0', { configurable: true, get: function () {
      trace.push('get:0');
      throw foreignMarker;
    } });
  }
  var caught;
  var prior = 17;
  var finalized = 0;
  try {
    prior = 18;
    source.splice(0, 1, ...Object.keys({}));
  } catch (error) { caught = error; }
  finally { finalized++; trace.push('finally'); }
  check(caught === foreignMarker, 'original foreign splice abrupt ' + stage);
  check(trace.join(',') === expected + ',finally', 'splice abrupt cutoff ' + stage);
  check(source.length === 3 && source[1] === 'b' && source[2] === 'c', 'no mutation before abrupt');
  if (stage !== 'deleted-get') check(source[0] === 'a', 'first element retained');
  check(prior === 18 && finalized === 1, 'splice prior assignment and finally');
}
for (var r = 0; r < 2; r++) {
  var ctor = r === 0 ? ForeignArray : LocalArray;
  var method = r === 0 ? localSplice : foreignSplice;
  spliceFailure(ctor, method, 'species', 'constructor,species');
  spliceFailure(ctor, method, 'construct', 'constructor,species,construct:1');
  spliceFailure(ctor, method, 'deleted-get', 'constructor,species,construct:1,get:0');
}

function shiftSetFailure(ctor, method) {
  var source = array(ctor);
  source.splice = method;
  var trace = [];
  LocalObject.defineProperty(source, '1', { configurable: true,
    get: function () { trace.push('get:1'); return 'b'; },
    set: function () { trace.push('set:1'); throw foreignMarker; }
  });
  var caught;
  var prior = 17;
  var finalized = 0;
  try {
    prior = 18;
    source.splice(0, 0, ...Object.keys({ inserted: true }));
  } catch (error) { caught = error; }
  finally { finalized++; trace.push('finally'); }
  check(caught === foreignMarker && trace.join(',') === 'get:1,set:1,finally',
    'original foreign Set abrupt after descending shift');
  check(source.length === 4 && source[3] === 'c' && source[2] === 'b' && source[0] === 'a',
    'prior descending shift effects retained');
  check(prior === 18 && finalized === 1, 'Set prior assignment and finally');
}
shiftSetFailure(ForeignArray, localSplice);
shiftSetFailure(LocalArray, foreignSplice);
print('array-splice-spread-abrupt:ok');
262;
