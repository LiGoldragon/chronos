//! `chronos` — CLI client.
//!
//! Parses a single DOTOS record from argv, archives it as a
//! [`chronos::Request`], sends it length-prefixed over the
//! daemon's UDS at `/run/chronos/<uid>.sock`, reads a length-
//! prefixed [`chronos::Response`], and prints it as DOTOS.

fn main() -> chronos::Result<()> {
    let mut args = std::env::args().skip(1);
    let request_text = args.next().unwrap_or_else(|| {
        eprintln!(
            "usage: chronos 'GetTime' | 'GetSchedule' | 'GetLocation' | '(SetLocation 47.6 -122.3)' | 'UseGeoclue' | …"
        );
        std::process::exit(2);
    });

    let request = chronos::Request::from_dotos(&request_text)?;
    let response = chronos::client::send(&request)?;
    println!("{}", response.to_dotos()?);
    Ok(())
}
