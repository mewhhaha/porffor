(function () {
const keys = []; for (let i = 0; i < 256; ++i) keys.push(i);
let sum = 0;
for (let round = 0; round < 8; ++round) {
  const table = Object.create(null);
  for (let i = 0; i < keys.length; ++i) { table[keys[i]] = i + round; }
  sum += Object.keys(table).length + table[keys[255]];
}
print(sum);
})();
