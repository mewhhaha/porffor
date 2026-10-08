function checkIdentityPair(literal, patternUnits, flags, inputUnits, expectedStart, expectedUnits, captureUnits, captureStart) {
  var expressions = [literal, new RegExp(fromUnits(patternUnits), flags)];
  var input = fromUnits(inputUnits);
  for (var e = 0; e < expressions.length; e++) {
    var expression = expressions[e];
    require(expression.source === fromUnits(patternUnits), 'original non-ASCII identity source');
    var match = expression.exec(input);
    if (expectedUnits === null) {
      require(match === null, 'identity atom keeps its required UTF-16 lead');
      continue;
    }
    require(match !== null && match.index === expectedStart, 'identity match position');
    require(match[0] === fromUnits(expectedUnits), 'identity match units');
    require(match.indices[0][0] === expectedStart && match.indices[0][1] === expectedStart + expectedUnits.length, 'identity match UTF-16 range');
    if (captureUnits !== null) {
      require(match[1] === fromUnits(captureUnits), 'identity capture units');
      require(match.indices[1][0] === captureStart && match.indices[1][1] === captureStart + captureUnits.length, 'identity capture UTF-16 range');
    }
  }
}

checkIdentityPair(/(\é+)/d, [40, 92, 233, 43, 41], 'd', [233, 233], 0, [233, 233], [233, 233], 0);
checkIdentityPair(/(\é+)/di, [40, 92, 233, 43, 41], 'di', [201, 201], 0, [201, 201], [201, 201], 0);
checkIdentityPair(/(\𠮷?)/d, [40, 92, 0xd842, 0xdfb7, 63, 41], 'd', [0xd842], 0, [0xd842], [0xd842], 0);
checkIdentityPair(/(\𠮷?)/d, [40, 92, 0xd842, 0xdfb7, 63, 41], 'd', [0xd842, 0xdfb7], 0, [0xd842, 0xdfb7], [0xd842, 0xdfb7], 0);
checkIdentityPair(/(\𠮷??)/d, [40, 92, 0xd842, 0xdfb7, 63, 63, 41], 'd', [0xd842, 0xdfb7], 0, [0xd842], [0xd842], 0);
checkIdentityPair(/\𠮷{0}/d, [92, 0xd842, 0xdfb7, 123, 48, 125], 'd', [], 0, null, null);
checkIdentityPair(/\𠮷{0}/d, [92, 0xd842, 0xdfb7, 123, 48, 125], 'd', [0xd842], 0, [0xd842], null);
checkIdentityPair(/(\𠮷{2})/d, [40, 92, 0xd842, 0xdfb7, 123, 50, 125, 41], 'd', [0xd842, 0xdfb7, 0xdfb7], 0, [0xd842, 0xdfb7, 0xdfb7], [0xd842, 0xdfb7, 0xdfb7], 0);
checkIdentityPair(/^(?:\𠮷){2}$/d, [94, 40, 63, 58, 92, 0xd842, 0xdfb7, 41, 123, 50, 125, 36], 'd', [0xd842, 0xdfb7, 0xd842, 0xdfb7], 0, [0xd842, 0xdfb7, 0xd842, 0xdfb7], null);
checkIdentityPair(/(?<=(\𠮷{2}))x/d, [40, 63, 60, 61, 40, 92, 0xd842, 0xdfb7, 123, 50, 125, 41, 41, 120], 'd', [0xd842, 0xdfb7, 0xdfb7, 120], 3, [120], [0xd842, 0xdfb7, 0xdfb7], 0);

for (var f = 0; f < 2; f++) {
  var unicodeFlags = fromUnits([f === 0 ? 117 : 118]);
  var invalidPatterns = [[92, 233], [92, 0xd842, 0xdfb7, 63]];
  for (var p = 0; p < invalidPatterns.length; p++) {
    var syntax = false;
    try {
      new RegExp(fromUnits(invalidPatterns[p]), unicodeFlags);
    } catch (error) {
      syntax = error instanceof SyntaxError;
    }
    require(syntax, 'Unicode non-ASCII identity uses genuine SyntaxError');
  }
}

print('ok');
262;
