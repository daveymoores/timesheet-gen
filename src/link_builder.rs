use crate::db;
use crate::timesheet::Timesheet;
use chrono::{Datelike, Month, Utc};
use futures::StreamExt;
use mongodb::bson::doc;
use num_traits::cast::FromPrimitive;
use regex::Regex;
use serde_json::json;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::format;
use std::io::ErrorKind;
use std::rc::Rc;
use std::{io, process};

fn get_string_month_year(
    month: &Option<String>,
    year: &Option<String>,
) -> Result<String, Box<dyn Error>> {
    let month_u32 = month
        .as_ref()
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, format!("Month not found")))?
        .parse::<u32>()?;

    let year_string = year
        .as_ref()
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, format!("Year not found")))?;

    let year_regex = Regex::new(r"^((19|20)\d{2})$").unwrap();
    let month_regex = Regex::new(r"^(1[0-2]|[1-9])$").unwrap();

    if !year_regex.is_match(year_string) {
        // into implements a conversion between &str and Box<dyn Error>
        return Err("Not a real year".into());
    }

    if !month_regex.is_match(&*month_u32.to_string()) {
        return Err("Not a real month".into());
    }

    Ok(format!(
        "{}, {}",
        Month::from_u32(month_u32).unwrap().name(),
        year_string
    ))
}

fn find_month_from_timesheet<'a>(
    sheet: &'a Timesheet,
    options: &'a Vec<Option<String>>,
) -> Result<&'a Vec<HashMap<String, i32>>, Box<dyn Error>> {
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

const EXPIRE_TIME_SECONDS: i32 = 1800;

pub async fn build_unique_uri(
    buffer: String,
    options: Vec<Option<String>>,
) -> Result<(), Box<dyn Error>> {
    let month_year_string = get_string_month_year(&options[0], &options[1])?;
    println!("Generating timesheet for {}...", month_year_string);

    let sheet: Timesheet = serde_json::from_str(&buffer)?;

    let timesheet_month = find_month_from_timesheet(&sheet, &options)?;
    let hours: Vec<&i32> = timesheet_month
        .into_iter()
        .map(|x| x.get("hours").unwrap())
        .collect();
    let total_hours: i32 = hours.iter().map(|&i| i).sum();

    let db = db::Db::new().await?;
    let collection = db
        .client
        .database("timesheet-gen")
        .collection("timesheet-temp-paths");

    let random_path = db.generate_random_path(&collection).await?;

    let document = doc! {
        "creation_date": Utc::now(),
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

    // Check for existing index for TTL on the collection
    let index_names = collection.list_index_names().await?;

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
                            "expireAfterSeconds": EXPIRE_TIME_SECONDS,
                            "unique": true
                        },
                    ]
                },
                None,
            )
            .await?;
    }

    collection.insert_one(document.clone(), None).await?;

    println!(
        "Timesheet now available for {} minutes @ http://localhost:8080/{}",
        EXPIRE_TIME_SECONDS / 60,
        &random_path
    );

    process::exit(exitcode::OK);
}

#[cfg(test)]
mod test {
    use crate::date_parser::get_timesheet_map_from_date_hashmap;
    use crate::link_builder::{find_month_from_timesheet, get_string_month_year};
    use crate::testing_helpers;
    use crate::timesheet::{GitLogDates, Timesheet};

    fn create_mock_timesheet() -> Timesheet {
        // testing utility that returns
        // {2021: {10: {20, 23, 21}, 9: {8}}, 2020: {8: {1}}, 2019: {1: {3}}}
        let date_hashmap: GitLogDates = testing_helpers::get_timesheet_hashmap();
        let timesheet = get_timesheet_map_from_date_hashmap(date_hashmap);

        let timesheet = Timesheet {
            timesheet: Option::from(timesheet),
            ..Default::default()
        };

        timesheet
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
