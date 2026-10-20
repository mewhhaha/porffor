function capture() {
    var token = {}, symbol = Symbol('live');
    token.self = token;
    var values = [token, symbol, 7n];
    return function () {
        return values[0] === token && token.self === token &&
            values[1] === symbol && values[2] === 7n;
    };
}
var retained = capture();
if (gc() !== undefined) throw new Error('collector completion');
if (!retained()) throw new Error('collector lost a captured whole value');
function throwOnly() {
    var token = {marker: 262};
    token.self = token;
    throw token;
}
var caughtSurvived = false;
try { throwOnly(); } catch (error) {
    gc();
    caughtSurvived = error.self === error && error.marker === 262 && retained();
}
if (!caughtSurvived) throw new Error('collector lost a caught abrupt value');
true;
