//! Tokio async streaming example demonstrating zero-allocation Base64 processing:
//! 1. Encoding data in-flight with `AsyncEncoderWriter`
//! 2. Decoding a stream on-the-fly with `AsyncDecoderReader`
//!
//! Run with: `cargo run --example tokio_streaming --features tokio`

use hyper64::{AsyncDecoderReader, AsyncEncoderWriter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ hyper64: Tokio Async Streaming I/O Demo\n");

    let raw_data = b"Streaming high-throughput payload via hyper64 + Tokio async runtime. Zero allocation bottleneck!";

    // =========================================================================
    // 1. ASYNC STREAM ENCODING
    // =========================================================================
    println!("1. Encoding stream on-the-fly into an async writer...");
    let mut encoded_sink = Vec::new();

    {
        let mut encoder = AsyncEncoderWriter::new(&mut encoded_sink);
        // Stream data in small chunks simulating network packets or file chunks
        for chunk in raw_data.chunks(16) {
            encoder.write_all(chunk).await?;
        }
        encoder.flush().await?;
        encoder.shutdown().await?;
    }

    let encoded_str = String::from_utf8(encoded_sink.clone())?;
    println!("   Encoded stream ({} bytes):", encoded_sink.len());
    println!("   {}\n", encoded_str);

    // =========================================================================
    // 2. ASYNC STREAM DECODING
    // =========================================================================
    println!("2. Decoding stream on-the-fly with AsyncDecoderReader...");
    let cursor = std::io::Cursor::new(encoded_sink);
    let mut decoder = AsyncDecoderReader::new(cursor);

    let mut decoded_output = Vec::new();
    let mut chunk_buf = [0u8; 32];

    loop {
        let n = decoder.read(&mut chunk_buf).await?;
        if n == 0 {
            break; // EOF reached
        }
        decoded_output.extend_from_slice(&chunk_buf[..n]);
    }

    assert_eq!(&decoded_output[..], &raw_data[..]);
    println!("   Successfully decoded {} original bytes streamingly:", decoded_output.len());
    println!("   \"{}\"\n", String::from_utf8_lossy(&decoded_output));

    println!("✅ All streaming assertions verified without memory buffering bottlenecks!");

    Ok(())
}
