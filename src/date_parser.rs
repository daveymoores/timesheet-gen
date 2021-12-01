use crate::repository::{GitLogDates, Repository};
use crate::utils::get_days_from_month;
use chrono::{TimeZone, Utc};
use serde_json::{Map, Number, Value};
use std::collections::{HashMap, HashSet};
use std::convert::{TryFrom, TryInto};
use std::process;

fn return_worked_hours_from_worked_days(
    worked_days: &Vec<u32>,
    day: &u32,
    adjacent_days_in_month: &Vec<HashSet<u32>>,
) -> f64 {
    // if day exists in adjacent days, then split the number of hours by number of occurrences
    let frequency_of_day_worked_in_adjacent_timesheets: f64 = (adjacent_days_in_month
        .into_iter()
        .map(|month| month.get(day))
        .filter(|e| e.is_some())
        .collect::<Vec<Option<&u32>>>()
        .len()
        + 1) as f64;

    let worked_day = worked_days.contains(day);
    match worked_day {
        true => (8.0 / frequency_of_day_worked_in_adjacent_timesheets),
        false => 0.0,
    }
}

pub fn is_weekend(date_tuple: &(i32, u32, u32), day: u32) -> bool {
    let day_of_week_index = Utc
        .ymd(date_tuple.0, date_tuple.1, day.try_into().unwrap())
        .format("%u")
        .to_string();

    match day_of_week_index.parse().unwrap() {
        6 | 7 => true,
        _ => false,
    }
}

pub type DayMap = [(String, Value); 3];

pub fn create_single_day_object(weekend: bool, hours: f64, edited: bool) -> DayMap {
    [
        ("weekend".to_string(), Value::Bool(weekend)),
        (
            "hours".to_string(),
            Value::Number(Number::from_f64(hours).unwrap()),
        ),
        ("user_edited".to_string(), Value::Bool(edited)),
    ]
}

fn set_day_map(weekend: bool, hours: f64, edited: bool, day_map: &mut Map<String, Value>) {
    day_map.extend(create_single_day_object(weekend, hours, edited));
}

fn parse_hours_from_date(
    date_tuple: (i32, u32, u32),
    worked_days: Vec<u32>,
    repository: &mut Repository,
    adjacent_days_in_month: Vec<HashSet<u32>>,
) -> Vec<Map<String, Value>> {
    // iterate through the number of days in the month
    // for each day return the calendar day
    // if its a Weekend or day that isn't worked, set to zero, otherwise 8
    let mut vector = vec![];

    for day in 1..date_tuple.2 + 1 {
        let is_weekend: bool = is_weekend(&date_tuple, day.clone());
        let mut day_map = Map::new();
        let hours_worked =
            return_worked_hours_from_worked_days(&worked_days, &day, &adjacent_days_in_month);

        // Each day denotes whether it is a Weekend, what the hours worked are
        // and whether it has been manually edited by the user to prevent these
        // changes being overwritten when the data is synced
        match repository.timesheet {
            // if there is no timesheet at all, then just add the days in
            None => {
                set_day_map(is_weekend, hours_worked, false, &mut day_map);
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

                // if it hasn't been edited or the month isn't in the old data, then just set it
                if is_user_edited.unwrap_or(&Value::Bool(false)) == &Value::Bool(false) {
                    set_day_map(is_weekend, hours_worked, false, &mut day_map);
                } else {
                    // otherwise get the existing value from the timesheet
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
                        hours_worked_for_user_edited_day.unwrap().as_f64().unwrap(),
                        true,
                        &mut day_map,
                    );
                }
            }
        }

        vector.push(day_map);
    }

    vector
}

pub type TimesheetMonths = HashMap<String, Vec<Map<String, Value>>>;
pub type TimesheetYears = HashMap<String, HashMap<String, Vec<Map<String, Value>>>>;

