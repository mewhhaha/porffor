function check(value, message) { if (!value) throw message; }
var foreign = __lilaCreateRealm().global;
var ForeignPromise = foreign.Promise;
var ForeignTypeError = foreign.TypeError;
var localFunctionPrototype = Function.prototype;
var foreignFunctionPrototype = foreign.Function.prototype;
var pending = [];

function checkResolvingFunctions(resolve, reject, prototype) {
  check(Object.getPrototypeOf(resolve) === prototype, "resolve function follows Job Realm");
  check(Object.getPrototypeOf(reject) === prototype, "reject function follows Job Realm");
  check(resolve !== reject && resolve.length === 1 && reject.length === 1, "resolving function metadata");
}

pending.push(ForeignPromise.resolve({ then: function (resolve, reject) {
  checkResolvingFunctions(resolve, reject, localFunctionPrototype);
  resolve(1);
  reject("already resolved");
} }).then(function (value) { check(value === 1, "shared already-resolved state"); }));

// GetFunctionRealm unwraps the Proxy's foreign target before its local apply
// trap executes. The trap's own Realm does not select the resolving functions.
var foreignThen = new Proxy(foreign.Object, { apply: function (target, receiver, args) {
  check(Object.getPrototypeOf(args) === foreign.Array.prototype, "Proxy argument Array follows Job Realm");
  checkResolvingFunctions(args[0], args[1], foreignFunctionPrototype);
  args[0](2);
} });
pending.push(Promise.resolve({ then: foreignThen }).then(function (value) {
  check(value === 2, "foreign then callback with local Promise");
}));

var localThen = function (resolve, reject) {
  checkResolvingFunctions(resolve, reject, localFunctionPrototype);
  resolve(3);
};
var boundLocalThen = foreign.Function.prototype.bind.call(localThen, null);
pending.push(ForeignPromise.resolve({ then: boundLocalThen }).then(function (value) {
  check(value === 3, "bound target Realm rather than bind method Realm");
}));

var cyclic;
cyclic = ForeignPromise.resolve({ then: function (resolve, reject) {
  checkResolvingFunctions(resolve, reject, localFunctionPrototype);
  resolve(cyclic);
} });
pending.push(cyclic.then(function () { throw "self-resolution fulfilled"; }, function (error) {
  check(error instanceof TypeError && !(error instanceof ForeignTypeError), "self-resolution error uses resolving function Realm");
}));

var revoked = Proxy.revocable(foreign.Object, {});
revoked.revoke();
pending.push(ForeignPromise.resolve({ then: revoked.proxy }).then(function () {
  throw "revoked then callback fulfilled";
}, function (error) {
  check(error instanceof ForeignTypeError && !(error instanceof TypeError), "revoked then uses enqueueing Realm fallback");
}));

var revokedHandler = Proxy.revocable(foreign.Object, {});
revokedHandler.revoke();
pending.push(ForeignPromise.resolve(4).then(revokedHandler.proxy).then(function () {
  throw "revoked reaction handler fulfilled";
}, function (error) {
  check(error instanceof ForeignTypeError && !(error instanceof TypeError), "revoked reaction uses enqueueing Realm fallback");
}));

pending.push(new ForeignPromise(function (resolve, reject) {
  checkResolvingFunctions(resolve, reject, foreignFunctionPrototype);
  resolve({ then: localThen });
}).then(function (value) { check(value === 3, "constructor Realm and thenable Job Realm differ"); }));

var thrown = Symbol("then callback abrupt identity");
pending.push(ForeignPromise.resolve({ then: function () { throw thrown; } }).then(function () {
  throw "throwing then callback fulfilled";
}, function (error) {
  check(error === thrown, "then callback Throw rejects the Promise without escaping the Job");
}));

Promise.all(pending).then(function () { print("thenable-job-realms:ok"); });
262;
