use std::time::SystemTime;

use time::{format_description::well_known::Rfc2822, OffsetDateTime};

pub fn format_date(date: SystemTime) -> String {
    let date = OffsetDateTime::from(date);
    date.format(&Rfc2822)
        .unwrap()
}
