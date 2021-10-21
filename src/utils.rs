use std::convert::Infallible;
use std::process::Output;

pub fn trim_output_from_utf8(output: Output) -> Result<String, Box<dyn std::error::Error>> {
    let x = String::from_utf8(output.stdout)?.trim().parse().unwrap();
    Ok(x)
}

#[cfg(test)]
mod tests {
    use crate::utils::trim_output_from_utf8;
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
}
