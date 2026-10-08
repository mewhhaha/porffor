function check(value, message) { if (!value) throw message; }
var other = __lilaCreateRealm().global;
var ForeignPromise = other.Promise;
var ForeignTypeError = other.TypeError;
var ForeignReferenceError = other.ReferenceError;
var ForeignSuppressedError = other.SuppressedError;
var foreignMethods = other.AsyncDisposableStack.prototype;
var localMethods = AsyncDisposableStack.prototype;
var pending = [];
other.Promise = function () { throw "mutable public Promise used"; };
other.TypeError = function () { throw "mutable public TypeError used"; };
other.ReferenceError = function () { throw "mutable public ReferenceError used"; };
other.SuppressedError = function () { throw "mutable public SuppressedError used"; };
var invalid = foreignMethods.disposeAsync.call({});
check(invalid instanceof ForeignPromise, "brand rejection capability defining Realm");
pending.push(invalid.then(function () { throw "brand fulfilled"; }, function (error) {
  check(error instanceof ForeignTypeError && !(error instanceof TypeError), "brand error defining Realm");
}));
var forward = new AsyncDisposableStack();
var reverse = new other.AsyncDisposableStack();
var retainedSymbol = Symbol("retained");
var retainedFunction = function () {};
foreignMethods.adopt.call(forward, retainedSymbol, function (value) {
  check(value === retainedSymbol, "foreign method retains Symbol across Await");
});
foreignMethods.adopt.call(forward, retainedFunction, function (value) {
  check(value === retainedFunction, "foreign method retains Function across Await");
  return { then: function (resolve) {
    check(Object.getPrototypeOf(resolve) === other.Function.prototype, "Await resolving function defining Realm");
    resolve();
  } };
});
var forwardResult = foreignMethods.disposeAsync.call(forward);
check(forwardResult instanceof ForeignPromise && !(forwardResult instanceof Promise), "borrowed foreign disposal Promise");
pending.push(forwardResult);
var reverseResult = localMethods.disposeAsync.call(reverse);
check(reverseResult instanceof Promise && !(reverseResult instanceof ForeignPromise), "borrowed local disposal Promise");
pending.push(reverseResult);
try { foreignMethods.use.call(forward, null); throw "disposed use accepted"; }
catch (error) { check(error instanceof ForeignReferenceError, "disposed synchronous error Realm"); }
try { foreignMethods.defer.call(new AsyncDisposableStack(), 1); throw "noncallable defer accepted"; }
catch (error) { check(error instanceof ForeignTypeError, "callability error defining Realm"); }
var forged = { "$AsyncDisposableState": 0, "$DisposableResourceStack": [] };
var proxyGets = 0;
var proxied = new Proxy(new AsyncDisposableStack(), { get: function () { proxyGets++; throw "proxy brand get"; } });
var disposedGetter = Object.getOwnPropertyDescriptor(foreignMethods, "disposed").get;
for (var receiver of [forged, proxied, foreignMethods]) {
  try { disposedGetter.call(receiver); throw "brand forged"; }
  catch (error) { check(error instanceof ForeignTypeError, "GC brand ignores mutable properties and Proxy target"); }
}
check(proxyGets === 0, "brand checks invoke no Proxy get");
var old = new AsyncDisposableStack();
var moved = foreignMethods.move.call(old);
check(Object.getPrototypeOf(moved) === foreignMethods && old.disposed && !moved.disposed, "Move uses called Realm intrinsic prototype");
pending.push(moved.disposeAsync());
var first = {};
var second = Symbol("foreign suppression");
var suppress = new AsyncDisposableStack();
suppress.defer(function () { return Promise.reject(first); });
suppress.defer(function () { throw second; });
pending.push(foreignMethods.disposeAsync.call(suppress).then(function () { throw "suppression fulfilled"; }, function (error) {
  check(error instanceof ForeignSuppressedError && !(error instanceof SuppressedError), "saved disposal Realm after Await");
  check(error.error === first && error.suppressed === second, "suppression stores whole references");
}));
var fallbackReads = 0;
var source = new AsyncDisposableStack();
var marker = {};
var order = [];
try {
  foreignMethods.use.call(source, {
    get [Symbol.asyncDispose]() { order.push("async"); throw marker; },
    get [Symbol.dispose]() { fallbackReads++; }
  });
} catch (error) { check(error === marker, "original getter Throw"); order.push("catch"); }
finally { order.push("finally"); }
check(order.join(",") === "async,catch,finally" && fallbackReads === 0, "Get abrupt chronology");
Promise.all(pending).then(function () { print("async-stack-realms:ok"); });
262;
