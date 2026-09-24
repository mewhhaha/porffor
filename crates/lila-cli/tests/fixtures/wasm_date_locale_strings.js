// Lila implements the ECMA-402 Internationalization API, so ECMA-402 20.4.1-3
// supersede the implementation-defined ECMA-262 21.4.4.38-40 definitions:
// each method formats through CreateDateTimeFormat(%Intl.DateTimeFormat%,
// locales, options, required, defaults). With no options, the defaults fill
// in exactly the numeric fields named below, so each result must equal an
// explicit Intl.DateTimeFormat over those fields. The ECMA-262 toDateString /
// toString / toTimeString forms are NOT a conforming result here.
const date = new Date(0);
const numeric = "numeric";

// toLocaleDateString: required "date", defaults "date".
const dateFields = { year: numeric, month: numeric, day: numeric };
// toLocaleString: required "any", defaults "all".
const allFields = {
  year: numeric,
  month: numeric,
  day: numeric,
  hour: numeric,
  minute: numeric,
  second: numeric,
};
// toLocaleTimeString: required "time", defaults "time".
const timeFields = { hour: numeric, minute: numeric, second: numeric };

function formatted(fields) {
  return new Intl.DateTimeFormat(undefined, fields).format(date);
}

if (date.toLocaleDateString() !== formatted(dateFields)) throw "date string";
if (date.toLocaleDateString() !== new Intl.DateTimeFormat().format(date)) {
  throw "date string matches the constructor's date defaults";
}
if (date.toLocaleString() !== formatted(allFields)) throw "date and time string";
if (date.toLocaleTimeString() !== formatted(timeFields)) throw "time string";

// The three ECMA-262 local-string formats stay distinct from the locale ones:
// ToDateString(tv) is DateString(t) + " " + TimeString(t) +
// TimeZoneString(tv), and toTimeString is TimeString(t) + TimeZoneString(tv)
// (21.4.4.35, 21.4.4.41, 21.4.4.42).
if (!/^[A-Z][a-z]{2} [A-Z][a-z]{2} \d{2} \d{4}$/.test(date.toDateString())) {
  throw "DateString shape";
}
if (!/^\d{2}:\d{2}:\d{2} GMT[+-]\d{4}/.test(date.toTimeString())) {
  throw "TimeString + TimeZoneString shape";
}
if (date.toString() !== date.toDateString() + " " + date.toTimeString()) {
  throw "ToDateString composition";
}
if (date.toLocaleDateString() === date.toDateString()) {
  throw "date string is locale formatted";
}
if (date.toLocaleString() === date.toString()) {
  throw "date and time string is locale formatted";
}
if (date.toLocaleTimeString() === date.toTimeString()) {
  throw "time string is locale formatted";
}

// A NaN time value short-circuits before CreateDateTimeFormat.
const invalid = new Date(NaN);
for (const name of ["toLocaleDateString", "toLocaleString", "toLocaleTimeString"]) {
  if (invalid[name]() !== "Invalid Date") throw `${name} invalid date`;
}

for (const [name, length] of [
  ["toLocaleDateString", 0],
  ["toLocaleString", 0],
  ["toLocaleTimeString", 0],
]) {
  const method = Date.prototype[name];
  if (method.name !== name) throw `${name} name`;
  if (method.length !== length) throw `${name} length`;

  const descriptor = Object.getOwnPropertyDescriptor(Date.prototype, name);
  if (!descriptor) throw `${name} descriptor`;
  if (!descriptor.writable) throw `${name} writable`;
  if (descriptor.enumerable) throw `${name} enumerable`;
  if (!descriptor.configurable) throw `${name} configurable`;
}

let threw = false;
try {
  Date.prototype.toLocaleString.call({});
} catch (error) {
  threw = error instanceof TypeError;
}
if (!threw) throw "incompatible receiver";

262;
