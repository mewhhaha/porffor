function same(actual, expected, label) {
  if (actual !== expected) throw label;
}

same(Date.UTC(1970), 0, "year only");
same(Date.UTC(1970, 0, 1, 0, 0, 0, 1), 1, "full args");
same(Date.UTC(1970, 12, 1), 31536000000, "month overflow");
same(Date.UTC(0, 0, 1), -2208988800000, "year remap");
if (Date.UTC() === Date.UTC()) throw "missing year NaN";
if (Date.UTC(1970, NaN) === Date.UTC(1970, NaN)) throw "NaN arg";

same(Number("   +00200.000E-0002\t"), 2, "trimmed exponent");
same(Number("1e3"), 1000, "positive exponent");
same(Number("1E-3"), 0.001, "negative exponent");
if (Number("not a number") === Number("not a number")) throw "malformed text";
if (Number("1e") === Number("1e")) throw "missing exponent digits";
if (Number("1e+") === Number("1e+")) throw "missing signed exponent digits";
if (Number("1e-") === Number("1e-")) throw "missing negative exponent digits";
if (Number("e1") === Number("e1")) throw "missing significand";

var d = new Date(0);
same(d.setTime("   +00200.000E-0002\t"), 2, "setTime exponent");
d = new Date(0);
same(d.setUTCMilliseconds("   +00200.000E-0002\t"), 2, "setter exponent");
d = new Date(0);
if (d.setYear("not a number") === d.setYear("not a number")) throw "setYear malformed return";
if (d.valueOf() === d.valueOf()) throw "setYear malformed stores NaN";

262;
