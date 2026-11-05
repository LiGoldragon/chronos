use chronos::wire::{MAX_FRAME_BYTES, write_frame, write_frame_blocking};

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
