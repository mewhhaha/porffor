var separator = {
  toString: function () {
    return /\u0037\u0037/g;
  },
};

Number.prototype.split = String.prototype.split;

var threw = false;
try {
  (6776767677.006771122677555).split(separator);
} catch (error) {
  threw = error instanceof TypeError;
}

if (!threw) throw "primitive number borrowed split regexp separator";

123;
