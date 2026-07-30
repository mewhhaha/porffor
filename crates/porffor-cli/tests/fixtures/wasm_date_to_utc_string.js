function expect(date, expected) {
  if (date.toUTCString() !== expected) throw expected;
}

expect(new Date(0), "Thu, 01 Jan 1970 00:00:00 GMT");
expect(new Date(Date.UTC(2014, 1, 1)), "Sat, 01 Feb 2014 00:00:00 GMT");
expect(new Date(Date.UTC(2014, 2, 23)), "Sun, 23 Mar 2014 00:00:00 GMT");
expect(new Date("2014-03-23T00:00:00Z"), "Sun, 23 Mar 2014 00:00:00 GMT");
expect(new Date("0020-01-01T00:00:00Z"), "Wed, 01 Jan 0020 00:00:00 GMT");
expect(new Date("-000001-07-01T00:00Z"), "Thu, 01 Jul -0001 00:00:00 GMT");
expect(new Date("-012345-07-01T00:00Z"), "Thu, 01 Jul -12345 00:00:00 GMT");

let date = new Date(0);
date.setUTCFullYear(20);
expect(date, "Wed, 01 Jan 0020 00:00:00 GMT");

date = new Date(0);
date.setUTCFullYear(-1);
expect(date, "Fri, 01 Jan -0001 00:00:00 GMT");

date = new Date(0);
date.setUTCFullYear(12345);
if (date.toUTCString().split(" ")[3] !== "12345") throw "positive extended year";

date = new Date(0);
date.setUTCFullYear(-12345);
if (date.toUTCString().split(" ")[3] !== "-12345") throw "negative extended year";

if (new Date(NaN).toUTCString() !== "Invalid Date") throw "invalid";
if (Date.prototype.toGMTString !== Date.prototype.toUTCString) throw "alias";

262;
