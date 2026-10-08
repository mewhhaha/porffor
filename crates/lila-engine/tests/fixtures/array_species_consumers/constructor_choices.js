function flatSource(source) { return source.flat(0); }
function concatSource(source) { return source.concat(); }

function speciesChoice(invoke, label) {
  var trace = [];
  var resultObject = {};
  function Constructor() { throw new Error('construct trap was bypassed'); }
  var speciesConstructor = new Proxy(Constructor, {
    construct: function (target, argumentsList, newTarget) {
      trace.push('construct:' + argumentsList[0] + ':' + (newTarget === speciesConstructor));
      if (argumentsList.length !== 1) throw new Error('species argument count');
      return resultObject;
    }
  });
  // An Array is an Object for the constructor's @@species lookup.
  var constructorObject = [];
  Object.defineProperty(constructorObject, Symbol.species, {
    get: function () { trace.push('species'); return speciesConstructor; }
  });
  var source = [0];
  Object.defineProperty(source, 'constructor', {
    get: function () { trace.push('constructor'); return constructorObject; }
  });
  Object.defineProperty(source, '0', {
    get: function () { trace.push('get:0'); return 7; }
  });
  var result = invoke(source);
  print(label + ':' + trace.join(',') + ':' + (result === resultObject) + ':' + result[0]);
}

speciesChoice(flatSource, 'flat');
speciesChoice(concatSource, 'concat');
