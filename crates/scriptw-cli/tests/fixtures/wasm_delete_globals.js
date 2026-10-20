globalThis.x = 1;
let deletedGlobal = delete x;

var kept = 2;
let keptDelete = delete kept;

function f() {}
let fnDelete = delete f;

let missingDelete = delete missingName;

function eraseImplicit() {
  y = 3;
  return delete y;
}

deletedGlobal
  && typeof x === "undefined"
  && keptDelete === false
  && kept === 2
  && fnDelete === false
  && typeof f === "function"
  && missingDelete
  && eraseImplicit();
