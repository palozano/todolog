use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::config::IdStrategy;
use crate::domain::{Fingerprint, TaskId};

pub(crate) fn fingerprint(file: &str, text: &str) -> Fingerprint {
    Fingerprint::new(format!(
        "{:016x}",
        fnv1a64(&format!("{file}\0{}", normalize_text(text)))
    ))
}

pub(crate) fn task_id(fingerprint: &Fingerprint, strategy: IdStrategy) -> TaskId {
    match strategy {
        IdStrategy::Timestamp => timestamp_id(SystemTime::now()),
        IdStrategy::Uid => uid(fingerprint),
        IdStrategy::Uuid => uuid(fingerprint),
    }
}

pub(crate) fn timestamp_id(now: SystemTime) -> TaskId {
    let duration = now.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
    let total_seconds = duration.as_secs();
    let days = (total_seconds / 86_400) as i64;
    let seconds_of_day = total_seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;

    TaskId::new(format!(
        "{year:04}{month:02}{day:02}-{hour:02}{minute:02}{second:02}"
    ))
}

fn uid(fingerprint: &Fingerprint) -> TaskId {
    TaskId::new(format!("T-{}", &fingerprint.as_str()[..12]))
}

fn uuid(fingerprint: &Fingerprint) -> TaskId {
    let seed = fingerprint.as_str();
    let high = fnv1a64(&format!("uuid-high\0{seed}"));
    let low = fnv1a64(&format!("uuid-low\0{seed}"));
    let mut hex: Vec<char> = format!("{high:016x}{low:016x}").chars().collect();
    hex[12] = '5';
    hex[16] = match hex[16] {
        '0'..='3' => '8',
        '4'..='7' => '9',
        '8'..='b' => 'a',
        _ => 'b',
    };
    let hex: String = hex.into_iter().collect();

    TaskId::new(format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    ))
}

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn fnv1a64(input: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i64, u32, u32) {
    let days = days_since_unix_epoch + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };

    (year, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::UNIX_EPOCH;

    #[test]
    fn generates_ids_for_each_strategy() {
        let fingerprint = Fingerprint::new("0123456789abcdef");

        assert_eq!(timestamp_id(UNIX_EPOCH), TaskId::new("19700101-000000"));
        assert_eq!(
            task_id(&fingerprint, IdStrategy::Uid),
            TaskId::new("T-0123456789ab")
        );

        let uuid = task_id(&fingerprint, IdStrategy::Uuid).to_string();
        assert_eq!(uuid.len(), 36);
        assert_eq!(&uuid[14..15], "5");
        assert!(matches!(&uuid[19..20], "8" | "9" | "a" | "b"));
    }
}
