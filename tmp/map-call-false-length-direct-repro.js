Boolean.prototype[0] = true;
Boolean.prototype.length = 1;
var m = Array.prototype.map;
m.call(false, function(val, idx, obj) {
  return Object(obj).length;
})[0];
