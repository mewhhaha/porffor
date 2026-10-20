//! Immutable Umm al-Qura native source authority for the emitted arithmetic.
//! Literal years, month flags and ISO starts are copied from
//! vendor/icu_calendar-2.0.6/src/cal/hijri/ummalqura_data.rs, SHA256
//! 9e489ef9fd0fc7205213ca3c7ed1edb5523d8cf89d66d7e626a859b49dcbbe30.
//! Upstream: ICU4X commit 31e2bfa8e39e069dcef6de3f6914c5d722e90d00,
//! components/calendar/src/cal/hijri/ummalqura_data.rs.
//! The corresponding vendor copyright and permission notice follows below.

// UNICODE LICENSE V3
//
// COPYRIGHT AND PERMISSION NOTICE
//
// Copyright © 2020-2024 Unicode, Inc.
//
// NOTICE TO USER: Carefully read the following legal agreement. BY
// DOWNLOADING, INSTALLING, COPYING OR OTHERWISE USING DATA FILES, AND/OR
// SOFTWARE, YOU UNEQUIVOCALLY ACCEPT, AND AGREE TO BE BOUND BY, ALL OF THE
// TERMS AND CONDITIONS OF THIS AGREEMENT. IF YOU DO NOT AGREE, DO NOT
// DOWNLOAD, INSTALL, COPY, DISTRIBUTE OR USE THE DATA FILES OR SOFTWARE.
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of data files and any associated documentation (the "Data Files") or
// software and any associated documentation (the "Software") to deal in the
// Data Files or Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, and/or sell
// copies of the Data Files or Software, and to permit persons to whom the
// Data Files or Software are furnished to do so, provided that either (a)
// this copyright and permission notice appear with all copies of the Data
// Files or Software, or (b) this copyright and permission notice appear in
// associated Documentation.
//
// THE DATA FILES AND SOFTWARE ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY
// KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT OF
// THIRD PARTY RIGHTS.
//
// IN NO EVENT SHALL THE COPYRIGHT HOLDER OR HOLDERS INCLUDED IN THIS NOTICE
// BE LIABLE FOR ANY CLAIM, OR ANY SPECIAL INDIRECT OR CONSEQUENTIAL DAMAGES,
// OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS,
// WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION,
// ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THE DATA
// FILES OR SOFTWARE.
//
// Except as contained in this notice, the name of a copyright holder shall
// not be used in advertising or otherwise to promote the sale, use or other
// dealings in these Data Files or Software without prior written
// authorization of the copyright holder.
//
// SPDX-License-Identifier: Unicode-3.0
//
// —
//
// Portions of ICU4X may have been adapted from ICU4C and/or ICU4J.
// ICU 1.8.1 to ICU 57.1 © 1995-2016 International Business Machines Corporation and others.

use super::super::temporal_plain_date::TemporalIslamicCalendar;

const FIRST_YEAR: i64 = 1300;
const YEAR_COUNT: usize = 301;
const MONTH_MASK: i64 = 0x0fff;

/// Only the checked literal constructor can mint a row. The emitted lookup
/// consumes this packed authority; it cannot choose an unrelated start/mask.
#[derive(Clone, Copy)]
pub(super) struct UmmAlQuraYearInfo {
    year: i64,
    packed: i64,
}

impl UmmAlQuraYearInfo {
    const fn new(year: i64, long_months: [bool; 12], iso: [i64; 3]) -> Self {
        assert!(year >= FIRST_YEAR && year < FIRST_YEAR + YEAR_COUNT as i64);
        let start = checked_iso_epoch_day(iso[0], iso[1], iso[2]);
        assert!(start >= (i64::MIN >> 12) && start <= (i64::MAX >> 12));
        let mut mask = 0i64;
        let mut month = 0;
        while month < 12 {
            if long_months[month] {
                mask |= 1i64 << month;
            }
            month += 1;
        }
        let days = 348 + mask.count_ones() as i64;
        assert!(days == 354 || days == 355);
        let packed = (start << 12) | mask;
        assert!((packed >> 12) == start && (packed & MONTH_MASK) == mask);
        Self { year, packed }
    }

    pub(super) const fn calendar_year(self) -> i64 {
        self.year
    }

    pub(super) const fn start_day(self) -> i64 {
        self.packed >> 12
    }

    pub(super) const fn month_mask(self) -> i64 {
        self.packed & MONTH_MASK
    }

    pub(super) const fn packed(self) -> i64 {
        self.packed
    }

    pub(super) const fn days_in_year(self) -> i64 {
        348 + self.month_mask().count_ones() as i64
    }
}

/// This native conversion checks committed ISO literals only. User-supplied
/// Temporal fields continue through the existing emitted conversion/owners.
const fn checked_iso_epoch_day(year: i64, month: i64, day: i64) -> i64 {
    assert!(year >= 1 && year <= 9999);
    assert!(month >= 1 && month <= 12);
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_length = match month {
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => panic!("invalid committed ISO month"),
    };
    assert!(day >= 1 && day <= month_length);
    let adjusted_year = if month <= 2 { year - 1 } else { year };
    let era = adjusted_year / 400;
    let year_of_era = adjusted_year - era * 400;
    let march_month = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * march_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146097 + day_of_era - 719468
}

/// Positive table boundary years make the native division Euclidean. Both
/// ends must meet the actual Civil fallback, not an independently chosen date.
const fn civil_year_start(year: i64) -> i64 {
    assert!(year >= 1);
    TemporalIslamicCalendar::Civil.epoch_day() + 354 * (year - 1) + (3 + 11 * year) / 30
}

const fn validate_years(rows: [UmmAlQuraYearInfo; YEAR_COUNT]) -> [UmmAlQuraYearInfo; YEAR_COUNT] {
    let mut index = 0;
    while index < YEAR_COUNT {
        let row = rows[index];
        assert!(row.calendar_year() == FIRST_YEAR + index as i64);
        assert!(row.days_in_year() == 354 || row.days_in_year() == 355);
        if index > 0 {
            let previous = rows[index - 1];
            assert!(row.start_day() == previous.start_day() + previous.days_in_year());
        }
        index += 1;
    }
    assert!(rows[0].start_day() == civil_year_start(FIRST_YEAR));
    let last = rows[YEAR_COUNT - 1];
    assert!(
        last.start_day() + last.days_in_year() == civil_year_start(FIRST_YEAR + YEAR_COUNT as i64)
    );
    rows
}

