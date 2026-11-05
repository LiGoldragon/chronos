use chronos::wire::{MAX_FRAME_BYTES, write_frame, write_frame_blocking};

#[cfg(unix)]
#[test]
fn socket_configuration_accepts_unix_paths_that_are_not_utf8() {
    use std::os::unix::ffi::OsStringExt;

    let configured = std::ffi::OsString::from_vec(b"/tmp/chronos-\xff.sock".to_vec());
    // This test binary has no other test that reads CHRONOS_SOCKET.
    unsafe { std::env::set_var("CHRONOS_SOCKET", &configured) };

    let path = chronos::wire::socket_path();

    unsafe { std::env::remove_var("CHRONOS_SOCKET") };
    assert_eq!(path, std::path::PathBuf::from(configured));
}

#[test]
fn blocking_writer_refuses_a_frame_its_reader_would_refuse() {
    let mut wire = Vec::new();
    let oversized = vec![0; MAX_FRAME_BYTES + 1];

    let error = write_frame_blocking(&mut wire, &oversized).expect_err("oversized frame must fail");

    assert!(error.to_string().contains("frame too large"));
    assert!(wire.is_empty());
}

#[tokio::test]
async fn async_writer_refuses_a_frame_its_reader_would_refuse() {
    let (mut writer, mut reader) = tokio::io::duplex(16);
    let oversized = vec![0; MAX_FRAME_BYTES + 1];

    let error = write_frame(&mut writer, &oversized).await.expect_err("oversized frame must fail");

    assert!(error.to_string().contains("frame too large"));
    drop(writer);
    let mut received = Vec::new();
    tokio::io::AsyncReadExt::read_to_end(&mut reader, &mut received).await.expect("read closed wire");
    assert!(received.is_empty());
}
