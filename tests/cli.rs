use std::process::Command;

#[test]
fn cli_refuses_more_than_one_request_datom() {
    let output =
        Command::new(env!("CARGO_BIN_EXE_chronos")).args(["GetTime", "GetLocation"]).output().expect("run Chronos CLI");

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(String::from_utf8(output.stderr).expect("UTF-8 diagnostic"), "chronos accepts one Request datom\n");
}
