Boolean.prototype[0] = true;
Boolean.prototype.length = 1;
Array.prototype.map.call(false, function(val, idx, obj) {
  return val;
})[0];
