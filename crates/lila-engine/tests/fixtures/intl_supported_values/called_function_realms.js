function check(value, message) { if (!value) throw new Error(message); }
function same(actual, expected, message) { check(Object.is(actual, expected), message); }
function throws(constructor, action, message) { let caught = false; try { action(); } catch (error) { caught = error instanceof constructor; } check(caught, message); }

const other = __lilaCreateRealm().global;
const f = other.Intl.supportedValuesOf;
const otherArrayPrototype = other.Array.prototype;
const otherRangeError = other.RangeError, otherTypeError = other.TypeError;
const saved = f("unit");
same(Object.getPrototypeOf(saved), otherArrayPrototype, "callee Array realm");
check(Object.getPrototypeOf(saved) !== Array.prototype, "foreign Array prototype differs");
other.Intl.supportedValuesOf = () => { throw new Error("public method consulted"); };
other.Array = () => { throw new Error("public Array consulted"); };
other.RangeError = () => { throw new Error("public RangeError consulted"); };
other.TypeError = () => { throw new Error("public TypeError consulted"); };
same(Object.getPrototypeOf(f.call({}, "timeZone")), otherArrayPrototype, "saved function uses immutable realm Array");
throws(otherRangeError, () => f("invalid"), "callee RangeError realm");
throws(otherTypeError, () => f(Symbol()), "callee ToString TypeError realm");
throws(TypeError, () => new f("unit"), "caller nonconstructor rejection");
let nested;
const result = f({ [Symbol.toPrimitive]() { nested = Intl.supportedValuesOf("unit"); return "currency"; } });
same(Object.getPrototypeOf(nested), Array.prototype, "nested local Array realm");
same(Object.getPrototypeOf(result), otherArrayPrototype, "outer called realm restored");
const sentinel = {};
let caught;
try { f({ toString() { throw sentinel; } }); } catch (error) { caught = error; }
same(caught, sentinel, "foreign builtin preserves local abrupt identity");

print("ok called_function_realms");
262;
