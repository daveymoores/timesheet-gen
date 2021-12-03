use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions

#[test]
fn runs_binary_with_a_command_that_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("timesheet-gen")?;
    cmd.arg("foo");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn runs_binary_with_a_command_that_does_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("timesheet-gen")?;
    cmd.arg("--help");
    cmd.assert().success();

    Ok(())
}

// #[test]
// fn runs_init_for_repo_outside_of_current_directory() {
//     let mut cmd = assert_cmd::Command::cargo_bin("timesheet-gen").unwrap();
//     let assert = cmd
//         .env("TEST_MODE", "true")
//         .arg("init")
//         .arg("--path=./tests")
//         .assert();
//
//     assert
//         .failure()
//         .stdout("No repositories found at path. Please check that the path is valid.\n");
// }

#[test]
fn runs_init_for_path_that_doesnt_exist() {
    let mut cmd = assert_cmd::Command::cargo_bin("timesheet-gen").unwrap();
    let assert = cmd
        .env("TEST_MODE", "true")
        .arg("init")
        .arg("--path=/not/a/path")
        .assert();

    assert
        .failure()
        .stdout("No repositories found at path. Please check that the path is valid.\n");
}

#[test]
fn runs_init_with_args() {
    let mut cmd = assert_cmd::Command::cargo_bin("timesheet-gen").unwrap();
    let assert = cmd.env("TEST_MODE", "true").arg("init").assert();

    assert.success().stdout(
        "timesheet-gen has already been initialised for this repository! \
        \nTry \'timesheet-gen make\' to create your first timesheet \
        \nor \'timesheet-gen help\' for more options.\n",
    );
}

#[test]
#[ignore]
fn runs_make_with_success() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("timesheet-gen")?;
    cmd.arg("make");
    cmd.assert().success();

    Ok(())
}

#[test]
#[ignore]
fn runs_edit_with_success() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("timesheet-gen")?;
    cmd.arg("make");
    cmd.assert().success();

    Ok(())
}

#[test]
fn runs_remove_with_success() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("timesheet-gen")?;
    cmd.arg("remove");
    cmd.assert().success();

    Ok(())
}
