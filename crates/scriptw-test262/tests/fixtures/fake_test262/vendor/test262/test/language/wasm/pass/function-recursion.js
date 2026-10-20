/*---
flags: [raw]
---*/

function up(n) {
  if (n === 0) {
    return 0;
  }
  return up(n - 1) + 1;
}

up(3);
