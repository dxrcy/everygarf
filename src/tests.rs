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
