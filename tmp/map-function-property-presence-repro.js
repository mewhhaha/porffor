var obj = function(a, b) {
  return a + b;
};
obj[0] = 11;
obj[1] = 9;

if (!(0 in obj)) throw "has 0";
if (!(1 in obj)) throw "has 1";
if (obj[0] !== 11) throw "read 0";
if (obj[1] !== 9) throw "read 1";
