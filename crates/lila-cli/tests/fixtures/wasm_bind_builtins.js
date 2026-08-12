function add(x, y) { return x + y; }
function F(x) { this.x = x; }

let inc = add.bind(null, 1);
let G = F.bind(null, 2);
let err = Error("x");

inc(2) === 3
  && new G().x === 2
  && err.toString() === "Error: x"
  && TypeError("y").toString() === "TypeError: y";
