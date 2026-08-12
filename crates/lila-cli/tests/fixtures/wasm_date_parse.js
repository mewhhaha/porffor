function same(actual, expected, label) {
  if (actual !== expected) throw label;
}

same(Date.parse.length, 1, "length");
same(Date.parse.name, "parse", "name");

var descriptor = Object.getOwnPropertyDescriptor(Date, "parse");
if (!descriptor.writable) throw "writable";
if (descriptor.enumerable) throw "enumerable";
if (!descriptor.configurable) throw "configurable";

var threw = false;
try {
  new Date.parse();
} catch (error) {
  threw = error instanceof TypeError;
}
if (!threw) throw "constructable";

same(Date.parse("-271821-04-20T00:00:00.000Z"), -8640000000000000, "minimum");
same(Date.parse("+275760-09-13T00:00:00.000Z"), 8640000000000000, "maximum");
if (Date.parse("-271821-04-19T23:59:59.999Z") === Date.parse("-271821-04-19T23:59:59.999Z")) throw "below minimum";
if (Date.parse("+275760-09-13T00:00:00.001Z") === Date.parse("+275760-09-13T00:00:00.001Z")) throw "above maximum";

same(Date.parse("1970-01-01"), 0, "date only");
same(Date.parse("1970-01-01T00:00:00"), 0, "local date time");
same(Date.parse("1970-01-01T01:00:00+01:00"), 0, "offset");

if (Date.parse("-000000-03-31T00:45Z") === Date.parse("-000000-03-31T00:45Z")) throw "negative zero Z";
if (Date.parse("-000000-03-31T00:45") === Date.parse("-000000-03-31T00:45")) throw "negative zero local";
if (Date.parse("-000000-03-31T00:45+01:00") === Date.parse("-000000-03-31T00:45+01:00")) throw "negative zero offset";

var epoch = new Date(0);
same(Date.parse(epoch.toString()), 0, "toString");
same(Date.parse(epoch.toUTCString()), 0, "toUTCString");
same(Date.parse(epoch.toISOString()), 0, "toISOString");

262;
