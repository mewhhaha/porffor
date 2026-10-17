var value = JSON.parse('[1e+0,-0,"\\uD800",{"n":2,"n":3e0}]', function (key, value, context) {
  print((key === "" ? "<root>" : key) + ":" + (context.source === undefined ? "none" : context.source));
  return value;
});
print(value.length + ":" + Object.is(value[1], -0) + ":" + value[2].charCodeAt(0) + ":" + value[3].n);
