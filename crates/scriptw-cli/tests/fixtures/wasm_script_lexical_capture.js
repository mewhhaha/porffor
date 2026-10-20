let x = 3;
let obj = { value: 4 };

function read() {
  if (x !== 3) throw "x";
  if (obj.value !== 4) throw "obj";
  obj.value = 5;
}

read();
if (obj.value !== 5) throw "write";
true;
