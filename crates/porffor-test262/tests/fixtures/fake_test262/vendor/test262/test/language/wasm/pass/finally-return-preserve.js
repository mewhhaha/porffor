/*---
flags: [raw]
---*/

function f() {
  try {
    return 1;
  } finally {}
}

f();