#[rustfmt::skip]
pub(super) const UMMALQURA_YEARS: [UmmAlQuraYearInfo; YEAR_COUNT] = validate_years({
    let l = true;
    let s = false;
    [
        UmmAlQuraYearInfo::new(1300, [l, s, l, s, l, s, l, s, l, s, l, s], [1882, 11, 12]),
        UmmAlQuraYearInfo::new(1301, [l, l, s, l, s, l, s, l, s, l, s, s], [1883, 11, 1]),
        UmmAlQuraYearInfo::new(1302, [l, l, l, s, l, l, s, s, l, s, s, l], [1884, 10, 20]),
        UmmAlQuraYearInfo::new(1303, [s, l, l, s, l, l, s, l, s, l, s, s], [1885, 10, 10]),
        UmmAlQuraYearInfo::new(1304, [s, l, l, s, l, l, l, s, l, s, l, s], [1886, 9, 29]),
        UmmAlQuraYearInfo::new(1305, [s, s, l, l, s, l, l, s, l, l, s, s], [1887, 9, 19]),
        UmmAlQuraYearInfo::new(1306, [l, s, l, s, l, s, l, s, l, l, s, l], [1888, 9, 7]),
        UmmAlQuraYearInfo::new(1307, [s, l, s, l, s, l, s, l, s, l, s, l], [1889, 8, 28]),
        UmmAlQuraYearInfo::new(1308, [s, l, l, s, l, s, l, s, l, s, s, l], [1890, 8, 17]),
        UmmAlQuraYearInfo::new(1309, [s, l, l, l, l, s, s, l, s, s, l, s], [1891, 8, 6]),
        UmmAlQuraYearInfo::new(1310, [l, s, l, l, l, s, l, s, l, s, s, l], [1892, 7, 25]),
        UmmAlQuraYearInfo::new(1311, [s, l, s, l, l, l, s, l, s, l, s, s], [1893, 7, 15]),
        UmmAlQuraYearInfo::new(1312, [l, s, l, s, l, l, s, l, l, s, l, s], [1894, 7, 4]),
        UmmAlQuraYearInfo::new(1313, [s, l, s, l, s, l, s, l, l, l, s, s], [1895, 6, 24]),
        UmmAlQuraYearInfo::new(1314, [l, l, s, l, s, s, l, s, l, l, s, l], [1896, 6, 12]),
        UmmAlQuraYearInfo::new(1315, [s, l, l, s, l, s, s, l, s, l, s, l], [1897, 6, 2]),
        UmmAlQuraYearInfo::new(1316, [s, l, l, l, s, l, s, s, l, s, l, s], [1898, 5, 22]),
        UmmAlQuraYearInfo::new(1317, [l, s, l, l, s, l, s, l, s, l, s, s], [1899, 5, 11]),
        UmmAlQuraYearInfo::new(1318, [l, s, l, l, s, l, l, s, l, s, l, s], [1900, 4, 30]),
        UmmAlQuraYearInfo::new(1319, [s, l, s, l, l, s, l, s, l, l, s, l], [1901, 4, 20]),
        UmmAlQuraYearInfo::new(1320, [s, l, s, s, l, s, l, s, l, l, l, s], [1902, 4, 10]),
        UmmAlQuraYearInfo::new(1321, [l, s, l, s, s, l, s, s, l, l, l, l], [1903, 3, 30]),
        UmmAlQuraYearInfo::new(1322, [s, l, s, l, s, s, s, l, s, l, l, l], [1904, 3, 19]),
        UmmAlQuraYearInfo::new(1323, [s, l, l, s, l, s, s, s, l, s, l, l], [1905, 3, 8]),
        UmmAlQuraYearInfo::new(1324, [s, l, l, s, l, s, l, s, s, l, s, l], [1906, 2, 25]),
        UmmAlQuraYearInfo::new(1325, [l, s, l, s, l, l, s, l, s, l, s, l], [1907, 2, 14]),
        UmmAlQuraYearInfo::new(1326, [s, s, l, s, l, l, s, l, s, l, l, s], [1908, 2, 4]),
        UmmAlQuraYearInfo::new(1327, [l, s, s, l, s, l, s, l, l, s, l, l], [1909, 1, 23]),
        UmmAlQuraYearInfo::new(1328, [s, l, s, s, l, s, s, l, l, l, s, l], [1910, 1, 13]),
        UmmAlQuraYearInfo::new(1329, [l, s, l, s, s, l, s, s, l, l, s, l], [1911, 1, 2]),
        UmmAlQuraYearInfo::new(1330, [l, l, s, l, s, s, l, s, s, l, l, s], [1911, 12, 22]),
        UmmAlQuraYearInfo::new(1331, [l, l, s, l, l, s, s, l, s, l, s, l], [1912, 12, 10]),
        UmmAlQuraYearInfo::new(1332, [s, l, s, l, l, s, l, s, l, l, s, s], [1913, 11, 30]),
        UmmAlQuraYearInfo::new(1333, [l, s, s, l, l, s, l, l, s, l, l, s], [1914, 11, 19]),
        UmmAlQuraYearInfo::new(1334, [s, s, l, s, l, s, l, l, l, s, l, s], [1915, 11, 9]),
        UmmAlQuraYearInfo::new(1335, [l, s, l, s, s, l, s, l, l, s, l, l], [1916, 10, 28]),
        UmmAlQuraYearInfo::new(1336, [s, l, s, l, s, s, l, s, l, s, l, l], [1917, 10, 18]),
        UmmAlQuraYearInfo::new(1337, [l, s, l, s, l, s, s, l, s, l, s, l], [1918, 10, 7]),
        UmmAlQuraYearInfo::new(1338, [s, l, l, s, l, l, s, s, l, s, l, s], [1919, 9, 26]),
        UmmAlQuraYearInfo::new(1339, [l, s, l, s, l, l, l, s, l, s, s, l], [1920, 9, 14]),
        UmmAlQuraYearInfo::new(1340, [s, s, l, s, l, l, l, l, s, l, s, s], [1921, 9, 4]),
        UmmAlQuraYearInfo::new(1341, [l, s, s, l, s, l, l, l, s, l, l, s], [1922, 8, 24]),
        UmmAlQuraYearInfo::new(1342, [s, s, l, s, l, s, l, l, s, l, l, s], [1923, 8, 14]),
        UmmAlQuraYearInfo::new(1343, [l, s, s, l, s, l, s, l, s, l, l, s], [1924, 8, 2]),
        UmmAlQuraYearInfo::new(1344, [l, s, l, s, l, l, s, s, l, s, l, s], [1925, 7, 22]),
        UmmAlQuraYearInfo::new(1345, [l, s, l, l, l, s, l, s, s, l, s, s], [1926, 7, 11]),
        UmmAlQuraYearInfo::new(1346, [l, s, l, l, l, l, s, l, s, s, l, s], [1927, 6, 30]),
        UmmAlQuraYearInfo::new(1347, [s, l, s, l, l, l, s, l, l, s, s, l], [1928, 6, 19]),
        UmmAlQuraYearInfo::new(1348, [s, s, l, s, l, l, s, l, l, l, s, s], [1929, 6, 9]),
        UmmAlQuraYearInfo::new(1349, [l, s, s, l, s, l, l, s, l, l, s, l], [1930, 5, 29]),
        UmmAlQuraYearInfo::new(1350, [s, l, s, l, s, l, s, s, l, l, s, l], [1931, 5, 19]),
        UmmAlQuraYearInfo::new(1351, [l, s, l, s, l, s, l, s, s, l, s, l], [1932, 5, 7]),
        UmmAlQuraYearInfo::new(1352, [l, s, l, l, s, l, s, l, s, s, l, s], [1933, 4, 26]),
        UmmAlQuraYearInfo::new(1353, [l, s, l, l, l, s, l, s, s, l, s, l], [1934, 4, 15]),
        UmmAlQuraYearInfo::new(1354, [s, l, s, l, l, s, l, l, s, l, s, s], [1935, 4, 5]),
        UmmAlQuraYearInfo::new(1355, [l, s, s, l, l, s, l, l, s, l, l, s], [1936, 3, 24]),
        UmmAlQuraYearInfo::new(1356, [s, l, s, l, s, l, s, l, s, l, l, l], [1937, 3, 14]),
        UmmAlQuraYearInfo::new(1357, [s, s, l, s, l, s, s, l, s, l, l, l], [1938, 3, 4]),
        UmmAlQuraYearInfo::new(1358, [s, l, s, l, s, l, s, s, l, s, l, l], [1939, 2, 21]),
        UmmAlQuraYearInfo::new(1359, [s, l, l, s, l, s, l, s, s, s, l, l], [1940, 2, 10]),
        UmmAlQuraYearInfo::new(1360, [s, l, l, l, s, l, s, l, s, s, l, s], [1941, 1, 29]),
        UmmAlQuraYearInfo::new(1361, [l, s, l, l, s, l, l, s, s, l, s, l], [1942, 1, 18]),
        UmmAlQuraYearInfo::new(1362, [s, l, s, l, s, l, l, s, l, s, l, s], [1943, 1, 8]),
        UmmAlQuraYearInfo::new(1363, [l, s, l, s, l, s, l, s, l, s, l, l], [1943, 12, 28]),
        UmmAlQuraYearInfo::new(1364, [s, l, s, l, s, s, l, s, l, s, l, l], [1944, 12, 17]),
        UmmAlQuraYearInfo::new(1365, [l, l, s, s, l, s, s, l, s, l, s, l], [1945, 12, 6]),
        UmmAlQuraYearInfo::new(1366, [l, l, s, l, s, l, s, s, l, s, l, s], [1946, 11, 25]),
        UmmAlQuraYearInfo::new(1367, [l, l, s, l, l, s, l, s, s, l, s, l], [1947, 11, 14]),
        UmmAlQuraYearInfo::new(1368, [s, l, s, l, l, l, s, s, l, s, l, s], [1948, 11, 3]),
        UmmAlQuraYearInfo::new(1369, [l, s, l, s, l, l, s, l, s, l, l, s], [1949, 10, 23]),
        UmmAlQuraYearInfo::new(1370, [l, s, s, l, s, l, s, l, s, l, l, l], [1950, 10, 13]),
        UmmAlQuraYearInfo::new(1371, [s, l, s, s, l, s, l, s, l, s, l, l], [1951, 10, 3]),
        UmmAlQuraYearInfo::new(1372, [l, s, s, l, s, l, s, s, l, s, l, l], [1952, 9, 21]),
        UmmAlQuraYearInfo::new(1373, [l, s, l, s, l, s, l, s, s, l, s, l], [1953, 9, 10]),
        UmmAlQuraYearInfo::new(1374, [l, s, l, l, s, l, s, l, s, s, l, s], [1954, 8, 30]),
        UmmAlQuraYearInfo::new(1375, [l, s, l, l, s, l, l, s, l, s, l, s], [1955, 8, 19]),
        UmmAlQuraYearInfo::new(1376, [s, l, s, l, s, l, l, l, s, l, s, l], [1956, 8, 8]),
        UmmAlQuraYearInfo::new(1377, [s, s, l, s, s, l, l, l, s, l, l, s], [1957, 7, 29]),
        UmmAlQuraYearInfo::new(1378, [l, s, s, s, l, s, l, l, s, l, l, l], [1958, 7, 18]),
        UmmAlQuraYearInfo::new(1379, [s, l, s, s, s, l, s, l, l, s, l, l], [1959, 7, 8]),
        UmmAlQuraYearInfo::new(1380, [s, l, s, l, s, l, s, l, s, l, s, l], [1960, 6, 26]),
        UmmAlQuraYearInfo::new(1381, [s, l, s, l, l, s, l, s, l, s, s, l], [1961, 6, 15]),
        UmmAlQuraYearInfo::new(1382, [s, l, s, l, l, s, l, l, s, l, s, s], [1962, 6, 4]),
        UmmAlQuraYearInfo::new(1383, [l, s, s, l, l, l, s, l, l, s, l, s], [1963, 5, 24]),
        UmmAlQuraYearInfo::new(1384, [s, l, s, s, l, l, s, l, l, l, s, l], [1964, 5, 13]),
        UmmAlQuraYearInfo::new(1385, [s, s, l, s, s, l, l, s, l, l, l, s], [1965, 5, 3]),
        UmmAlQuraYearInfo::new(1386, [l, s, s, l, s, s, l, l, s, l, l, s], [1966, 4, 22]),
        UmmAlQuraYearInfo::new(1387, [l, s, l, s, l, s, l, s, l, s, l, s], [1967, 4, 11]),
        UmmAlQuraYearInfo::new(1388, [l, l, s, l, s, l, s, l, s, l, s, s], [1968, 3, 30]),
        UmmAlQuraYearInfo::new(1389, [l, l, s, l, l, s, l, l, s, s, l, s], [1969, 3, 19]),
        UmmAlQuraYearInfo::new(1390, [s, l, s, l, l, l, s, l, s, l, s, l], [1970, 3, 9]),
        UmmAlQuraYearInfo::new(1391, [s, s, l, s, l, l, s, l, l, s, l, s], [1971, 2, 27]),
        UmmAlQuraYearInfo::new(1392, [l, s, s, l, s, l, s, l, l, s, l, l], [1972, 2, 16]),
        UmmAlQuraYearInfo::new(1393, [s, l, s, s, l, s, l, s, l, s, l, l], [1973, 2, 5]),
        UmmAlQuraYearInfo::new(1394, [l, s, l, s, s, l, s, l, s, l, s, l], [1974, 1, 25]),
        UmmAlQuraYearInfo::new(1395, [l, s, l, l, s, l, s, s, l, s, s, l], [1975, 1, 14]),
        UmmAlQuraYearInfo::new(1396, [l, s, l, l, s, l, l, s, s, l, s, s], [1976, 1, 3]),
        UmmAlQuraYearInfo::new(1397, [l, s, l, l, s, l, l, l, s, s, s, l], [1976, 12, 22]),
        UmmAlQuraYearInfo::new(1398, [s, l, s, l, l, s, l, l, s, l, s, s], [1977, 12, 12]),
        UmmAlQuraYearInfo::new(1399, [l, s, l, s, l, s, l, l, s, l, s, l], [1978, 12, 1]),
        UmmAlQuraYearInfo::new(1400, [l, s, l, s, s, l, s, l, s, l, s, l], [1979, 11, 21]),
        UmmAlQuraYearInfo::new(1401, [l, l, s, l, s, s, l, s, s, l, s, l], [1980, 11, 9]),
        UmmAlQuraYearInfo::new(1402, [l, l, l, s, l, s, s, l, s, s, l, s], [1981, 10, 29]),
        UmmAlQuraYearInfo::new(1403, [l, l, l, s, l, l, s, s, l, s, s, l], [1982, 10, 18]),
        UmmAlQuraYearInfo::new(1404, [s, l, l, s, l, l, s, l, s, l, s, s], [1983, 10, 8]),
        UmmAlQuraYearInfo::new(1405, [l, s, l, s, l, l, l, s, l, s, s, l], [1984, 9, 26]),
        UmmAlQuraYearInfo::new(1406, [l, s, s, l, s, l, l, s, l, s, l, l], [1985, 9, 16]),
        UmmAlQuraYearInfo::new(1407, [s, l, s, s, l, s, l, s, l, s, l, l], [1986, 9, 6]),
        UmmAlQuraYearInfo::new(1408, [l, s, l, s, l, s, s, l, s, s, l, l], [1987, 8, 26]),
        UmmAlQuraYearInfo::new(1409, [l, l, s, l, s, l, s, s, l, s, s, l], [1988, 8, 14]),
        UmmAlQuraYearInfo::new(1410, [l, l, s, l, l, s, l, s, s, l, s, s], [1989, 8, 3]),
        UmmAlQuraYearInfo::new(1411, [l, l, s, l, l, s, l, l, s, s, l, s], [1990, 7, 23]),
        UmmAlQuraYearInfo::new(1412, [l, s, l, s, l, s, l, l, l, s, s, l], [1991, 7, 13]),
        UmmAlQuraYearInfo::new(1413, [s, l, s, s, l, s, l, l, l, s, l, s], [1992, 7, 2]),
        UmmAlQuraYearInfo::new(1414, [l, s, l, s, s, l, s, l, l, s, l, l], [1993, 6, 21]),
        UmmAlQuraYearInfo::new(1415, [s, l, s, l, s, s, l, s, l, s, l, l], [1994, 6, 11]),
        UmmAlQuraYearInfo::new(1416, [l, s, l, s, l, s, s, l, s, l, s, l], [1995, 5, 31]),
        UmmAlQuraYearInfo::new(1417, [l, s, l, l, s, s, l, s, l, s, l, s], [1996, 5, 19]),
        UmmAlQuraYearInfo::new(1418, [l, s, l, l, s, l, s, l, s, l, s, l], [1997, 5, 8]),
        UmmAlQuraYearInfo::new(1419, [s, l, s, l, s, l, s, l, l, l, s, s], [1998, 4, 28]),
        UmmAlQuraYearInfo::new(1420, [s, l, s, s, l, s, l, l, l, l, s, l], [1999, 4, 17]),
        UmmAlQuraYearInfo::new(1421, [s, s, l, s, s, s, l, l, l, l, s, l], [2000, 4, 6]),
        UmmAlQuraYearInfo::new(1422, [l, s, s, l, s, s, s, l, l, l, s, l], [2001, 3, 26]),
        UmmAlQuraYearInfo::new(1423, [l, s, l, s, l, s, s, l, s, l, s, l], [2002, 3, 15]),
        UmmAlQuraYearInfo::new(1424, [l, s, l, l, s, l, s, s, l, s, l, s], [2003, 3, 4]),
        UmmAlQuraYearInfo::new(1425, [l, s, l, l, s, l, s, l, l, s, l, s], [2004, 2, 21]),
        UmmAlQuraYearInfo::new(1426, [s, l, s, l, s, l, l, s, l, l, s, l], [2005, 2, 10]),
        UmmAlQuraYearInfo::new(1427, [s, s, l, s, l, s, l, l, s, l, l, s], [2006, 1, 31]),
        UmmAlQuraYearInfo::new(1428, [l, s, s, l, s, s, l, l, l, s, l, l], [2007, 1, 20]),
        UmmAlQuraYearInfo::new(1429, [s, l, s, s, l, s, s, l, l, s, l, l], [2008, 1, 10]),
        UmmAlQuraYearInfo::new(1430, [s, l, l, s, s, l, s, l, s, l, s, l], [2008, 12, 29]),
        UmmAlQuraYearInfo::new(1431, [s, l, l, s, l, s, l, s, l, s, s, l], [2009, 12, 18]),
        UmmAlQuraYearInfo::new(1432, [s, l, l, l, s, l, s, l, s, l, s, s], [2010, 12, 7]),
        UmmAlQuraYearInfo::new(1433, [l, s, l, l, s, l, l, s, l, s, l, s], [2011, 11, 26]),
        UmmAlQuraYearInfo::new(1434, [s, l, s, l, s, l, l, s, l, l, s, s], [2012, 11, 15]),
        UmmAlQuraYearInfo::new(1435, [l, s, l, s, l, s, l, s, l, l, s, l], [2013, 11, 4]),
        UmmAlQuraYearInfo::new(1436, [s, l, s, l, s, l, s, l, s, l, s, l], [2014, 10, 25]),
        UmmAlQuraYearInfo::new(1437, [l, s, l, l, s, s, l, s, l, s, s, l], [2015, 10, 14]),
        UmmAlQuraYearInfo::new(1438, [l, s, l, l, l, s, s, l, s, s, l, s], [2016, 10, 2]),
        UmmAlQuraYearInfo::new(1439, [l, s, l, l, l, s, l, s, l, s, s, l], [2017, 9, 21]),
        UmmAlQuraYearInfo::new(1440, [s, l, s, l, l, l, s, l, s, l, s, s], [2018, 9, 11]),
        UmmAlQuraYearInfo::new(1441, [l, s, l, s, l, l, s, l, l, s, l, s], [2019, 8, 31]),
        UmmAlQuraYearInfo::new(1442, [s, l, s, l, s, l, s, l, l, s, l, s], [2020, 8, 20]),
        UmmAlQuraYearInfo::new(1443, [l, s, l, s, l, s, l, s, l, s, l, l], [2021, 8, 9]),
        UmmAlQuraYearInfo::new(1444, [s, l, s, l, l, s, s, l, s, l, s, l], [2022, 7, 30]),
        UmmAlQuraYearInfo::new(1445, [s, l, l, l, s, l, s, s, l, s, s, l], [2023, 7, 19]),
        UmmAlQuraYearInfo::new(1446, [s, l, l, l, s, l, l, s, s, l, s, s], [2024, 7, 7]),
        UmmAlQuraYearInfo::new(1447, [l, s, l, l, l, s, l, s, l, s, l, s], [2025, 6, 26]),
        UmmAlQuraYearInfo::new(1448, [s, l, s, l, l, s, l, l, s, l, s, l], [2026, 6, 16]),
        UmmAlQuraYearInfo::new(1449, [s, s, l, s, l, s, l, l, s, l, l, s], [2027, 6, 6]),
        UmmAlQuraYearInfo::new(1450, [l, s, l, s, s, l, s, l, s, l, l, s], [2028, 5, 25]),
        UmmAlQuraYearInfo::new(1451, [l, l, l, s, s, l, s, s, l, l, s, l], [2029, 5, 14]),
        UmmAlQuraYearInfo::new(1452, [l, s, l, l, s, s, l, s, s, l, s, l], [2030, 5, 4]),
        UmmAlQuraYearInfo::new(1453, [l, s, l, l, s, l, s, l, s, s, l, s], [2031, 4, 23]),
        UmmAlQuraYearInfo::new(1454, [l, s, l, l, s, l, l, s, l, s, l, s], [2032, 4, 11]),
        UmmAlQuraYearInfo::new(1455, [s, l, s, l, l, s, l, s, l, l, s, l], [2033, 4, 1]),
        UmmAlQuraYearInfo::new(1456, [s, s, l, s, l, s, l, s, l, l, l, s], [2034, 3, 22]),
        UmmAlQuraYearInfo::new(1457, [l, s, s, l, s, s, l, s, l, l, l, l], [2035, 3, 11]),
        UmmAlQuraYearInfo::new(1458, [s, l, s, s, l, s, s, l, s, l, l, l], [2036, 2, 29]),
        UmmAlQuraYearInfo::new(1459, [s, l, l, s, s, l, s, s, l, s, l, l], [2037, 2, 17]),
        UmmAlQuraYearInfo::new(1460, [s, l, l, s, l, s, l, s, s, l, s, l], [2038, 2, 6]),
        UmmAlQuraYearInfo::new(1461, [s, l, l, s, l, s, l, s, l, l, s, s], [2039, 1, 26]),
        UmmAlQuraYearInfo::new(1462, [l, s, l, s, l, l, s, l, s, l, l, s], [2040, 1, 15]),
        UmmAlQuraYearInfo::new(1463, [s, l, s, l, s, l, s, l, l, l, s, l], [2041, 1, 4]),
        UmmAlQuraYearInfo::new(1464, [s, l, s, s, l, s, s, l, l, l, s, l], [2041, 12, 25]),
        UmmAlQuraYearInfo::new(1465, [l, s, l, s, s, l, s, s, l, l, s, l], [2042, 12, 14]),
        UmmAlQuraYearInfo::new(1466, [l, l, s, l, s, s, s, l, s, l, l, s], [2043, 12, 3]),
        UmmAlQuraYearInfo::new(1467, [l, l, s, l, l, s, s, l, s, l, s, l], [2044, 11, 21]),
        UmmAlQuraYearInfo::new(1468, [s, l, s, l, l, s, l, s, l, s, l, s], [2045, 11, 11]),
        UmmAlQuraYearInfo::new(1469, [s, l, s, l, l, s, l, l, s, l, s, l], [2046, 10, 31]),
        UmmAlQuraYearInfo::new(1470, [s, s, l, s, l, l, s, l, l, s, l, s], [2047, 10, 21]),
        UmmAlQuraYearInfo::new(1471, [l, s, s, l, s, l, s, l, l, s, l, l], [2048, 10, 9]),
        UmmAlQuraYearInfo::new(1472, [s, l, s, s, l, s, l, s, l, l, s, l], [2049, 9, 29]),
        UmmAlQuraYearInfo::new(1473, [s, l, s, l, l, s, s, l, s, l, s, l], [2050, 9, 18]),
        UmmAlQuraYearInfo::new(1474, [s, l, l, s, l, l, s, s, l, s, l, s], [2051, 9, 7]),
        UmmAlQuraYearInfo::new(1475, [s, l, l, s, l, l, l, s, s, l, s, s], [2052, 8, 26]),
        UmmAlQuraYearInfo::new(1476, [l, s, l, s, l, l, l, s, l, s, l, s], [2053, 8, 15]),
        UmmAlQuraYearInfo::new(1477, [s, l, s, s, l, l, l, l, s, l, s, l], [2054, 8, 5]),
        UmmAlQuraYearInfo::new(1478, [s, s, l, s, l, s, l, l, s, l, l, s], [2055, 7, 26]),
        UmmAlQuraYearInfo::new(1479, [l, s, s, l, s, l, s, l, s, l, l, s], [2056, 7, 14]),
        UmmAlQuraYearInfo::new(1480, [l, s, l, s, l, s, l, s, l, s, l, s], [2057, 7, 3]),
        UmmAlQuraYearInfo::new(1481, [l, s, l, l, s, l, s, l, s, l, s, s], [2058, 6, 22]),
        UmmAlQuraYearInfo::new(1482, [l, s, l, l, l, l, s, l, s, s, l, s], [2059, 6, 11]),
        UmmAlQuraYearInfo::new(1483, [s, l, s, l, l, l, s, l, l, s, s, l], [2060, 5, 31]),
        UmmAlQuraYearInfo::new(1484, [s, s, l, s, l, l, l, s, l, s, l, s], [2061, 5, 21]),
        UmmAlQuraYearInfo::new(1485, [l, s, s, l, s, l, l, s, l, l, s, l], [2062, 5, 10]),
        UmmAlQuraYearInfo::new(1486, [s, l, s, s, l, s, l, s, l, l, s, l], [2063, 4, 30]),
        UmmAlQuraYearInfo::new(1487, [l, s, l, s, l, s, s, l, s, l, s, l], [2064, 4, 18]),
        UmmAlQuraYearInfo::new(1488, [l, s, l, l, s, l, s, s, l, s, l, s], [2065, 4, 7]),
        UmmAlQuraYearInfo::new(1489, [l, s, l, l, l, s, l, s, s, l, s, l], [2066, 3, 27]),
        UmmAlQuraYearInfo::new(1490, [s, l, s, l, l, s, l, l, s, s, l, s], [2067, 3, 17]),
        UmmAlQuraYearInfo::new(1491, [l, s, s, l, l, s, l, l, s, l, s, l], [2068, 3, 5]),
        UmmAlQuraYearInfo::new(1492, [s, l, s, s, l, l, s, l, s, l, l, s], [2069, 2, 23]),
        UmmAlQuraYearInfo::new(1493, [l, s, l, s, l, s, s, l, s, l, l, l], [2070, 2, 12]),
        UmmAlQuraYearInfo::new(1494, [s, l, s, l, s, l, s, s, s, l, l, l], [2071, 2, 2]),
        UmmAlQuraYearInfo::new(1495, [s, l, l, s, l, s, s, l, s, s, l, l], [2072, 1, 22]),
        UmmAlQuraYearInfo::new(1496, [s, l, l, l, s, l, s, s, l, s, s, l], [2073, 1, 10]),
        UmmAlQuraYearInfo::new(1497, [l, s, l, l, s, l, s, l, s, l, s, l], [2073, 12, 30]),
        UmmAlQuraYearInfo::new(1498, [s, l, s, l, s, l, l, s, l, s, l, s], [2074, 12, 20]),
        UmmAlQuraYearInfo::new(1499, [l, s, l, s, s, l, l, s, l, s, l, l], [2075, 12, 9]),
        UmmAlQuraYearInfo::new(1500, [s, l, s, l, s, s, l, s, l, s, l, l], [2076, 11, 28]),
        UmmAlQuraYearInfo::new(1501, [l, s, l, s, l, s, s, s, l, s, l, l], [2077, 11, 17]),
        UmmAlQuraYearInfo::new(1502, [l, l, s, l, s, l, s, s, s, l, l, s], [2078, 11, 6]),
        UmmAlQuraYearInfo::new(1503, [l, l, s, l, l, s, l, s, s, s, l, l], [2079, 10, 26]),
        UmmAlQuraYearInfo::new(1504, [s, l, s, l, l, l, s, s, l, s, l, s], [2080, 10, 15]),
        UmmAlQuraYearInfo::new(1505, [l, s, l, s, l, l, s, l, s, l, l, s], [2081, 10, 4]),
        UmmAlQuraYearInfo::new(1506, [s, l, s, s, l, l, s, l, l, s, l, l], [2082, 9, 24]),
        UmmAlQuraYearInfo::new(1507, [s, s, l, s, s, l, l, s, l, s, l, l], [2083, 9, 14]),
        UmmAlQuraYearInfo::new(1508, [l, s, s, l, s, l, s, s, l, s, l, l], [2084, 9, 2]),
        UmmAlQuraYearInfo::new(1509, [l, s, l, s, l, s, l, s, s, l, s, l], [2085, 8, 22]),
        UmmAlQuraYearInfo::new(1510, [l, s, l, l, s, l, s, l, s, s, l, s], [2086, 8, 11]),
        UmmAlQuraYearInfo::new(1511, [l, s, l, l, s, l, l, s, l, s, s, l], [2087, 7, 31]),
        UmmAlQuraYearInfo::new(1512, [s, l, s, l, s, l, l, l, s, l, s, l], [2088, 7, 20]),
        UmmAlQuraYearInfo::new(1513, [s, s, s, l, s, l, l, l, s, l, l, s], [2089, 7, 10]),
        UmmAlQuraYearInfo::new(1514, [l, s, s, s, l, s, l, l, s, l, l, l], [2090, 6, 29]),
        UmmAlQuraYearInfo::new(1515, [s, s, l, s, s, l, s, l, l, s, l, l], [2091, 6, 19]),
        UmmAlQuraYearInfo::new(1516, [s, l, s, l, s, s, l, s, l, s, l, l], [2092, 6, 7]),
        UmmAlQuraYearInfo::new(1517, [s, l, s, l, s, l, l, s, s, l, s, l], [2093, 5, 27]),
        UmmAlQuraYearInfo::new(1518, [s, l, s, l, l, s, l, l, s, l, s, s], [2094, 5, 16]),
        UmmAlQuraYearInfo::new(1519, [l, s, s, l, l, l, s, l, l, s, l, s], [2095, 5, 5]),
        UmmAlQuraYearInfo::new(1520, [s, l, s, s, l, l, l, s, l, l, s, l], [2096, 4, 24]),
        UmmAlQuraYearInfo::new(1521, [s, s, s, l, s, l, l, s, l, l, s, l], [2097, 4, 14]),
        UmmAlQuraYearInfo::new(1522, [l, s, s, s, l, s, l, l, s, l, l, s], [2098, 4, 3]),
        UmmAlQuraYearInfo::new(1523, [l, s, l, s, l, s, l, s, s, l, l, s], [2099, 3, 23]),
        UmmAlQuraYearInfo::new(1524, [l, l, s, l, s, l, s, l, s, s, l, s], [2100, 3, 12]),
        UmmAlQuraYearInfo::new(1525, [l, l, s, l, l, s, l, s, l, s, s, l], [2101, 3, 1]),
        UmmAlQuraYearInfo::new(1526, [s, l, s, l, l, l, s, l, s, l, s, s], [2102, 2, 19]),
        UmmAlQuraYearInfo::new(1527, [l, s, l, s, l, l, s, l, l, s, l, s], [2103, 2, 8]),
        UmmAlQuraYearInfo::new(1528, [l, s, s, l, s, l, s, l, l, s, l, l], [2104, 1, 29]),
        UmmAlQuraYearInfo::new(1529, [s, l, s, s, l, s, l, s, l, s, l, l], [2105, 1, 18]),
        UmmAlQuraYearInfo::new(1530, [s, l, l, s, s, l, s, l, s, s, l, l], [2106, 1, 7]),
        UmmAlQuraYearInfo::new(1531, [s, l, l, l, s, s, l, s, l, s, s, l], [2106, 12, 27]),
        UmmAlQuraYearInfo::new(1532, [s, l, l, l, s, l, l, s, s, s, l, s], [2107, 12, 16]),
        UmmAlQuraYearInfo::new(1533, [l, s, l, l, l, s, l, s, l, s, s, l], [2108, 12, 4]),
        UmmAlQuraYearInfo::new(1534, [s, l, s, l, l, s, l, l, s, s, l, s], [2109, 11, 24]),
        UmmAlQuraYearInfo::new(1535, [l, s, l, s, l, s, l, l, s, l, s, l], [2110, 11, 13]),
        UmmAlQuraYearInfo::new(1536, [s, l, s, l, s, l, s, l, s, l, s, l], [2111, 11, 3]),
        UmmAlQuraYearInfo::new(1537, [l, s, l, l, s, s, l, s, s, l, s, l], [2112, 10, 22]),
        UmmAlQuraYearInfo::new(1538, [l, l, s, l, l, s, s, l, s, s, l, s], [2113, 10, 11]),
        UmmAlQuraYearInfo::new(1539, [l, l, l, s, l, l, s, s, l, s, s, l], [2114, 9, 30]),
        UmmAlQuraYearInfo::new(1540, [s, l, l, s, l, l, s, l, s, s, l, s], [2115, 9, 20]),
        UmmAlQuraYearInfo::new(1541, [l, s, l, s, l, l, l, s, l, s, s, l], [2116, 9, 8]),
        UmmAlQuraYearInfo::new(1542, [s, l, s, l, s, l, l, s, l, s, l, l], [2117, 8, 29]),
        UmmAlQuraYearInfo::new(1543, [s, l, s, s, l, s, l, s, l, s, l, l], [2118, 8, 19]),
        UmmAlQuraYearInfo::new(1544, [l, s, l, s, s, l, s, l, s, l, s, l], [2119, 8, 8]),
        UmmAlQuraYearInfo::new(1545, [l, l, s, l, s, s, l, s, l, s, s, l], [2120, 7, 27]),
        UmmAlQuraYearInfo::new(1546, [l, l, s, l, s, l, s, l, s, l, s, s], [2121, 7, 16]),
        UmmAlQuraYearInfo::new(1547, [l, l, s, l, l, s, l, s, l, s, l, s], [2122, 7, 5]),
        UmmAlQuraYearInfo::new(1548, [l, s, s, l, l, s, l, l, s, l, s, l], [2123, 6, 25]),
        UmmAlQuraYearInfo::new(1549, [s, l, s, s, l, s, l, l, l, s, l, s], [2124, 6, 14]),
        UmmAlQuraYearInfo::new(1550, [l, s, l, s, s, s, l, l, l, s, l, l], [2125, 6, 3]),
        UmmAlQuraYearInfo::new(1551, [s, l, s, s, l, s, s, l, l, s, l, l], [2126, 5, 24]),
        UmmAlQuraYearInfo::new(1552, [l, s, l, s, s, l, s, s, l, l, s, l], [2127, 5, 13]),
        UmmAlQuraYearInfo::new(1553, [l, s, l, s, l, s, l, s, l, s, l, s], [2128, 5, 1]),
        UmmAlQuraYearInfo::new(1554, [l, s, l, s, l, l, s, l, s, l, s, l], [2129, 4, 20]),
        UmmAlQuraYearInfo::new(1555, [s, s, l, s, l, l, s, l, l, s, l, s], [2130, 4, 10]),
        UmmAlQuraYearInfo::new(1556, [l, s, s, l, s, l, s, l, l, l, s, l], [2131, 3, 30]),
        UmmAlQuraYearInfo::new(1557, [s, l, s, s, s, l, s, l, l, l, l, s], [2132, 3, 19]),
        UmmAlQuraYearInfo::new(1558, [l, s, l, s, s, s, l, s, l, l, l, s], [2133, 3, 8]),
        UmmAlQuraYearInfo::new(1559, [l, l, s, s, l, s, s, l, l, s, l, s], [2134, 2, 25]),
        UmmAlQuraYearInfo::new(1560, [l, l, s, l, s, l, s, l, s, l, s, l], [2135, 2, 14]),
        UmmAlQuraYearInfo::new(1561, [s, l, l, s, l, s, l, l, s, s, l, s], [2136, 2, 4]),
        UmmAlQuraYearInfo::new(1562, [s, l, l, s, l, s, l, l, l, s, s, l], [2137, 1, 23]),
        UmmAlQuraYearInfo::new(1563, [s, l, s, s, l, s, l, l, l, s, l, s], [2138, 1, 13]),
        UmmAlQuraYearInfo::new(1564, [l, s, l, s, s, l, s, l, l, l, s, l], [2139, 1, 2]),
        UmmAlQuraYearInfo::new(1565, [s, l, s, l, s, s, l, s, l, l, s, l], [2139, 12, 23]),
        UmmAlQuraYearInfo::new(1566, [l, s, l, s, l, s, s, l, s, l, s, l], [2140, 12, 11]),
        UmmAlQuraYearInfo::new(1567, [l, s, l, l, s, l, s, l, s, s, l, s], [2141, 11, 30]),
        UmmAlQuraYearInfo::new(1568, [l, s, l, l, l, s, l, s, l, s, s, s], [2142, 11, 19]),
        UmmAlQuraYearInfo::new(1569, [l, s, l, l, l, s, l, l, s, l, s, s], [2143, 11, 8]),
        UmmAlQuraYearInfo::new(1570, [s, l, s, l, l, s, l, l, l, s, s, l], [2144, 10, 28]),
        UmmAlQuraYearInfo::new(1571, [s, s, l, s, l, l, s, l, l, s, l, s], [2145, 10, 18]),
        UmmAlQuraYearInfo::new(1572, [l, s, s, l, s, l, s, l, l, s, l, s], [2146, 10, 7]),
        UmmAlQuraYearInfo::new(1573, [l, s, l, l, s, l, s, s, l, s, l, s], [2147, 9, 26]),
        UmmAlQuraYearInfo::new(1574, [l, l, s, l, l, s, l, s, s, l, s, s], [2148, 9, 14]),
        UmmAlQuraYearInfo::new(1575, [l, l, l, s, l, l, s, l, s, s, s, l], [2149, 9, 3]),
        UmmAlQuraYearInfo::new(1576, [s, l, l, s, l, l, l, s, l, s, s, s], [2150, 8, 24]),
        UmmAlQuraYearInfo::new(1577, [l, s, l, l, s, l, l, s, l, s, l, s], [2151, 8, 13]),
        UmmAlQuraYearInfo::new(1578, [s, l, s, l, s, l, l, s, l, l, s, l], [2152, 8, 2]),
        UmmAlQuraYearInfo::new(1579, [s, l, s, l, s, s, l, l, s, l, s, l], [2153, 7, 23]),
        UmmAlQuraYearInfo::new(1580, [s, l, l, s, l, s, s, l, s, l, s, l], [2154, 7, 12]),
        UmmAlQuraYearInfo::new(1581, [l, l, s, l, s, l, s, s, l, s, l, s], [2155, 7, 1]),
        UmmAlQuraYearInfo::new(1582, [l, l, s, l, l, s, l, s, l, s, s, s], [2156, 6, 19]),
        UmmAlQuraYearInfo::new(1583, [l, l, s, l, l, l, s, l, s, l, s, s], [2157, 6, 8]),
        UmmAlQuraYearInfo::new(1584, [s, l, l, s, l, l, s, l, l, s, l, s], [2158, 5, 29]),
        UmmAlQuraYearInfo::new(1585, [s, l, s, l, s, l, s, l, l, s, l, l], [2159, 5, 19]),
        UmmAlQuraYearInfo::new(1586, [s, s, l, s, l, s, s, l, l, l, s, l], [2160, 5, 8]),
        UmmAlQuraYearInfo::new(1587, [s, l, l, s, s, s, l, s, l, s, l, l], [2161, 4, 27]),
        UmmAlQuraYearInfo::new(1588, [l, s, l, l, s, s, s, l, s, l, s, l], [2162, 4, 16]),
        UmmAlQuraYearInfo::new(1589, [l, s, l, l, s, l, s, s, l, s, l, s], [2163, 4, 5]),
        UmmAlQuraYearInfo::new(1590, [l, s, l, l, l, s, s, l, s, l, s, l], [2164, 3, 24]),
        UmmAlQuraYearInfo::new(1591, [s, l, s, l, l, s, l, s, l, s, l, s], [2165, 3, 14]),
        UmmAlQuraYearInfo::new(1592, [l, s, l, s, l, s, l, s, l, l, l, s], [2166, 3, 3]),
        UmmAlQuraYearInfo::new(1593, [l, s, s, l, s, s, l, s, l, l, l, s], [2167, 2, 21]),
        UmmAlQuraYearInfo::new(1594, [l, l, s, s, l, s, s, s, l, l, l, l], [2168, 2, 10]),
        UmmAlQuraYearInfo::new(1595, [s, l, s, l, s, s, l, s, s, l, l, l], [2169, 1, 30]),
        UmmAlQuraYearInfo::new(1596, [s, l, l, s, l, s, s, l, s, l, s, l], [2170, 1, 19]),
        UmmAlQuraYearInfo::new(1597, [s, l, l, s, l, s, l, s, l, s, l, s], [2171, 1, 8]),
        UmmAlQuraYearInfo::new(1598, [l, s, l, s, l, l, s, l, s, l, l, s], [2171, 12, 28]),
        UmmAlQuraYearInfo::new(1599, [s, l, s, l, s, l, s, l, l, l, s, l], [2172, 12, 17]),
        UmmAlQuraYearInfo::new(1600, [s, s, l, s, l, s, s, l, l, l, s, l], [2173, 12, 7]),
    ]
});

