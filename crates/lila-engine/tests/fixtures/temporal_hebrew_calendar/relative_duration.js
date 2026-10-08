function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
const calendar = 'hebrew';
const plain = Temporal.PlainDate.from({calendar,year:5784,monthCode:'M01',day:1});
const time = plain.toPlainDateTime({hour:12});
const fixed = Temporal.ZonedDateTime.from({calendar,year:5784,monthCode:'M01',day:1,hour:12,timeZone:'+05:30'});
const named = Temporal.ZonedDateTime.from({calendar,year:5784,monthCode:'M01',day:1,hour:12,timeZone:'America/New_York'});
const year = Temporal.Duration.from({years:1});
const twelve = Temporal.Duration.from({months:12});
const thirteen = Temporal.Duration.from({months:13});
const month = Temporal.Duration.from({months:1});
for (const relativeTo of [plain,time,fixed,named,plain.toString(),named.toString(),{calendar,year:5784,monthCode:'M01',day:1}]) {
  same(year.total({unit:'days',relativeTo}),383,'actual deficient leap year');
  same(year.total({unit:'months',relativeTo}),13,'year crosses thirteen actual months');
  same(thirteen.total({unit:'years',relativeTo}),1,'thirteen-month exact year');
  same(Temporal.Duration.compare(twelve,year,{relativeTo}),-1,'twelve months below this year');
  same(Temporal.Duration.compare(thirteen,year,{relativeTo}),0,'actual year equality');
  same(month.total({unit:'days',relativeTo}),30,'actual M01 length');
  same(Temporal.Duration.from({days:383}).round({largestUnit:'years',smallestUnit:'years',relativeTo}).years,1,'actual year anchor round');
  same(Temporal.Duration.from({days:30}).round({largestUnit:'months',smallestUnit:'months',relativeTo}).months,1,'actual month anchor round');
}
const next = plain.add({years:1});
same(year.total({unit:'days',relativeTo:next}),355,'following complete common year');
same(year.total({unit:'months',relativeTo:next}),12,'following twelve-month year');
same(Temporal.Duration.from({months:-13}).total({unit:'years',relativeTo:next}),-1,'negative actual leap year');
same(Temporal.Duration.from({months:-13}).total({unit:'days',relativeTo:next}),-383,'negative year day count');
same(month.total({unit:'days',relativeTo:plain.add({months:1})}),29,'short M02 in deficient year');
same(month.total({unit:'days',relativeTo:next.add({months:1})}),30,'long M02 in complete year');
const adar = Temporal.PlainDate.from({calendar,year:5784,monthCode:'M05L',day:1});
same(month.total({unit:'days',relativeTo:adar}),30,'inserted month span');
same(month.total({unit:'days',relativeTo:adar.add({months:1})}),29,'regular Adar span');
same(Temporal.Duration.from({days:382}).round({largestUnit:'years',smallestUnit:'years',roundingMode:'floor',relativeTo:plain}).years,0,'lower real year bracket');
same(Temporal.Duration.from({days:382}).round({largestUnit:'years',smallestUnit:'years',roundingMode:'ceil',relativeTo:plain}).years,1,'upper real year bracket');
for (const relativeTo of [plain,time,fixed,named]) {
  for (const name of ['year','month','monthCode','day','calendarId','timeZoneId']) {
    Object.defineProperty(relativeTo,name,{get(){throw new Error('public relative field read');}});
  }
  same(thirteen.total({unit:'years',relativeTo}),1,'retained private calendar owner');
  same(Temporal.Duration.compare(thirteen,year,{relativeTo}),0,'retained private comparison');
}
let log = [];
same(month.total({get relativeTo(){log.push('relative');return plain;},get unit(){log.push('unit');return 'days';}}),30,'ordered relative total');
same(log.join('|'),'relative|unit','relative option order');
const marker = {};
try {
  month.round({get relativeTo(){throw marker;},get roundingIncrement(){log.push('late');return 1;}});
  throw new Error('missing relative throw');
} catch (caught) {if(caught!==marker)throw new Error('relative throw identity');}
same(log.join('|'),'relative|unit','no late options after abrupt');
print('hebrew-relative-duration:ok');
262;
