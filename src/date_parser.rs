use chrono::{DateTime, Datelike, NaiveDate};

pub fn get_days_from_month(year: i32, month: u32) -> i64 {
    NaiveDate::from_ymd(
        match month {
            12 => year + 1,
            _ => year,
        },
        match month {
            12 => 1,
            _ => month + 1,
        },
        1,
    )
    .signed_duration_since(NaiveDate::from_ymd(year, month, 1))
    .num_days()
}

fn parse_year_month_from_date_string(date_string: &str) -> (String, String, i64) {
    let date_time = DateTime::parse_from_rfc2822(date_string);
    let date = date_time.unwrap().date();

    let year = date.year();
    let month = date.month();
    let days_in_month = get_days_from_month(year, month);

    let year_string = date.format("%C%y").to_string();
    let month_string = date.format("%B").to_string();
    (year_string, month_string, days_in_month)
}

fn parse_hours_from_date() {
    // iterate through the number of days in the month
    // for each day return the calendar day
    // if its a weekend, set to zero, otherwise 8
}

#[cfg(test)]
mod tests {
    use crate::date_parser::{get_days_from_month, parse_year_month_from_date_string};

    #[test]
    fn parse_year_month_from_date_string_returns_month_year() {
        let return_value = ("2021".to_string(), "October".to_string(), 31);
        let date_string = "Tue, 19 Oct 2021 10:52:28 +0200";
        assert_eq!(parse_year_month_from_date_string(date_string), return_value);
    }

    #[test]
    fn it_finds_the_number_of_days_for_a_specific_month_and_year() {
        assert_eq!(get_days_from_month(2021, 10), 31);
        assert_eq!(get_days_from_month(1989, 2), 28);
        assert_eq!(get_days_from_month(1945, 6), 30);
        // leap year
        assert_eq!(get_days_from_month(2024, 2), 29);
    }
}
