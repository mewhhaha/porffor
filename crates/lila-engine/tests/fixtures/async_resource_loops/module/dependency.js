export let count = 0;
await 0;
for (using resource = { [Symbol.dispose]() { count++; } }; false;) {}
for (using resource of [{ [Symbol.dispose]() { count++; } }]) {}
await 0;
