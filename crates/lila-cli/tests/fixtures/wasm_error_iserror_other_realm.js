var other = __lilaCreateRealm().global;

function check(label, value) {
  if (value !== true) {
    return "missing " + label;
  }
  return label;
}

[
  check("Error", Error.isError(new other.Error())),
  check("EvalError", Error.isError(new other.EvalError())),
  check("RangeError", Error.isError(new other.RangeError())),
  check("ReferenceError", Error.isError(new other.ReferenceError())),
  check("SyntaxError", Error.isError(new other.SyntaxError())),
  check("TypeError", Error.isError(new other.TypeError())),
  check("URIError", Error.isError(new other.URIError())),
  check("AggregateError", Error.isError(new other.AggregateError([]))),
  check("SuppressedError", Error.isError(new other.SuppressedError()))
].join(",");
