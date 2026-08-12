function same(a, b) {
  if (a.valueOf() !== b.valueOf()) throw "local";
}

let date = new Date(0);
if (date.setUTCFullYear(2000, 1, 29) !== 951782400000) throw "fullYear";
if (date.getUTCFullYear() !== 2000) throw "fullYear value";

date = new Date(0);
date.setUTCMonth(12);
if (date.getUTCFullYear() !== 1971 || date.getUTCMonth() !== 0) throw "month";

date = new Date(2000, 0, 1);
date.setUTCDate(0);
if (date.getUTCFullYear() !== 1999 || date.getUTCMonth() !== 11 || date.getUTCDate() !== 31) throw "date";

date = new Date(0);
date.setUTCHours(1, 2, 3, 4);
date.setUTCSeconds(60);
date.setUTCMilliseconds(-1);
if (date.getUTCHours() !== 1 || date.getUTCMinutes() !== 2 || date.getUTCSeconds() !== 59 || date.getUTCMilliseconds() !== 999) throw "time";

same(new Date(new Date(0).setFullYear(2001, 2, 4)), new Date(new Date(0).setUTCFullYear(2001, 2, 4)));
same(new Date(new Date(0).setHours(8, 9, 10, 11)), new Date(new Date(0).setUTCHours(8, 9, 10, 11)));

date = new Date(NaN);
if (date.setUTCFullYear(2000) !== 946684800000) throw "invalid fullYear";
date = new Date(NaN);
if (date.setUTCMonth(0) === date.setUTCMonth(0)) throw "invalid month";

if (Date.prototype.setUTCFullYear.length !== 3) throw "length";
if (Date.prototype.setUTCMilliseconds.name !== "setUTCMilliseconds") throw "name";

262;
