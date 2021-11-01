use random_string::generate;
use std::process::Output;

pub fn trim_output_from_utf8(output: Output) -> Result<String, Box<dyn std::error::Error>> {
    let x = String::from_utf8(output.stdout)?.trim().parse().unwrap();
    Ok(x)
}

pub fn generate_random_path() -> String {
    let charset = "0123456789abcdefghijklmnopqrstuvwxyz";
    generate(10, charset)
}

#[cfg(test)]
mod tests {
    use crate::utils::{generate_random_path, trim_output_from_utf8};
    use std::os::unix::process::ExitStatusExt;
    use std::process::{ExitStatus, Output};

    #[test]
    fn it_trims_output_from_utf8() {
        let output_path = Output {
            status: ExitStatus::from_raw(0),
            stdout: vec![68, 97, 118, 101, 121, 32, 77, 111, 111, 114, 101, 115, 10],
            stderr: vec![],
        };

        assert_eq!(trim_output_from_utf8(output_path).unwrap(), "Davey Moores");
    }

    #[test]
    fn it_generates_a_random_string() {
        let random_string = generate_random_path();
        let regex = regex::Regex::new(r"^[a-z0-9]{10}$");
        match regex.unwrap().find(&*random_string) {
            Some(_x) => assert!(true),
            None => panic!("Pattern not matched"),
        }
    }
}
