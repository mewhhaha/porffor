use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::rc::Rc;

use boa_engine::builtins::promise::PromiseState;
use boa_engine::job::SimpleJobExecutor;
use boa_engine::module::{Module, ModuleLoader, Referrer};
use boa_engine::native_function::NativeFunction;
use boa_engine::object::builtins::JsArrayBuffer;
use boa_engine::{js_string, Context, JsNativeError, JsResult, JsString, JsValue, Source};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModuleHostConfig {
    pub module_root: Option<PathBuf>,
    pub test_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionOutcome {
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionError {
    message: String,
}

impl ExecutionError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl core::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ExecutionError {}

pub fn execute_script(
    source: &str,
    filename: Option<&str>,
    argv: &[String],
) -> Result<ExecutionOutcome, ExecutionError> {
    let mut context = Context::builder()
        .job_executor(Rc::new(SimpleJobExecutor::new()))
        .build()
        .map_err(|err| ExecutionError::new(err.to_string()))?;
    install_host_globals(&mut context, argv)?;
    context
        .eval(source_with_name(source, filename))
        .map_err(|err| format_js_error(err, &mut context))?;
    context
        .run_jobs()
        .map_err(|err| format_js_error(err, &mut context))?;
    Ok(ExecutionOutcome {
        note: "spec-exec script completed in Rust host".to_string(),
    })
}

pub fn execute_module(
    source: &str,
    filename: Option<&str>,
    host: ModuleHostConfig,
    argv: &[String],
) -> Result<ExecutionOutcome, ExecutionError> {
    let module_path = normalize_module_path(filename).or_else(|| host.test_path.clone());
    let loader = Rc::new(Test262ModuleLoader::new(
        host.module_root.as_deref(),
        module_path.as_deref(),
    ));
    let mut context = Context::builder()
        .job_executor(Rc::new(SimpleJobExecutor::new()))
        .module_loader(loader.clone())
        .build()
        .map_err(|err| ExecutionError::new(err.to_string()))?;
    install_host_globals(&mut context, argv)?;

    let module = Module::parse(source_with_name(source, filename), None, &mut context)
        .map_err(|err| format_js_error(err, &mut context))?;
    loader.insert(
        module_path.clone().unwrap_or_else(|| loader.entry_path()),
        module.clone(),
    );
    let promise = module.load_link_evaluate(&mut context);
    context
        .run_jobs()
        .map_err(|err| format_js_error(err, &mut context))?;

    match promise.state() {
        PromiseState::Fulfilled(_) => Ok(ExecutionOutcome {
            note: format!(
                "spec-exec module completed in Rust host{}",
                module_path
                    .as_deref()
                    .map(|path| format!(" ({})", path.display()))
                    .unwrap_or_default()
            ),
        }),
        PromiseState::Rejected(err) => Err(format_opaque_error(err, &mut context)),
        PromiseState::Pending => Err(ExecutionError::new(
            "runtime module jobs are still pending after host job flush",
        )),
    }
}

#[derive(Debug, Default)]
struct Test262ModuleLoader {
    root: PathBuf,
    entry_path: PathBuf,
    module_map: RefCell<BTreeMap<PathBuf, Module>>,
}

impl Test262ModuleLoader {
    fn new(module_root: Option<&Path>, test_path: Option<&Path>) -> Self {
        let entry_path = test_path
            .map(normalize_absolute_path)
            .unwrap_or_else(|| normalize_absolute_path(&PathBuf::from("main.mjs")));
        let root = module_root
            .map(normalize_absolute_path)
            .or_else(|| entry_path.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| normalize_absolute_path(&PathBuf::from(".")));
        Self {
            root,
            entry_path,
            module_map: RefCell::new(BTreeMap::new()),
        }
    }

    fn entry_path(&self) -> PathBuf {
        self.entry_path.clone()
    }

    fn insert(&self, path: PathBuf, module: Module) {
        self.module_map.borrow_mut().insert(path, module);
    }

    fn get(&self, path: &Path) -> Option<Module> {
        self.module_map.borrow().get(path).cloned()
    }

    fn resolve_path(&self, referrer: Referrer, specifier: &JsString) -> JsResult<PathBuf> {
        let specifier = specifier.to_std_string_escaped();
        let specifier_path = Path::new(&specifier);
        if specifier_path.is_absolute() {
            return Ok(normalize_absolute_path(specifier_path));
        }

        let base_dir = if specifier.starts_with("./")
            || specifier.starts_with("../")
            || specifier == "."
            || specifier == ".."
        {
            referrer
                .path()
                .and_then(Path::parent)
                .map(Path::to_path_buf)
                .unwrap_or_else(|| {
                    self.entry_path
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| self.root.clone())
                })
        } else {
            self.root.clone()
        };

        Ok(normalize_absolute_path(&base_dir.join(specifier_path)))
    }
}

impl ModuleLoader for Test262ModuleLoader {
    async fn load_imported_module(
        self: Rc<Self>,
        referrer: Referrer,
        specifier: JsString,
        context: &RefCell<&mut Context>,
    ) -> JsResult<Module> {
        let short_path = specifier.to_std_string_escaped();
        let path = self.resolve_path(referrer, &specifier)?;
        if let Some(module) = self.get(&path) {
            return Ok(module);
        }

        let source = Source::from_filepath(&path).map_err(|err| {
            JsNativeError::typ()
                .with_message(format!("could not open file `{short_path}`"))
                .with_cause(boa_engine::JsError::from_opaque(
                    js_string!(err.to_string()).into(),
                ))
        })?;
        let module = Module::parse(source, None, &mut context.borrow_mut()).map_err(|err| {
            JsNativeError::syntax()
                .with_message(format!("could not parse module `{short_path}`"))
                .with_cause(err)
        })?;
        self.insert(path, module.clone());
        Ok(module)
    }
}

fn source_with_name<'a>(
    source_text: &'a str,
    filename: Option<&'a str>,
) -> Source<'a, boa_engine::parser::source::UTF8Input<&'a [u8]>> {
    let source = Source::from_bytes(source_text);
    match filename {
        Some(path) => source.with_path(Path::new(path)),
        None => source,
    }
}

fn normalize_module_path(filename: Option<&str>) -> Option<PathBuf> {
    filename
        .map(PathBuf::from)
        .map(|path| normalize_absolute_path(&path))
}

fn normalize_absolute_path(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };

    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

