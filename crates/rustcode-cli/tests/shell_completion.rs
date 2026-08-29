use assert_cmd::Command;

#[test]
fn completion_exits_before_startup_side_effects() {
    let rustcode_home = tempfile::tempdir().unwrap();

    Command::cargo_bin("rustcode")
        .unwrap()
        .env("RUSTCODE_HOME", rustcode_home.path())
        .arg("completion")
        .arg("bash")
        .assert()
        .success()
        .stderr("")
        .stdout(predicates::str::contains("rustcode"));

    assert!(
        rustcode_home.path().read_dir().unwrap().next().is_none(),
        "completion generation must not create logs, config, or telemetry state"
    );
}
