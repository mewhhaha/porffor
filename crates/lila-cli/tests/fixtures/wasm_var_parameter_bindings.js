function preservesParameterBinding(x) {
  var x;
  return x;
}

function capturesParameterBinding(x) {
  var x;
  return function () {
    return x;
  };
}

function initializesParameterBinding(x) {
  var x = x + 1;
  return x;
}

function preservesDefaultParameterBinding(x = 11) {
  var x;
  return x;
}

if (
  preservesParameterBinding(7) !== 7 ||
  capturesParameterBinding(8)() !== 8 ||
  initializesParameterBinding(9) !== 10 ||
  preservesDefaultParameterBinding() !== 11
) {
  throw "var parameter binding";
}

true;
