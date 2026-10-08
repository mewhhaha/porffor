function same(actual, expected) { if (!Object.is(actual, expected)) throw new Error('compare '+actual+' != '+expected); }
var spring=Temporal.ZonedDateTime.from('2024-03-09T12:00[America/New_York]');
var fall=Temporal.ZonedDateTime.from('2024-11-02T12:00[America/New_York]');
same(Temporal.Duration.compare({days:1},{hours:24},{relativeTo:spring}),-1);
same(Temporal.Duration.compare({days:1},{hours:24},{relativeTo:fall}),1);
same(Temporal.Duration.compare({days:1},{hours:23,minutes:30},{relativeTo:'2020-10-03T12:00[Australia/Lord_Howe]'}),0);
same(Temporal.Duration.compare({days:-1},{hours:-24},{relativeTo:'2024-03-10T12:00[America/New_York]'}),1);
// Date-free comparisons do not require AddZonedDateTime or a far boundary.
same(Temporal.Duration.compare({hours:1},{minutes:60},{relativeTo:new Temporal.ZonedDateTime(8640000000000000000000n,'UTC')}),0);
same(Temporal.Duration.compare({days:1},{days:1},{relativeTo:new Temporal.ZonedDateTime(8640000000000000000000n,'UTC')}),0);
262;
