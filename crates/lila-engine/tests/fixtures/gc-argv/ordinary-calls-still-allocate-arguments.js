// Regression for the former eager Arguments-object allocation on every call.
// Deferred native-GC frames now make this four-million-call case pass.
// Parameter environments still allocate in linear memory; this is not a
// demonstration of semantic-object reclamation or unbounded execution.
function add(left, right) { return left + right; }
var result = 0;
for (var index = 0; index < 4000000; index++) result = add(result, 1);
print(result);
