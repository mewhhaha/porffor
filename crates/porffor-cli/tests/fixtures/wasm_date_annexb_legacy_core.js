if (new Date(1899, 0).getYear() !== -1) throw "getYear 1899";
if (new Date(1970, 0).getYear() !== 70) throw "getYear 1970";
if (new Date({}).getYear() === new Date({}).getYear()) throw "invalid getYear";

let date = new Date(1970, 1, 2, 3, 4, 5);
let expected = new Date(1971, 1, 2, 3, 4, 5).valueOf();
if (date.setYear(71) !== expected) throw "setYear relative";
if (date.valueOf() !== expected) throw "setYear value";

date = new Date(1970, 0);
date.setYear(2000);
if (date.getFullYear() !== 2000) throw "setYear absolute";

date = new Date(0);
if (date.setYear(NaN) === date.setYear(NaN)) throw "setYear NaN";

date = new Date(0);
if (date.setYear() === date.setYear()) throw "setYear undefined";

let threw = 0;
try {
  Date.prototype.getYear.call({});
} catch (e) {
  if (e.name === "TypeError") threw += 1;
}
try {
  Date.prototype.setYear.call(null, 1);
} catch (e) {
  if (e.name === "TypeError") threw += 1;
}
if (threw !== 2) throw "receiver TypeError";

if (Date.prototype.toGMTString !== Date.prototype.toUTCString) throw "GMT alias";
if (Date.prototype.getYear.length !== 0) throw "getYear length";
if (Date.prototype.setYear.length !== 1) throw "setYear length";
if (Date.prototype.getYear.name !== "getYear") throw "getYear name";
if (Date.prototype.setYear.name !== "setYear") throw "setYear name";

262;
