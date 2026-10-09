use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_dtf_numbering(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let observation = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("DateTimeFormat numbering regression must use Wasm AOT");
    assert!(
        matches!(observation.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observation.completion,
    );
    assert_eq!(
        observation.output_events,
        vec![HostOutputEvent::PrintLine("true".to_string())],
        "{source}",
    );
}

#[test]
fn numeric_fields_padding_and_fractional_separator_follow_selected_numbering() {
    assert_dtf_numbering(
        r#"
var rows = [
  ['latn', '02:35:06.789'],
  ['arab', '٠٢:٣٥:٠٦٫٧٨٩'],
  ['deva', '०२:३५:०६.७८९'],
  ['hanidec', '〇二:三五:〇六.七八九'],
  ['mathbold', '𝟎𝟐:𝟑𝟓:𝟎𝟔.𝟕𝟖𝟗']
];
for (var row of rows) {
  var formatter = new Intl.DateTimeFormat('en-US-u-nu-' + row[0], {
    timeZone:'UTC', hourCycle:'h23', hour:'2-digit', minute:'2-digit',
    second:'2-digit', fractionalSecondDigits:3
  });
  var result = formatter.format(1704076506789);
  if (formatter.resolvedOptions().numberingSystem !== row[0]) throw 'resolved numbering system: ' + row[0];
  if (result !== row[1]) throw 'numeric fields: ' + row[0] + ' ' + result;
}
print(true);
"#,
    );
}

#[test]
fn numbering_options_and_keywords_share_the_supported_rendering_domain() {
    assert_dtf_numbering(
        r#"
var retained = new Intl.DateTimeFormat('en-US-u-nu-arab', {numberingSystem:'unlisted'}).resolvedOptions();
if (retained.numberingSystem !== 'arab' || retained.locale !== 'en-US-u-nu-arab') throw 'unsupported option replaced supported keyword';
var overridden = new Intl.DateTimeFormat('en-US-u-nu-arab', {numberingSystem:'deva'}).resolvedOptions();
if (overridden.numberingSystem !== 'deva' || overridden.locale !== 'en-US') throw 'option override';
var same = new Intl.DateTimeFormat('en-US-u-nu-arab', {numberingSystem:'ARAB'}).resolvedOptions();
if (same.numberingSystem !== 'arab' || same.locale !== 'en-US-u-nu-arab') throw 'case canonicalization';
var fallback = new Intl.DateTimeFormat('en-US-u-nu-roman').resolvedOptions();
if (fallback.numberingSystem !== 'latn' || fallback.locale !== 'en-US') throw 'algorithmic system advertised';
print(true);
"#,
    );
}

#[test]
fn numbered_parts_and_ranges_preserve_join_and_source_attribution() {
    assert_dtf_numbering(
        r#"
var formatter = new Intl.DateTimeFormat('en-US', {timeZone:'UTC', numberingSystem:'arab', month:'short', day:'numeric', year:'numeric'});
var start = 1704067200000;
var end = start + 86400000;
var parts = formatter.formatToParts(start);
var joined = '';
for (var part of parts) {
  joined += part.value;
  if (part.type === 'day' && part.value !== '١') throw 'day part';
  if (part.type === 'year' && part.value !== '٢٠٢٤') throw 'year part';
}
if (joined !== formatter.format(start)) throw 'parts join';
var rangeParts = formatter.formatRangeToParts(start, end);
joined = '';
var sawStart = false;
var sawEnd = false;
for (var part of rangeParts) {
  joined += part.value;
  if (part.type === 'day' && part.source === 'startRange') { if (part.value !== '١') throw 'range start day'; sawStart = true; }
  if (part.type === 'day' && part.source === 'endRange') { if (part.value !== '٢') throw 'range end day'; sawEnd = true; }
}
if (!sawStart || !sawEnd || joined !== formatter.formatRange(start,end)) throw 'range parts join and attribution';
print(true);
"#,
    );
}

#[test]
fn every_pinned_positional_system_renders_all_ten_digits() {
    assert_dtf_numbering(
        r#"
var rows = [
  ["adlm", ["𞥐", "𞥑", "𞥒", "𞥓", "𞥔", "𞥕", "𞥖", "𞥗", "𞥘", "𞥙"]],
  ["ahom", ["𑜰", "𑜱", "𑜲", "𑜳", "𑜴", "𑜵", "𑜶", "𑜷", "𑜸", "𑜹"]],
  ["arab", ["٠", "١", "٢", "٣", "٤", "٥", "٦", "٧", "٨", "٩"]],
  ["arabext", ["۰", "۱", "۲", "۳", "۴", "۵", "۶", "۷", "۸", "۹"]],
  ["bali", ["᭐", "᭑", "᭒", "᭓", "᭔", "᭕", "᭖", "᭗", "᭘", "᭙"]],
  ["beng", ["০", "১", "২", "৩", "৪", "৫", "৬", "৭", "৮", "৯"]],
  ["bhks", ["𑱐", "𑱑", "𑱒", "𑱓", "𑱔", "𑱕", "𑱖", "𑱗", "𑱘", "𑱙"]],
  ["brah", ["𑁦", "𑁧", "𑁨", "𑁩", "𑁪", "𑁫", "𑁬", "𑁭", "𑁮", "𑁯"]],
  ["cakm", ["𑄶", "𑄷", "𑄸", "𑄹", "𑄺", "𑄻", "𑄼", "𑄽", "𑄾", "𑄿"]],
  ["cham", ["꩐", "꩑", "꩒", "꩓", "꩔", "꩕", "꩖", "꩗", "꩘", "꩙"]],
  ["deva", ["०", "१", "२", "३", "४", "५", "६", "७", "८", "९"]],
  ["diak", ["𑥐", "𑥑", "𑥒", "𑥓", "𑥔", "𑥕", "𑥖", "𑥗", "𑥘", "𑥙"]],
  ["fullwide", ["０", "１", "２", "３", "４", "５", "６", "７", "８", "９"]],
  ["gara", ["𐵀", "𐵁", "𐵂", "𐵃", "𐵄", "𐵅", "𐵆", "𐵇", "𐵈", "𐵉"]],
  ["gong", ["𑶠", "𑶡", "𑶢", "𑶣", "𑶤", "𑶥", "𑶦", "𑶧", "𑶨", "𑶩"]],
  ["gonm", ["𑵐", "𑵑", "𑵒", "𑵓", "𑵔", "𑵕", "𑵖", "𑵗", "𑵘", "𑵙"]],
  ["gujr", ["૦", "૧", "૨", "૩", "૪", "૫", "૬", "૭", "૮", "૯"]],
  ["gukh", ["𖄰", "𖄱", "𖄲", "𖄳", "𖄴", "𖄵", "𖄶", "𖄷", "𖄸", "𖄹"]],
  ["guru", ["੦", "੧", "੨", "੩", "੪", "੫", "੬", "੭", "੮", "੯"]],
  ["hanidec", ["〇", "一", "二", "三", "四", "五", "六", "七", "八", "九"]],
  ["hmng", ["𖭐", "𖭑", "𖭒", "𖭓", "𖭔", "𖭕", "𖭖", "𖭗", "𖭘", "𖭙"]],
  ["hmnp", ["𞅀", "𞅁", "𞅂", "𞅃", "𞅄", "𞅅", "𞅆", "𞅇", "𞅈", "𞅉"]],
  ["java", ["꧐", "꧑", "꧒", "꧓", "꧔", "꧕", "꧖", "꧗", "꧘", "꧙"]],
  ["kali", ["꤀", "꤁", "꤂", "꤃", "꤄", "꤅", "꤆", "꤇", "꤈", "꤉"]],
  ["kawi", ["𑽐", "𑽑", "𑽒", "𑽓", "𑽔", "𑽕", "𑽖", "𑽗", "𑽘", "𑽙"]],
  ["khmr", ["០", "១", "២", "៣", "៤", "៥", "៦", "៧", "៨", "៩"]],
  ["knda", ["೦", "೧", "೨", "೩", "೪", "೫", "೬", "೭", "೮", "೯"]],
  ["krai", ["𖵰", "𖵱", "𖵲", "𖵳", "𖵴", "𖵵", "𖵶", "𖵷", "𖵸", "𖵹"]],
  ["lana", ["᪀", "᪁", "᪂", "᪃", "᪄", "᪅", "᪆", "᪇", "᪈", "᪉"]],
  ["lanatham", ["᪐", "᪑", "᪒", "᪓", "᪔", "᪕", "᪖", "᪗", "᪘", "᪙"]],
  ["laoo", ["໐", "໑", "໒", "໓", "໔", "໕", "໖", "໗", "໘", "໙"]],
  ["latn", ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]],
  ["lepc", ["᱀", "᱁", "᱂", "᱃", "᱄", "᱅", "᱆", "᱇", "᱈", "᱉"]],
  ["limb", ["᥆", "᥇", "᥈", "᥉", "᥊", "᥋", "᥌", "᥍", "᥎", "᥏"]],
  ["mathbold", ["𝟎", "𝟏", "𝟐", "𝟑", "𝟒", "𝟓", "𝟔", "𝟕", "𝟖", "𝟗"]],
  ["mathdbl", ["𝟘", "𝟙", "𝟚", "𝟛", "𝟜", "𝟝", "𝟞", "𝟟", "𝟠", "𝟡"]],
  ["mathmono", ["𝟶", "𝟷", "𝟸", "𝟹", "𝟺", "𝟻", "𝟼", "𝟽", "𝟾", "𝟿"]],
  ["mathsanb", ["𝟬", "𝟭", "𝟮", "𝟯", "𝟰", "𝟱", "𝟲", "𝟳", "𝟴", "𝟵"]],
  ["mathsans", ["𝟢", "𝟣", "𝟤", "𝟥", "𝟦", "𝟧", "𝟨", "𝟩", "𝟪", "𝟫"]],
  ["mlym", ["൦", "൧", "൨", "൩", "൪", "൫", "൬", "൭", "൮", "൯"]],
  ["modi", ["𑙐", "𑙑", "𑙒", "𑙓", "𑙔", "𑙕", "𑙖", "𑙗", "𑙘", "𑙙"]],
  ["mong", ["᠐", "᠑", "᠒", "᠓", "᠔", "᠕", "᠖", "᠗", "᠘", "᠙"]],
  ["mroo", ["𖩠", "𖩡", "𖩢", "𖩣", "𖩤", "𖩥", "𖩦", "𖩧", "𖩨", "𖩩"]],
  ["mtei", ["꯰", "꯱", "꯲", "꯳", "꯴", "꯵", "꯶", "꯷", "꯸", "꯹"]],
  ["mymr", ["၀", "၁", "၂", "၃", "၄", "၅", "၆", "၇", "၈", "၉"]],
  ["mymrepka", ["𑛚", "𑛛", "𑛜", "𑛝", "𑛞", "𑛟", "𑛠", "𑛡", "𑛢", "𑛣"]],
  ["mymrpao", ["𑛐", "𑛑", "𑛒", "𑛓", "𑛔", "𑛕", "𑛖", "𑛗", "𑛘", "𑛙"]],
  ["mymrshan", ["႐", "႑", "႒", "႓", "႔", "႕", "႖", "႗", "႘", "႙"]],
  ["mymrtlng", ["꧰", "꧱", "꧲", "꧳", "꧴", "꧵", "꧶", "꧷", "꧸", "꧹"]],
  ["nagm", ["𞓰", "𞓱", "𞓲", "𞓳", "𞓴", "𞓵", "𞓶", "𞓷", "𞓸", "𞓹"]],
  ["newa", ["𑑐", "𑑑", "𑑒", "𑑓", "𑑔", "𑑕", "𑑖", "𑑗", "𑑘", "𑑙"]],
  ["nkoo", ["߀", "߁", "߂", "߃", "߄", "߅", "߆", "߇", "߈", "߉"]],
  ["olck", ["᱐", "᱑", "᱒", "᱓", "᱔", "᱕", "᱖", "᱗", "᱘", "᱙"]],
  ["onao", ["𞗱", "𞗲", "𞗳", "𞗴", "𞗵", "𞗶", "𞗷", "𞗸", "𞗹", "𞗺"]],
  ["orya", ["୦", "୧", "୨", "୩", "୪", "୫", "୬", "୭", "୮", "୯"]],
  ["osma", ["𐒠", "𐒡", "𐒢", "𐒣", "𐒤", "𐒥", "𐒦", "𐒧", "𐒨", "𐒩"]],
  ["outlined", ["𜳰", "𜳱", "𜳲", "𜳳", "𜳴", "𜳵", "𜳶", "𜳷", "𜳸", "𜳹"]],
  ["rohg", ["𐴰", "𐴱", "𐴲", "𐴳", "𐴴", "𐴵", "𐴶", "𐴷", "𐴸", "𐴹"]],
  ["saur", ["꣐", "꣑", "꣒", "꣓", "꣔", "꣕", "꣖", "꣗", "꣘", "꣙"]],
  ["segment", ["🯰", "🯱", "🯲", "🯳", "🯴", "🯵", "🯶", "🯷", "🯸", "🯹"]],
  ["shrd", ["𑇐", "𑇑", "𑇒", "𑇓", "𑇔", "𑇕", "𑇖", "𑇗", "𑇘", "𑇙"]],
  ["sind", ["𑋰", "𑋱", "𑋲", "𑋳", "𑋴", "𑋵", "𑋶", "𑋷", "𑋸", "𑋹"]],
  ["sinh", ["෦", "෧", "෨", "෩", "෪", "෫", "෬", "෭", "෮", "෯"]],
  ["sora", ["𑃰", "𑃱", "𑃲", "𑃳", "𑃴", "𑃵", "𑃶", "𑃷", "𑃸", "𑃹"]],
  ["sund", ["᮰", "᮱", "᮲", "᮳", "᮴", "᮵", "᮶", "᮷", "᮸", "᮹"]],
  ["sunu", ["𑯰", "𑯱", "𑯲", "𑯳", "𑯴", "𑯵", "𑯶", "𑯷", "𑯸", "𑯹"]],
  ["takr", ["𑛀", "𑛁", "𑛂", "𑛃", "𑛄", "𑛅", "𑛆", "𑛇", "𑛈", "𑛉"]],
  ["talu", ["᧐", "᧑", "᧒", "᧓", "᧔", "᧕", "᧖", "᧗", "᧘", "᧙"]],
  ["tamldec", ["௦", "௧", "௨", "௩", "௪", "௫", "௬", "௭", "௮", "௯"]],
  ["tnsa", ["𖫀", "𖫁", "𖫂", "𖫃", "𖫄", "𖫅", "𖫆", "𖫇", "𖫈", "𖫉"]],
  ["telu", ["౦", "౧", "౨", "౩", "౪", "౫", "౬", "౭", "౮", "౯"]],
  ["thai", ["๐", "๑", "๒", "๓", "๔", "๕", "๖", "๗", "๘", "๙"]],
  ["tibt", ["༠", "༡", "༢", "༣", "༤", "༥", "༦", "༧", "༨", "༩"]],
  ["tirh", ["𑓐", "𑓑", "𑓒", "𑓓", "𑓔", "𑓕", "𑓖", "𑓗", "𑓘", "𑓙"]],
  ["vaii", ["꘠", "꘡", "꘢", "꘣", "꘤", "꘥", "꘦", "꘧", "꘨", "꘩"]],
  ["wara", ["𑣠", "𑣡", "𑣢", "𑣣", "𑣤", "𑣥", "𑣦", "𑣧", "𑣨", "𑣩"]],
  ["wcho", ["𞋰", "𞋱", "𞋲", "𞋳", "𞋴", "𞋵", "𞋶", "𞋷", "𞋸", "𞋹"]]
];
for (var row of rows) {
  var formatter = new Intl.DateTimeFormat('en-US', {timeZone:'UTC', numberingSystem:row[0], second:'numeric'});
  if (formatter.resolvedOptions().numberingSystem !== row[0]) throw 'unsupported positional system: ' + row[0];
  for (var second = 0; second < 10; second++) {
    var actual = formatter.format(second * 1000);
    if (actual !== row[1][second]) throw 'digit ' + second + ' in ' + row[0] + ': ' + actual;
  }
}
print(true);
"#,
    );
}