fn get_adjacent_git_log_days_for_month<'a>(
    adjacent_git_log_days: Vec<GitLogDates>,
    year: &'a i32,
    month: &'a u32,
) -> Vec<HashSet<u32>> {
    let mut repo_days = vec![];
    for i in 0..adjacent_git_log_days.len() {
        if let Some(year) = adjacent_git_log_days[i].get(year) {
            if let Some(month) = year.get(month) {
                repo_days.push(month.clone());
            }
        }
    }

    repo_days
}

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
                    let adjacent_days_in_month = get_adjacent_git_log_days_for_month(
                        adjacent_git_log_dates.clone(),
                        &year_tuple.0,
                        &month_tuple.0,
                    );
                    let worked_hours_for_month = parse_hours_from_date(
                        (year_tuple.0, month_tuple.0, days_in_month),
                        worked_days,
                        repository,
                        adjacent_days_in_month,
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
        get_adjacent_git_log_days_for_month, is_weekend, parse_hours_from_date,
        return_worked_hours_from_worked_days,
    };
    use crate::repository::GitLogDates;
    use serde_json::{Map, Number, Value};
    use std::collections::{HashMap, HashSet};

    #[test]
    fn it_returns_worked_hours_from_worked_days() {
        let adjacent_days_in_month = vec![HashSet::from([1, 2, 3]), HashSet::from([2, 3, 4])];
        assert_eq!(
            return_worked_hours_from_worked_days(&vec![1, 3, 6, 22], &2, &adjacent_days_in_month),
            0.0
        );
        assert_eq!(
            return_worked_hours_from_worked_days(&vec![1, 3, 6, 22], &22, &adjacent_days_in_month),
            8.0
        );
        assert_eq!(
            return_worked_hours_from_worked_days(&vec![1, 3, 4, 22], &4, &adjacent_days_in_month),
            4.0
        );
        assert_eq!(
            return_worked_hours_from_worked_days(&vec![1, 3, 4, 22], &3, &adjacent_days_in_month),
            2.6666666666666665
        );
    }

    #[test]
    fn is_weekend_returns_truth_if_weekend() {
        assert_eq!(is_weekend(&(2021, 11, 6), 6), true);
        assert_eq!(is_weekend(&(2021, 11, 28), 28), true);
    }

    #[test]
    fn is_weekend_returns_false_if_not_weekend() {
        assert_eq!(is_weekend(&(2021, 11, 8), 8), false);
        assert_eq!(is_weekend(&(2021, 11, 23), 23), false);
    }

    #[test]
    fn it_parses_hours_from_date() {
        let adjacent_days_in_month = vec![HashSet::from([1, 2, 3]), HashSet::from([2, 3, 4])];

        let mut weekday_map = Map::new();
        weekday_map.extend(vec![
            (
                "hours".to_string(),
                Value::Number(Number::from_f64(8.0).unwrap()),
            ),
            ("user_edited".to_string(), Value::Bool(false)),
            ("Weekend".to_string(), Value::Bool(false)),
        ]);

        let mut weekend_map = Map::new();
        weekend_map.extend(vec![
            (
                "hours".to_string(),
                Value::Number(Number::from_f64(0.0).unwrap()),
            ),
            ("user_edited".to_string(), Value::Bool(false)),
            ("Weekend".to_string(), Value::Bool(true)),
        ]);

        let day_vec = parse_hours_from_date(
            (2021 as i32, 10 as u32, 31 as u32),
            vec![1, 4, 6],
            &mut Default::default(),
            adjacent_days_in_month,
        );

        assert_eq!(
            *day_vec[0].get("hours").unwrap(),
            Value::Number(Number::from_f64(4.0).unwrap())
        );
        assert_eq!(
            *day_vec[1].get("hours").unwrap(),
            Value::Number(Number::from_f64(0.0).unwrap())
        );
        assert_eq!(
            *day_vec[3].get("hours").unwrap(),
            Value::Number(Number::from_f64(4.0).unwrap())
        );
        assert_eq!(
            *day_vec[5].get("hours").unwrap(),
            Value::Number(Number::from_f64(8.0).unwrap())
        );
        assert_eq!(day_vec.len(), 31);
    }

    #[test]
    fn it_finds_adjacent_git_log_days_for_a_given_month() {
        let git_log_dates: Vec<GitLogDates> = vec![
            HashMap::from([(
                2019,
                HashMap::from([(1, HashSet::from([1, 2, 3])), (2, HashSet::from([1, 2, 3]))]),
            )]),
            HashMap::from([(
                2019,
                HashMap::from([(1, HashSet::from([2, 3, 4])), (2, HashSet::from([2, 3, 4]))]),
            )]),
        ];

        let vec_of_days = get_adjacent_git_log_days_for_month(git_log_dates, &2019, &2);
        assert_eq!(
            vec_of_days,
            vec![HashSet::from([1, 2, 3]), HashSet::from([2, 3, 4])]
        )
    }
}
