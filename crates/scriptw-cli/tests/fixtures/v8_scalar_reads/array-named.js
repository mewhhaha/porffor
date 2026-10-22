var values = [];
values.alpha = 11; values.beta = 23; values.gamma = 37; values.delta = 53;
function read(target, key) { return target[key]; }
var sum = 0;
for (var i = 0; i < 5000; i++) sum = (sum + read(values, 'alpha') + read(values, 'beta') + read(values, 'gamma') + read(values, 'delta')) | 0;
print(sum);
