function check(condition, label) { if (!condition) throw new Error(label); }
function step(iterator, sent, value, done, label) {
    var result = iterator.next(sent);
    check(result.value === value && result.done === done, label);
}
var strictMode = (function () { return this === undefined; })();
var trace = '';
var whole = { label: 'whole' }; whole.self = whole;
var leaf = {};
var original = function (argument) {
    trace += 'C';
    check(this === leaf, 'original method receiver');
    return argument;
};
Object.defineProperty(leaf, 'make', {
    configurable: true,
    get: function () { trace += 'M'; gc(); return original; }
});
var root = {};
Object.defineProperty(root, 'leaf', {
    get: function () { trace += 'G'; gc(); return leaf; }
});
var key = { [Symbol.toPrimitive]: function () { trace += 'K'; return 'leaf'; } };
function* selected() {
    return (yield 'b1', yield 'b2')?.[yield 'k1', yield 'k2']?.make(yield 'a1', yield 'a2');
}
var iterator = selected();
step(iterator, undefined, 'b1', false, 'first base');
step(iterator, 0, 'b2', false, 'second base');
step(iterator, root, 'k1', false, 'first key');
step(iterator, 0, 'k2', false, 'second key');
step(iterator, key, 'a1', false, 'first argument');
check(trace === 'KGM', 'key conversion and getter precede arguments');
Object.defineProperty(leaf, 'make', { value: function () { throw new Error('callee replay'); } });
gc();
step(iterator, 0, 'a2', false, 'second argument');
step(iterator, whole, whole, true, 'retained callee receiver and whole argument');
check(trace === 'KGMC', 'no duplicate key or Get');
iterator = selected();
step(iterator, undefined, 'b1', false, 'skipped first base still runs');
step(iterator, 0, 'b2', false, 'skipped second base still runs');
step(iterator, null, undefined, true, 'nullish skips keys arguments and their yields');

var locked = {};
Object.defineProperty(locked, 'locked', {
    configurable: false,
    get: function () { throw new Error('terminal getter must never run'); }
});
function* remove() {
    try { return delete ((yield 'target')?.[yield 'key']); }
    catch (error) { check(error instanceof TypeError, 'failed strict delete'); return 'strict'; }
}
iterator = remove();
step(iterator, undefined, 'target', false, 'delete target');
step(iterator, locked, 'key', false, 'delete key');
gc();
step(iterator, 'locked', strictMode ? 'strict' : false, true, 'actual nonconfigurable deletion');
iterator = remove();
step(iterator, undefined, 'target', false, 'nullish delete target');
step(iterator, undefined, true, true, 'nullish delete skips key yield');
var ordinary = false;
try { ordinary = delete locked?.locked; }
catch (error) { check(strictMode && error instanceof TypeError, 'ordinary strict delete'); ordinary = 'strict'; }
check(ordinary === (strictMode ? 'strict' : false), 'ordinary actual terminal deletion');
var configurable = {};
Object.defineProperty(configurable, 'present', {
    configurable: true,
    get: function () { throw new Error('configurable terminal getter must not run'); }
});
check(delete configurable?.present && !Object.prototype.hasOwnProperty.call(configurable, 'present'), 'ordinary configurable Delete');

var symbol = Symbol('delete-key');
var proxyTarget = {}; proxyTarget[symbol] = whole;
var deleteTrace = '';
var proxy = new Proxy(proxyTarget, {
    get: function () { throw new Error('terminal proxy Get must not run'); },
    deleteProperty: function (target, selectedKey) {
        deleteTrace += 'D'; gc();
        check(target === proxyTarget && selectedKey === symbol, 'original native delete Reference');
        return Reflect.deleteProperty(target, selectedKey);
    }
});
var symbolKey = { [Symbol.toPrimitive]: function () { deleteTrace += 'K'; return symbol; } };
iterator = remove();
step(iterator, undefined, 'target', false, 'proxy target');
step(iterator, proxy, 'key', false, 'proxy key');
step(iterator, symbolKey, true, true, 'proxy actual Delete');
check(deleteTrace === 'KD' && !Object.prototype.hasOwnProperty.call(proxyTarget, symbol), 'symbol conversion once before Delete');
var thrown = { label: 'delete throw' }; thrown.self = thrown;
var throwing = new Proxy({}, { deleteProperty: function () { throw thrown; } });
function* throwingDelete() { return delete (yield 'throw-target')?.p; }
iterator = throwingDelete();
step(iterator, undefined, 'throw-target', false, 'throwing proxy target');
var caught;
try { iterator.next(throwing); } catch (error) { caught = error; }
check(caught === thrown, 'whole Proxy Delete Throw');

var intermediateGets = 0;
var parent = {};
Object.defineProperty(parent, 'child', { get: function () { intermediateGets++; return undefined; } });
var forbiddenConversion = { [Symbol.toPrimitive]: function () { throw new Error('nullish nonshorted receiver precedes key conversion'); } };
function* missing() { return delete parent?.child[yield 'missing-key']; }
iterator = missing();
step(iterator, undefined, 'missing-key', false, 'intermediate actual Get');
caught = undefined;
try { iterator.next(forbiddenConversion); } catch (error) { caught = error; }
check(caught instanceof TypeError && intermediateGets === 1, 'nonshorted undefined is not a skipped chain');

var called = 0;
var callObject = { method: function (argument) { called++; check(this === callObject && argument === whole, 'call terminal receiver'); return locked; } };
function* deleteCall() { return delete (yield 'call-target')?.method(yield 'call-argument'); }
iterator = deleteCall();
step(iterator, undefined, 'call-target', false, 'call Value target');
step(iterator, callObject, 'call-argument', false, 'call Value argument');
step(iterator, whole, true, true, 'completed Call Value Delete');
check(called === 1, 'terminal Call executes once');

function* delegate() { yield 'delegate'; return whole; }
var delegateObject = { receive: function (argument) { check(this === delegateObject, 'delegated receiver'); return argument; } };
function* delegated() { return (yield 'delegate-target')?.receive(yield* delegate()); }
iterator = delegated();
step(iterator, undefined, 'delegate-target', false, 'delegated base');
step(iterator, delegateObject, 'delegate', false, 'selected delegated operand');
gc();
step(iterator, 0, whole, true, 'delegated completion whole Value');

function* interrupted() {
    try { return delete (yield 'interrupt-target')?.[yield 'interrupt-key']; }
    finally { yield 'finally'; }
}
iterator = interrupted();
step(iterator, undefined, 'interrupt-target', false, 'pending Return base');
step(iterator, locked, 'interrupt-key', false, 'pending Return key');
var result = iterator.return(whole);
check(result.value === 'finally' && !result.done, 'injected Return enters yielding finally');
gc();
step(iterator, 0, whole, true, 'pending whole Return preserved');
iterator = interrupted();
step(iterator, undefined, 'interrupt-target', false, 'pending Throw base');
step(iterator, locked, 'interrupt-key', false, 'pending Throw key');
result = iterator.throw(thrown);
check(result.value === 'finally' && !result.done, 'injected Throw enters yielding finally');
gc();
caught = undefined;
try { iterator.next(); } catch (error) { caught = error; }
check(caught === thrown, 'pending whole Throw preserved');
print('generator-optional-regions:ok');
