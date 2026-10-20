function check(value, message) { if (!value) throw new Error(message); }
function throws(kind, fn, message) { var caught; try { fn(); } catch (error) { caught = error; } check(caught instanceof kind, message); }
var formatter = new Intl.DurationFormat('en',{fractionalDigits:3}), bag = {years:1,days:2};
var first = formatter.formatToParts(bag), second = formatter.formatToParts(bag);
check(first !== second && first[0] !== second[0], 'fresh arrays and part objects');
first[0].value = 'changed'; first.push({type:'literal',value:'changed'});
check(second.map(p => p.value).join('') === formatter.format(bag), 'mutations cannot change native owner');
var keys = ['locale','numberingSystem','style'];
for (var unit of ['years','months','weeks','days','hours','minutes','seconds','milliseconds','microseconds','nanoseconds']) { keys.push(unit); keys.push(unit+'Display'); }
keys.push('fractionalDigits');
check(Object.keys(formatter.resolvedOptions()).join(',') === keys.join(','), 'resolved property order');
var resolved = formatter.resolvedOptions(); resolved.style = 'changed';
check(formatter.resolvedOptions().style === 'short', 'fresh resolved object');
check(!Object.prototype.hasOwnProperty.call(new Intl.DurationFormat('en').resolvedOptions(),'fractionalDigits'), 'absent precision stays absent');
for (var name of ['format','formatToParts','resolvedOptions']) {
  var descriptor = Object.getOwnPropertyDescriptor(Intl.DurationFormat.prototype,name);
  check(descriptor.writable && !descriptor.enumerable && descriptor.configurable && typeof descriptor.value === 'function', 'normal method descriptor');
  check(!Object.prototype.hasOwnProperty.call(descriptor.value,'prototype'), 'methods lack Construct');
}
check(Intl.DurationFormat.prototype.format.length === 1 && Intl.DurationFormat.prototype.formatToParts.length === 1 && Intl.DurationFormat.prototype.resolvedOptions.length === 0, 'method lengths');
check(Object.prototype.toString.call(formatter) === '[object Intl.DurationFormat]', 'genuine tag');
for (var key of Object.keys(second[0])) { var d = Object.getOwnPropertyDescriptor(second[0],key); check(d.writable && d.enumerable && d.configurable, 'CreateDataProperty attributes'); }
print('ok fresh_results_and_descriptors'); 262;
