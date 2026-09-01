var locale = new Intl.Locale("EN-lAtN-us-u-CA-iso8601");

if (locale.toString() !== "en-Latn-US-u-ca-iso8601") throw "canonical tag role";
if (locale.language !== "en") throw "language role";
if (locale.script !== "Latn") throw "script role";
if (locale.region !== "US") throw "region role";
if (locale.baseName !== "en-Latn-US") throw "baseName role";

var canonical = Intl.getCanonicalLocales(["EN-lAtN-us-u-CA-iso8601"]);
if (canonical.length !== 1) throw "canonical list length";
if (canonical[0] !== "en-Latn-US-u-ca-iso8601") throw "canonical list tag role";

var resolved = new Intl.DateTimeFormat("EN-us-u-ca-iso8601").resolvedOptions();
if (resolved.locale !== "en-US-u-ca-iso8601") throw "matched tag role";
if (resolved.calendar !== "iso8601") throw "extension role";

262
