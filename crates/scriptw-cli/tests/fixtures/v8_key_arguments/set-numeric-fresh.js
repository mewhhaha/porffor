(function () {
const keys = []; for (let i = 0; i < 256; ++i) keys.push(i);
let sum = 0;
for (let round = 0; round < 8; ++round) {
  const table = new Set();
  for (let i = 0; i < keys.length; ++i) { table.add(keys[i]); }
  sum += table.size + (table.has(keys[0]) ? 1 : 0);
}
print(sum);
})();
