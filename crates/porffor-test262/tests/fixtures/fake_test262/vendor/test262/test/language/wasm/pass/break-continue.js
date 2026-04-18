/*---
flags: [raw]
---*/

let i = 0;
let sum = 0;
for (; i < 5; i = i + 1) {
  if (i === 2) {
    continue;
  }
  if (i === 4) {
    break;
  }
  sum = sum + i;
}
sum;
