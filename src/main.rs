extern crate clap;
use chrono::prelude::*;
use clap::{App, Arg, ArgMatches, Error};
use regex::Regex;
use std::ffi::OsString;
use std::process;

#[derive(Debug, Default)]
struct Cli<'a> {
    matches: ArgMatches<'a>,
    command: Option<String>,
    options: Option<String>,
}

fn validate_date_options(v: String) -> Result<(), String> {
    // create regex for the date string
    let re = Regex::new(
        r"(?P<year>\d{4})?\s?(?P<month>\d{1,2})?\s?(?P<day>\d{1,2})?\s?(?P<hour>\d{1,2})",
    )
    .unwrap();
    let date_values = re.captures(&*v).unwrap();
    let date_value_ok: bool = !&date_values["year"].is_empty()
        | !&date_values["month"].is_empty()
        | !&date_values["day"].is_empty()
        | !&date_values["hour"].is_empty();

    if date_value_ok {
        Ok(())
    } else {
        Err(String::from("The value did not contain numeric values"))
    }
}

impl Cli<'_> {
    fn new() -> Self {
        Self::new_from(std::env::args_os().into_iter()).unwrap_or_else(|e| e.exit())
    }

    fn new_from<I, T>(args: I) -> Result<Self, clap::Error>
    where
        I: Iterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let app: App = App::new("timesheet-gen")
            .version("0.1")
            .author("Davey Moores")
            .about(
                "Minimal configuration, simple timesheets for sharing via pdf download or unique link.",
            ).arg(
            Arg::with_name("init")
                .short("i")
                .long("init")
                .value_name("path")
                .help(
                    "Pass optional 'path' to git repository. Defaults \n\
            to current directory",
                )
                .takes_value(true)
                .min_values(0)
                .max_values(1),
        ).arg(
            Arg::with_name("run-mode")
                .short("o")
                .long("run-mode")
                .value_names(&["cd | d"])
                .help(
                    "Whether the repository should \n\
               be initialised as commit based (cb) or default (d)",
                )
                .takes_value(true)
                .min_values(0)
                .max_values(1),
        ).arg(
            Arg::with_name("edit")
                .short("e")
                .long("edit")
                .value_names(&["year", "month", "day", "hour"])
                .validator(validate_date_options)
                .help(
                    "Sets the hours worked for a given day. \n\
            The arguments are parsed from left to right, and omitted \n\
            values default to the current day/month/year",
                )
                .takes_value(true)
        ).arg(
            Arg::with_name("remove")
                .short("r")
                .long("remove")
                .value_names(&["year", "month", "day", "hour"])
                .help("Removes the work entry for a given day/month/year.")
                .takes_value(true)
                .min_values(1)
                .max_values(4),
        ).arg(
            Arg::with_name("make")
                .short("m")
                .long("make")
                .value_names(&["month"])
                .help(
                    "Generates a timesheet for a given month. \n\
            Defaults to last completed calendar month.",
                )
                .takes_value(true)
                .min_values(0)
                .max_values(1),
        );

        // extract the matches
        let matches = app.get_matches_from_safe(args)?;

        if matches.args.is_empty() {
            return Err(Error {
                message: "No matches for inputs".to_string(),
                kind: clap::ErrorKind::EmptyValue,
                info: None,
            });
        }

        Ok(Cli {
            matches,
            command: None,
            options: None,
        })
    }

    pub fn parse_commands(&self, matches: &ArgMatches) -> Result<Cli, clap::Error> {
        let mut options = None;
        let mut command = None;

        let date_time: DateTime<Local> = Local::now();
        let month = date_time.month().to_string();
        let day = date_time.day().to_string();

        if matches.is_present("init") {
            options = Some(matches.value_of("init").unwrap_or(".").to_string());
            command = Some("init".to_string());
        }

        if matches.is_present("make") {
            options = Some(matches.value_of("make").unwrap_or(&*month).to_string());
            command = Some("make".to_string());
        }

        if matches.is_present("edit") {
            match matches.value_of("edit") {
                None => {
                    return Err(Error {
                        message: "Missing required argument".to_string(),
                        kind: clap::ErrorKind::MissingRequiredArgument,
                        info: None,
                    });
                }
                Some(matches) => {
                    options = Some(matches.to_string());
                    command = Some("edit".to_string());
                }
            }
        }

        if matches.is_present("remove") {
            options = Some(matches.value_of("remove").unwrap_or(&*day).to_string());
            command = Some("remove".to_string());
        }

        if matches.is_present("run_mode") {
            options = Some(matches.value_of("run_mode").unwrap_or("d").to_string());
            command = Some("run_mode".to_string());
        }

        Ok(Cli {
            options,
            command,
            ..Default::default()
        })
    }

    fn run(&self) -> Result<(), clap::Error> {
        let matches = &self.matches;
        let _cli = self.parse_commands(&matches)?;

        Ok(())
    }
}

fn main() {
    let cli = Cli::new();
    cli.run().unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_an_error_when_no_command_args_are_passed() {
        let result = Cli::new_from([""].iter());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind, clap::ErrorKind::EmptyValue);
    }

    #[test]
    fn finds_a_match_from_command_args() {
        let result = Cli::new_from(["exename", "--init"].iter()).unwrap();
        assert!(result.matches.args.contains_key("init"));
    }

    #[test]
    fn returns_a_default_option_for_init() {
        let cli: Cli = Cli::new_from(["exename", "--init"].iter()).unwrap();
        let new_cli = cli.parse_commands(&cli.matches);
        let result = new_cli.unwrap();
        assert_eq!(result.options.unwrap(), ".");
        assert_eq!(result.command.unwrap().clone(), "init");
    }

    #[test]
    fn returns_a_passed_value_for_init() {
        let cli: Cli = Cli::new_from(["exename", "--init", "/usr/path"].iter()).unwrap();
        let new_cli = cli.parse_commands(&cli.matches);
        assert_eq!(new_cli.unwrap().options.unwrap(), "/usr/path");
    }

    #[test]
    fn returns_a_default_option_for_make() {
        let date_time: DateTime<Local> = Local::now();
        let month = date_time.month();

        let cli: Cli = Cli::new_from(["exename", "--make"].iter()).unwrap();
        let new_cli = cli.parse_commands(&cli.matches);
        let result = new_cli.unwrap();
        assert_eq!(
            result.options.unwrap().parse::<u32>().unwrap().clone(),
            month
        );
        assert_eq!(result.command.unwrap().clone(), "make");
    }

    #[test]
    fn returns_a_passed_value_for_make() {
        let cli: Cli = Cli::new_from(["exename", "--make", "10"].iter()).unwrap();
        let new_cli = cli.parse_commands(&cli.matches);

        assert_eq!(
            new_cli.unwrap().options.unwrap().parse::<u32>().unwrap(),
            10
        );
    }

    #[test]
    fn returns_an_error_when_no_arg_is_present_for_edit() {
        let result = Cli::new_from(["exename", "--edit"].iter());
        assert!(result.is_err());
    }

    // #[test]
    // fn returns_a_passed_value_for_edit() {
    //     let cli: Cli = Cli::new_from(["exename", "--edit", "2021 10 30 6"].iter()).unwrap();
    //     let new_cli = cli.parse_commands(&cli.matches);
    //
    //     assert_eq!(
    //         new_cli.unwrap().options.unwrap().parse::<u32>().unwrap(),
    //         10
    //     );
    // }
}
