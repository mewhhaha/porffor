function check(actual, expected, label) {
  if (actual !== expected) {
    throw label;
  }
}

function checkExec(result, value, index, input, label) {
  check(result !== null, true, label + " result");
  check(result[0], value, label + " value");
  check(result.index, index, label + " index");
  check(result.input, input, label + " input");
}

var dot = /./y;
dot.lastIndex = 1;
checkExec(dot.exec("abc"), "b", 1, "abc", "dot exec");
check(dot.lastIndex, 2, "dot lastIndex");

var abc = /abc/y;
checkExec(abc.exec("abc"), "abc", 0, "abc", "abc exec");
check(abc.lastIndex, 3, "abc lastIndex");

var c = /c/y;
c.lastIndex = 1;
check(c.exec("abc"), null, "c failure result");
check(c.lastIndex, 0, "c failure lastIndex");

var b = /b/y;
check(b.test("ab"), false, "b test initial result");
check(b.lastIndex, 0, "b test initial lastIndex");
b.lastIndex = 1;
check(b.test("ab"), true, "b test match result");
check(b.lastIndex, 2, "b test match lastIndex");

var newline = /./y;
check(newline.exec("\n"), null, "newline result");
check(newline.lastIndex, 0, "newline lastIndex");

var execReadonly = /a/y;
Object.defineProperty(execReadonly, "lastIndex", { writable: false });
var execReadonlyThrows = false;
try {
  execReadonly.exec("a");
} catch (error) {
  execReadonlyThrows = error instanceof TypeError;
}
check(execReadonlyThrows, true, "readonly exec TypeError");

var testReadonly = /a/y;
Object.defineProperty(testReadonly, "lastIndex", { writable: false });
var testReadonlyThrows = false;
try {
  testReadonly.test("a");
} catch (error) {
  testReadonlyThrows = error instanceof TypeError;
}
check(testReadonlyThrows, true, "readonly test TypeError");

var execCoercion = /a/y;
execCoercion.lastIndex = {
  valueOf: function() {
    throw 42;
  }
};
var execCoercionCaught = false;
try {
  execCoercion.exec("a");
} catch (error) {
  execCoercionCaught = error === 42;
}
check(execCoercionCaught, true, "exec lastIndex coercion throw");

function catchesTestCoercion() {
  var testCoercion = /a/y;
  testCoercion.lastIndex = {
    valueOf: function() {
      throw 43;
    }
  };
  try {
    testCoercion.test("a");
  } catch (error) {
    return error === 43;
  }
  return false;
}
check(catchesTestCoercion(), true, "test lastIndex coercion throw");

true;
