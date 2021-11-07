use crate::db;
use crate::timesheet::Timesheet;
use crate::utils::{check_for_valid_month, check_for_valid_year};
use bson::Document;
use chrono::{DateTime, Month, Utc};
use mongodb::bson::doc;
use num_traits::cast::FromPrimitive;
use serde_json::json;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::io::ErrorKind;
use std::rc::Rc;
use std::{env, io, process};

type TimesheetHoursForMonth = Vec<HashMap<String, i32>>;

fn get_string_month_year(
    month: &Option<String>,
    year: &Option<String>,
) -> Result<String, Box<dyn Error>> {
    let month_u32 = check_for_valid_month(month)?;
    let year_string = check_for_valid_year(year)?;

    Ok(format!(
        "{}, {}",
        Month::from_u32(month_u32).unwrap().name(),
        year_string
    ))
}

fn find_month_from_timesheet<'a>(
    sheet: &'a Timesheet,
    options: &'a Vec<Option<String>>,
) -> Result<&'a TimesheetHoursForMonth, Box<dyn Error>> {
    // safe to unwrap options here as it would have been caught above
    let timesheet_month = sheet
        .timesheet
        .as_ref()
        .unwrap()
        .get(&options[1].as_ref().unwrap().to_string())
        .ok_or_else(|| {
            io::Error::new(
                ErrorKind::InvalidInput,
                format!("Year not found in interaction data"),
            )
        })?
        .get(&options[0].as_ref().unwrap().to_string())
        .ok_or_else(|| {
            io::Error::new(
                ErrorKind::InvalidInput,
                format!("Month not found in interaction data"),
            )
        })?;

    Ok(timesheet_month)
}

fn build_document(
    date: DateTime<Utc>,
    sheet: &Timesheet,
    random_path: &String,
    month_year_string: &String,
    total_hours: &i32,
    timesheet_month: &TimesheetHoursForMonth,
) -> Document {
    let document = doc! {
        "creation_date": date,
        "random_path": &random_path,
        "name" : sheet.name.as_ref(),
        "email" : sheet.email.as_ref(),
        "namespace" : sheet.namespace.as_ref(),
        "client_name" : sheet.client_name.as_ref(),
        "client_contact_person" : sheet.client_contact_person.as_ref(),
        "address" : sheet.client_address.as_ref(),
        "timesheet" : json!(timesheet_month).to_string(),
        "total_hours" : total_hours,
        "month_year": month_year_string,
    };

    document
}

fn calculate_total_hours(timesheet_month: &TimesheetHoursForMonth) -> i32 {
    let hours: Vec<&i32> = timesheet_month
        .into_iter()
        .map(|x| x.get("hours").unwrap())
        .collect();

    let total_hours: i32 = hours.iter().map(|&i| i).sum();
    total_hours
}

pub async fn build_unique_uri(
    timesheet: Rc<RefCell<Timesheet>>,
    options: Vec<Option<String>>,
) -> Result<(), Box<dyn Error>> {
    let month_year_string = get_string_month_year(&options[0], &options[1])?;
    println!("Generating timesheet for {}...", month_year_string);

    let db = db::Db::new().await?;
    let collection = db
        .client
        .database("timesheet-gen")
        .collection("timesheet-temp-paths");

    let sheet = timesheet.borrow_mut();

    let timesheet_month = find_month_from_timesheet(&sheet, &options)?;
    let total_hours = calculate_total_hours(&timesheet_month);
    let random_path: String = db.generate_random_path(&collection).await?;
    let document = build_document(
        Utc::now(),
        &sheet,
        &random_path,
        &month_year_string,
        &total_hours,
        &timesheet_month,
    );

    // Check for existing index for TTL on the collection
    let index_names = collection.list_index_names().await?;

    let expire_time_seconds: i32 = env::var("EXPIRE_TIME_SECONDS")
        .expect("You must set the EXPIRE_TIME_SECONDS environment var!")
        .parse()
        .expect("Expire time can't be parsed to i32");

    if !index_names.contains(&String::from("expiration_date")) {
        // create TTL index to expire documents after 30 minutes
        db.client
            .database("timesheet-gen")
            .run_command(
                doc! {
                    "createIndexes": "timesheet-temp-paths",
                    "indexes": [
                        {
                            "key": { "creation_date": 1 },
                            "name": "expiration_date",
                            "expireAfterSeconds": expire_time_seconds,
                            "unique": true
                        },
                    ]
                },
                None,
            )
            .await?;
    }

    collection.insert_one(document.clone(), None).await?;

    let timesheet_gen_uri: String = format!(
        "{}/{}",
        env::var("TIMESHEET_GEN_URI").expect("You must set the TIMESHEET_GEN_URI environment var!"),
        &random_path
    );

    println!(
        "Timesheet now available for {} minutes @ {}",
        expire_time_seconds / 60,
        timesheet_gen_uri
    );

    process::exit(exitcode::OK);
}

#[cfg(test)]
mod test {
    use crate::date_parser::get_timesheet_map_from_date_hashmap;
    use crate::link_builder::{
        build_document, calculate_total_hours, find_month_from_timesheet, get_string_month_year,
        TimesheetHoursForMonth,
    };
    use crate::timesheet::{GitLogDates, Timesheet};
    use chrono::{TimeZone, Utc};
    use mongodb::bson::doc;
    use serde_json::json;
    use std::collections::{HashMap, HashSet};

