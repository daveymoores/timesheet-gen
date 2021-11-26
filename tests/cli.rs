use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn runs_binary_with_a_command_that_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("timesheet-gen")?;
    cmd.arg("foo");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn runs_binary_with_a_command_that_does_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("timesheet-gen")?;
    cmd.arg("--help");
    cmd.assert().success();

    Ok(())
}

#[test]
fn runs_init_with_success() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("timesheet-gen")?;
    cmd.arg("init");
    cmd.assert().success();

    Ok(())
}

// #[test]
// fn runs_init_with_args() {
//     let mut cmd = Command::cargo_bin("timesheet-gen").unwrap();
//     let output = cmd.env("TEST_MODE", "true").arg("init").output().unwrap();
//     println!("{:?}", String::from_utf8_lossy(&output.stderr));
//     assert_eq!(String::from_utf8_lossy(&output.stderr), "");
// }

#[test]
#[ignore]
fn runs_make_with_success() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("timesheet-gen")?;
    cmd.arg("make");
    cmd.assert().success();

    Ok(())
}

#[test]
#[ignore]
fn runs_edit_with_success() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("timesheet-gen")?;
    cmd.arg("make");
    cmd.assert().success();

    Ok(())
}

#[test]
fn runs_remove_with_success() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("timesheet-gen")?;
    cmd.arg("remove");
    cmd.assert().success();

    Ok(())
}
