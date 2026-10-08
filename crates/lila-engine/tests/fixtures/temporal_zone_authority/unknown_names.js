function range(operation, label) {
  var caught = false;
  try { operation(); }
  catch (error) {
    if (!(error instanceof RangeError)) throw error;
    caught = true;
  }
  if (!caught) throw new Error(label);
}

// Both spellings satisfy the named-zone grammar and are absent from the
// catalogue. A slash does not establish that a time zone exists.
for (var zone of ['NoSuch/Zone', 'NoSuchZone']) {
  var source = '1970-01-01T12:00+01:00[' + zone + ']';
  range(function () { Temporal.ZonedDateTime.from(source); }, zone + ' default');
  for (var offset of ['reject', 'use', 'prefer', 'ignore']) {
    var optionReads = 0;
    var options = {
      get disambiguation() { optionReads++; return 'compatible'; },
      get offset() { optionReads++; return offset; },
      get overflow() { optionReads++; return 'constrain'; }
    };
    range(function () { Temporal.ZonedDateTime.from(source, options); }, zone + ' ' + offset);
    if (optionReads !== 0) throw new Error('unknown zone read options: ' + zone);
  }
  optionReads = 0;
  range(function () {
    Temporal.ZonedDateTime.from('1970-01-01T12:00Z[' + zone + ']', options);
  }, zone + ' with Z');
  if (optionReads !== 0) throw new Error('unknown Z zone read options: ' + zone);
  range(function () {
    Temporal.ZonedDateTime.from('1970-01-01T12:00+01:00[!' + zone + ']');
  }, zone + ' critical annotation');
  range(function () { new Temporal.ZonedDateTime(0n, zone); }, zone + ' constructor');
  range(function () {
    Temporal.ZonedDateTime.from({year:1970, month:1, day:1, hour:12,
      offset:'+01:00', timeZone:zone});
  }, zone + ' property bag');
  range(function () {
    new Temporal.Duration(0, 0, 0, 1).total({unit:'hour', relativeTo:source});
  }, zone + ' relativeTo');
}

print('ok');
262;
