var values = [];
values['01'] = 11; values['1e0'] = 23; values['4294967295'] = 37; values['-0'] = 53;
function read(target, key) { return target[key]; }
var sum = 0;
for (var i = 0; i < 5000; i++) sum = (sum + read(values, '01') + read(values, '1e0') + read(values, '4294967295') + read(values, '-0')) | 0;
print(sum);
