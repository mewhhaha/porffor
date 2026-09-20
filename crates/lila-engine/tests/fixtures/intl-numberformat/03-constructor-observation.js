function check(value, message) { if (!value) throw new Error(message); }
const keys = ["localeMatcher", "numberingSystem", "style", "currency", "currencyDisplay", "currencySign", "unit", "unitDisplay", "notation", "minimumIntegerDigits", "minimumFractionDigits", "maximumFractionDigits", "minimumSignificantDigits", "maximumSignificantDigits", "roundingIncrement", "roundingMode", "roundingPriority", "trailingZeroDisplay", "compactDisplay", "useGrouping", "signDisplay"];
const reads = [];
const options = {};
for (const key of keys) Object.defineProperty(options, key, { get() { reads.push(key); return undefined; } });
new Intl.NumberFormat("en-US", options);
check(reads.join(",") === keys.join(","), "option observation order " + reads.join(","));
const events = [];
const marker = {};
let caught;
try {
  new Intl.NumberFormat({ length: 2, 1: { toString() { events.push("locale"); return "en-US"; } } }, {
    get style() { events.push("style"); throw marker; },
    get currency() { events.push("late currency"); }
  });
} catch (error) { caught = error; }
check(caught === marker && events.join(",") === "locale,style", "locale/option abrupt order");
const local = new Intl.Locale("en-US");
local.toString = function () { throw new Error("Locale internal tag must be used"); };
check(new Intl.NumberFormat(local).resolvedOptions().locale === "en-US", "Locale single item");
print("ok constructor observation");
