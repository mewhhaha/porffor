function flatSource(source) { return source.flat(0); }
function concatSource(source) { return source.concat(); }

function lookupCount(invoke, label) {
  var constructorGets = 0;
  var speciesGets = 0;
  var constructions = 0;
  var resultObject = {};
  function Constructor(length) {
    if (length !== 0) throw new Error('initial species length');
    constructions++;
    return resultObject;
  }
  var constructorObject = {};
  Object.defineProperty(constructorObject, Symbol.species, {
    get: function () { speciesGets++; return Constructor; }
  });
  var source = new Proxy([], {
    get: function (target, key, receiver) {
      if (key === 'constructor') {
        constructorGets++;
        return constructorObject;
      }
      return Reflect.get(target, key, receiver);
    }
  });
  var result = invoke(source);
  print(label + ':' + constructorGets + ':' + speciesGets + ':' + constructions + ':' +
        (result === resultObject));
}

lookupCount(flatSource, 'flat');
lookupCount(concatSource, 'concat');
