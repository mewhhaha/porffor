// Temporal availability is recorded explicitly for comparison engines.
if (typeof Temporal === "undefined") {
  print(JSON.stringify({ temporalAvailable: false }));
} else {
  var johnston = new Temporal.ZonedDateTime(0n, "Pacific/Johnston");
  var honolulu = new Temporal.ZonedDateTime(0n, "Pacific/Honolulu");
  print(JSON.stringify({
    temporalAvailable: true,
    identifiers: [johnston.timeZoneId, honolulu.timeZoneId],
    equal: johnston.equals(honolulu),
    sameOffset: johnston.offsetNanoseconds === honolulu.offsetNanoseconds
  }));
}