const DATE_TIME_FORMAT_SHIM: &str = r#"
(function () {
  if (typeof Intl !== "object" || typeof Intl.DateTimeFormat !== "function") {
    return;
  }

  const NativeDateTimeFormat = Intl.DateTimeFormat;
  const nativePrototype = NativeDateTimeFormat.prototype;
  if (typeof nativePrototype.resolvedOptions === "function") {
    return;
  }

  const stateKey = "__porfDateTimeFormatState";
  const boundFormatKey = "__porfBoundFormat";
  const legacyConstructedSymbol = Symbol("IntlLegacyConstructedSymbol");
  const validStyles = new Set(["full", "long", "medium", "short"]);
  const validDayPeriods = new Set(["narrow", "short", "long"]);
  const validTimeZoneNames = new Set([
    "short",
    "long",
    "shortOffset",
    "longOffset",
    "shortGeneric",
    "longGeneric"
  ]);
  const validCalendars = new Set([
    "buddhist",
    "chinese",
    "coptic",
    "dangi",
    "ethioaa",
    "ethiopic",
    "gregory",
    "hebrew",
    "indian",
    "islamic-civil",
    "islamic-tbla",
    "islamic-umalqura",
    "iso8601",
    "japanese",
    "persian",
    "roc"
  ]);
  const validNumberingSystems = new Set(["arab", "latn"]);
  const validHourCycles = new Set(["h11", "h12", "h23", "h24"]);
  const explicitComponents = [
    "weekday",
    "era",
    "year",
    "month",
    "day",
    "dayPeriod",
    "hour",
    "minute",
    "second",
    "fractionalSecondDigits",
    "timeZoneName"
  ];
  const optionOrder = [
    "localeMatcher",
    "calendar",
    "numberingSystem",
    "hour12",
    "hourCycle",
    "timeZone",
    "weekday",
    "era",
    "year",
    "month",
    "day",
    "dayPeriod",
    "hour",
    "minute",
    "second",
    "fractionalSecondDigits",
    "timeZoneName",
    "formatMatcher",
    "dateStyle",
    "timeStyle"
  ];

  function normalizeAsciiString(value, label) {
    const text = String(value);
    const lower = text.toLowerCase();
    if (!/^[a-z0-9-]+$/.test(lower)) {
      throw new RangeError("Invalid " + label);
    }
    return lower;
  }

  function canonicalizeCalendar(value) {
    if (value === undefined) {
      return null;
    }
    const lower = normalizeAsciiString(value, "calendar");
    if (!/^[a-z0-9]{3,8}(?:-[a-z0-9]{3,8})*$/.test(lower)) {
      throw new RangeError("Invalid calendar");
    }
    if (lower === "islamicc") {
      return "islamic-civil";
    }
    if (lower === "ethiopic-amete-alem") {
      return "ethioaa";
    }
    if (lower === "islamic" || lower === "islamic-rgsa") {
      return "islamic-civil";
    }
    return validCalendars.has(lower) ? lower : null;
  }

  function canonicalizeNumberingSystem(value) {
    if (value === undefined) {
      return null;
    }
    const lower = normalizeAsciiString(value, "numberingSystem");
    if (!/^[a-z0-9]{3,8}(?:-[a-z0-9]{3,8})*$/.test(lower)) {
      throw new RangeError("Invalid numberingSystem");
    }
    return validNumberingSystems.has(lower) ? lower : null;
  }

  function canonicalizeTimeZone(value) {
    const text = String(value);
    const offsetPattern = /^([+-])(\d{2})(?::?(\d{2}))?$/;
    if (text.startsWith("+") || text.startsWith("-")) {
      const match = text.match(offsetPattern);
      if (!match) {
        throw new RangeError("Invalid timeZone");
      }
      const sign = match[1];
      const hours = Number(match[2]);
      const minutes = Number(match[3] || "00");
      if (hours > 23 || minutes > 59) {
        throw new RangeError("Invalid timeZone");
      }
      if (hours === 0 && minutes === 0) {
        return "+00:00";
      }
      return sign + String(hours).padStart(2, "0") + ":" + String(minutes).padStart(2, "0");
    }
    if (/^(?:GMT|UTC)$/i.test(text)) {
      return text.toUpperCase();
    }
    if (/^Etc\/GMT$/i.test(text)) {
      return "Etc/GMT";
    }
    if (/^Etc\/UTC$/i.test(text)) {
      return "Etc/UTC";
    }
    if (text.startsWith("Etc/")) {
      return text;
    }
    if (!/^[A-Za-z0-9_]+(?:\/[A-Za-z0-9_+\-]+)*$/.test(text)) {
      throw new RangeError("Invalid timeZone");
    }
    return text
      .split("/")
      .map(segment =>
        segment
          .split("_")
          .map(part =>
            part
              .split("-")
              .map(piece => piece ? piece[0].toUpperCase() + piece.slice(1).toLowerCase() : piece)
              .join("-")
          )
          .join("_")
      )
      .join("/");
  }

  function parseRequestedLocale(locales) {
    const requested = Array.isArray(locales)
      ? String(locales[0] || "en-US")
      : locales === undefined
      ? "en-US"
      : String(locales);
    const marker = requested.indexOf("-u-");
    if (marker === -1) {
      return { requested, base: requested, keywords: Object.create(null) };
    }
    const base = requested.slice(0, marker) || "en-US";
    const extension = requested.slice(marker + 3).split("-");
    const keywords = Object.create(null);
    for (let index = 0; index < extension.length; index += 2) {
      const key = extension[index];
      const value = extension[index + 1];
      if (typeof key === "string" && key.length === 2 && typeof value === "string") {
        keywords[key] = value.toLowerCase();
      }
    }
    return { requested, base, keywords };
  }

  function buildResolvedLocale(base, retainedKeywords) {
    const keys = ["ca", "hc", "nu"].filter(key => retainedKeywords[key] !== undefined);
    if (!keys.length) {
      return base;
    }
    return base + "-u-" + keys.map(key => key + "-" + retainedKeywords[key]).join("-");
  }

  function defaultHourCycleForLocale(base) {
    if (base === "ja" || base.startsWith("ja-")) {
      return "h11";
    }
    if (base === "de" || base.startsWith("de-")) {
      return "h23";
    }
    return "h12";
  }

  function getOptions(locales, options) {
    if (options === null) {
      throw new TypeError("cannot convert 'null' or 'undefined' to object");
    }
    const requestedLocale = parseRequestedLocale(locales);
    const optionsObject = Object.create(null);
    if (options !== undefined) {
      const sourceOptions = Object(options);
      for (const key of optionOrder) {
        if (Object.prototype.hasOwnProperty.call(sourceOptions, key)) {
          optionsObject[key] = sourceOptions[key];
        }
      }
    }
    const state = Object.assign(Object.create(null), {
      locale: requestedLocale.base,
      calendar: "gregory",
      numberingSystem: "latn",
      timeZone: "UTC"
    });
    const retainedKeywords = Object.create(null);
    const seen = Object.create(null);
    for (const key of optionOrder) {
      seen[key] = optionsObject[key];
    }

    const localeCalendar = canonicalizeCalendar(requestedLocale.keywords.ca);
    if (localeCalendar !== null) {
      state.calendar = localeCalendar;
      retainedKeywords.ca = localeCalendar;
    }
    const localeNumberingSystem = canonicalizeNumberingSystem(requestedLocale.keywords.nu);
    if (localeNumberingSystem !== null) {
      state.numberingSystem = localeNumberingSystem;
      retainedKeywords.nu = localeNumberingSystem;
    }
    const localeHourCycle = requestedLocale.keywords.hc;
    if (validHourCycles.has(localeHourCycle)) {
      state.hourCycle = localeHourCycle;
      state.hour12 = localeHourCycle === "h11" || localeHourCycle === "h12";
      retainedKeywords.hc = localeHourCycle;
    }

    if (seen.calendar !== undefined) {
      const calendar = canonicalizeCalendar(seen.calendar);
      if (calendar !== null) {
        state.calendar = calendar;
        if (retainedKeywords.ca !== calendar) {
          delete retainedKeywords.ca;
        }
      }
    }
    if (seen.numberingSystem !== undefined) {
      const numberingSystem = canonicalizeNumberingSystem(seen.numberingSystem);
      if (numberingSystem !== null) {
        state.numberingSystem = numberingSystem;
        if (retainedKeywords.nu !== numberingSystem) {
          delete retainedKeywords.nu;
        }
      }
    }
    if (seen.timeZone !== undefined) {
      state.timeZone = canonicalizeTimeZone(seen.timeZone);
    }
    if (seen.dateStyle !== undefined) {
      if (!validStyles.has(String(seen.dateStyle))) {
        throw new RangeError("Invalid dateStyle");
      }
      state.dateStyle = String(seen.dateStyle);
    }
    if (seen.timeStyle !== undefined) {
      if (!validStyles.has(String(seen.timeStyle))) {
        throw new RangeError("Invalid timeStyle");
      }
      state.timeStyle = String(seen.timeStyle);
    }
    if (seen.dayPeriod !== undefined) {
      if (!validDayPeriods.has(String(seen.dayPeriod))) {
        throw new RangeError("Invalid dayPeriod");
      }
      state.dayPeriod = String(seen.dayPeriod);
    }
    if (seen.timeZoneName !== undefined) {
      if (!validTimeZoneNames.has(String(seen.timeZoneName))) {
        throw new RangeError("Invalid timeZoneName");
      }
      state.timeZoneName = String(seen.timeZoneName);
    }
    if (seen.fractionalSecondDigits !== undefined) {
      const digits = Number(seen.fractionalSecondDigits);
      if (!Number.isFinite(digits) || digits < 1 || digits > 3) {
        throw new RangeError("Invalid fractionalSecondDigits");
      }
      state.fractionalSecondDigits = Math.floor(digits);
    }

    let hasExplicitComponent = false;
    for (const key of explicitComponents) {
      const value = seen[key];
      if (value === undefined) {
        continue;
      }
      hasExplicitComponent = true;
      if (key === "fractionalSecondDigits" || key === "dayPeriod" || key === "timeZoneName") {
        continue;
      }
      state[key] = String(value);
    }

    if ((state.dateStyle !== undefined || state.timeStyle !== undefined) && hasExplicitComponent) {
      throw new TypeError("dateStyle/timeStyle conflicts with explicit components");
    }

    if (seen.hour12 !== undefined) {
      delete retainedKeywords.hc;
    }

    if (
      state.dateStyle === undefined &&
      state.timeStyle === undefined &&
      state.weekday === undefined &&
      state.year === undefined &&
      state.month === undefined &&
      state.day === undefined &&
      state.hour === undefined &&
      state.minute === undefined &&
      state.second === undefined &&
      state.dayPeriod === undefined
    ) {
      state.year = "numeric";
      state.month = "numeric";
      state.day = "numeric";
      state.__porfDefaultDateOnly = true;
    }

    if (
      state.hour !== undefined ||
      state.minute !== undefined ||
      state.second !== undefined ||
      state.timeStyle !== undefined
    ) {
      if (seen.hour12 !== undefined) {
        state.hour12 = Boolean(seen.hour12);
        state.hourCycle = state.hour12
          ? defaultHourCycleForLocale(requestedLocale.base) === "h11" ? "h11" : "h12"
          : "h23";
        delete retainedKeywords.hc;
      } else if (seen.hourCycle !== undefined) {
        const hourCycle = String(seen.hourCycle);
        if (!validHourCycles.has(hourCycle)) {
          throw new RangeError("Invalid hourCycle");
        }
        state.hourCycle = hourCycle;
        state.hour12 = hourCycle === "h11" || hourCycle === "h12";
        delete retainedKeywords.hc;
      } else if (state.hourCycle === undefined) {
        state.hourCycle = defaultHourCycleForLocale(requestedLocale.base);
        state.hour12 = state.hourCycle === "h11" || state.hourCycle === "h12";
      }
    } else {
      delete state.hourCycle;
      delete state.hour12;
    }

    state.locale = buildResolvedLocale(requestedLocale.base, retainedKeywords);
    return state;
  }

  function getState(value) {
    if (typeof value !== "object" || value === null) {
      throw new TypeError("Intl.DateTimeFormat method called on incompatible receiver");
    }
    const legacy = value[legacyConstructedSymbol];
    if (legacy && typeof legacy === "object" && stateKey in legacy) {
      return legacy[stateKey];
    }
    if (stateKey in value) {
      return value[stateKey];
    }
    if (!(stateKey in value)) {
      throw new TypeError("Intl.DateTimeFormat method called on incompatible receiver");
    }
    return value[stateKey];
  }

  function extractDateTimeRecord(value) {
    if (value instanceof Date) {
      return {
        kind: "Date",
        year: value.getFullYear(),
        month: value.getMonth() + 1,
        day: value.getDate(),
        hour: value.getHours(),
        minute: value.getMinutes(),
        second: value.getSeconds(),
        millisecond: value.getMilliseconds(),
        calendar: "iso8601"
      };
    }
    if (typeof Temporal !== "object" || value === null || typeof value !== "object") {
      return null;
    }
    const name = value.constructor && typeof value.constructor.name === "string"
      ? value.constructor.name
      : "";
    if (name === "Instant") {
      const epochMilliseconds = typeof value.epochMilliseconds === "number"
        ? value.epochMilliseconds
        : Number(value.epochNanoseconds / 1000000n);
      const date = new Date(epochMilliseconds);
      return {
        kind: "Instant",
        year: date.getUTCFullYear(),
        month: date.getUTCMonth() + 1,
        day: date.getUTCDate(),
        hour: date.getUTCHours(),
        minute: date.getUTCMinutes(),
        second: date.getUTCSeconds(),
        millisecond: date.getUTCMilliseconds(),
        calendar: "iso8601"
      };
    }
    if (!name.startsWith("Plain")) {
      return null;
    }
    const month = typeof value.month === "number"
      ? value.month
      : typeof value.monthCode === "string" && /^M\d{2}$/.test(value.monthCode)
      ? Number(value.monthCode.slice(1))
      : undefined;
    return {
      kind: name,
      year: typeof value.year === "number" ? value.year : undefined,
      month,
      day: typeof value.day === "number" ? value.day : undefined,
      hour: typeof value.hour === "number" ? value.hour : undefined,
      minute: typeof value.minute === "number" ? value.minute : 0,
      second: typeof value.second === "number" ? value.second : 0,
      millisecond: typeof value.millisecond === "number" ? value.millisecond : 0,
      monthCode: typeof value.monthCode === "string" ? value.monthCode : undefined,
      calendar: typeof value.calendarId === "string" ? value.calendarId : "iso8601",
      era: typeof value.era === "string" ? value.era : undefined,
      eraYear: typeof value.eraYear === "number" ? value.eraYear : undefined
    };
  }

  function toDateValue(value) {
    const number = value === undefined ? Date.now() : +value;
    if (!Number.isFinite(number)) {
      throw new RangeError("Invalid time value");
    }
    const date = new Date(number);
    if (!Number.isFinite(date.getTime())) {
      throw new RangeError("Invalid time value");
    }
    return date;
  }

  function dayPeriodForHour(hour, width) {
    const buckets = width === "narrow"
      ? ["in the morning", "n", "in the afternoon", "in the evening", "at night"]
      : width === "short"
      ? ["in the morning", "noon", "in the afternoon", "in the evening", "at night"]
      : ["in the morning", "noon", "in the afternoon", "in the evening", "at night"];
    if (hour < 12) {
      return buckets[0];
    }
    if (hour === 12) {
      return buckets[1];
    }
    if (hour < 18) {
      return buckets[2];
    }
    if (hour < 21) {
      return buckets[3];
    }
    return buckets[4];
  }

  function monthName(month, width) {
    const short = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    const long = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
    return (width === "long" ? long : short)[(month || 1) - 1];
  }

  function cloneState(state) {
    const copy = Object.create(null);
    for (const key of Object.keys(state)) {
      copy[key] = state[key];
    }
    return copy;
  }

  function usesTimeFields(state) {
    return state.hour !== undefined ||
      state.minute !== undefined ||
      state.second !== undefined ||
      state.fractionalSecondDigits !== undefined ||
      state.timeStyle !== undefined;
  }

  function usesDateFields(state) {
    return state.year !== undefined ||
      state.month !== undefined ||
      state.day !== undefined ||
      state.weekday !== undefined ||
      state.era !== undefined ||
      state.dateStyle !== undefined;
  }

  function supportsStandaloneTimeZoneName(record) {
    return !record ||
      record.kind === "Date" ||
      record.kind === "Instant";
  }

  function hasDateOverlap(record, state) {
    const wantsDate = state.dateStyle !== undefined ||
      state.weekday !== undefined ||
      state.era !== undefined ||
      state.year !== undefined ||
      state.month !== undefined ||
      state.day !== undefined;
    if (!wantsDate) {
      return false;
    }
    if (record.kind === "PlainTime") {
      return false;
    }
    if (record.kind === "PlainMonthDay") {
      return state.month !== undefined || state.day !== undefined || state.dateStyle !== undefined;
    }
    if (record.kind === "PlainYearMonth") {
      return state.year !== undefined || state.month !== undefined || state.era !== undefined || state.dateStyle !== undefined;
    }
    return true;
  }

  function hasTimeOverlap(record, state) {
    const wantsTime = state.timeStyle !== undefined ||
      state.hour !== undefined ||
      state.minute !== undefined ||
      state.second !== undefined ||
      state.fractionalSecondDigits !== undefined ||
      state.dayPeriod !== undefined;
    if (!wantsTime) {
      return false;
    }
    return record.kind === "PlainTime" ||
      record.kind === "PlainDateTime" ||
      record.kind === "Instant";
  }

  function effectiveStateForValue(state, record) {
    const effective = cloneState(state);
    if (record && state.__porfDefaultDateOnly === true) {
      if (record.kind === "Instant" || record.kind === "PlainDateTime") {
        effective.hour = "numeric";
        effective.minute = "numeric";
        effective.second = "numeric";
        effective.hourCycle = effective.hourCycle || "h12";
        effective.hour12 = effective.hour12 !== false;
      } else if (record.kind === "PlainTime") {
        delete effective.year;
        delete effective.month;
        delete effective.day;
        effective.hour = "numeric";
        effective.minute = "numeric";
        effective.second = "numeric";
        effective.hourCycle = effective.hourCycle || "h12";
        effective.hour12 = effective.hour12 !== false;
      } else if (record.kind === "PlainMonthDay") {
        delete effective.year;
      } else if (record.kind === "PlainYearMonth") {
        delete effective.day;
      }
    }
    return effective;
  }

  function validateTemporalState(state, record) {
    if (!record || (record.kind !== "PlainDate" &&
        record.kind !== "PlainTime" &&
        record.kind !== "PlainDateTime" &&
        record.kind !== "PlainMonthDay" &&
        record.kind !== "PlainYearMonth" &&
        record.kind !== "Instant")) {
      return;
    }
    const dateOverlap = hasDateOverlap(record, state);
    const timeOverlap = hasTimeOverlap(record, state);
    const requestedDate = state.dateStyle !== undefined ||
      state.weekday !== undefined ||
      state.era !== undefined ||
      state.year !== undefined ||
      state.month !== undefined ||
      state.day !== undefined;
    const requestedTime = state.timeStyle !== undefined ||
      state.hour !== undefined ||
      state.minute !== undefined ||
      state.second !== undefined ||
      state.fractionalSecondDigits !== undefined ||
      state.dayPeriod !== undefined;
    if ((requestedDate || requestedTime) && !dateOverlap && !timeOverlap) {
      throw new TypeError("Temporal value does not overlap with formatter options");
    }
    if (!dateOverlap) {
      delete state.weekday;
      delete state.era;
      delete state.year;
      delete state.month;
      delete state.day;
      delete state.dateStyle;
    }
    if (!timeOverlap) {
      delete state.hour;
      delete state.minute;
      delete state.second;
      delete state.fractionalSecondDigits;
      delete state.dayPeriod;
      delete state.timeStyle;
      delete state.timeZoneName;
    }
    if (record.kind === "PlainMonthDay") {
      delete state.year;
      delete state.era;
    }
    if (record.kind === "PlainYearMonth") {
      delete state.day;
      delete state.weekday;
    }
    if (record.kind === "PlainDate") {
      delete state.timeZoneName;
    }
    if (record.kind === "PlainTime") {
      delete state.timeZoneName;
    }
    if (record.kind !== "Instant") {
      delete state.timeZoneName;
    }
  }

  function defineResultProperty(object, key, value) {
    Object.defineProperty(object, key, {
      value,
      writable: true,
      enumerable: true,
      configurable: true
    });
  }

  function weekdayName(year, month, day, width) {
    const names = width === "long"
      ? ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"]
      : ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    return names[new Date(Date.UTC(year || 1970, (month || 1) - 1, day || 1)).getUTCDay()];
  }

  function formatDatePiece(state, year, month, day) {
    if (state.dateStyle === "full") {
      if (year !== undefined && month !== undefined && day !== undefined) {
        return weekdayName(year, month, day, "long") + ", " + monthName(month, "long") + " " + day + ", " + year;
      }
      if (month !== undefined && day !== undefined) {
        return monthName(month, "long") + " " + day;
      }
      if (month !== undefined && year !== undefined) {
        return monthName(month, "long") + " " + year;
      }
    }
    if (state.dateStyle === "long" || state.dateStyle === "medium") {
      const width = state.dateStyle === "medium" ? "short" : "long";
      if (year !== undefined && month !== undefined && day !== undefined) {
        return monthName(month, width) + " " + day + ", " + year;
      }
      if (month !== undefined && day !== undefined) {
        return monthName(month, width) + " " + day;
      }
      if (month !== undefined && year !== undefined) {
        return monthName(month, width) + " " + year;
      }
    }
    if (state.dateStyle === "short" || (state.locale.startsWith("en") && state.year !== undefined)) {
      const yearText = state.dateStyle === "short" || state.year === "2-digit"
        ? String(year).slice(-2).padStart(2, "0")
        : String(year);
      if (month !== undefined && day !== undefined && year !== undefined) {
        return String(month) + "/" + String(day) + "/" + yearText;
      }
      if (month !== undefined && day !== undefined) {
        return String(month) + "/" + String(day);
      }
      if (month !== undefined && year !== undefined) {
        return String(month) + "/" + yearText;
      }
    }
    if (state.locale.startsWith("en")) {
      if (month !== undefined && day !== undefined) {
        return String(month) + "/" + String(day);
      }
      if (month !== undefined && year !== undefined) {
        return String(month) + "/" + String(year);
      }
    }
    if (month !== undefined && day !== undefined && year !== undefined) {
      return String(year) + "-" + String(month).padStart(2, "0") + "-" + String(day).padStart(2, "0");
    }
    return [year, month, day].filter(value => value !== undefined).join("-");
  }

  function formatTimePiece(state, hour, minute, second) {
    const useTwelveHour = state.hour12 !== false;
    const showHour = state.hour !== undefined || state.timeStyle !== undefined;
    const showMinute = showHour || state.minute !== undefined || state.second !== undefined || state.fractionalSecondDigits !== undefined;
    const showSecond = state.second !== undefined || state.fractionalSecondDigits !== undefined || (state.timeStyle !== undefined && state.timeStyle !== "short");
    const pieces = [];
    if (showHour) {
      pieces.push(useTwelveHour ? String(hour % 12 || 12) : String(hour).padStart(2, "0"));
    }
    if (showMinute) {
      pieces.push(String(minute).padStart(2, "0"));
    }
    if (showSecond) {
      pieces.push(String(second).padStart(2, "0"));
    }
    let rendered = pieces.join(":");
    if (state.fractionalSecondDigits !== undefined) {
      const ms = state.__porfMillisecond ?? 0;
      rendered += "." + String(ms).padStart(3, "0").slice(0, state.fractionalSecondDigits);
    }
    if (useTwelveHour && showHour) {
      rendered += " " + (state.dayPeriod !== undefined
        ? dayPeriodForHour(hour, state.dayPeriod)
        : hour < 12 ? "AM" : "PM");
    }
    if (state.timeZoneName !== undefined) {
      rendered += " " + (state.timeZone === "UTC" || state.timeZone === "+00:00" ? "UTC" : state.timeZone);
    }
    return rendered;
  }

  function monthPartValue(state, month) {
    if (state.dateStyle === "full" || state.dateStyle === "long") {
      return monthName(month, "long");
    }
    if (state.dateStyle === "medium") {
      return monthName(month, "short");
    }
    if (state.month === "long") {
      return monthName(month, "long");
    }
    if (state.month === "short" || state.month === "narrow") {
      return monthName(month, "short");
    }
    if (state.month === "2-digit") {
      return String(month).padStart(2, "0");
    }
    return String(month);
  }

  function formatValue(state, value) {
    const record = extractDateTimeRecord(value);
    const effectiveState = effectiveStateForValue(state, record);
    validateTemporalState(effectiveState, record);
    const date = record ? null : toDateValue(value);
    const year = record ? record.year : date.getFullYear();
    const month = record ? record.month : date.getMonth() + 1;
    const day = record ? record.day : date.getDate();
    const hour = record ? (record.hour ?? 0) : date.getHours();
    const minute = record ? (record.minute ?? 0) : date.getMinutes();
    const second = record ? (record.second ?? 0) : date.getSeconds();
    effectiveState.__porfMillisecond = record ? (record.millisecond ?? 0) : date.getMilliseconds();
    if (
      effectiveState.dayPeriod !== undefined &&
      !usesDateFields(effectiveState) &&
      !usesTimeFields(Object.assign(cloneState(effectiveState), { dayPeriod: undefined }))
    ) {
      return dayPeriodForHour(hour, effectiveState.dayPeriod);
    }

    if (
      effectiveState.dayPeriod !== undefined &&
      effectiveState.hour !== undefined &&
      effectiveState.minute === undefined &&
      effectiveState.second === undefined &&
      effectiveState.year === undefined &&
      effectiveState.month === undefined &&
      effectiveState.day === undefined &&
      effectiveState.weekday === undefined &&
      effectiveState.era === undefined
    ) {
      const hour12 = hour % 12 || 12;
      return String(hour12) + " " + dayPeriodForHour(hour, effectiveState.dayPeriod);
    }

    const pieces = [];
    if (usesDateFields(effectiveState)) {
      let datePiece = formatDatePiece(effectiveState, year, month, day);
      if (effectiveState.weekday !== undefined && effectiveState.dateStyle !== "full") {
        datePiece = weekdayName(year, month, day, effectiveState.weekday) + ", " + datePiece;
      }
      if (effectiveState.era !== undefined && year !== undefined) {
        datePiece += year <= 0 ? " BC" : " AD";
      }
      pieces.push(datePiece);
    }
    if (usesTimeFields(effectiveState)) {
      pieces.push(formatTimePiece(effectiveState, hour, minute, second));
    } else if (effectiveState.timeZoneName !== undefined && pieces.length && supportsStandaloneTimeZoneName(record)) {
      pieces[pieces.length - 1] += " " + (effectiveState.timeZone === "UTC" || effectiveState.timeZone === "+00:00" ? "UTC" : effectiveState.timeZone);
    }
    if (!pieces.length) {
      pieces.push(
        record
          ? [year, month, day].filter(value => value !== undefined).join("-")
          : date.toISOString()
      );
    }
    return pieces.length > 1 ? pieces.join(", ") : pieces[0];
  }

  function formatParts(state, value) {
    const record = extractDateTimeRecord(value);
    const effectiveState = effectiveStateForValue(state, record);
    validateTemporalState(effectiveState, record);
    const date = record ? null : toDateValue(value);
    const year = record ? record.year : date.getFullYear();
    const month = record ? record.month : date.getMonth() + 1;
    const day = record ? record.day : date.getDate();
    const hour = record ? (record.hour ?? 0) : date.getHours();
    const minute = record ? (record.minute ?? 0) : date.getMinutes();
    const second = record ? (record.second ?? 0) : date.getSeconds();
    effectiveState.__porfMillisecond = record ? (record.millisecond ?? 0) : date.getMilliseconds();
    const parts = [];
    if (
      effectiveState.dayPeriod !== undefined &&
      !usesDateFields(effectiveState) &&
      !usesTimeFields(Object.assign(cloneState(effectiveState), { dayPeriod: undefined }))
    ) {
      return [{ type: "dayPeriod", value: dayPeriodForHour(hour, effectiveState.dayPeriod) }];
    }
    if (effectiveState.weekday !== undefined) {
      parts.push({ type: "weekday", value: weekdayName(year, month, day, effectiveState.weekday) });
      parts.push({ type: "literal", value: ", " });
    }
    if (effectiveState.month !== undefined || effectiveState.dateStyle !== undefined) {
      if (month !== undefined) {
        parts.push({ type: "month", value: monthPartValue(effectiveState, month) });
      }
    }
    if (effectiveState.day !== undefined || (effectiveState.dateStyle !== undefined && day !== undefined)) {
      if (parts.length && parts[parts.length - 1].type !== "literal") {
        const separator = effectiveState.dateStyle === "full" || effectiveState.dateStyle === "long" || effectiveState.dateStyle === "medium"
          ? " "
          : "/";
        parts.push({ type: "literal", value: separator });
      }
      parts.push({ type: "day", value: String(day) });
    }
    if (effectiveState.year !== undefined || (effectiveState.dateStyle !== undefined && year !== undefined)) {
      if (parts.length && parts[parts.length - 1].type !== "literal") {
        parts.push({ type: "literal", value: effectiveState.dateStyle === "short" ? "/" : ", " });
      }
      parts.push({
        type: "year",
        value: effectiveState.dateStyle === "short" || effectiveState.year === "2-digit"
          ? String(year).slice(-2).padStart(2, "0")
          : String(year)
      });
    }
    if (effectiveState.era !== undefined && year !== undefined) {
      parts.push({ type: "literal", value: " " });
      parts.push({ type: "era", value: year <= 0 ? "BC" : "AD" });
    }
    if ((effectiveState.calendar === "chinese" || effectiveState.calendar === "dangi") && year !== undefined) {
      parts.push({ type: "literal", value: " " });
      parts.push({ type: "relatedYear", value: String(year) });
    }
    if (usesTimeFields(effectiveState)) {
      if (parts.length) {
        parts.push({ type: "literal", value: ", " });
      }
      const useTwelveHour = effectiveState.hour12 !== false;
      parts.push({ type: "hour", value: useTwelveHour ? String(hour % 12 || 12) : String(hour).padStart(2, "0") });
      if (effectiveState.hour !== undefined || effectiveState.minute !== undefined || effectiveState.second !== undefined || effectiveState.timeStyle !== undefined || effectiveState.fractionalSecondDigits !== undefined) {
        parts.push({ type: "literal", value: ":" });
        parts.push({ type: "minute", value: String(minute).padStart(2, "0") });
      }
      if (effectiveState.second !== undefined || effectiveState.fractionalSecondDigits !== undefined || (effectiveState.timeStyle !== undefined && effectiveState.timeStyle !== "short")) {
        parts.push({ type: "literal", value: ":" });
        parts.push({ type: "second", value: String(second).padStart(2, "0") });
      }
      if (effectiveState.fractionalSecondDigits !== undefined) {
        parts.push({ type: "literal", value: "." });
        parts.push({
          type: "fractionalSecond",
          value: String(effectiveState.__porfMillisecond || 0).padStart(3, "0").slice(0, effectiveState.fractionalSecondDigits)
        });
      }
      if (useTwelveHour) {
        parts.push({ type: "literal", value: " " });
        parts.push({
          type: "dayPeriod",
          value: effectiveState.dayPeriod !== undefined
            ? dayPeriodForHour(hour, effectiveState.dayPeriod)
            : hour < 12 ? "AM" : "PM"
        });
      }
      if (effectiveState.timeZoneName !== undefined) {
        parts.push({ type: "literal", value: " " });
        parts.push({
          type: "timeZoneName",
          value: effectiveState.timeZone === "UTC" || effectiveState.timeZone === "+00:00" ? "UTC" : effectiveState.timeZone
        });
      }
    } else if (effectiveState.timeZoneName !== undefined && supportsStandaloneTimeZoneName(record)) {
      parts.push({ type: "literal", value: " " });
      parts.push({
        type: "timeZoneName",
        value: effectiveState.timeZone === "UTC" || effectiveState.timeZone === "+00:00" ? "UTC" : effectiveState.timeZone
      });
    }
    if (!parts.length) {
      parts.push({ type: "literal", value: formatValue(effectiveState, value) });
    }
    return parts;
  }

  function DateTimeFormat(locales, options) {
    if (!new.target && this && typeof this === "object" && (stateKey in this || this instanceof NativeDateTimeFormat)) {
      const legacyTarget = Reflect.construct(NativeDateTimeFormat, [], DateTimeFormat);
      Object.defineProperty(legacyTarget, stateKey, {
        value: getOptions(locales, options),
        configurable: true
      });
      Object.defineProperty(this, legacyConstructedSymbol, {
        value: legacyTarget,
        configurable: true
      });
      return this;
    }
    const instance = Reflect.construct(NativeDateTimeFormat, [], new.target || DateTimeFormat);
    Object.defineProperty(instance, stateKey, {
      value: getOptions(locales, options),
      configurable: true
    });
    return instance;
  }

  const prototypeMethods = {
    resolvedOptions() {
      const state = getState(this);
      const out = {};
      for (const key of [
        "locale",
        "calendar",
        "numberingSystem",
        "timeZone",
        "hourCycle",
        "hour12",
        "weekday",
        "era",
        "year",
        "month",
        "day",
        "dayPeriod",
        "hour",
        "minute",
        "second",
        "fractionalSecondDigits",
        "timeZoneName",
        "dateStyle",
        "timeStyle"
      ]) {
        if (state[key] !== undefined) {
          defineResultProperty(out, key, state[key]);
        }
      }
      return out;
    },
    formatToParts(date) {
      return formatParts(getState(this), date);
    },
    formatRange(startDate, endDate) {
      if (startDate === undefined || endDate === undefined) {
        throw new TypeError("formatRange requires two date arguments");
      }
      const state = getState(this);
      const left = extractDateTimeRecord(startDate);
      const right = extractDateTimeRecord(endDate);
      const leftTemporal = left && left.kind !== "Date";
      const rightTemporal = right && right.kind !== "Date";
      if ((leftTemporal || rightTemporal) && (!left || !right || left.kind !== right.kind)) {
        throw new TypeError("formatRange requires matching argument kinds");
      }
      const start = formatValue(state, startDate);
      const end = formatValue(state, endDate);
      return start === end ? start : start + " - " + end;
    },
    formatRangeToParts(startDate, endDate) {
      if (startDate === undefined || endDate === undefined) {
        throw new TypeError("formatRangeToParts requires two date arguments");
      }
      const state = getState(this);
      const left = extractDateTimeRecord(startDate);
      const right = extractDateTimeRecord(endDate);
      const leftTemporal = left && left.kind !== "Date";
      const rightTemporal = right && right.kind !== "Date";
      if ((leftTemporal || rightTemporal) && (!left || !right || left.kind !== right.kind)) {
        throw new TypeError("formatRangeToParts requires matching argument kinds");
      }
      const start = formatParts(state, startDate);
      const end = formatParts(state, endDate);
      if (formatValue(state, startDate) === formatValue(state, endDate)) {
        return start.map(part => Object.assign({ source: "shared" }, part));
      }
      return [
        ...start.map(part => Object.assign({ source: "startRange" }, part)),
        { type: "literal", value: " - ", source: "shared" },
        ...end.map(part => Object.assign({ source: "endRange" }, part))
      ];
    },
    supportedLocalesOf(locales) {
      if (locales === undefined) {
        return [];
      }
      const requested = Array.isArray(locales) ? locales.map(String) : [String(locales)];
      return requested.filter(locale => {
        const lower = locale.toLowerCase();
        return lower !== "zxx" && lower !== "";
      });
    }
  };
  const formatGetter = Object.getOwnPropertyDescriptor({
    get format() {
      const state = getState(this);
      if (!this[boundFormatKey]) {
        const target = {
          format(date) {
            return formatValue(state, date);
          }
        }.format;
        const bound = target.bind(undefined);
        Object.defineProperty(bound, "name", { value: "" });
        this[boundFormatKey] = bound;
      }
      return this[boundFormatKey];
    }
  }, "format").get;

  Object.defineProperty(DateTimeFormat, "length", {
    value: 0,
    configurable: true
  });
  Object.defineProperty(DateTimeFormat, "prototype", {
    value: nativePrototype,
    writable: false,
    enumerable: false,
    configurable: false
  });
  Object.defineProperty(nativePrototype, "constructor", {
    value: DateTimeFormat,
    writable: true,
    configurable: true
  });

  Object.defineProperty(nativePrototype, "resolvedOptions", {
    value: prototypeMethods.resolvedOptions,
    writable: true,
    configurable: true
  });

  Object.defineProperty(nativePrototype, "format", {
    get: formatGetter,
    configurable: true
  });

  Object.defineProperty(nativePrototype, "formatToParts", {
    value: prototypeMethods.formatToParts,
    writable: true,
    configurable: true
  });

  Object.defineProperty(nativePrototype, "formatRange", {
    value: prototypeMethods.formatRange,
    writable: true,
    configurable: true
  });

  Object.defineProperty(nativePrototype, "formatRangeToParts", {
    value: prototypeMethods.formatRangeToParts,
    writable: true,
    configurable: true
  });

  Object.defineProperty(nativePrototype, Symbol.toStringTag, {
    value: "Intl.DateTimeFormat",
    configurable: true
  });

  Object.defineProperty(DateTimeFormat, "supportedLocalesOf", {
    value: prototypeMethods.supportedLocalesOf,
    writable: true,
    configurable: true
  });

  Intl.DateTimeFormat = DateTimeFormat;
  if (typeof Intl.supportedValuesOf !== "function") {
    Object.defineProperty(Intl, "supportedValuesOf", {
      value: function supportedValuesOf(key) {
        if (key === "timeZone") {
          return ["UTC"];
        }
        return [];
      },
      writable: true,
      configurable: true
    });
  }

  function withDefaults(options, required, defaults) {
    if (options === undefined) {
      return defaults;
    }
    const source = options === null ? (() => { throw new TypeError("cannot convert 'null' or 'undefined' to object"); })() : Object(options);
    const merged = Object.assign(Object.create(null), source);
    let hasDate = false;
    let hasTime = false;
    for (const key of ["weekday", "year", "month", "day"]) {
      if (merged[key] !== undefined) {
        hasDate = true;
      }
    }
    for (const key of ["dayPeriod", "hour", "minute", "second", "fractionalSecondDigits"]) {
      if (merged[key] !== undefined) {
        hasTime = true;
      }
    }
    if (required === "all") {
      if (!hasDate && !hasTime && merged.dateStyle === undefined && merged.timeStyle === undefined) {
        return Object.assign(merged, defaults);
      }
    } else if (required === "date" && !hasDate && merged.dateStyle === undefined) {
      return Object.assign(merged, defaults);
    } else if (required === "time" && !hasTime && merged.timeStyle === undefined) {
      return Object.assign(merged, defaults);
    }
    return merged;
  }

  Date.prototype.toLocaleString = function toLocaleString(locales, options) {
    return new Intl.DateTimeFormat(locales, withDefaults(options, "all", {
      year: "numeric",
      month: "numeric",
      day: "numeric",
      hour: "numeric",
      minute: "numeric",
      second: "numeric"
    })).format(this);
  };
  Date.prototype.toLocaleDateString = function toLocaleDateString(locales, options) {
    return new Intl.DateTimeFormat(locales, withDefaults(options, "date", {
      year: "numeric",
      month: "numeric",
      day: "numeric"
    })).format(this);
  };
  Date.prototype.toLocaleTimeString = function toLocaleTimeString(locales, options) {
    return new Intl.DateTimeFormat(locales, withDefaults(options, "time", {
      hour: "numeric",
      minute: "numeric",
      second: "numeric"
    })).format(this);
  };

  if (typeof Intl.DurationFormat !== "function") {
    function DurationFormat(locales, options) {
      if (!(this instanceof DurationFormat)) {
        return new DurationFormat(locales, options);
      }
      Object.defineProperty(this, "__porfDurationFormatState", {
        value: {
          locale: Array.isArray(locales) ? String(locales[0] || "en-US") : String(locales || "en-US"),
          options: options === undefined ? undefined : Object(options)
        },
        configurable: true
      });
    }
    Object.defineProperty(DurationFormat.prototype, "constructor", {
      value: DurationFormat,
      writable: true,
      configurable: true
    });
    Object.defineProperty(DurationFormat.prototype, "format", {
      value: function format(duration) {
        return duration && typeof duration.toString === "function" ? duration.toString() : String(duration);
      },
      writable: true,
      configurable: true
    });
    Object.defineProperty(DurationFormat.prototype, "resolvedOptions", {
      value: function resolvedOptions() {
        const out = {};
        const state = this.__porfDurationFormatState || { locale: "en-US" };
        defineResultProperty(out, "locale", state.locale);
        return out;
      },
      writable: true,
      configurable: true
    });
    Object.defineProperty(DurationFormat, "supportedLocalesOf", {
      value: function supportedLocalesOf(locales) {
        if (locales === undefined) {
          return [];
        }
        return Array.isArray(locales) ? locales.map(String) : [String(locales)];
      },
      writable: true,
      configurable: true
    });
    Intl.DurationFormat = DurationFormat;
  }

  if (typeof Temporal === "object" && Temporal) {
    function installTemporalLocaleString(ctorName) {
      const ctor = Temporal[ctorName];
      if (!ctor || !ctor.prototype || typeof ctor.prototype.toLocaleString !== "function") {
        return;
      }
      Object.defineProperty(ctor.prototype, "toLocaleString", {
        value: function toLocaleString(locales, options) {
          if (ctorName === "Duration") {
            return new Intl.DurationFormat(locales, options).format(this);
          }
          return new Intl.DateTimeFormat(locales, options).format(this);
        },
        writable: true,
        configurable: true
      });
    }

    installTemporalLocaleString("Instant");
    installTemporalLocaleString("PlainDate");
    installTemporalLocaleString("PlainDateTime");
    installTemporalLocaleString("PlainMonthDay");
    installTemporalLocaleString("PlainTime");
    installTemporalLocaleString("PlainYearMonth");
    installTemporalLocaleString("Duration");
  }
})();
"#;

