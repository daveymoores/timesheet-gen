use crate::repository::{GitLogDates, Repository};
use crate::utils::get_days_from_month;
use chrono::{TimeZone, Utc};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::process;

fn return_worked_hours_from_worked_days(worked_days: &Vec<u32>, day: &u32) -> i32 {
    let worked_day = worked_days.contains(day);
    match worked_day {
        true => 8,
        false => 0,
    }
}

pub fn is_weekend(date_tuple: &(i32, u32, u32), day: u32) -> i32 {
    let day_of_week_index = Utc
        .ymd(date_tuple.0, date_tuple.1, day.try_into().unwrap())
        .format("%u")
        .to_string();

    match day_of_week_index.parse().unwrap() {
        6 | 7 => 1,
        _ => 0,
    }
}

fn set_day_map(weekend: i32, hours: i32, edited: i32, day_map: &mut HashMap<String, i32>) {
    day_map.extend(vec![
        ("weekend".to_string(), weekend),
        ("hours".to_string(), hours),
        ("user_edited".to_string(), edited),
    ]);
}

fn parse_hours_from_date(
    date_tuple: (i32, u32, u32),
    worked_days: Vec<u32>,
    repository: &mut Repository,
) -> Vec<HashMap<String, i32>> {
    // iterate through the number of days in the month
    // for each day return the calendar day
    // if its a Weekend or day that isn't worked, set to zero, otherwise 8
    let mut vector = vec![];

    for day in 1..date_tuple.2 + 1 {
        let is_weekend = is_weekend(&date_tuple, day.clone());
        let mut day_map: HashMap<String, i32> = HashMap::new();
        let hours_worked = return_worked_hours_from_worked_days(&worked_days, &day);

        // Each day denotes whether it is a Weekend, what the hours worked are
        // and whether it has been manually edited by the user to prevent these
        // changes being overwritten when the data is synced
        match repository.timesheet {
            // if there is no timesheet at all, then just add the days in
            None => {
                set_day_map(is_weekend, hours_worked, 0, &mut day_map);
            }
            // if there is a timesheet then lets check whether the day value has been edited
            // before setting the hour value
            Some(_) => {
                let day_index: usize = usize::try_from(day).unwrap() - 1;
                let is_user_edited = match repository.get_timesheet_entry(
                    &date_tuple.0.to_string(),
                    &date_tuple.1,
                    day_index,
                    "user_edited".to_string(),
                ) {
                    Ok(result) => result,
                    Err(err) => {
                        eprintln!("Error retrieving timesheet entry: {}", err);
                        process::exit(exitcode::DATAERR);
                    }
                };

                if is_user_edited.unwrap() == &0 {
                    set_day_map(is_weekend, hours_worked, 0, &mut day_map);
                } else {
                    let hours_worked_for_user_edited_day = match repository.get_timesheet_entry(
                        &date_tuple.0.to_string(),
                        &date_tuple.1,
                        day_index,
                        "hours".to_string(),
                    ) {
                        Ok(result) => result,
                        Err(err) => {
                            eprintln!("Error retrieving timesheet entry: {}", err);
                            process::exit(exitcode::DATAERR);
                        }
                    };

                    set_day_map(
                        is_weekend,
                        *hours_worked_for_user_edited_day.unwrap(),
                        1,
                        &mut day_map,
                    );
                }
            }
        }

        vector.push(day_map);
    }

    vector
}

pub type TimesheetMonths = HashMap<String, Vec<HashMap<String, i32>>>;
pub type TimesheetYears = HashMap<String, HashMap<String, Vec<HashMap<String, i32>>>>;

pub fn get_timesheet_map_from_date_hashmap(
    git_log_dates: GitLogDates,
    repository: &mut Repository,
    adjacent_git_log_dates: Vec<GitLogDates>,
) -> TimesheetYears {
    let timesheet_years: TimesheetYears = git_log_dates
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
                        repository,
                    );
                    (month_tuple.0.to_string(), worked_hours_for_month)
                })
                .collect();
            (year_tuple.0.to_string(), month_map)
        })
        .collect();

    timesheet_years
}

#[cfg(test)]
mod tests {
    use crate::date_parser::{
        is_weekend, parse_hours_from_date, return_worked_hours_from_worked_days,
    };
    use std::collections::HashMap;

    #[test]
    fn it_returns_worked_hours_from_worked_days() {
        assert_eq!(
            return_worked_hours_from_worked_days(&vec![1, 3, 6, 22], &13),
            0
        );
        assert_eq!(
            return_worked_hours_from_worked_days(&vec![1, 3, 6, 22], &22),
            8
        );
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
        weekday_map.extend(vec![
            ("hours".to_string(), 8),
            ("user_edited".to_string(), 0),
            ("Weekend".to_string(), 0),
        ]);

        let mut weekend_map = HashMap::new();
        weekend_map.extend(vec![
            ("hours".to_string(), 0),
            ("user_edited".to_string(), 0),
            ("Weekend".to_string(), 1),
        ]);

        let day_vec = parse_hours_from_date(
            (2021 as i32, 10 as u32, 31 as u32),
            vec![1, 4, 6],
            &mut Default::default(),
        );

        assert_eq!(*day_vec[0].get("hours").unwrap(), 8);
        assert_eq!(*day_vec[3].get("hours").unwrap(), 8);
        assert_eq!(*day_vec[5].get("hours").unwrap(), 8);
        assert_eq!(*day_vec[1].get("hours").unwrap(), 0);
        assert_eq!(day_vec.len(), 31);
    }
}
