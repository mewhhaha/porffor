// Temporal.ZonedDateTime.prototype.{toJSON,valueOf,toLocaleString,toPlainTime}.
//
// `toJSON` shares `toString`'s emitter but must not share its function object
// and must never read its argument; `valueOf` is an unconditional TypeError;
// `toLocaleString` delegates to `Intl.DateTimeFormat.prototype.format`, which
// already dispatches the zoned brand; `toPlainTime` projects the six wall-clock
// slots out of the shared `toPlain` field layout.
//
// Rendered values are printed as well as compared, so the CLI test holds the
// literal strings and a wrong rendering cannot match itself.

function expectError(errorConstructor, callback) {
  var thrown = false;
  try {
    callback();
  } catch (error) {
    if (!(error instanceof errorConstructor)) throw error;
    thrown = true;
  }
  if (!thrown) throw errorConstructor.name;
}

var proto = Temporal.ZonedDateTime.prototype;
var toJSON = proto.toJSON;
var valueOf = proto.valueOf;
var toLocaleString = proto.toLocaleString;
var toPlainTime = proto.toPlainTime;

if (typeof toJSON !== "function") throw "toJSON missing";
if (typeof valueOf !== "function") throw "valueOf missing";
if (typeof toLocaleString !== "function") throw "toLocaleString missing";
if (typeof toPlainTime !== "function") throw "toPlainTime missing";

if (toJSON.name !== "toJSON") throw "toJSON name";
if (toJSON.length !== 0) throw "toJSON length";
if (valueOf.name !== "valueOf") throw "valueOf name";
if (valueOf.length !== 0) throw "valueOf length";
if (toLocaleString.name !== "toLocaleString") throw "toLocaleString name";
if (toLocaleString.length !== 0) throw "toLocaleString length";
if (toPlainTime.name !== "toPlainTime") throw "toPlainTime name";
if (toPlainTime.length !== 0) throw "toPlainTime length";

if (toJSON === proto.toString) throw "toJSON identity";

var epoch = new Temporal.ZonedDateTime(0n, "UTC");
var frac = new Temporal.ZonedDateTime(30_123_400_000n, "UTC");

var epochJson = epoch.toJSON();
var fracJson = frac.toJSON();
if (epochJson !== "1970-01-01T00:00:00+00:00[UTC]") throw "epoch toJSON";
if (fracJson !== "1970-01-01T00:00:30.1234+00:00[UTC]") throw "frac toJSON";
if (epochJson !== epoch.toString()) throw "toJSON/toString agreement";

// The argument is never observed, not even a throwing Proxy.
var bomb = new Proxy({}, { get: function () { throw "toJSON read its argument"; } });
if (epoch.toJSON(bomb) !== epochJson) throw "toJSON argument";

// `valueOf` throws before any brand check, on every receiver.
expectError(TypeError, function () { epoch.valueOf(); });
expectError(TypeError, function () { valueOf.call(undefined); });
expectError(TypeError, function () { valueOf.call({}); });

if (typeof epoch.toLocaleString("en", { dateStyle: "short" }) !== "string") {
  throw "toLocaleString dateStyle";
}
if (typeof epoch.toLocaleString("en", { timeStyle: "short" }) !== "string") {
  throw "toLocaleString timeStyle";
}
expectError(TypeError, function () { toLocaleString.call({}); });

function renderTime(t) {
  return t.hour + ":" + t.minute + ":" + t.second + "." +
    t.millisecond + t.microsecond + t.nanosecond;
}
var epochTime = epoch.toPlainTime();
var fracTime = frac.toPlainTime();
if (!(epochTime instanceof Temporal.PlainTime)) throw "toPlainTime brand";
if (Object.getPrototypeOf(epochTime) !== Temporal.PlainTime.prototype) {
  throw "toPlainTime prototype";
}
if (renderTime(epochTime) !== "0:0:0.000") throw "epoch toPlainTime";
if (renderTime(fracTime) !== "0:0:30.1234000") throw "frac toPlainTime";

print("temporal-zdt-json:" + epochJson + "|" + fracJson);
print("temporal-zdt-plaintime:" + renderTime(epochTime) + "|" + renderTime(fracTime));
print("temporal-zdt-valueof:TypeError|TypeError|TypeError");

262;
