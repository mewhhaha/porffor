var callbackCount = 0;
var method = Array.prototype.forEach;
var frozen = Object.freeze(method);

["z"].forEach(function(value) {
  callbackCount++;
  Object.freeze(Array.prototype.forEach);
});

frozen === method && Object.isFrozen(method) && callbackCount === 1;
