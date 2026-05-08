//! `chronos-daemon` — the daemon binary.
//!
//! Loads the DE440 ephemeris, opens the location source
//! (`geoclue2` or persisted manual), wires the event
//! scheduler to the subscription hub, and runs the UDS
//! server at `/run/chronos/<uid>.sock` until SIGTERM.

#[tokio::main(flavor = "multi_thread")]
async fn main() -> chronos::Result<()> {
    chronos::daemon::run().await
}