    pub fn get_timesheet_hashmap() -> GitLogDates {
        let date_hashmap: GitLogDates = vec![
            (2020, vec![(8, vec![1])]),
            (2019, vec![(1, vec![3])]),
            (2021, vec![(10, vec![23, 20, 21]), (9, vec![8])]),
        ]
        .into_iter()
        .map(|x| {
            let y: HashMap<u32, HashSet<u32>> =
                x.1.into_iter()
                    .map(|k| {
                        let n: HashSet<u32> = k.1.into_iter().collect();
                        (k.0, n)
                    })
                    .collect();
            (x.0, y)
        })
        .collect();

        date_hashmap
    }

    fn create_mock_timesheet() -> Timesheet {
        // testing utility that returns
        // {2021: {10: {20, 23, 21}, 9: {8}}, 2020: {8: {1}}, 2019: {1: {3}}}
        let date_hashmap: GitLogDates = get_timesheet_hashmap();
        let timesheet = get_timesheet_map_from_date_hashmap(date_hashmap, &mut Default::default());

        let timesheet = Timesheet {
            timesheet: Option::from(timesheet),
            ..Default::default()
        };

        timesheet
    }

    fn create_mock_timesheet_hours_for_month() -> TimesheetHoursForMonth {
        let month: TimesheetHoursForMonth = vec![
            vec![("hours".to_string(), 8 as i32)].into_iter().collect(),
            vec![("hours".to_string(), 8 as i32)].into_iter().collect(),
            vec![("hours".to_string(), 8 as i32)].into_iter().collect(),
        ];

        month
    }

    #[test]
    fn it_builds_document() {
        let timesheet = Timesheet {
            namespace: Option::from("Some project".to_string()),
            name: Option::from("Barry Balls".to_string()),
            email: Option::from("barry.balls@123.reg".to_string()),
            client_name: Option::from("Alphabet".to_string()),
            client_contact_person: Option::from("Jenny boomers".to_string()),
            client_address: Option::from("Tron, Tron, Tron".to_string()),
            ..Default::default()
        };
        let month = create_mock_timesheet_hours_for_month();

        let doc = doc! {
        "creation_date": Utc.ymd(2014, 11, 28).and_hms(12, 0, 9),
        "random_path": "fbfxhs",
        "name" : "Barry Balls",
        "email" : "barry.balls@123.reg",
        "namespace" : "Some project",
        "client_name" :"Alphabet",
        "client_contact_person" : "Jenny boomers",
        "address" : "Tron, Tron, Tron",
        "timesheet" : json!(month).to_string(),
        "total_hours" : 36,
        "month_year": "November, 2021",
        };

        assert_eq!(
            build_document(
                Utc.ymd(2014, 11, 28).and_hms(12, 0, 9),
                &timesheet,
                &"fbfxhs".to_string(),
                &"November, 2021".to_string(),
                &36,
                &month
            ),
            doc
        );
    }

    #[test]
    fn it_calculates_total_hours() {
        let month = create_mock_timesheet_hours_for_month();
        assert_eq!(calculate_total_hours(&month), 24);
    }

    #[test]
    fn it_throws_error_getting_string_month_year_with_incorrect_month() {
        let options = vec![
            Option::from("blah blah".to_owned()),
            Option::from("2021".to_owned()),
        ];
        assert!(get_string_month_year(&options[0], &options[1]).is_err());
    }

    #[test]
    fn it_throws_error_getting_string_month_year_with_incorrect_year() {
        let options = vec![
            Option::from("10".to_owned()),
            Option::from("blah blah".to_owned()),
        ];
        assert!(get_string_month_year(&options[0], &options[1]).is_err());
    }

    #[test]
    fn it_throws_error_getting_string_month_year_with_year_that_doesnt_exist() {
        let options = vec![
            Option::from("10".to_owned()),
            Option::from("1345".to_owned()),
        ];
        assert!(get_string_month_year(&options[0], &options[1]).is_err());
    }

    #[test]
    fn it_throws_error_getting_string_month_year_with_month_that_doesnt_exist() {
        let options = vec![
            Option::from("15".to_owned()),
            Option::from("1345".to_owned()),
        ];
        assert!(get_string_month_year(&options[0], &options[1]).is_err());
    }

    #[test]
    fn it_gets_string_for_month_year() {
        let options = vec![
            Option::from("10".to_owned()),
            Option::from("2021".to_owned()),
        ];
        assert_eq!(
            get_string_month_year(&options[0], &options[1]).unwrap(),
            "October, 2021".to_string()
        );
    }

    #[test]
    fn it_throws_error_if_month_cannot_be_found() {
        let options = vec![
            Option::from("2".to_owned()),
            Option::from("2021".to_owned()),
        ];

        let timesheet = create_mock_timesheet();
        assert!(find_month_from_timesheet(&timesheet, &options).is_err());
    }

    #[test]
    fn it_returns_month_from_timesheet() {
        let options = vec![
            Option::from("10".to_owned()),
            Option::from("2021".to_owned()),
        ];

        let timesheet = create_mock_timesheet();
        assert!(find_month_from_timesheet(&timesheet, &options).is_ok());
    }
}