#[test]
fn calendar_fields_and_gmt_labels_localize_without_changing_zone_identifiers() {
    assert_dtf_numbering(
        r#"

var formatter = new Intl.DateTimeFormat('en-US', {
  timeZone:'+05:45', numberingSystem:'arab', hourCycle:'h23',
  year:'2-digit', month:'2-digit', day:'2-digit', hour:'2-digit',
  minute:'2-digit', second:'2-digit', timeZoneName:'longOffset'
});
var resolved = formatter.resolvedOptions();
if (resolved.timeZone !== '+05:45' || resolved.numberingSystem !== 'arab') throw 'protocol slots';
var expected = {year:'٢٤', month:'٠١', day:'٠٢', hour:'٠١', minute:'٠٠', second:'٠٦', timeZoneName:'GMT+٠٥:٤٥'};
var parts = formatter.formatToParts(1704136506000);
var joined = '';
var fields = 0;
for (var part of parts) {
  joined += part.value;
  if (part.type !== 'literal') {
    if (part.value !== expected[part.type]) throw 'field ' + part.type + ': ' + part.value;
    fields++;
  }
}
if (fields !== 7 || joined !== formatter.format(1704136506000)) throw 'full field/parts agreement';
var textual = new Intl.DateTimeFormat('en-US', {timeZone:'UTC', numberingSystem:'hanidec', year:'numeric',month:'short',day:'numeric'});
if (textual.format(1704067200000) !== 'Jan 一, 二〇二四') throw 'text month with numbered date fields';
print(true);
"#,
    );
}

