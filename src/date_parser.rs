use chrono::{Date, DateTime, Datelike, FixedOffset, NaiveDate, TimeZone, Utc};
use serde_json::{Map, Value};
use std::convert::TryInto;

type YearTuple = (Date<FixedOffset>, i32, u32, i64);

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

fn parse_year_month_days_from_date_string(date_string: &str) -> YearTuple {
    let date_time = DateTime::parse_from_rfc2822(date_string);
    let date = date_time.unwrap().date();

    let year = date.year();
    let month = date.month();
    let days_in_month = get_days_from_month(year, month);

    (date, year, month, days_in_month)
}

fn parse_hours_from_date(date_tuple: YearTuple) -> Vec<Map<String, Value>> {
    // iterate through the number of days in the month
    // for each day return the calendar day
    // if its a weekend, set to zero, otherwise 8
    let mut vector = vec![];

    for day in 1..date_tuple.3 + 1 {
        let mut day_map: Map<String, Value> = Map::new();

        let day_of_week_index = Utc
            .ymd(date_tuple.1, date_tuple.2, day.try_into().unwrap())
            .format("%u")
            .to_string();

        if day_of_week_index == "6" || day_of_week_index == "7" {
            day_map.insert("weekend".to_string(), Value::from(true));
            day_map.insert("hours".to_string(), Value::from(0));
        } else {
            day_map.insert("weekend".to_string(), Value::from(false));
            day_map.insert("hours".to_string(), Value::from(8));
        }
        vector.push(day_map);
    }

    vector
}

fn get_date_map_from_date_string(rfc_date: String) -> Map<String, Value> {
    let mut date_map: Map<String, Value> = Map::new();

    let year_tuple: YearTuple = parse_year_month_days_from_date_string(rfc_date.as_ref());
    let default_hours: Vec<Map<String, Value>> = parse_hours_from_date(year_tuple);

    date_map.insert(
        "year".to_string(),
        Value::from(year_tuple.0.format("%C%y").to_string()),
    );
    date_map.insert(
        "month".to_string(),
        Value::from(year_tuple.0.format("%B").to_string()),
    );
    date_map.insert("total_days".to_string(), Value::from(year_tuple.3));
    date_map.insert("hours_worked".to_string(), Value::from(default_hours));

    date_map
}

#[cfg(test)]
mod tests {
    use crate::date_parser::{
        get_date_map_from_date_string, get_days_from_month, parse_hours_from_date,
        parse_year_month_days_from_date_string,
    };
    use chrono::{Date, DateTime, FixedOffset, TimeZone};
    use serde_json::{json, Map, Value};

    fn mock_date_fixed_offset() -> Date<FixedOffset> {
        let date_time = DateTime::parse_from_rfc2822("Tue, 19 Oct 2021 10:52:28 +0200");
        let date = date_time.unwrap().date();
        date
    }

    #[test]
    fn parse_year_month_from_date_string_returns_month_year() {
        let return_value = (mock_date_fixed_offset(), 2021 as i32, 10 as u32, 31 as i64);
        let date_string = "Tue, 19 Oct 2021 10:52:28 +0200";
        assert_eq!(
            parse_year_month_days_from_date_string(date_string),
            return_value
        );
    }

    #[test]
    fn it_finds_the_number_of_days_for_a_specific_month_and_year() {
        assert_eq!(get_days_from_month(2021, 10), 31);
        assert_eq!(get_days_from_month(1989, 2), 28);
        assert_eq!(get_days_from_month(1945, 6), 30);
        // leap year
        assert_eq!(get_days_from_month(2024, 2), 29);
    }

    #[test]
    fn it_parses_hours_from_date() {
        let mut weekday_map = Map::new();
        weekday_map.insert("weekend".to_string(), Value::from(false));
        weekday_map.insert("hours".to_string(), Value::from(8));

        let mut weekend_map = Map::new();
        weekend_map.insert("weekend".to_string(), Value::from(true));
        weekend_map.insert("hours".to_string(), Value::from(0));

        let day_vec =
            parse_hours_from_date((mock_date_fixed_offset(), 2021 as i32, 10 as u32, 31 as i64));

        assert_eq!(day_vec[0], weekday_map);
        assert_eq!(day_vec[1], weekend_map);
        assert_eq!(day_vec.len(), 31);
    }

    #[test]
    fn it_gets_date_map_from_date_string() {
        let map = get_date_map_from_date_string("Tue, 19 Oct 2021 10:52:28 +0200".to_string());
        let x: String = json!(map).to_string();
        assert_eq!(x, "{\"hours_worked\":[{\"hours\":8,\"weekend\":false},{\"hours\":0,\"weekend\":true},{\"hours\":0,\"weekend\":true},{\"hours\":8,\"weekend\":false},{\"hours\":8,\"weekend\":false},{\"hours\":8,\"weekend\":false},{\"hours\":8,\"weekend\":false},{\"hours\":8,\"weekend\":false},{\"hours\":0,\"weekend\":true},{\"hours\":0,\"weekend\":true},{\"hours\":8,\"weekend\":false},{\"hours\":8,\"weekend\":false},{\"hours\":8,\"weekend\":false},{\"hours\":8,\"weekend\":false},{\"hours\":8,\"weekend\":false},{\"hours\":0,\"weekend\":true},{\"hours\":0,\"weekend\":true},{\"hours\":8,\"weekend\":false},{\"hours\":8,\"weekend\":false},{\"hours\":8,\"weekend\":false},{\"hours\":8,\"weekend\":false},{\"hours\":8,\"weekend\":false},{\"hours\":0,\"weekend\":true},{\"hours\":0,\"weekend\":true},{\"hours\":8,\"weekend\":false},{\"hours\":8,\"weekend\":false},{\"hours\":8,\"weekend\":false},{\"hours\":8,\"weekend\":false},{\"hours\":8,\"weekend\":false},{\"hours\":0,\"weekend\":true},{\"hours\":0,\"weekend\":true}],\"month\":\"October\",\"total_days\":31,\"year\":\"2021\"}");
    }
}
