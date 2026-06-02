Boolean.prototype[0] = true;
Boolean.prototype.length = 1;
Array.prototype.map.call(new Boolean(false), function(val, idx, obj) {
  return obj instanceof Boolean;
})[0];