#[test]
fn fractional_parts_and_extended_temporal_dates_share_numbering() {
    assert_dtf_numbering(
        r#"

for (var width = 1; width <= 3; width++) {
  var formatter = new Intl.DateTimeFormat('en-US', {timeZone:'UTC',numberingSystem:'arabext',second:'numeric',fractionalSecondDigits:width});
  var expected = ['۷','۷۸','۷۸۹'][width - 1];
  var parts = formatter.formatToParts(6789);
  var joined = '';
  var found = false;
  var separator = false;
  for (var part of parts) {
    joined += part.value;
    if (part.type === 'fractionalSecond') { if (part.value !== expected) throw 'fraction truncation'; found = true; }
    if (part.type === 'literal' && part.value === '٫') separator = true;
  }
  if (!found || !separator || joined !== formatter.format(6789)) throw 'fractional part structure';
  var range = formatter.formatRangeToParts(6789,8791);
  joined = '';
  var startFraction = false;
  var endFraction = false;
  for (var part of range) {
    joined += part.value;
    if (part.type === 'fractionalSecond' && part.source === 'startRange') startFraction = true;
    if (part.type === 'fractionalSecond' && part.source === 'endRange') endFraction = true;
    if (part.type === 'literal' && part.value === '٫' && part.source === 'shared') throw 'fraction separator attribution';
  }
  if (!startFraction || !endFraction || joined !== formatter.formatRange(6789,8791)) throw 'fraction range agreement';
}
var date = new Intl.DateTimeFormat('en-US-u-nu-arab', {year:'numeric', month:'numeric', day:'numeric', calendar:'iso8601', timeZone:'UTC'});
var maximum = date.format(new Temporal.PlainDate(275760,9,13));
var minimum = date.format(new Temporal.PlainDate(-271821,4,19));
if (!maximum.includes('١٣') || !maximum.includes('٩') || !minimum.includes('١٩') || !minimum.includes('٤')) throw 'extended Temporal dates';
var time = new Intl.DateTimeFormat('en-US', {numberingSystem:'deva',hourCycle:'h23',hour:'2-digit',minute:'2-digit',second:'2-digit'});
if (time.format(new Temporal.PlainTime(2,35,6)) !== '०२:३५:०६') throw 'Temporal wall time';
print(true);
"#,
    );
}