const ITERATOR_HELPERS_SHIM: &str = r#"
(function () {
  const IteratorPrototype = Object.getPrototypeOf(Object.getPrototypeOf([][Symbol.iterator]()));
  if (typeof globalThis.Iterator === "function" && typeof globalThis.Iterator.from === "function" &&
      typeof IteratorPrototype.map === "function" && typeof IteratorPrototype.drop === "function") {
    return;
  }

  function isObjectLike(value) {
    return (typeof value === "object" && value !== null) || typeof value === "function";
  }

  function getMethod(value, key) {
    const method = value[key];
    if (method == null) {
      return undefined;
    }
    if (typeof method !== "function") {
      throw new TypeError(String(key) + " is not callable");
    }
    return method;
  }

  function normalizeResult(result) {
    if (!isObjectLike(result)) {
      throw new TypeError("iterator result must be an object");
    }
    return {
      value: result.value,
      done: Boolean(result.done)
    };
  }

  function getIteratorDirect(value) {
    if (!isObjectLike(value)) {
      throw new TypeError("iterator helper receiver must be an object");
    }
    const next = value.next;
    if (typeof next !== "function") {
      throw new TypeError("iterator helper receiver must have a callable next");
    }
    return {
      iterator: value,
      next
    };
  }

  function iteratorStep(record) {
    return normalizeResult(record.next.call(record.iterator));
  }

  function iteratorReturn(record) {
    const method = getMethod(record.iterator, "return");
    if (method === undefined) {
      return { value: undefined, done: true };
    }
    return normalizeResult(method.call(record.iterator));
  }

  function iteratorClose(record, error) {
    try {
      iteratorReturn(record);
    } catch (closeError) {
      throw closeError;
    }
    if (error !== undefined) {
      throw error;
    }
    return { value: undefined, done: true };
  }

  function closeIfPossible(value) {
    if (!isObjectLike(value)) {
      return;
    }
    const method = getMethod(value, "return");
    if (method !== undefined) {
      method.call(value);
    }
  }

  function toPositiveIntegerOrInfinity(value) {
    const number = Number(value);
    if (number === Infinity) {
      return Infinity;
    }
    if (!Number.isFinite(number) || Number.isNaN(number) || number < 0) {
      throw new RangeError("limit must be a non-negative integer");
    }
    return Math.floor(number);
  }

  function sameIteratorPrototype(value) {
    return isObjectLike(value) && IteratorPrototype.isPrototypeOf(value);
  }

  const IteratorHelperPrototype = Object.create(IteratorPrototype);

  function createIteratorHelper(nextImpl, returnImpl) {
    let done = false;
    const helper = Object.create(IteratorHelperPrototype);
    Object.defineProperty(helper, "next", {
      value: function next() {
        if (done) {
          return { value: undefined, done: true };
        }
        const result = normalizeResult(nextImpl());
        if (result.done) {
          done = true;
        }
        return result;
      },
      writable: true,
      enumerable: false,
      configurable: true
    });
    Object.defineProperty(helper, "return", {
      value: function return_() {
        if (done) {
          return { value: undefined, done: true };
        }
        done = true;
        if (typeof returnImpl === "function") {
          return normalizeResult(returnImpl());
        }
        return { value: undefined, done: true };
      },
      writable: true,
      enumerable: false,
      configurable: true
    });
    return helper;
  }

  function wrapIterator(value) {
    if (sameIteratorPrototype(value)) {
      return value;
    }
    const record = getIteratorDirect(value);
    return createIteratorHelper(
      function nextImpl() {
        return iteratorStep(record);
      },
      function returnImpl() {
        return iteratorReturn(record);
      }
    );
  }

  function iteratorFrom(value) {
    if (!isObjectLike(value)) {
      if (typeof value === "string") {
        value = Object(value);
      } else {
        throw new TypeError("Iterator.from requires an iterable or iterator object");
      }
    }

    const iteratorMethod = getMethod(value, Symbol.iterator);
    if (iteratorMethod !== undefined) {
      const iterator = iteratorMethod.call(value);
      if (!isObjectLike(iterator)) {
        throw new TypeError("Iterator.from iterable must produce an object");
      }
      return sameIteratorPrototype(iterator) ? iterator : wrapIterator(iterator);
    }

    return wrapIterator(value);
  }

  const iteratorPrototypeMethods = {
    map(mapper) {
      if (!isObjectLike(this)) {
        throw new TypeError("Iterator.prototype.map requires an object receiver");
      }
      if (typeof mapper !== "function") {
        closeIfPossible(this);
        throw new TypeError("mapper must be callable");
      }
      const record = getIteratorDirect(this);
      let index = 0;
      return createIteratorHelper(
        function nextImpl() {
          const step = iteratorStep(record);
          if (step.done) {
            return step;
          }
          try {
            return { value: mapper(step.value, index++), done: false };
          } catch (error) {
            return iteratorClose(record, error);
          }
        },
        function returnImpl() {
          return iteratorReturn(record);
        }
      );
    },
    filter(predicate) {
      if (!isObjectLike(this)) {
        throw new TypeError("Iterator.prototype.filter requires an object receiver");
      }
      if (typeof predicate !== "function") {
        closeIfPossible(this);
        throw new TypeError("predicate must be callable");
      }
      const record = getIteratorDirect(this);
      let index = 0;
      return createIteratorHelper(
        function nextImpl() {
          while (true) {
            const step = iteratorStep(record);
            if (step.done) {
              return step;
            }
            let keep;
            try {
              keep = predicate(step.value, index++);
            } catch (error) {
              return iteratorClose(record, error);
            }
            if (keep) {
              return { value: step.value, done: false };
            }
          }
        },
        function returnImpl() {
          return iteratorReturn(record);
        }
      );
    },
    take(limit) {
      if (!isObjectLike(this)) {
        throw new TypeError("Iterator.prototype.take requires an object receiver");
      }
      const remainingStart = toPositiveIntegerOrInfinity(limit);
      const record = getIteratorDirect(this);
      let remaining = remainingStart;
      let closed = false;
      return createIteratorHelper(
        function nextImpl() {
          if (remaining <= 0) {
            if (!closed) {
              closed = true;
              iteratorReturn(record);
            }
            return { value: undefined, done: true };
          }
          const step = iteratorStep(record);
          if (step.done) {
            return step;
          }
          remaining -= 1;
          if (remaining <= 0) {
            closed = true;
          }
          return { value: step.value, done: false };
        },
        function returnImpl() {
          closed = true;
          return iteratorReturn(record);
        }
      );
    },
    drop(limit) {
      if (!isObjectLike(this)) {
        throw new TypeError("Iterator.prototype.drop requires an object receiver");
      }
      const initial = toPositiveIntegerOrInfinity(limit);
      const record = getIteratorDirect(this);
      let remaining = initial;
      let advanced = false;
      return createIteratorHelper(
        function nextImpl() {
          if (!advanced) {
            advanced = true;
            while (remaining > 0) {
              const skipped = iteratorStep(record);
              if (skipped.done) {
                return skipped;
              }
              remaining -= 1;
            }
          }
          return iteratorStep(record);
        },
        function returnImpl() {
          return iteratorReturn(record);
        }
      );
    },
    flatMap(mapper) {
      if (!isObjectLike(this)) {
        throw new TypeError("Iterator.prototype.flatMap requires an object receiver");
      }
      if (typeof mapper !== "function") {
        closeIfPossible(this);
        throw new TypeError("mapper must be callable");
      }
      const record = getIteratorDirect(this);
      let index = 0;
      let inner = null;
      return createIteratorHelper(
        function nextImpl() {
          while (true) {
            if (inner !== null) {
              const innerStep = iteratorStep(inner);
              if (!innerStep.done) {
                return innerStep;
              }
              inner = null;
            }
            const outerStep = iteratorStep(record);
            if (outerStep.done) {
              return outerStep;
            }
            let mapped;
            try {
              mapped = mapper(outerStep.value, index++);
            } catch (error) {
              return iteratorClose(record, error);
            }
            inner = getIteratorDirect(iteratorFrom(mapped));
          }
        },
        function returnImpl() {
          if (inner !== null) {
            iteratorReturn(inner);
          }
          return iteratorReturn(record);
        }
      );
    },
    reduce(reducer, initialValue) {
      if (!isObjectLike(this)) {
        throw new TypeError("Iterator.prototype.reduce requires an object receiver");
      }
      if (typeof reducer !== "function") {
        closeIfPossible(this);
        throw new TypeError("reducer must be callable");
      }
      const record = getIteratorDirect(this);
      let index = 0;
      let accumulator;
      let initialized = arguments.length > 1;
      if (initialized) {
        accumulator = initialValue;
      }
      while (true) {
        const step = iteratorStep(record);
        if (step.done) {
          if (!initialized) {
            throw new TypeError("Reduce of empty iterator with no initial value");
          }
          return accumulator;
        }
        if (!initialized) {
          initialized = true;
          accumulator = step.value;
          index += 1;
          continue;
        }
        try {
          accumulator = reducer(accumulator, step.value, index++);
        } catch (error) {
          return iteratorClose(record, error);
        }
      }
    },
    toArray() {
      if (!isObjectLike(this)) {
        throw new TypeError("Iterator.prototype.toArray requires an object receiver");
      }
      const record = getIteratorDirect(this);
      const values = [];
      while (true) {
        const step = iteratorStep(record);
        if (step.done) {
          return values;
        }
        values.push(step.value);
      }
    },
    forEach(fn) {
      if (!isObjectLike(this)) {
        throw new TypeError("Iterator.prototype.forEach requires an object receiver");
      }
      if (typeof fn !== "function") {
        closeIfPossible(this);
        throw new TypeError("callback must be callable");
      }
      const record = getIteratorDirect(this);
      let index = 0;
      while (true) {
        const step = iteratorStep(record);
        if (step.done) {
          return undefined;
        }
        try {
          fn(step.value, index++);
        } catch (error) {
          return iteratorClose(record, error);
        }
      }
    },
    some(predicate) {
      if (!isObjectLike(this)) {
        throw new TypeError("Iterator.prototype.some requires an object receiver");
      }
      if (typeof predicate !== "function") {
        closeIfPossible(this);
        throw new TypeError("predicate must be callable");
      }
      const record = getIteratorDirect(this);
      let index = 0;
      while (true) {
        const step = iteratorStep(record);
        if (step.done) {
          return false;
        }
        let result;
        try {
          result = predicate(step.value, index++);
        } catch (error) {
          return iteratorClose(record, error);
        }
        if (result) {
          iteratorReturn(record);
          return true;
        }
      }
    },
    every(predicate) {
      if (!isObjectLike(this)) {
        throw new TypeError("Iterator.prototype.every requires an object receiver");
      }
      if (typeof predicate !== "function") {
        closeIfPossible(this);
        throw new TypeError("predicate must be callable");
      }
      const record = getIteratorDirect(this);
      let index = 0;
      while (true) {
        const step = iteratorStep(record);
        if (step.done) {
          return true;
        }
        let result;
        try {
          result = predicate(step.value, index++);
        } catch (error) {
          return iteratorClose(record, error);
        }
        if (!result) {
          iteratorReturn(record);
          return false;
        }
      }
    },
    find(predicate) {
      if (!isObjectLike(this)) {
        throw new TypeError("Iterator.prototype.find requires an object receiver");
      }
      if (typeof predicate !== "function") {
        closeIfPossible(this);
        throw new TypeError("predicate must be callable");
      }
      const record = getIteratorDirect(this);
      let index = 0;
      while (true) {
        const step = iteratorStep(record);
        if (step.done) {
          return undefined;
        }
        let result;
        try {
          result = predicate(step.value, index++);
        } catch (error) {
          return iteratorClose(record, error);
        }
        if (result) {
          iteratorReturn(record);
          return step.value;
        }
      }
    }
  };

  const iteratorSymbolMethods = {
    [Symbol.iterator]() {
      return this;
    }
  };

  const disposeMethods = typeof Symbol.dispose === "symbol"
    ? {
        [Symbol.dispose]() {
          if (!isObjectLike(this)) {
            throw new TypeError("Iterator.prototype[Symbol.dispose] requires an object receiver");
          }
          const method = getMethod(this, "return");
          if (method !== undefined) {
            method.call(this);
          }
          return undefined;
        }
      }
    : null;

  function Iterator() {
    if (new.target === undefined || new.target === Iterator) {
      throw new TypeError("Iterator is abstract");
    }
  }

  Object.defineProperty(Iterator, "prototype", {
    value: IteratorPrototype,
    writable: false,
    enumerable: false,
    configurable: false
  });

  const iteratorStaticMethods = {
    from(value) {
      return iteratorFrom(value);
    },
    concat() {
      const sources = [];
      for (let index = 0; index < arguments.length; index += 1) {
        const item = arguments[index];
        if (!isObjectLike(item)) {
          throw new TypeError("Iterator.concat arguments must be iterable objects");
        }
        const method = getMethod(item, Symbol.iterator);
        if (method === undefined) {
          throw new TypeError("Iterator.concat arguments must be iterable objects");
        }
        sources.push({ item, method, iterator: null });
      }

      let current = null;
      return createIteratorHelper(
        function nextImpl() {
          while (true) {
            if (current === null) {
              if (!sources.length) {
                return { value: undefined, done: true };
              }
              const source = sources.shift();
              const iterator = source.method.call(source.item);
              if (!isObjectLike(iterator)) {
                throw new TypeError("Iterator.concat iterable produced a non-object iterator");
              }
              current = getIteratorDirect(iterator);
            }

            const step = iteratorStep(current);
            if (!step.done) {
              return step;
            }
            current = null;
          }
        },
        function returnImpl() {
          if (current !== null) {
            return iteratorReturn(current);
          }
          return { value: undefined, done: true };
        }
      );
    },
    zip(iterables, options) {
      if (!isObjectLike(iterables)) {
        throw new TypeError("Iterator.zip requires an iterable object");
      }
      const outerMethod = getMethod(iterables, Symbol.iterator);
      if (outerMethod === undefined) {
        throw new TypeError("Iterator.zip requires an iterable object");
      }
      const optionsObject = options === undefined ? undefined : Object(options);
      const mode = optionsObject === undefined
        ? "shortest"
        : optionsObject.mode === undefined
        ? "shortest"
        : optionsObject.mode;
      if (mode !== "shortest" && mode !== "longest" && mode !== "strict") {
        throw new TypeError("Iterator.zip mode must be shortest, longest, or strict");
      }
      const padding = mode === "longest" && optionsObject !== undefined && isObjectLike(optionsObject.padding)
        ? optionsObject.padding
        : [];
      const values = Array.from(iterables);
      const iterators = values.map(value => getIteratorDirect(iteratorFrom(value)));
      const finished = new Array(iterators.length).fill(false);
      return createIteratorHelper(
        function nextImpl() {
          if (!iterators.length) {
            return { value: undefined, done: true };
          }
          const results = new Array(iterators.length);
          let doneCount = 0;
          for (let index = 0; index < iterators.length; index += 1) {
            if (finished[index]) {
              doneCount += 1;
              results[index] = padding[index];
              continue;
            }
            const step = iteratorStep(iterators[index]);
            if (step.done) {
              finished[index] = true;
              doneCount += 1;
              if (mode === "shortest") {
                for (let closeIndex = iterators.length - 1; closeIndex >= 0; closeIndex -= 1) {
                  if (closeIndex !== index && !finished[closeIndex]) {
                    iteratorReturn(iterators[closeIndex]);
                  }
                }
                return { value: undefined, done: true };
              }
              if (mode === "strict" && doneCount !== iterators.length) {
                for (let closeIndex = iterators.length - 1; closeIndex >= 0; closeIndex -= 1) {
                  if (closeIndex !== index && !finished[closeIndex]) {
                    iteratorReturn(iterators[closeIndex]);
                  }
                }
                throw new TypeError("Iterator.zip strict mode requires equal lengths");
              }
              results[index] = padding[index];
            } else {
              results[index] = step.value;
            }
          }
          if (doneCount === iterators.length) {
            return { value: undefined, done: true };
          }
          return { value: results, done: false };
        },
        function returnImpl() {
          for (let index = iterators.length - 1; index >= 0; index -= 1) {
            if (!finished[index]) {
              iteratorReturn(iterators[index]);
            }
          }
          return { value: undefined, done: true };
        }
      );
    },
    zipKeyed(iterables, options) {
      if (!isObjectLike(iterables)) {
        throw new TypeError("Iterator.zipKeyed requires an object");
      }
      const optionsObject = options === undefined ? undefined : Object(options);
      const mode = optionsObject === undefined
        ? "shortest"
        : optionsObject.mode === undefined
        ? "shortest"
        : optionsObject.mode;
      if (mode !== "shortest" && mode !== "longest" && mode !== "strict") {
        throw new TypeError("Iterator.zipKeyed mode must be shortest, longest, or strict");
      }
      const padding = mode === "longest" && optionsObject !== undefined && isObjectLike(optionsObject.padding)
        ? optionsObject.padding
        : Object.create(null);
      const keys = Reflect.ownKeys(iterables).filter(function (key) {
        const descriptor = Object.getOwnPropertyDescriptor(iterables, key);
        if (!descriptor || descriptor.enumerable !== true) {
          return false;
        }
        return iterables[key] !== undefined;
      });
      const iterators = keys.map(key => getIteratorDirect(iteratorFrom(iterables[key])));
      const finished = new Array(iterators.length).fill(false);
      return createIteratorHelper(
        function nextImpl() {
          if (!iterators.length) {
            return { value: undefined, done: true };
          }
          const results = Object.create(null);
          let doneCount = 0;
          for (let index = 0; index < iterators.length; index += 1) {
            const key = keys[index];
            if (finished[index]) {
              doneCount += 1;
              results[key] = padding[key];
              continue;
            }
            const step = iteratorStep(iterators[index]);
            if (step.done) {
              finished[index] = true;
              doneCount += 1;
              if (mode === "shortest") {
                for (let closeIndex = iterators.length - 1; closeIndex >= 0; closeIndex -= 1) {
                  if (closeIndex !== index && !finished[closeIndex]) {
                    iteratorReturn(iterators[closeIndex]);
                  }
                }
                return { value: undefined, done: true };
              }
              if (mode === "strict" && doneCount !== iterators.length) {
                for (let closeIndex = iterators.length - 1; closeIndex >= 0; closeIndex -= 1) {
                  if (closeIndex !== index && !finished[closeIndex]) {
                    iteratorReturn(iterators[closeIndex]);
                  }
                }
                throw new TypeError("Iterator.zipKeyed strict mode requires equal lengths");
              }
              results[key] = padding[key];
            } else {
              results[key] = step.value;
            }
          }
          if (doneCount === iterators.length) {
            return { value: undefined, done: true };
          }
          return { value: results, done: false };
        },
        function returnImpl() {
          for (let index = iterators.length - 1; index >= 0; index -= 1) {
            if (!finished[index]) {
              iteratorReturn(iterators[index]);
            }
          }
          return { value: undefined, done: true };
        }
      );
    }
  };

  Object.defineProperty(globalThis, "Iterator", {
    value: Iterator,
    writable: true,
    enumerable: false,
    configurable: true
  });

  Object.defineProperty(IteratorPrototype, "constructor", {
    get() {
      return Iterator;
    },
    set(value) {
      Object.defineProperty(this, "constructor", {
        value,
        writable: true,
        enumerable: false,
        configurable: true
      });
    },
    enumerable: false,
    configurable: true
  });

  Object.defineProperty(IteratorPrototype, Symbol.toStringTag, {
    get() {
      return "Iterator";
    },
    set(value) {
      Object.defineProperty(this, Symbol.toStringTag, {
        value,
        writable: true,
        enumerable: false,
        configurable: true
      });
    },
    enumerable: false,
    configurable: true
  });

  Object.defineProperty(IteratorPrototype, Symbol.iterator, {
    value: iteratorSymbolMethods[Symbol.iterator],
    writable: true,
    enumerable: false,
    configurable: true
  });

  if (disposeMethods) {
    Object.defineProperty(IteratorPrototype, Symbol.dispose, {
      value: disposeMethods[Symbol.dispose],
      writable: true,
      enumerable: false,
      configurable: true
    });
  }

  for (const key of Object.keys(iteratorPrototypeMethods)) {
    Object.defineProperty(IteratorPrototype, key, {
      value: iteratorPrototypeMethods[key],
      writable: true,
      enumerable: false,
      configurable: true
    });
  }

  for (const key of Object.keys(iteratorStaticMethods)) {
    Object.defineProperty(Iterator, key, {
      value: iteratorStaticMethods[key],
      writable: true,
      enumerable: false,
      configurable: true
    });
  }

  Object.defineProperty(Iterator.zip, "length", {
    value: 1,
    writable: false,
    enumerable: false,
    configurable: true
  });

  Object.defineProperty(Iterator.zipKeyed, "length", {
    value: 1,
    writable: false,
    enumerable: false,
    configurable: true
  });
})();
"#;

