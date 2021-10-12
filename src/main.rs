extern crate clap;
use chrono::prelude::*;
use clap::{App, Arg, ArgMatches, Error};
use std::ffi::OsString;
use std::process;

#[derive(Debug, Default)]
struct Cli<'a> {
    matches: ArgMatches<'a>,
    command: Option<String>,
    options: Vec<Option<String>>,
}

fn has_cb_or_d(v: String) -> Result<(), String> {
    if v == "d" || v == "cb" {
        return Ok(());
    }
    Err(String::from(
        "Permitted values are 'cb' (commit-based) or 'd' (default)",
    ))
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
        let hour_arg = Arg::with_name("hour")
            .short("h")
            .long("hour")
            .value_name("xx")
            .help(
                "sets the hour value. When the day/month/year \n\
                    isn't set, it defaults to the current day",
            )
            .required(true);

        let day_arg = Arg::with_name("day")
            .requires("hour")
            .short("d")
            .long("day")
            .value_name("xx")
            .help(
                "sets the day value. When the month/year \n\
                    isn't set, it defaults to the current day",
            );

        let month_arg = Arg::with_name("month")
            .requires("day")
            .short("m")
            .long("month")
            .value_name("xx")
            .help(
                "sets the day value. When the month/year \n\
                    isn't set, it defaults to the current day",
            );

        let year_arg = Arg::with_name("year")
            .requires("month")
            .short("y")
            .long("year")
            .value_name("xxxx")
            .help(
                "sets the day value. When the month/year \n\
                    isn't set, it defaults to the current day",
            );

        let app: App = App::new("timesheet-gen")
            .version("0.1")
            .author("Davey Moores")
            .about(
                "Minimal configuration, simple timesheets for sharing via pdf download or unique link.",
            ).subcommand(
            App::new("init")
                .about("Initialise for current or specified repository")
                .arg(Arg::with_name("path")
                    .value_name("path")
                    .help(
                        "Pass optional 'path' to git repository. Defaults \n\
                            to current directory",
                    )
                    .required(false)))
            .subcommand(
            App::new("run-mode")
                .about("Whether the repository should \n\
               be initialised as commit based (cb) or default (d)").arg(
                Arg::with_name("mode")
                .value_name("mode")
                    .validator(has_cb_or_d)
                .help(
                    "Whether the repository should \n\
               be initialised as commit based (cb) or default (d)",
                )))
            .subcommand(App::new("edit")
                .about("Change the hours worked value for a given day")
                .arg(&hour_arg)
                .arg(&day_arg)
                .arg(&month_arg)
                .arg(&year_arg))
            .subcommand(App::new("remove")
                .about("Remove the entry for a given day")
                .arg(Arg::with_name("day")
                    .short("d")
                    .long("day")
                    .value_name("xx")
                    .help(
                        "sets the day value. When the month/year \n\
                    isn't set, it defaults to the current day",
                    ))
                .arg(&month_arg)
                .arg(&year_arg))
            .subcommand(App::new("make")
                .about("Change the hours worked value for a given day")
                .arg(Arg::with_name("month")
                    .short("m")
                    .long("month")
                    .value_name("xx")
                    .help(
                        "sets the day value. When the month/year \n\
                    isn't set, it defaults to the current day",
                    ))
                .arg(&year_arg));

        // extract the matches
        let matches = app.get_matches_from_safe(args)?;

        Ok(Cli {
            matches,
            command: None,
            options: vec![None],
        })
    }

    pub fn parse_commands(&self, matches: &ArgMatches) -> Result<Cli, clap::Error> {
        let mut options: Vec<Option<String>> = vec![];
        let command: Option<String>;

        let date_time: DateTime<Local> = Local::now();
        let year = date_time.year().to_string();
        let month = date_time.month().to_string();
        let day = date_time.day().to_string();

        if let Some(init) = matches.subcommand_matches("init") {
            // set default value of current directory
            options.push(Some(init.value_of("path").unwrap_or(".").to_string()));
            command = Some("init".to_string());
        } else if let Some(make) = matches.subcommand_matches("make") {
            // set default value of current month
            options.push(Some(make.value_of("month").unwrap_or(&*month).to_string()));
            command = Some("make".to_string());
        } else if let Some(edit) = matches.subcommand_matches("edit") {
            // this will error out if the preceding date value isn't passed
            // so I can happily set default here knowing that just the day/month/year will make it through
            options.push(Some(edit.value_of("hour").unwrap().to_string()));
            options.push(Some(edit.value_of("day").unwrap_or(&day).to_string()));
            options.push(Some(edit.value_of("month").unwrap_or(&month).to_string()));
            options.push(Some(edit.value_of("year").unwrap_or(&year).to_string()));
            command = Some("edit".to_string());
        } else if let Some(remove) = matches.subcommand_matches("remove") {
            // same here...
            options.push(Some(remove.value_of("day").unwrap_or(&day).to_string()));
            options.push(Some(remove.value_of("month").unwrap_or(&month).to_string()));
            options.push(Some(remove.value_of("year").unwrap_or(&year).to_string()));
            command = Some("remove".to_string());
        } else if let Some(run_mode) = matches.subcommand_matches("run-mode") {
            options.push(Some(run_mode.value_of("mode").unwrap_or("d").to_string()));
            command = Some("run_mode".to_string());
        } else {
            return Err(Error {
                message: "No matches for inputs".to_string(),
                kind: clap::ErrorKind::EmptyValue,
                info: None,
            });
        }

        // println!("matches: {:#?}", self.matches);
        println!("options: {:?}", options);

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
    use std::fmt::Debug;
    use std::str::FromStr;

    fn unwrap_iter_with_option<T: FromStr + Clone>(args: Vec<Option<String>>) -> Vec<T>
    where
        <T as FromStr>::Err: Debug,
    {
        args.into_iter()
            .map(|x| x.unwrap().parse::<T>().unwrap().clone())
            .collect()
    }

    #[test]
    fn returns_an_error_when_no_command_args_are_passed() {
        let cli = Cli::new_from([""].iter()).unwrap();
        let result = cli.parse_commands(&cli.matches);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind, clap::ErrorKind::EmptyValue);
    }

    #[test]
    fn finds_a_match_without_options_from_command_args() {
        let cli = Cli::new_from(["exename", "init"].iter()).unwrap();
        let m = cli.matches.subcommand_matches("init").unwrap();
        let (command, _value) = m.subcommand();
        assert_eq!(command, "");
    }

    #[test]
    fn returns_a_default_option_for_init() {
        let cli: Cli = Cli::new_from(["exename", "init"].iter()).unwrap();
        let new_cli = cli.parse_commands(&cli.matches);
        let result = new_cli.unwrap();
        let values = unwrap_iter_with_option::<String>(result.options);
        assert_eq!(values, vec!["."]);
        assert_eq!(result.command.unwrap().clone(), "init");
    }

    #[test]
    fn returns_a_passed_value_for_init() {
        let cli: Cli = Cli::new_from(["exename", "init", "/usr/path"].iter()).unwrap();
        let new_cli = cli.parse_commands(&cli.matches);
        let result = new_cli.unwrap();
        let values = unwrap_iter_with_option::<String>(result.options);
        assert_eq!(values, vec!["/usr/path"]);
    }

    #[test]
    fn returns_a_default_option_for_make() {
        let date_time: DateTime<Local> = Local::now();
        let month = date_time.month();

        let cli: Cli = Cli::new_from(["exename", "make"].iter()).unwrap();
        let new_cli = cli.parse_commands(&cli.matches);
        let result = new_cli.unwrap();
        let values = unwrap_iter_with_option::<u32>(result.options);
        assert_eq!(values, vec![month]);
        assert_eq!(result.command.unwrap().clone(), "make");
    }

    #[test]
    fn returns_a_passed_value_for_make() {
        let cli: Cli = Cli::new_from(["exename", "make", "-m5"].iter()).unwrap();
        let new_cli = cli.parse_commands(&cli.matches);
        let result = new_cli.unwrap();
        let values = unwrap_iter_with_option::<u32>(result.options);
        assert_eq!(values, vec![5]);
    }

    #[test]
    fn returns_an_error_when_no_arg_is_present_for_edit() {
        let result = Cli::new_from(["exename", "edit"].iter());
        assert!(result.is_err());
    }

    #[test]
    fn returns_an_error_when_a_day_is_passed_without_an_hour() {
        let result = Cli::new_from(["exename", "edit", "-d8"].iter());
        assert!(result.is_err());
    }

    #[test]
    fn returns_an_error_when_a_month_is_passed_without_a_day() {
        let result = Cli::new_from(["exename", "edit", "-m8"].iter());
        assert!(result.is_err());
    }

    #[test]
    fn returns_an_error_when_a_year_is_passed_to_edit_without_a_month() {
        let result = Cli::new_from(["exename", "edit", "-y2020"].iter());
        assert!(result.is_err());
    }

    #[test]
    fn returns_a_passed_value_for_edit() {
        let cli: Cli =
            Cli::new_from(["exename", "edit", "-h5", "-d15", "-m12", "-y2021"].iter()).unwrap();
        let new_cli = cli.parse_commands(&cli.matches);
        let result = new_cli.unwrap();
        let values = unwrap_iter_with_option::<u32>(result.options);
        assert_eq!(values, vec![5, 15, 12, 2021]);
    }

    #[test]
    fn returns_an_error_when_a_year_is_passed_to_remove_without_a_month() {
        let result = Cli::new_from(["exename", "remove", "-y2020"].iter());
        assert!(result.is_err());
    }

    #[test]
    fn returns_an_error_when_an_hour_is_passed_to_remove() {
        let result = Cli::new_from(["exename", "remove", "-h5"].iter());
        assert!(result.is_err());
    }

    #[test]
    fn returns_a_passed_value_for_remove() {
        let cli: Cli =
            Cli::new_from(["exename", "remove", "-d21", "-m03", "-y2021"].iter()).unwrap();
        let new_cli = cli.parse_commands(&cli.matches);
        let result = new_cli.unwrap();
        let values = unwrap_iter_with_option::<u32>(result.options);
        assert_eq!(values, vec![21, 03, 2021]);
    }

    #[test]
    fn returns_a_default_value_for_run_mode() {
        let cli: Cli = Cli::new_from(["exename", "run-mode"].iter()).unwrap();
        let new_cli = cli.parse_commands(&cli.matches);
        let result = new_cli.unwrap();
        let values = unwrap_iter_with_option::<String>(result.options);
        assert_eq!(values, vec!["d"]);
    }

    #[test]
    fn throws_an_error_if_an_incorrect_arg_is_passed_in_run_mode() {
        let result: Result<Cli, Error> = Cli::new_from(["exename", "run-mode", "nn"].iter());
        assert!(result.is_err());
    }
}
