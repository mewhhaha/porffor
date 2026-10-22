var values = [11, 23, 37, 53];
function read(target, key) { return target[key]; }
var sum = 0;
for (var i = 0; i < 20000; i++) sum = (sum + read(values, i & 3)) | 0;
print(sum);
