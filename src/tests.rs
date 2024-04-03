use super::*;

#[test]
fn format_bytes_works() {
    assert_eq!(format_bytes(0), "0B");
    assert_eq!(format_bytes(1), "1B");
    assert_eq!(format_bytes(10), "10B");
    assert_eq!(format_bytes(12), "12B");
    assert_eq!(format_bytes(100), "100B");
    assert_eq!(format_bytes(120), "120B");
    assert_eq!(format_bytes(123), "123B");

    assert_eq!(format_bytes(1_000), "1.0kB");
    assert_eq!(format_bytes(1_200), "1.2kB");
    assert_eq!(format_bytes(1_230), "1.2kB");
    assert_eq!(format_bytes(1_234), "1.2kB");

    assert_eq!(format_bytes(10_000), "10.0kB");
    assert_eq!(format_bytes(12_000), "12.0kB");
    assert_eq!(format_bytes(12_300), "12.3kB");
    assert_eq!(format_bytes(12_345), "12.3kB");
    assert_eq!(format_bytes(100_000), "100.0kB");
    assert_eq!(format_bytes(120_000), "120.0kB");
    assert_eq!(format_bytes(123_450), "123.4kB");

    assert_eq!(format_bytes(1_000_000), "1.0MB");
    assert_eq!(format_bytes(1_234_000), "1.2MB");
    assert_eq!(format_bytes(10_000_000), "10.0MB");
    assert_eq!(format_bytes(12_345_678), "12.3MB");
    assert_eq!(format_bytes(100_000_000), "100.0MB");

    assert_eq!(format_bytes(1_000_000_000), "1.0GB");
    assert_eq!(format_bytes(10_000_000_000), "10.0GB");
    assert_eq!(format_bytes(100_000_000_000), "100.0GB");
    assert_eq!(format_bytes(123_456_789_123), "123.5GB");

    assert_eq!(format_bytes(1_000_000_000_000), "1.0TB");
    assert_eq!(format_bytes(100_000_000_000_000), "100.0TB");
    assert_eq!(format_bytes(123_456_789_123_456), "123.5TB");
}

#[test]
fn format_duration_works() {
    assert_eq!(format_duration(Duration::from_secs(0)), "0s");
    assert_eq!(format_duration(Duration::from_secs(1)), "1s");
    assert_eq!(format_duration(Duration::from_secs(10)), "10s");
    assert_eq!(format_duration(Duration::from_secs(59)), "59s");
    assert_eq!(format_duration(Duration::from_secs(60)), "1m");
    assert_eq!(format_duration(Duration::from_secs(61)), "1m 1s");
    assert_eq!(format_duration(Duration::from_secs(119)), "1m 59s");
    assert_eq!(format_duration(Duration::from_secs(120)), "2m");
    assert_eq!(format_duration(Duration::from_secs(150)), "2m 30s");
    assert_eq!(format_duration(Duration::from_secs(3_540)), "59m");
    assert_eq!(format_duration(Duration::from_secs(3_541)), "59m 1s");
    assert_eq!(format_duration(Duration::from_secs(3_599)), "59m 59s");
    assert_eq!(format_duration(Duration::from_secs(3_600)), "1h");
    assert_eq!(format_duration(Duration::from_secs(3_601)), "1h 1s");
    assert_eq!(format_duration(Duration::from_secs(3_659)), "1h 59s");
    assert_eq!(format_duration(Duration::from_secs(3_660)), "1h 1m");
    assert_eq!(format_duration(Duration::from_secs(3_661)), "1h 1m 1s");
    assert_eq!(format_duration(Duration::from_secs(7_140)), "1h 59m");
    assert_eq!(format_duration(Duration::from_secs(7_199)), "1h 59m 59s");
    assert_eq!(format_duration(Duration::from_secs(7_200)), "2h");
    assert_eq!(format_duration(Duration::from_secs(7_201)), "2h 1s");
    assert_eq!(format_duration(Duration::from_secs(82_800)), "23h");
    assert_eq!(format_duration(Duration::from_secs(82_859)), "23h 59s");
    assert_eq!(format_duration(Duration::from_secs(86_340)), "23h 59m");
    assert_eq!(format_duration(Duration::from_secs(86_399)), "23h 59m 59s");
    assert_eq!(format_duration(Duration::from_secs(86_400)), "1d");
    assert_eq!(format_duration(Duration::from_secs(86_401)), "1d 1s");
    assert_eq!(format_duration(Duration::from_secs(864_000)), "10d");
    assert_eq!(format_duration(Duration::from_secs(8_643_600)), "100d 1h");
}
