use assert_cmd::Command;

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
#[ignore]
fn runs_init_for_repo_outside_of_current_directory() {
    let mut cmd = Command::cargo_bin("timesheet-gen").unwrap();
    let assert = cmd
        .env("TEST_MODE", "true")
        .arg("init")
        .arg("--path=./tests")
        .assert();

    assert
        .failure()
        .stdout("Initialising new repository.\nWould you like to add it to any of these existing clients?\n");
}

#[test]
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
fn runs_init_with_args() {
    let mut cmd = Command::cargo_bin("timesheet-gen").unwrap();
    let assert = cmd.env("TEST_MODE", "true").arg("init").assert();

    assert.success().stdout(
        "\u{1F916} timesheet-gen has already been initialised for this repository.\n\
        Try \'timesheet-gen make\' to create your first timesheet \n\
        or \'timesheet-gen help\' for more options.\n",
    );
}

#[test]
#[ignore]
fn runs_make_with_success() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("timesheet-gen")?;
    let assert = cmd.arg("make").assert();
    assert
        .failure()
        .stdout("Finding project data for \'apple\'...\nDoes \'timesheet-gen\' require a project/PO number?\n");
    Ok(())
}

#[test]
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
        .assert();

    assert
        .failure()
        .stdout("Remove \'timesheet-gen\' from client \'apple\'?\n");

    Ok(())
}