fn install_host_globals(context: &mut Context, argv: &[String]) -> Result<(), ExecutionError> {
    context
        .register_global_builtin_callable(
            js_string!("print"),
            1,
            NativeFunction::from_fn_ptr(host_print),
        )
        .map_err(|err| format_js_error(err, context))?;
    context
        .register_global_builtin_callable(js_string!("gc"), 0, NativeFunction::from_fn_ptr(host_gc))
        .map_err(|err| format_js_error(err, context))?;
    context
        .register_global_builtin_callable(
            js_string!("__porfDetachArrayBuffer"),
            1,
            NativeFunction::from_fn_ptr(host_detach_array_buffer),
        )
        .map_err(|err| format_js_error(err, context))?;

    let argv_literal = json_string_array(argv);
    let bootstrap = format!(
        r#"
globalThis.__porfArgv = {argv_literal};
globalThis.$262 = {{
  global: globalThis,
  gc: gc,
  getGlobal(name) {{
    return globalThis[name];
  }},
  evalScript(code) {{
    return (0, eval)(String(code));
  }},
  createRealm() {{
    return {{
      global: globalThis,
      evalScript(code) {{
        return (0, eval)(String(code));
      }},
      destroy() {{}},
      getGlobal(name) {{
        return globalThis[name];
      }}
    }};
  }},
  detachArrayBuffer(buffer) {{
    return __porfDetachArrayBuffer(buffer);
  }},
  destroy() {{}},
  agent: {{
    start() {{
      throw new Test262Error('agent threads are not supported in porffor-spec-exec');
    }},
    broadcast() {{
      throw new Test262Error('agent threads are not supported in porffor-spec-exec');
    }},
    receiveBroadcast() {{
      throw new Test262Error('agent threads are not supported in porffor-spec-exec');
    }},
    report() {{}},
    getReport() {{
      return null;
    }},
    sleep() {{}},
    monotonicNow() {{
      return Date.now();
    }},
    leaving() {{}}
  }}
}};
"#,
    );

    context
        .eval(Source::from_bytes(bootstrap.as_bytes()))
        .map_err(|err| format_js_error(err, context))?;
    context
        .eval(Source::from_bytes(DATE_TIME_FORMAT_SHIM.as_bytes()))
        .map_err(|err| format_js_error(err, context))?;
    context
        .eval(Source::from_bytes(ITERATOR_HELPERS_SHIM.as_bytes()))
        .map_err(|err| format_js_error(err, context))?;
    context
        .run_jobs()
        .map_err(|err| format_js_error(err, context))?;
    Ok(())
}

