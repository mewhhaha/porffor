function check(value, message) { if (!value) throw new Error(message); }
const events = [];
const prototype = {};
const target = new Proxy(function Target() {}, {
  get(object, key, receiver) {
    if (key === "prototype") { events.push("prototype"); return prototype; }
    return Reflect.get(object, key, receiver);
  }
});
const locales = { length: 1, 0: { toString() { events.push("locale"); return "en-US"; } } };
const options = { get localeMatcher() { events.push("localeMatcher"); return "lookup"; } };
const result = Reflect.construct(Intl.NumberFormat, [locales, options], target);
check(Object.getPrototypeOf(result) === prototype, "newTarget prototype");
check(events.join(",") === "prototype,locale,localeMatcher", "newTarget precedes locale/options " + events.join(","));
const marker = {};
const abruptTarget = new Proxy(function AbruptTarget() {}, {
  get(object, key, receiver) {
    if (key === "prototype") throw marker;
    return Reflect.get(object, key, receiver);
  }
});
events.length = 0;
let caught;
try { Reflect.construct(Intl.NumberFormat, [locales, options], abruptTarget); }
catch (error) { caught = error; }
check(caught === marker && events.length === 0, "prototype abrupt completion wins");
print("ok newTarget observation");
