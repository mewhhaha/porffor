function check(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}

// B.3.2.1 evaluation writes fenv directly; the catch parameter (B.3.4) keeps
// its value while the function-scope var receives the function.
function captured() {
  var seen;
  try {
    throw "param";
  } catch (f) {
    {
      function f() {
        return "fn";
      }
    }
    seen = f;
  }
  var read = function () {
    return typeof f === "function" ? f() : "none";
  };
  return seen + "/" + read();
}
check(captured(), "param/fn", "captured var copy");

// A var declaration in the catch block still assigns the catch parameter
// (B.3.4 note), unlike the Annex B copy.
function varInitializer() {
  var inner;
  try {
    throw 1;
  } catch (x) {
    var x = 2;
    inner = x;
  }
  return inner + "/" + x;
}
check(varInitializer(), "2/undefined", "var initializer assigns catch param");

// Nested catch with the same name: the copy lands in the function's var.
function nested() {
  var observed = [];
  try {
    throw "outer";
  } catch (g) {
    try {
      throw "inner";
    } catch (g) {
      {
        function g() {
          return "g";
        }
      }
      observed.push(g);
    }
    observed.push(g);
  }
  observed.push(typeof g === "function" ? g() : typeof g);
  return observed.join(",");
}
check(nested(), "inner,outer,g", "nested catch params keep values");

// Arrow owner.
var arrowOwner = () => {
  try {
    throw 5;
  } catch (h) {
    {
      function h() {
        return 6;
      }
    }
    check(h, 5, "arrow catch param");
  }
  return h();
};
check(arrowOwner(), 6, "arrow owner copy");
true;
