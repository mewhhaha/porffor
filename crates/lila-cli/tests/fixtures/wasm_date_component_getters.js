function check(date, year, month, day, weekDay, hour, minute, second, ms, label) {
  if (date.getUTCFullYear() !== year) throw "component";
  if (date.getUTCMonth() !== month) throw "component";
  if (date.getUTCDate() !== day) throw "component";
  if (date.getUTCDay() !== weekDay) throw "component";
  if (date.getUTCHours() !== hour) throw "component";
  if (date.getUTCMinutes() !== minute) throw "component";
  if (date.getUTCSeconds() !== second) throw "component";
  if (date.getUTCMilliseconds() !== ms) throw "component";
  if (date.getMonth() !== month) throw "component";
  if (date.getDate() !== day) throw "component";
  if (date.getDay() !== weekDay) throw "component";
  if (date.getHours() !== hour) throw "component";
  if (date.getMinutes() !== minute) throw "component";
  if (date.getSeconds() !== second) throw "component";
  if (date.getMilliseconds() !== ms) throw "component";
}

check(new Date(0), 1970, 0, 1, 4, 0, 0, 0, 0, "epoch");
check(new Date(-1), 1969, 11, 31, 3, 23, 59, 59, 999, "negative");
check(new Date(951868799999), 2000, 1, 29, 2, 23, 59, 59, 999, "leap boundary");
check(new Date(951868800000), 2000, 2, 1, 3, 0, 0, 0, 0, "march boundary");
if (new Date(NaN).getUTCHours() === new Date(NaN).getUTCHours()) throw "invalid";
var receiverThrew = 0;
try { Date.prototype.getUTCDay.call({}); } catch (e) { receiverThrew = 1; }
if (receiverThrew !== 1) throw "receiver";
if (Date.prototype.getUTCMilliseconds.length !== 0) throw "length";
if (Date.prototype.getUTCMilliseconds.name !== "getUTCMilliseconds") throw "name";
var constructThrew = 0;
try { new Date.prototype.getUTCSeconds(); } catch (e) { constructThrew = 1; }
if (constructThrew !== 1) throw "construct";
262;
