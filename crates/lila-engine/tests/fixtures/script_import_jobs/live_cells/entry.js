globalThis.liveEvents = [];
import('./first.js').then(first => {
  if (first.value !== 'first' || liveEvents.join(',') !== 'first') throw 'nested target eager';
  print('first:' + first.value);
  return first.loadOther().then(other => {
    if (first.value !== 'live' || other.default !== 42 ||
        liveEvents.join(',') !== 'first,dependency,other') throw 'transitive live update';
    print('other:' + other.default + ':' + first.value);
  });
});
print('Script body');
true;
