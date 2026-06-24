function check(value, label) {
  if (!value) throw "String slice fixture failed: " + label;
}

function Test262Error(message) {}
function __porfCheck(value, message) {
  if (!value) {
    throw new Test262Error(message);
  }
}

__porfCheck("gnulluna".slice(null, -3) === "gnull", "materialized Test262 helper");

check("gnulluna".slice(0, 5) === "gnull", "basic end");
check("gnulluna".slice() === "gnulluna", "omitted args");
check("gnulluna".slice(null, -3) === "gnull", "null start negative end");
check("gnulluna".slice(undefined, undefined) === "gnulluna", "undefined args");
check("report".slice(function() {}()) === "report", "undefined function start");
check("12345".slice(-3, -1) === "34", "negative bounds");
check("12345".slice(4, 1) === "", "end before start");
check("true".slice(false, true) === "t", "boolean indexes");

var indexObject = {
  valueOf: function() {
    return 2;
  }
};
var indexString = "\u0035ABBBABAB";
check(indexString.slice(indexObject, indexString.slice(0, 1)) === "BBB", "object index coercion");

var bool = new Boolean;
bool.slice = String.prototype.slice;
var x;
check(bool.slice(function() {
  return true;
}(), x) === "alse", "copied boolean wrapper slice");

var object = new Object(true);
object.slice = String.prototype.slice;
check(object.slice(false, true) === "t", "copied Object(true) slice");

check(String.prototype.slice.call(11.001002) === "11.001002", "number receiver");

try {
  "ABB\u0041BABAB".slice({
    valueOf: function() {
      throw "instart";
    }
  }, {
    valueOf: function() {
      throw "inend";
    }
  });
  check(false, "start valueOf throw did not propagate");
} catch (error) {
  check(error === "instart", "start valueOf throw");
}

try {
  "ABB\u0041BABAB\u0031BBAA".slice({
    valueOf: function() {
      return {};
    },
    toString: function() {
      return 1;
    }
  }, {
    toString: function() {
      throw "inend";
    }
  });
  check(false, "end toString throw did not propagate");
} catch (error) {
  check(error === "inend", "end toString throw");
}

true;
