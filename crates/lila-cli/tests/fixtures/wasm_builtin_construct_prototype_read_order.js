// Built-in [[Construct]] bodies that allocate their own result read
// Get(newTarget, "prototype") exactly once, at their own spec step, and the
// ones that throw on construction never read it. A Proxy newTarget makes every
// read observable. The shared construct dispatcher once pre-allocated an
// ordinary receiver for these callees, adding a leading "prototype" read.

function observedNewTarget(log) {
  return new Proxy(function () {}, {
    get(target, key, receiver) {
      if (key === "prototype") {
        log.push("prototype");
        return null;
      }
      return Reflect.get(target, key, receiver);
    },
  });
}

function coerced(log, label, value) {
  return {
    valueOf() {
      log.push(label);
      return value;
    },
  };
}

function reads(constructor, makeArguments) {
  const log = [];
  let outcome;
  try {
    Reflect.construct(constructor, makeArguments(log), observedNewTarget(log));
    outcome = "ok";
  } catch (error) {
    outcome = error instanceof TypeError ? "TypeError" : "other";
  }
  return log.join(",") + ":" + outcome;
}

const rows = [
  // ECMA-262 21.4.2.1 steps 3-4 coerce before OrdinaryCreateFromConstructor.
  ["Date(value)", Date, (log) => [coerced(log, "value", 4)], "value,prototype:ok"],
  ["Date()", Date, () => [], "prototype:ok"],
  [
    "Date(year, month)",
    Date,
    (log) => [coerced(log, "year", 1970), coerced(log, "month", 0)],
    "year,month,prototype:ok",
  ],
  // Temporal constructors validate their arguments before
  // OrdinaryCreateFromConstructor inside CreateTemporal*.
  [
    "Temporal.PlainDate",
    Temporal.PlainDate,
    (log) => [coerced(log, "year", 2020), 1, 1],
    "year,prototype:ok",
  ],
  ["Temporal.PlainTime", Temporal.PlainTime, () => [], "prototype:ok"],
  [
    "Temporal.PlainDateTime",
    Temporal.PlainDateTime,
    () => [2020, 1, 1],
    "prototype:ok",
  ],
  [
    "Temporal.PlainYearMonth",
    Temporal.PlainYearMonth,
    () => [2020, 1],
    "prototype:ok",
  ],
  [
    "Temporal.PlainMonthDay",
    Temporal.PlainMonthDay,
    () => [1, 1],
    "prototype:ok",
  ],
  ["Temporal.Duration", Temporal.Duration, () => [], "prototype:ok"],
  ["Temporal.Instant", Temporal.Instant, () => [0n], "prototype:ok"],
  // BigInt, Symbol and %TypedArray% throw before reading NewTarget.
  ["BigInt", BigInt, () => [1], ":TypeError"],
  ["Symbol", Symbol, () => [], ":TypeError"],
  ["%TypedArray%", Object.getPrototypeOf(Int8Array), () => [], ":TypeError"],
  // A bound function forwards Construct(target, args, newTarget); only the
  // target reads NewTarget.prototype.
  ["bound ordinary", function () {}.bind(null), () => [], "prototype:ok"],
  [
    "bound Date",
    Date.bind(null),
    (log) => [coerced(log, "value", 4)],
    "value,prototype:ok",
  ],
];

const failures = [];
for (const [label, constructor, makeArguments, expected] of rows) {
  const actual = reads(constructor, makeArguments);
  if (actual !== expected) failures.push(label + " => " + actual);
}
if (failures.length !== 0) throw failures.join("; ");

true;