pub(super) const UMMALQURA_FIRST_START_DAY: i64 = UMMALQURA_YEARS[0].start_day();
pub(super) const UMMALQURA_EXCLUSIVE_END_DAY: i64 = {
    let last = UMMALQURA_YEARS[YEAR_COUNT - 1];
    last.start_day() + last.days_in_year()
};

/// The policy is derived from complete, valid day-30 dates inside the actual
/// reference window. A long month starting before the cutoff is insufficient.
const fn latest_day30_reference_years() -> [i64; 12] {
    let minimum = checked_iso_epoch_day(1900, 1, 1);
    let cutoff = checked_iso_epoch_day(1972, 12, 31);
    let mut years = [0i64; 12];
    let mut latest = [i64::MIN; 12];
    let mut index = 0;
    while index < YEAR_COUNT {
        let row = UMMALQURA_YEARS[index];
        let mut prefix = 0i64;
        let mut month = 0;
        while month < 12 {
            let long = (row.month_mask() >> month) & 1;
            if long == 1 {
                let candidate = row.start_day() + prefix + 29;
                if candidate >= minimum && candidate <= cutoff && candidate > latest[month] {
                    latest[month] = candidate;
                    years[month] = row.calendar_year();
                }
            }
            prefix += 29 + long;
            month += 1;
        }
        assert!(prefix == row.days_in_year());
        index += 1;
    }
    let mut month = 0;
    while month < 12 {
        assert!(latest[month] >= minimum && latest[month] <= cutoff);
        assert!(years[month] >= FIRST_YEAR && years[month] < FIRST_YEAR + YEAR_COUNT as i64);
        month += 1;
    }
    years
}

const DAY30_REFERENCE_YEARS: [i64; 12] = latest_day30_reference_years();

/// Only the emitted ordinal selector consumes this native policy. It retains
/// the user's original month for the existing regulator and its error order.
pub(super) const fn ummalqura_day30_reference_year(month: usize) -> i64 {
    assert!(month >= 1 && month <= 12);
    DAY30_REFERENCE_YEARS[month - 1]
}
