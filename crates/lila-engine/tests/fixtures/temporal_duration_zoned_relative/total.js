function same(actual, expected) { if (!Object.is(actual, expected)) throw new Error('total '+actual+' != '+expected); }
var day=Temporal.Duration.from({days:1});
same(day.total({unit:'hours',relativeTo:'2024-03-09T12:00[America/New_York]'}),23);
same(day.total({unit:'hours',relativeTo:'2024-11-02T12:00[America/New_York]'}),25);
same(day.total({unit:'hours',relativeTo:'2020-10-03T12:00[Australia/Lord_Howe]'}),23.5);
same(Temporal.Duration.from({days:-1}).total({unit:'hours',relativeTo:'2024-03-10T12:00[America/New_York]'}),-23);
var value=Temporal.Duration.from({days:32}).total({unit:'months',relativeTo:'2024-02-01T12:00[America/New_York]'});
if (Math.abs(value-(1+72/743))>1e-14) throw new Error('elapsed month window '+value);
same(Temporal.Duration.from({days:1}).total({unit:'days',relativeTo:'2024-03-09T12:00[America/New_York]'}),1);
same(Temporal.Duration.from({days:1}).total({unit:'hours',relativeTo:'2024-03-09'}),24);
same(Temporal.Duration.from({hours:25}).total('days'),25/24);
262;
