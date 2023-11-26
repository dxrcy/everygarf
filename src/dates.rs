use chrono::{Datelike, Duration, NaiveDate, NaiveTime, Utc};

/// Convert `NaiveDate` to YYYY/MM/DD format, with '/' being a given string separator, and
/// optionally includes leading zeros for months and days (Eg. '03')
pub fn date_to_string(date: NaiveDate, separator: &str, leading_zeros: bool) -> String {
    let month = date.month();
    let day = date.day();

    date.year().to_string()
        + separator
        + if leading_zeros && month < 10 { "0" } else { "" }
        + &month.to_string()
        + separator
        + if leading_zeros && day < 10 { "0" } else { "" }
        + &day.to_string()
}

pub fn get_all_dates() -> Vec<NaiveDate> {
    get_dates_between(first(), latest())
}

fn first() -> NaiveDate {
    NaiveDate::from_ymd_opt(1978, 6, 19)
        .expect("Static date failed to parse. This error should never occur.")
}

fn latest() -> NaiveDate {
    let now = Utc::now();

    // Get naive time (UTC) for when comic is published to gocomics.com
    // Estimated time is:
    //      0000-0300 EST
    //      0400-0700 UTC
    //      1400-1700 AEST
    // And a margin of error is added just in case
    let time_of_publish = NaiveTime::from_hms_opt(7, 0, 0)
        .expect("Static time failed to parse. This error should never occur.");

    // Today if currently AFTER time of publish for todays comic
    // Yesterday if currently BEFORE time of publish for todays comic
    now.date_naive() - Duration::days(if now.time() > time_of_publish { 0 } else { 1 })
}

fn get_dates_between(start: NaiveDate, end: NaiveDate) -> Vec<NaiveDate> {
    (0..=(end - start).num_days())
        .map(|days| start + Duration::days(days))
        .collect()
}

pub fn date_from_filename(filename: &str) -> Option<NaiveDate> {
    let name = filename.split('/').last()?.split('.').next()?;
    let mut parts = name.split('-');

    let year = parts.next()?;
    let month = parts.next()?;
    let day = parts.next()?;

    let year: i32 = year.parse().ok()?;
    let month: u32 = month.parse().ok()?;
    let day: u32 = day.parse().ok()?;

    NaiveDate::from_ymd_opt(year, month, day)
}
