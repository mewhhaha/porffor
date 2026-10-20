// Scriptw regression probe for its pinned IANA2026a transition provider.
// This is not an upstream Test262 test or a primary-identifier equivalence rule.
// Keep data-sharing checks separate from the amended portable test's scope.
const epochs = [0n];
for (const year of [1900, 1950, 1960, 1970, 1980, 1990, 2000, 2010, 2020, 2030]) {
  epochs.push(new Temporal.PlainDate(year, 1, 1).toZonedDateTime("UTC").epochNanoseconds);
}
for (const epoch of epochs) {
  const johnston = new Temporal.ZonedDateTime(epoch, "Pacific/Johnston");
  const honolulu = new Temporal.ZonedDateTime(epoch, "Pacific/Honolulu");
  if (johnston.timeZoneId !== "Pacific/Johnston" || honolulu.timeZoneId !== "Pacific/Honolulu") {
    throw new Error("The provider lost a requested named identity");
  }
  if (johnston.offsetNanoseconds !== honolulu.offsetNanoseconds) {
    throw new Error("Pinned shared transition data disagrees at " + epoch);
  }
}
print("Johnston/Honolulu: " + epochs.length + " matching offset samples; requested names preserved");
