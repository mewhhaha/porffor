var insertedFunction, insertedArguments, functionCalls = 0;
var result = JSON.parse('{"first":0,"function":1,"arguments":2}', function (key, value, context) {
  print((key === '' ? '<root>' : key) + ':' + (Object.prototype.hasOwnProperty.call(context, 'source') ? context.source : 'none'));
  if (key === 'first') {
    insertedFunction = function inserted() { print('function-effect'); functionCalls++; return 'called'; };
    insertedFunction.branch = {leaf:3};
    insertedFunction.tail = 4;
    insertedArguments = (function () { return arguments; })({leaf:5});
    insertedArguments.extra = [6];
    this.function = insertedFunction;
    this.arguments = insertedArguments;
  }
  if (this === insertedFunction && key === 'tail') print('function-call:' + this() + ':' + functionCalls);
  return key === 'leaf' ? value + 10 : value;
});
print((result.function === insertedFunction) + ':' + result.function.branch.leaf + ':' + result.arguments[0].leaf + ':' + result.arguments.extra[0] + ':' + functionCalls);
