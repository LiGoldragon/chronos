//! The daemon entry point.
//!
//! Skeleton — the real implementation wires:
//!
//! - the redb store (one row: `LocationSource`);
//! - the [`Sky`](crate::sky::Sky) loader (DE440 via `anise`);
//! - the [`LocationTracker`] (zbus to `geoclue2` or persisted
//!   manual override);
//! - the `EventScheduler` (next-fire deadline per kind,
//!   `tokio::time::sleep_until`-driven; per
//!   `~/primary/skills/push-not-pull.md`);
//! - the `SubscriptionHub` (live subscriber set + cursors);
//! - the UDS server at `/run/chronos/<uid>.sock`.
//!
//! Each connection: read one [`Request`] frame; if it's a
//! one-shot verb, write one [`Response`] frame and close;
//! if it's `Subscribe`, register with the hub and stream
//! `Event` frames until the peer disconnects.

use tokio::net::{UnixListener, UnixStream};

use crate::error::Result;
use crate::request::Request;
use crate::response::Response;
use crate::wire::{read_frame, socket_path, write_frame};

/// Run the daemon until SIGTERM / Ctrl-C.
pub async fn run() -> Result<()> {
    let path = socket_path();
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    let listener = UnixListener::bind(&path)?;
    eprintln!("chronos-daemon listening on {}", path.display());

    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            biased;
            _ = &mut shutdown => {
                eprintln!("chronos-daemon shutting down");
                break;
            }
            accepted = listener.accept() => {
                let (stream, _addr) = accepted?;
                tokio::spawn(async move {
                    if let Err(error) = serve_connection(stream).await {
                        eprintln!("chronos-daemon connection error: {error}");
                    }
                });
            }
        }
    }

    let _ = std::fs::remove_file(&path);
    Ok(())
}

// TODO(chronos-impl): both `serve_connection` and
// `dispatch_one_shot` are free functions doing real work on
// typed values — provisional only. Per
// `~/primary/skills/abstractions.md` §"Verb belongs to noun",
// the implementation phase attaches them to typed nouns:
// `Daemon` (owns the listener + state stores), `Connection`
// (owns one stream + its protocol state), and `SubscriptionHub`
// (owns the live subscriber set).

async fn serve_connection(mut stream: UnixStream) -> Result<()> {
    let frame = read_frame(&mut stream).await?;
    let request = Request::from_archive(&frame)?;
    match request {
        Request::Subscribe { .. } => {
            // TODO(chronos-impl): register with SubscriptionHub
            // and stream Event frames until disconnect, per
            // ~/primary/skills/push-not-pull.md §"Subscription
            // contract" — emit current schedule first, then
            // deltas at deadline fires.
            let response = Response::Error { message: "Subscribe not yet implemented".into() };
            let archive = response.archive()?;
            write_frame(&mut stream, &archive).await?;
        }
        other => {
            let response = dispatch_one_shot(other);
            let archive = response.archive()?;
            write_frame(&mut stream, &archive).await?;
        }
    }
    Ok(())
}

fn dispatch_one_shot(request: Request) -> Response {
    // TODO(chronos-impl): route through the Observer + Sky
    // (DE440 + Location) for GetTime / GetSchedule, the
    // StateStore for GetLocation / SetLocation / UseGeoclue.
    // The skeleton answers every verb with `Error` so the
    // wire shape is exercised before the astronomy lands.
    Response::Error {
        message: format!("not yet implemented: {}", request.to_dotos().unwrap_or_else(|_| "<unrenderable>".into())),
    }
}
