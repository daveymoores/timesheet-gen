use assert_cmd::Command;

//TODO - why does this need a timeout
const TIMEOUT_MILLISECONDS: u64 = 500;

#[test]
#[ignore]
fn runs_binary_with_a_command_that_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("timesheet-gen")?;
    cmd.arg("foo");
    cmd.assert().failure();

    Ok(())
}

#[test]
#[ignore]
fn runs_binary_with_a_command_that_does_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("timesheet-gen")?;
    cmd.arg("--help");
    cmd.assert().success();

    Ok(())
}

#[test]
#[ignore]
fn runs_init_for_repo_outside_of_current_directory() {
    let mut cmd = Command::cargo_bin("timesheet-gen").unwrap();
    let assert = cmd
        .env("TEST_MODE", "true")
        .arg("init")
        .arg("--path=./tests")
        .timeout(std::time::Duration::from_millis(TIMEOUT_MILLISECONDS))
        .assert();

    assert
        .failure()
        .stdout("Initialising new repository.\nWould you like to add it to any of these existing clients?\n");
}

#[test]
#[ignore]
fn runs_init_for_path_that_doesnt_exist() {
    let mut cmd = Command::cargo_bin("timesheet-gen").unwrap();
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
#[ignore]
fn runs_init_with_args() {
    let mut cmd = Command::cargo_bin("timesheet-gen").unwrap();
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
    let mut cmd = Command::cargo_bin("timesheet-gen")?;
    let assert = cmd
        .arg("make")
        .timeout(std::time::Duration::from_millis(TIMEOUT_MILLISECONDS))
        .assert();
    assert
        .failure()
        .stdout("Finding project data for \'apple\'...\nDoes \'timesheet-gen\' require a project/PO number?\n");
    Ok(())
}

#[test]
#[ignore]
fn runs_remove_with_failure() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("timesheet-gen")?;
    let assert = cmd
        .env("TEST_MODE", "true")
        .arg("remove")
        .arg("--client=doesn't exist")
        .assert();

    assert
        .failure()
        .stderr("The client, or client + namespace combination you passed has not be found.\n");

    Ok(())
}

#[test]
#[ignore]
fn runs_remove_and_prompts_to_remove_client() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("timesheet-gen")?;
    let assert = cmd
        .env("TEST_MODE", "true")
        .arg("remove")
        .arg("--client=apple")
        .timeout(std::time::Duration::from_millis(TIMEOUT_MILLISECONDS))
        .assert();

    assert.failure().stdout("Remove client \'apple\'?\n");

    Ok(())
}

#[test]
#[ignore]
fn runs_remove_and_prompts_to_remove_namespace() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("timesheet-gen")?;
    let assert = cmd
        .env("TEST_MODE", "true")
        .arg("remove")
        .arg("--client=apple")
        .arg("--namespace=timesheet-gen")
        .timeout(std::time::Duration::from_millis(TIMEOUT_MILLISECONDS))
        .assert();

    assert
        .failure()
        .stdout("Remove \'timesheet-gen\' from client \'apple\'?\n");

    Ok(())
}