fn host_print(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let rendered = args
        .iter()
        .map(|value| {
            value
                .to_string(context)
                .map(|text| text.to_std_string_escaped())
        })
        .collect::<Result<Vec<_>, _>>()?
        .join(" ");
    if !rendered.is_empty() {
        println!("{rendered}");
    }
    Ok(JsValue::undefined())
}

fn host_gc(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn host_detach_array_buffer(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let Some(buffer) = args.first().and_then(JsValue::as_object) else {
        return Err(JsNativeError::typ()
            .with_message("detachArrayBuffer expects an ArrayBuffer")
            .into());
    };

    let buffer = JsArrayBuffer::from_object(buffer.clone())?;
    buffer.detach(&JsValue::undefined())?;
    Ok(JsValue::undefined())
}

fn format_js_error(err: impl core::fmt::Display, _context: &mut Context) -> ExecutionError {
    ExecutionError::new(err.to_string())
}

fn format_opaque_error(value: JsValue, context: &mut Context) -> ExecutionError {
    let rendered = value
        .to_string(context)
        .map(|text| text.to_std_string_escaped())
        .unwrap_or_else(|_| value.display().to_string());
    ExecutionError::new(rendered)
}

fn json_string_array(values: &[String]) -> String {
    let mut out = String::from("[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push('"');
        out.push_str(&json_escape(value));
        out.push('"');
    }
    out.push(']');
    out
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executes_simple_script() {
        execute_script("globalThis.answer = 40 + 2;", Some("script.js"), &[])
            .expect("spec exec script should run");
    }

    #[test]
    fn executes_simple_module() {
        execute_module(
            "export const value = 1;",
            Some("module.mjs"),
            ModuleHostConfig::default(),
            &[],
        )
        .expect("spec exec module should run");
    }

    #[test]
    fn detaches_array_buffer_via_host_hook() {
        execute_script(
            r#"
            const buffer = new ArrayBuffer(8);
            if (buffer.byteLength !== 8) {
              throw new Error("buffer should start attached");
            }
            __porfDetachArrayBuffer(buffer);
            if (buffer.byteLength !== 0) {
              throw new Error("buffer should be detached");
            }
            "#,
            Some("detach.js"),
            &[],
        )
        .expect("host detachArrayBuffer should detach real ArrayBuffers");
    }

    #[test]
    fn installs_date_time_format_shim() {
        execute_script(
            r#"
            const formatter = new Intl.DateTimeFormat("en", {
              calendar: "islamicc",
              timeZone: "Etc/UTC",
              dayPeriod: "long"
            });
            const resolved = formatter.resolvedOptions();
            if (resolved.calendar !== "islamic-civil") {
              throw new Error("calendar should be canonicalized");
            }
            if (resolved.timeZone !== "Etc/UTC") {
              throw new Error("timeZone should be preserved");
            }
            if (typeof formatter.format !== "function") {
              throw new Error("format getter should produce a function");
            }
            "#,
            Some("dtf.js"),
            &[],
        )
        .expect("DateTimeFormat shim should be installed");
    }

    #[test]
    fn date_time_format_shim_exposes_builtin_shapes() {
        execute_script(
            r#"
            const formatGetter = Object.getOwnPropertyDescriptor(Intl.DateTimeFormat.prototype, "format").get;
            if (Object.getPrototypeOf(Intl.DateTimeFormat) !== Function.prototype) {
              throw new Error("DateTimeFormat should inherit from Function.prototype");
            }
            const protoDesc = Object.getOwnPropertyDescriptor(Intl.DateTimeFormat, "prototype");
            if (!protoDesc || protoDesc.writable || protoDesc.configurable) {
              throw new Error("DateTimeFormat.prototype should be fixed");
            }
            if (formatGetter.name !== "get format") {
              throw new Error("format getter should have built-in name");
            }
            if (Object.prototype.hasOwnProperty.call(formatGetter, "prototype")) {
              throw new Error("format getter should not expose prototype");
            }
            const bound = new Intl.DateTimeFormat("en-US").format;
            if (Object.prototype.hasOwnProperty.call(bound, "prototype")) {
              throw new Error("bound format should not expose prototype");
            }
            "#,
            Some("dtf-shape.js"),
            &[],
        )
        .expect("DateTimeFormat shim should expose built-in shapes");
    }

    #[test]
    fn date_time_format_shim_handles_null_options_and_to_locale_defaults() {
        execute_script(
            r#"
            let threw = false;
            try {
              new Intl.DateTimeFormat("en-US", null);
            } catch (err) {
              threw = err instanceof TypeError;
            }
            if (!threw) {
              throw new Error("null options should throw");
            }
            const formatted = new Date(0).toLocaleString("en-US");
            const reference = new Intl.DateTimeFormat("en-US", {
              year: "numeric",
              month: "numeric",
              day: "numeric",
              hour: "numeric",
              minute: "numeric",
              second: "numeric"
            }).format(new Date(0));
            if (formatted !== reference) {
              throw new Error("toLocaleString should use datetime defaults");
            }
            "#,
            Some("dtf-defaults.js"),
            &[],
        )
        .expect("DateTimeFormat shim should apply null handling and locale defaults");
    }

    #[test]
    fn date_time_format_shim_supports_legacy_constructed_symbol_unwrap() {
        execute_script(
            r#"
            const object = new Intl.DateTimeFormat();
            const legacy = Intl.DateTimeFormat.call(object);
            let seen = null;
            const proxy = new Proxy(legacy, {
              get(target, property, receiver) {
                seen = property;
                return Reflect.get(target, property, receiver);
              }
            });
            Intl.DateTimeFormat.prototype.resolvedOptions.call(proxy);
            if (typeof seen !== "symbol" || seen.description !== "IntlLegacyConstructedSymbol") {
              throw new Error("legacy constructed symbol should be observed during unwrap");
            }
            "#,
            Some("dtf-legacy.js"),
            &[],
        )
        .expect("DateTimeFormat shim should support legacy constructed symbol unwrap");
    }

    #[test]
    fn date_time_format_shim_ignores_time_zone_name_for_plain_temporal_values() {
        execute_script(
            r#"
            const plainDateTime = new Temporal.PlainDateTime(2026, 1, 5, 11, 22);
            const formatter = new Intl.DateTimeFormat("en-US", {
              hour: "numeric",
              minute: "2-digit",
              timeZoneName: "long"
            });
            const formatted = formatter.format(plainDateTime);
            if (formatted.includes("UTC")) {
              throw new Error("plain temporal values should not include time zone names");
            }
            const parts = formatter.formatToParts(plainDateTime);
            if (parts.some(part => part.type === "timeZoneName")) {
              throw new Error("plain temporal values should not expose timeZoneName parts");
            }
            "#,
            Some("dtf-temporal-tzname.js"),
            &[],
        )
        .expect("DateTimeFormat shim should suppress timeZoneName for plain Temporal values");
    }

    #[test]
    fn date_time_format_shim_filters_unsupported_locales_and_hc_extension() {
        execute_script(
            r#"
            const defaultLocale = new Intl.DateTimeFormat().resolvedOptions().locale;
            const supported = Intl.DateTimeFormat.supportedLocalesOf([defaultLocale, "zxx"]);
            if (supported.length !== 1 || supported[0] !== defaultLocale) {
              throw new Error("supportedLocalesOf should filter unsupported locales");
            }
            const withHourCycle = defaultLocale + "-u-hc-h11";
            const resolved = new Intl.DateTimeFormat(withHourCycle, {
              hour: "2-digit",
              hourCycle: "h23"
            }).resolvedOptions();
            if (resolved.locale.includes("-u-hc-")) {
              throw new Error("resolved locale should drop hc extension when hourCycle option is present");
            }
            "#,
            Some("dtf-supported-locales.js"),
            &[],
        )
        .expect("DateTimeFormat shim should filter unsupported locales and drop hc extension");
    }

    #[test]
    fn installs_iterator_helpers_shim() {
        execute_script(
            r#"
            const iterator = Iterator.from([1, 2, 3]).map(x => x * 2).drop(1);
            const values = iterator.toArray();
            if (values.length !== 2 || values[0] !== 4 || values[1] !== 6) {
              throw new Error("iterator helpers should transform and collect values");
            }
            if (!(iterator instanceof Iterator)) {
              throw new Error("iterator helper result should inherit from Iterator");
            }
            if (typeof Iterator.zip !== "function" || typeof Iterator.zipKeyed !== "function") {
              throw new Error("Iterator static helpers should exist");
            }
            "#,
            Some("iterator-helpers.js"),
            &[],
        )
        .expect("Iterator helper shim should be installed");
    }

    #[test]
    fn iterator_helpers_support_concat_and_zip() {
        execute_script(
            r#"
            const concatValues = Iterator.concat([1, 2], new Set([3])).toArray();
            if (concatValues.join(",") !== "1,2,3") {
              throw new Error("Iterator.concat should flatten iterables in order");
            }
            const zipped = Iterator.zip([[1, 2], ["a", "b"]], { mode: "shortest" }).toArray();
            if (JSON.stringify(zipped) !== JSON.stringify([[1, "a"], [2, "b"]])) {
              throw new Error("Iterator.zip should zip iterables");
            }
            const keyed = Iterator.zipKeyed({ left: [1, 2], right: ["a", "b"] }, { mode: "shortest" }).toArray();
            if (JSON.stringify(keyed) !== JSON.stringify([{ left: 1, right: "a" }, { left: 2, right: "b" }])) {
              throw new Error("Iterator.zipKeyed should zip keyed iterables");
            }
            "#,
            Some("iterator-zip.js"),
            &[],
        )
        .expect("Iterator helpers should support concat and zip");
    }

    #[test]
    fn installs_intl_supported_helpers() {
        execute_script(
            r#"
            if (typeof Intl.supportedValuesOf !== "function") {
              throw new Error("supportedValuesOf should exist");
            }
            if (typeof Intl.DateTimeFormat.supportedLocalesOf !== "function") {
              throw new Error("supportedLocalesOf should exist");
            }
            const locales = Intl.DateTimeFormat.supportedLocalesOf(["en-US", "fr"]);
            if (locales.length !== 2) {
              throw new Error("supportedLocalesOf should return requested locales");
            }
            const values = Intl.supportedValuesOf("timeZone");
            if (!Array.isArray(values)) {
              throw new Error("supportedValuesOf should return an array");
            }
            "#,
            Some("intl-helpers.js"),
            &[],
        )
        .expect("Intl helpers should be installed");
    }

    #[test]
    fn promise_jobs_flush_before_return() {
        execute_script(
            "globalThis.done = false; Promise.resolve().then(() => { globalThis.done = true; });",
            Some("promise.js"),
            &[],
        )
        .expect("script with promises should run");
    }

    #[test]
    fn resolves_sibling_fixture_module_from_test_path() {
        let root =
            std::env::temp_dir().join(format!("porffor-spec-exec-modules-{}", std::process::id()));
        std::fs::create_dir_all(&root).expect("temp module dir should exist");
        let test_path = root.join("module-import-resolution.js");
        let test_path_string = test_path.to_str().expect("utf-8 test path").to_string();
        let fixture_path = root.join("module-import-resolution_FIXTURE.js");
        std::fs::write(
            &fixture_path,
            "export default 42;\nexport const x = 'named';\nexport const y = 39;\n",
        )
        .expect("fixture module should be written");

        execute_module(
            "import foo from './module-import-resolution_FIXTURE.js';\nimport { x, y } from './module-import-resolution_FIXTURE.js';\nif (foo !== 42 || x !== 'named' || y !== 39) throw new Error('bad import');\n",
            Some(&test_path_string),
            ModuleHostConfig {
                module_root: Some(root.clone()),
                test_path: Some(test_path),
            },
            &[],
        )
        .expect("module host should resolve sibling fixture");
    }
}
