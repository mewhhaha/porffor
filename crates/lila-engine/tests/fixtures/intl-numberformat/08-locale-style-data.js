function check(value, message) { if (!value) throw new Error(message); }
check(new Intl.NumberFormat("en-US").format(1234.5) === "1,234.5", "English grouping");
check(new Intl.NumberFormat("de-DE").format(1234.5) === "1.234,5", "German symbols");
check(new Intl.NumberFormat("en-IN").format(1234567) === "12,34,567", "Indian grouping");
check(new Intl.NumberFormat("en-US-u-nu-arab", { useGrouping: false }).format(123) === "١٢٣", "numbering extension");
check(new Intl.NumberFormat("en-US", { style: "currency", currency: "USD" }).format(12.5) === "$12.50", "currency pattern");
check(new Intl.NumberFormat("en-US", { style: "percent" }).format(0.125) === "13%", "percent scaling");
check(new Intl.NumberFormat("en-US", { style: "unit", unit: "meter", unitDisplay: "long" }).format(2) === "2 meters", "unit plural");
check(new Intl.NumberFormat("en-US", { notation: "compact" }).format(1200) === "1.2K", "compact pattern");
check(new Intl.NumberFormat("ja-JP", { style: "unit", unit: "kilometer-per-hour", unitDisplay: "long" }).format(-987) === "時速 -987 キロメートル", "signed number inside unit pattern");
var taiwan = new Intl.NumberFormat("zh-TW", { style: "unit", unit: "kilometer-per-hour", unitDisplay: "long" });
check(taiwan.resolvedOptions().locale === "zh-TW", "script-elided available locale");
check(taiwan.format(-987) === "每小時 -987 公里", "zh-TW uses its likely Traditional data");
print("ok locale style data");
