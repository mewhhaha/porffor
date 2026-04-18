/*---
flags: [raw]
---*/

let x = 0;
switch (3) {
  case 1:
    x = 1;
    break;
  default:
    x = 9;
    break;
  case 3:
    x = 3;
}
x;
