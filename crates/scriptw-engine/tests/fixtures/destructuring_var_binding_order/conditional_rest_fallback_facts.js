function arrayFallback(scope) {
  var target = 1;
  with (scope) { var [...target] = [2]; }
  return target + 1;
}
function objectFallback(scope) {
  var target = 3;
  with (scope) { var { ...target } = { p: 5 }; }
  return target + 1;
}
var arrayScope = { target: 0 };
var objectScope = { target: 0 };
arrayFallback(arrayScope) === 2 && Array.isArray(arrayScope.target)
  && arrayScope.target.length === 1 && arrayScope.target[0] === 2
  && objectFallback(objectScope) === 4 && objectScope.target.p === 5;
