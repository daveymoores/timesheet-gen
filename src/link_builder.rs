use crate::db;
use crate::repository::Repository;
use crate::utils::{check_for_valid_month, check_for_valid_year};
use bson::Document;
use chrono::{DateTime, Month, Utc};
use dotenv;
use mongodb::bson::doc;
use num_traits::cast::FromPrimitive;
use serde_json::{json, Map, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::io::ErrorKind;
use std::rc::Rc;
use std::{env, io, process};

type TimesheetHoursForMonth = Vec<Map<String, Value>>;

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
    sheet: &'a Repository,
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
    sheet: &Repository,
    random_path: &String,
    month_year_string: &String,
    total_hours: &f64,
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

fn calculate_total_hours(timesheet_month: &TimesheetHoursForMonth) -> f64 {
    let hours: Vec<f64> = timesheet_month
        .into_iter()
        .map(|x| x.get("hours").unwrap().as_f64().unwrap())
        .collect();

    let total_hours: f64 = hours.iter().map(|&i| i).sum();
    total_hours
}

pub async fn build_unique_uri(
    repository: Rc<RefCell<Repository>>,
    options: Vec<Option<String>>,
) -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    let mongodb_db = env::var("MONGODB_DB").expect("You must set the MONGODB_DB environment var!");
    let mongodb_collection = env::var("MONGODB_COLLECTION")
        .expect("You must set the MONGODB_COLLECTION environment var!");

    let month_year_string = get_string_month_year(&options[0], &options[1])?;
    println!("Generating timesheet for {}...", month_year_string);

    let db = db::Db::new().await?;
    let collection = db
        .client
        .database(&mongodb_db)
        .collection(&mongodb_collection);

    let sheet = repository.borrow_mut();

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
            .database(&mongodb_db)
            .run_command(
                doc! {
                    "createIndexes": &mongodb_collection,
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
    use crate::repository::{GitLogDates, Repository};
    use chrono::{TimeZone, Utc};
    use mongodb::bson::doc;
    use serde_json::{json, Map, Number, Value};
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

    fn create_mock_repository() -> Repository {
        // testing utility that returns
        // {2021: {10: {20, 23, 21}, 9: {8}}, 2020: {8: {1}}, 2019: {1: {3}}}
        let date_hashmap: GitLogDates = get_timesheet_hashmap();
        let timesheet =
            get_timesheet_map_from_date_hashmap(date_hashmap, &mut Default::default(), vec![]);

        let repository = Repository {
            timesheet: Option::from(timesheet),
            ..Default::default()
        };

        repository
    }

    fn create_mock_timesheet_hours_for_month() -> TimesheetHoursForMonth {
        let f64_value = Value::Number(Number::from_f64(8.0).unwrap());

        let mut map = Map::new();
        map.extend(vec![("hours".to_string(), f64_value)]);

        let month: TimesheetHoursForMonth = vec![map.clone(), map.clone(), map.clone()];
        month
    }

    #[test]
    fn it_builds_document() {
        let timesheet = Repository {
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
        "total_hours" : 36.0,
        "month_year": "November, 2021",
        };

        assert_eq!(
            build_document(
                Utc.ymd(2014, 11, 28).and_hms(12, 0, 9),
                &timesheet,
                &"fbfxhs".to_string(),
                &"November, 2021".to_string(),
                &36.0,
                &month
            ),
            doc
        );
    }

    #[test]
    fn it_calculates_total_hours() {
        let month = create_mock_timesheet_hours_for_month();
        assert_eq!(calculate_total_hours(&month), 24.0);
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

        let timesheet = create_mock_repository();
        assert!(find_month_from_timesheet(&timesheet, &options).is_err());
    }

    #[test]
    fn it_returns_month_from_timesheet() {
        let options = vec![
            Option::from("10".to_owned()),
            Option::from("2021".to_owned()),
        ];

        let timesheet = create_mock_repository();
        assert!(find_month_from_timesheet(&timesheet, &options).is_ok());
    }
}
