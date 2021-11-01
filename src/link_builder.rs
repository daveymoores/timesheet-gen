use crate::db;
use crate::timesheet::Timesheet;
use chrono::{Datelike, Month, Utc};
use mongodb::bson::doc;
use num_traits::cast::FromPrimitive;
use serde_json::json;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::error::Error;
use std::process;
use std::rc::Rc;

fn get_date() -> (i32, u32, &'static str) {
    let date_today = Utc::now();
    (
        date_today.year(),
        date_today.month(),
        Month::from_u32(date_today.month()).unwrap().name(),
    )
}

const EXPIRE_TIME_SECONDS: i32 = 1800;

pub async fn build_unique_uri(buffer: String) -> Result<(), Box<dyn Error>> {
    let (year, month, string_month) = get_date();
    println!("Generating timesheet for {}...", string_month);

    let db = db::Db::new().await?;
    let collection = db
        .client
        .database("timesheet-gen")
        .collection("timesheet-temp-paths");

    let random_path = db.generate_random_path(&collection).await?;

    let sheet: Timesheet = serde_json::from_str(&buffer)?;

    // TODO what happens if this is indexed and the month doesn't exist?
    // shouldn't unwrap this, but instead fallback to an empty month
    let timesheet_years = sheet
        .timesheet
        .as_ref()
        .unwrap()
        .get(&*year.to_string())
        .unwrap()
        .get(&*month.to_string())
        .unwrap();

    let document = doc! {
        "creation_date": Utc::now(),
        "random_path": &random_path,
        "name" : sheet.name.as_ref(),
        "email" : sheet.email.as_ref(),
        "namespace" : sheet.namespace.as_ref(),
        "client_name" : sheet.client_name.as_ref(),
        "client_contact_person" : sheet.client_contact_person.as_ref(),
        "address" : sheet.client_address.as_ref(),
        "timesheet" : json!(timesheet_years).to_string(),
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
        "Timesheet now available for {} minutes @ https://timesheet-gen.io/{}",
        EXPIRE_TIME_SECONDS / 60,
        &random_path
    );

    process::exit(exitcode::OK);
}
