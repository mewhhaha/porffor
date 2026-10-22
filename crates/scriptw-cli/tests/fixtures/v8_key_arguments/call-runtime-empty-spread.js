(function () {
function f(a, b) { return a === undefined ? 1 : a + b; }
const args = [];
let sum = 0;
for (let i = 0; i < 4096; ++i) sum += f(...args);
print(sum);
})();
