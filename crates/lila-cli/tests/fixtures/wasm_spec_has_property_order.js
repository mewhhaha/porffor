let trace = "";

function key() {
  trace += "k";
  return "x";
}

function object() {
  trace += "o";
  return { x: 1 };
}

(key() in object()) && trace === "ko";
