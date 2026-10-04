use std::path::Path;

use chronos::daemon::Chronos;
use chronos::{Latitude, Location, Request, Response};
use nexus::Entering;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

#[tokio::test]
async fn set_location_then_get_location_uses_the_unix_socket() {
    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("chronos.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    let (_main, door) = Chronos::open(directory.path().to_path_buf()).unwrap();

    let server = async {
        for _ in 0..2 {
            let (stream, _) = listener.accept().await.unwrap();
            door.serve(stream).await.unwrap();
        }
    };
    let client = async {
        let location = Location {
            latitude: Latitude::try_new(19.4326).unwrap(),
            longitude: chronos::Longitude::try_new(-99.1332).unwrap(),
        };
        let set =
            request(&socket, Request::SetLocation { latitude: location.latitude, longitude: location.longitude }).await;
        assert_eq!(set, Response::Acked);
        let got = request(&socket, Request::GetLocation).await;
        assert_eq!(got, Response::Location { location, source: chronos::LocationSource::Manual });
    };
    tokio::join!(server, client);
}

async fn request(socket: &Path, request: Request) -> Response {
    let mut stream = UnixStream::connect(socket).await.unwrap();
    let frame = request.archive().unwrap();
    stream.write_all(&(frame.len() as u32).to_be_bytes()).await.unwrap();
    stream.write_all(&frame).await.unwrap();
    let mut length = [0; 4];
    stream.read_exact(&mut length).await.unwrap();
    let mut response = vec![0; u32::from_be_bytes(length) as usize];
    stream.read_exact(&mut response).await.unwrap();
    Response::from_archive(&response).unwrap()
}
