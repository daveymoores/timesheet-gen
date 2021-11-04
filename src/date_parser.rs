use crate::timesheet::GitLogDates;
use crate::utils::get_days_from_month;
use chrono::{TimeZone, Utc};
use std::collections::HashMap;
use std::convert::TryInto;

fn return_worked_hours_from_worked_days(worked_days: &Vec<u32>, day: &u32) -> i32 {
    let worked_day = worked_days.contains(day);
    match worked_day {
        true => 8,
        false => 0,
    }
}

fn is_weekend(date_tuple: &(i32, u32, u32), day: u32) -> i32 {
    let day_of_week_index = Utc
        .ymd(date_tuple.0, date_tuple.1, day.try_into().unwrap())
        .format("%u")
        .to_string();

    match day_of_week_index.parse().unwrap() {
        6 | 7 => 1,
        _ => 0,
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
        let is_weekend = is_weekend(&date_tuple, day.clone());
        let mut day_map: HashMap<String, i32> = HashMap::new();
        let hours_worked = return_worked_hours_from_worked_days(&worked_days, &day);

        // Each day denotes whether it is a weekend, what the hours worked are
        // and whether it has been manually edited by the user to prevent these
        // changes being overwritten when the data is synced
        day_map.insert("weekend".to_string(), is_weekend);
        day_map.insert("hours".to_string(), hours_worked);
        // all data is initially unedited
        day_map.insert("user_edited".to_string(), 0);

        vector.push(day_map);
    }

    vector
}

pub type TimesheetMonths = HashMap<String, Vec<HashMap<String, i32>>>;
pub type TimesheetYears = HashMap<String, HashMap<String, Vec<HashMap<String, i32>>>>;

// TODO export types and replace here
pub fn get_timesheet_map_from_date_hashmap(date_map: GitLogDates) -> TimesheetYears {
    let timesheet: TimesheetYears = date_map
        .into_iter()
        .map(|year_tuple| {
            let month_map: TimesheetMonths = year_tuple
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
        get_days_from_month, get_timesheet_map_from_date_hashmap, is_weekend, parse_hours_from_date,
    };
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
    fn is_weekend_returns_truthy_value_if_weekend() {
        assert_eq!(is_weekend(&(2021, 11, 6), 6), 1);
        assert_eq!(is_weekend(&(2021, 11, 28), 28), 1);
    }

    #[test]
    fn is_weekend_returns_falsy_value_if_not_weekend() {
        assert_eq!(is_weekend(&(2021, 11, 8), 8), 0);
        assert_eq!(is_weekend(&(2021, 11, 23), 23), 0);
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
}
