use crate::timesheet::GitLogDates;
use chrono::{Date, FixedOffset, NaiveDate, TimeZone, Utc};
use std::collections::HashMap;
use std::convert::TryInto;

type YearTuple = (Date<FixedOffset>, i32, u32, u32);

pub fn get_days_from_month(year: i32, month: u32) -> u32 {
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
    .num_days() as u32
}

fn return_worked_hours_from_worked_days(worked_days: &Vec<u32>, day: &u32) -> i32 {
    let worked_day = worked_days.contains(day);
    match worked_day {
        true => 8,
        false => 0,
    }
}

fn parse_hours_from_date(
    date_tuple: (i32, u32, u32),
    worked_days: Vec<u32>,
) -> Vec<HashMap<String, i32>> {
    // iterate through the number of days in the month
    // for each day return the calendar day
    // if its a weekend or day that isn't worked, set to zero, otherwise 8
    let mut vector = vec![];

    for day in 1..date_tuple.2 + 1 {
        let mut day_map: HashMap<String, i32> = HashMap::new();

        let day_of_week_index = Utc
            .ymd(date_tuple.0, date_tuple.1, day.try_into().unwrap())
            .format("%u")
            .to_string();

        let weekend = match day_of_week_index.parse().unwrap() {
            6 | 7 => 1,
            _ => 0,
        };

        let hours_worked = return_worked_hours_from_worked_days(&worked_days, &day);

        day_map.insert("weekend".to_string(), weekend);
        day_map.insert("hours".to_string(), hours_worked);

        vector.push(day_map);
    }

    vector
}

pub type TimesheetYears = HashMap<String, HashMap<String, Vec<HashMap<String, i32>>>>;

// TODO export types and replace here
pub fn get_timesheet_map_from_date_hashmap(date_map: GitLogDates) -> TimesheetYears {
    let timesheet: TimesheetYears = date_map
        .into_iter()
        .map(|year_tuple| {
            let month_map: HashMap<String, Vec<HashMap<String, i32>>> = year_tuple
                .1
                .clone()
                .into_iter()
                .map(|month_tuple| {
                    let mut worked_days = month_tuple.1.into_iter().collect::<Vec<u32>>();
                    worked_days.sort();
                    let days_in_month = get_days_from_month(year_tuple.0, month_tuple.0);
                    let worked_hours_for_month = parse_hours_from_date(
                        (year_tuple.0, month_tuple.0, days_in_month),
                        worked_days,
                    );
                    (month_tuple.0.to_string(), worked_hours_for_month)
                })
                .collect();
            (year_tuple.0.to_string(), month_map)
        })
        .collect();

    timesheet
}

#[cfg(test)]
mod tests {
    use crate::date_parser::{
        get_days_from_month, get_timesheet_map_from_date_hashmap, parse_hours_from_date,
    };
    use crate::testing_helpers;
    use crate::timesheet::GitLogDates;
    use chrono::{Date, DateTime, FixedOffset, TimeZone};
    use serde_json::{json, Map, Value};
    use std::collections::{HashMap, HashSet};

    fn mock_date_fixed_offset() -> Date<FixedOffset> {
        let date_time = DateTime::parse_from_rfc2822("Tue, 19 Oct 2021 10:52:28 +0200");
        let date = date_time.unwrap().date();
        date
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
        let mut weekday_map = HashMap::new();
        weekday_map.insert("weekend".to_string(), 0);
        weekday_map.insert("hours".to_string(), 8);

        let mut weekend_map = HashMap::new();
        weekend_map.insert("weekend".to_string(), 1);
        weekend_map.insert("hours".to_string(), 0);

        let day_vec = parse_hours_from_date((2021 as i32, 10 as u32, 31 as u32), vec![1, 4, 6]);

        assert_eq!(day_vec[0], weekday_map);
        assert_eq!(day_vec[3], weekday_map);
        assert_eq!(day_vec[5], weekday_map);
        assert_eq!(day_vec[1], weekend_map);
        assert_eq!(day_vec.len(), 31);
    }

    // TODO ignored because this fails as the keys are in a different order
    #[test]
    #[ignore]
    fn it_gets_date_map_from_date_hashmap() {
        // testing utility that returns
        // {2021: {10: {20, 23, 21}, 9: {8}}, 2020: {8: {1}}, 2019: {1: {3}}}
        let date_hashmap: GitLogDates = testing_helpers::get_timesheet_hashmap();

        let map = get_timesheet_map_from_date_hashmap(date_hashmap);
        let x: String = json!(map).to_string();
        assert_eq!(x, "{\"2019\":{\"1\":[{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":8,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0}]},\"2020\":{\"8\":[{\"hours\":8,\"weekend\":1},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":0}]},\"2021\":{\"10\":[{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":8,\"weekend\":0},{\"hours\":8,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":8,\"weekend\":1},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":1}],\"9\":[{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":8,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":1},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0},{\"hours\":0,\"weekend\":0}]}}");
    }
}
