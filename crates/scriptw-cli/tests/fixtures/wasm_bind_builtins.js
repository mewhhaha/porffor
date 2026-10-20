function add(x, y) { return x + y; }
function F(x) { this.x = x; }
function strictThis() { "use strict"; return this; }
function sloppyThis() { return this; }

let inc = add.bind(null, 1);
let G = F.bind(null, 2);
let err = Error("x");
let token = Symbol("bound this");
let integer = 123n;
let strictNumber = strictThis.bind(7);
let strictSymbol = strictThis.bind(token);
let strictBigInt = strictThis.bind(integer);
let sloppyNumber = sloppyThis.bind(7);

inc(2) === 3
  && new G().x === 2
  && err.toString() === "Error: x"
  && TypeError("y").toString() === "TypeError: y"
  && strictNumber() === 7
  && strictSymbol() === token
  && strictBigInt() === integer
  && sloppyNumber() !== sloppyNumber();
