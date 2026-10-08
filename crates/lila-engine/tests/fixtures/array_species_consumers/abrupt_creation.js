function flatSource(source) { return source.flat(0); }
function concatSource(source) { return source.concat(); }

function abruptCreation(invoke, label, stage) {
  var original = { stage: stage };
  var reads = 0;
  var source = [0];
  Object.defineProperty(source, '0', {
    get: function () { reads++; return 9; }
  });
  function Constructor() { throw original; }
  var constructorObject = {};
  Object.defineProperty(constructorObject, Symbol.species, {
    get: function () {
      if (stage === 'species') throw original;
      if (stage === 'nonconstructor') return () => ({});
      return Constructor;
    }
  });
  Object.defineProperty(source, 'constructor', {
    get: function () {
      if (stage === 'constructor') throw original;
      return constructorObject;
    }
  });
  var correct = false;
  try { invoke(source); }
  catch (error) {
    correct = stage === 'nonconstructor' ? error instanceof TypeError : error === original;
  }
  print(label + '/' + stage + ':' + correct + ':' + reads);
}

for (var stage of ['constructor', 'species', 'construct', 'nonconstructor']) {
  abruptCreation(flatSource, 'flat', stage);
  abruptCreation(concatSource, 'concat', stage);
}
