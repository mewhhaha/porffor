(function () {
function f(a, b) { return a === undefined ? 1 : a + b; }
const args = [2, 3];
let sum = 0;
for (let i = 0; i < 4096; ++i) sum += Reflect.apply(f, null, args);
print(sum);
})();
