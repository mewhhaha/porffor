function owner() {
  let f = function () {
    return 2;
  };
  {
    function f() {
      return 1;
    }
    function g() {
      return f();
    }
    return g();
  }
}

function sameBlockLet() {
  "use strict";
  let value = 2;
  {
    let value = 1;
    function read() {
      return value;
    }
    return read();
  }
}

function nearestNestedShadow() {
  "use strict";
  let value = 0;
  {
    let value = 1;
    {
      const value = 2;
      function read() {
        return value;
      }
      return read();
    }
  }
}

owner() === 1 && sameBlockLet() === 1 && nearestNestedShadow() === 2;
