function check(condition, label) { if (!condition) throw label; }
var supported = Intl.DateTimeFormat.supportedLocalesOf(['fr', 'zh-Hans-CN', 'ar-EG', 'en-US', 'ja']);
check(supported.join(',') === 'zh-Hans-CN,ar-EG,en-US', 'datetime-public-rows-and-parent-lookup');
check(new Intl.DateTimeFormat('fr').resolvedOptions().locale === 'en-US', 'excluded-datetime-row-falls-back');
check(new Intl.DateTimeFormat('zh-Hans-CN').resolvedOptions().locale === 'zh', 'selected-datetime-parent-row');
check(new Intl.DateTimeFormat('ar-EG').resolvedOptions().numberingSystem === 'arab', 'selected-datetime-row-default-digits');

var fields = {timeZone: 'UTC', year: 'numeric', month: 'numeric', day: 'numeric', formatMatcher: 'basic'};
var arabic = new Intl.DateTimeFormat('ar-EG', fields);
var january = Date.UTC(2020, 0, 25);
check(arabic.format(january) === '٢٥\u200f/١\u200f/٢٠٢٠', 'selected-arabic-pattern-and-digits');
check(arabic.formatToParts(january).map(function (part) { return part.value; }).join('') === arabic.format(january), 'selected-datetime-parts');
var chinese = new Intl.DateTimeFormat('zh-u-ca-chinese', {timeZone: 'UTC', dateStyle: 'full'});
check(chinese.format(Date.UTC(2024, 1, 10)) === '2024甲辰年正月初一星期六', 'selected-chinese-calendar-fields');
check(chinese.formatToParts(Date.UTC(2024, 1, 10)).some(function (part) { return part.type === 'yearName' && part.value === '甲辰'; }), 'real-selected-related-year-name');

var intervals = new Intl.DateTimeFormat('ar-EG', {timeZone: 'UTC', year: 'numeric', month: 'short', day: 'numeric', formatMatcher: 'basic'});
var begin = Date.UTC(2020, 0, 10), end = Date.UTC(2020, 0, 12);
check(intervals.formatRange(begin, end) === '١٠–١٢ يناير ٢٠٢٠', 'selected-calendar-interval-pool');
var range = intervals.formatRangeToParts(begin, end);
check(range.map(function (part) { return part.value; }).join('') === intervals.formatRange(begin, end), 'selected-range-parts');
check(range.some(function (part) { return part.source === 'startRange'; }) && range.some(function (part) { return part.source === 'endRange'; }), 'selected-range-endpoint-identity');

var zone = new Intl.DateTimeFormat('ar-EG', {timeZone: 'America/New_York', hour: 'numeric', minute: 'numeric', second: 'numeric', hourCycle: 'h23', timeZoneName: 'longOffset'});
var before = zone.formatToParts(1710053999000), after = zone.formatToParts(1710054000000);
check(before.find(function (part) { return part.type === 'hour'; }).value === '٠١' && after.find(function (part) { return part.type === 'hour'; }).value === '٠٣', 'selected-original-iana-transition');
check(before.find(function (part) { return part.type === 'timeZoneName'; }).value === 'غرينتش-٠٥:٠٠' && after.find(function (part) { return part.type === 'timeZoneName'; }).value === 'غرينتش-٠٤:٠٠', 'selected-zone-name-and-offset-pool');

var calendars = Intl.supportedValuesOf('calendar');
check(calendars.length === 16, 'complete-selected-calendar-kernel-domain');
for (var calendar of calendars) {
  var formatter = new Intl.DateTimeFormat('en-US', {timeZone: 'UTC', calendar: calendar, year: 'numeric', month: 'numeric', day: 'numeric'});
  check(formatter.resolvedOptions().calendar === calendar && formatter.format(january).length > 0, 'complete-selected-calendar-association-' + calendar);
}
check(Intl.supportedValuesOf('numberingSystem').length === 78, 'complete-numbering-authority-outside-date-filter');
check(new Intl.Locale('ja-u-ca-japanese').getCalendars()[0] === 'japanese', 'locale-calendar-owner-outside-public-date-domain');
var instant = Temporal.Instant.fromEpochNanoseconds(BigInt(january) * 1000000n);
check(instant.toLocaleString('ar-EG', fields) === arabic.format(january), 'temporal-intrinsic-selected-datetime-owner');
print('intl-datetime-projection:ok');
