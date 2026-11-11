//! Streaming I/O adapters for `std::io::Write` and `std::io::Read`.
//!
//! Provides zero-allocation, buffer-conscious Base64 encoding and decoding streams.

use crate::alphabet::Config;
use crate::decode::decode_to_slice;
use crate::encode::encode_to_slice;
#[cfg(feature = "std")]
use std::io::{self, Read, Write};

#[cfg(feature = "tokio")]
use core::pin::Pin;
#[cfg(feature = "tokio")]
use core::task::{Context, Poll};
#[cfg(feature = "tokio")]
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// An encoder stream that wraps a writer and encodes incoming bytes as Base64 on the fly.
#[cfg(feature = "std")]
pub struct EncoderWriter<W: Write> {
    writer: W,
    config: &'static Config,
    buf: [u8; 3],
    buf_len: usize,
}

#[cfg(feature = "std")]
impl<W: Write> EncoderWriter<W> {
    /// Creates a new `EncoderWriter` wrapping `writer` with the standard Base64 configuration.
    pub fn new(writer: W) -> Self {
        Self::with_config(writer, &Config::STANDARD)
    }

    /// Creates a new `EncoderWriter` with custom configuration.
    pub fn with_config(writer: W, config: &'static Config) -> Self {
        Self {
            writer,
            config,
            buf: [0u8; 3],
            buf_len: 0,
        }
    }

    /// Flushes any pending data, writes final padding, and returns the underlying writer.
    pub fn finish(mut self) -> io::Result<W> {
        self.flush_tail()?;
        Ok(self.writer)
    }

    fn flush_tail(&mut self) -> io::Result<()> {
        if self.buf_len > 0 {
            let mut out = [0u8; 4];
            let written = encode_to_slice(&self.buf[..self.buf_len], &mut out, self.config);
            self.buf_len = 0;
            self.writer.write_all(&out[..written])?;
        }
        self.writer.flush()
    }
}

#[cfg(feature = "std")]
impl<W: Write> Write for EncoderWriter<W> {
    fn write(&mut self, mut input: &[u8]) -> io::Result<usize> {
        let original_len = input.len();

        // 1. Fill partial buffer if any
        if self.buf_len > 0 {
            let needed = 3 - self.buf_len;
            let take = needed.min(input.len());
            self.buf[self.buf_len..self.buf_len + take].copy_from_slice(&input[..take]);
            self.buf_len += take;
            input = &input[take..];

            if self.buf_len == 3 {
                let mut out = [0u8; 4];
                encode_to_slice(&self.buf, &mut out, self.config);
                self.writer.write_all(&out)?;
                self.buf_len = 0;
            }
        }

        // 2. Process large chunks directly
        let mut enc_buf = [0u8; 1024];
        while input.len() >= 3 {
            let chunk_in_len = (input.len() / 3 * 3).min(768);
            let written = encode_to_slice(&input[..chunk_in_len], &mut enc_buf, self.config);
            self.writer.write_all(&enc_buf[..written])?;
            input = &input[chunk_in_len..];
        }

        // 3. Save remainder (1 or 2 bytes)
        if !input.is_empty() {
            self.buf[..input.len()].copy_from_slice(input);
            self.buf_len = input.len();
        }

        Ok(original_len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

/// A decoder stream that wraps a reader and decodes Base64 data on the fly.
#[cfg(feature = "std")]
pub struct DecoderReader<R: Read> {
    reader: R,
    config: &'static Config,
    buf: [u8; 4],
    buf_len: usize,
    decoded_buf: [u8; 3],
    decoded_len: usize,
    decoded_pos: usize,
    eof: bool,
}

#[cfg(feature = "std")]
impl<R: Read> DecoderReader<R> {
    /// Creates a new `DecoderReader` with standard Base64 configuration.
    pub fn new(reader: R) -> Self {
        Self::with_config(reader, &Config::STANDARD)
    }

    /// Creates a new `DecoderReader` with custom configuration.
    pub fn with_config(reader: R, config: &'static Config) -> Self {
        Self {
            reader,
            config,
            buf: [0u8; 4],
            buf_len: 0,
            decoded_buf: [0u8; 3],
            decoded_len: 0,
            decoded_pos: 0,
            eof: false,
        }
    }
}

#[cfg(feature = "std")]
impl<R: Read> Read for DecoderReader<R> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if out.is_empty() {
            return Ok(0);
        }

        // Deliver leftover decoded bytes if any
        if self.decoded_pos < self.decoded_len {
            let available = self.decoded_len - self.decoded_pos;
            let take = available.min(out.len());
            out[..take]
                .copy_from_slice(&self.decoded_buf[self.decoded_pos..self.decoded_pos + take]);
            self.decoded_pos += take;
            return Ok(take);
        }

        if self.eof {
            return Ok(0);
        }

        // Read 4 Base64 bytes from inner reader
        while self.buf_len < 4 {
            let n = self.reader.read(&mut self.buf[self.buf_len..])?;
            if n == 0 {
                self.eof = true;
                break;
            }
            self.buf_len += n;
        }

        if self.buf_len == 0 {
            return Ok(0);
        }

        let written = decode_to_slice(
            &self.buf[..self.buf_len],
            &mut self.decoded_buf,
            self.config,
        )
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        self.buf_len = 0;
        self.decoded_len = written;
        self.decoded_pos = 0;

        let take = written.min(out.len());
        out[..take].copy_from_slice(&self.decoded_buf[..take]);
        self.decoded_pos = take;

        Ok(take)
    }
}

/// An asynchronous Base64 encoder wrapping any [`tokio::io::AsyncWrite`].
#[cfg(feature = "tokio")]
pub struct AsyncEncoderWriter<W> {
    writer: W,
    config: &'static Config,
    in_buf: [u8; 3],
    in_len: usize,
    out_buf: [u8; 4],
    out_len: usize,
    out_pos: usize,
}

#[cfg(feature = "tokio")]
impl<W: AsyncWrite + Unpin> AsyncEncoderWriter<W> {
    /// Creates a new `AsyncEncoderWriter` wrapping `writer` with the standard configuration.
    pub fn new(writer: W) -> Self {
        Self::with_config(writer, &Config::STANDARD)
    }

    /// Creates a new `AsyncEncoderWriter` with custom configuration.
    pub fn with_config(writer: W, config: &'static Config) -> Self {
        Self {
            writer,
            config,
            in_buf: [0u8; 3],
            in_len: 0,
            out_buf: [0u8; 4],
            out_len: 0,
            out_pos: 0,
        }
    }

    /// Unwraps the inner writer.
    pub fn into_inner(self) -> W {
        self.writer
    }

    fn poll_flush_out(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        while self.out_pos < self.out_len {
            let n = match Pin::new(&mut self.writer)
                .poll_write(cx, &self.out_buf[self.out_pos..self.out_len])
            {
                Poll::Ready(Ok(n)) => n,
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            };
            if n == 0 {
                return Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "underlying writer wrote 0 bytes",
                )));
            }
            self.out_pos += n;
        }
        self.out_len = 0;
        self.out_pos = 0;
        Poll::Ready(Ok(()))
    }
}

#[cfg(feature = "tokio")]
impl<W: AsyncWrite + Unpin> AsyncWrite for AsyncEncoderWriter<W> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        match this.poll_flush_out(cx) {
            Poll::Ready(Ok(())) => {}
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            Poll::Pending => return Poll::Pending,
        }

        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }

        // Fill partial buffer if any
        if this.in_len > 0 {
            let needed = 3 - this.in_len;
            let take = needed.min(buf.len());
            this.in_buf[this.in_len..this.in_len + take].copy_from_slice(&buf[..take]);
            this.in_len += take;

            if this.in_len == 3 {
                let written = encode_to_slice(&this.in_buf, &mut this.out_buf, this.config);
                this.in_len = 0;
                this.out_len = written;
                this.out_pos = 0;
                let _ = this.poll_flush_out(cx);
            }
            return Poll::Ready(Ok(take));
        }

        // Process a 3-byte chunk
        if buf.len() >= 3 {
            let written = encode_to_slice(&buf[..3], &mut this.out_buf, this.config);
            this.out_len = written;
            this.out_pos = 0;
            let _ = this.poll_flush_out(cx);
            return Poll::Ready(Ok(3));
        }

        // Buffer leftover bytes < 3
        this.in_buf[..buf.len()].copy_from_slice(buf);
        this.in_len = buf.len();
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        match this.poll_flush_out(cx) {
            Poll::Ready(Ok(())) => {}
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            Poll::Pending => return Poll::Pending,
        }
        Pin::new(&mut this.writer).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if this.in_len > 0 {
            let written =
                encode_to_slice(&this.in_buf[..this.in_len], &mut this.out_buf, this.config);
            this.in_len = 0;
            this.out_len = written;
            this.out_pos = 0;
        }

        match this.poll_flush_out(cx) {
            Poll::Ready(Ok(())) => {}
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            Poll::Pending => return Poll::Pending,
        }

        Pin::new(&mut this.writer).poll_shutdown(cx)
    }
}

/// An asynchronous Base64 decoder wrapping any [`tokio::io::AsyncRead`].
#[cfg(feature = "tokio")]
pub struct AsyncDecoderReader<R> {
    reader: R,
    config: &'static Config,
    buf: [u8; 4],
    buf_len: usize,
    decoded_buf: [u8; 3],
    decoded_len: usize,
    decoded_pos: usize,
    eof: bool,
}

#[cfg(feature = "tokio")]
impl<R: AsyncRead + Unpin> AsyncDecoderReader<R> {
    /// Creates a new `AsyncDecoderReader` wrapping `reader` with standard Base64 configuration.
    pub fn new(reader: R) -> Self {
        Self::with_config(reader, &Config::STANDARD)
    }

    /// Creates a new `AsyncDecoderReader` with custom configuration.
    pub fn with_config(reader: R, config: &'static Config) -> Self {
        Self {
            reader,
            config,
            buf: [0u8; 4],
            buf_len: 0,
            decoded_buf: [0u8; 3],
            decoded_len: 0,
            decoded_pos: 0,
            eof: false,
        }
    }

    /// Unwraps the inner reader.
    pub fn into_inner(self) -> R {
        self.reader
    }
}

#[cfg(feature = "tokio")]
impl<R: AsyncRead + Unpin> AsyncRead for AsyncDecoderReader<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();

        // Deliver leftover decoded bytes if any
        if this.decoded_pos < this.decoded_len {
            let available = this.decoded_len - this.decoded_pos;
            let take = available.min(buf.remaining());
            buf.put_slice(&this.decoded_buf[this.decoded_pos..this.decoded_pos + take]);
            this.decoded_pos += take;
            return Poll::Ready(Ok(()));
        }

        if this.eof {
            return Poll::Ready(Ok(()));
        }

        // Read up to 4 Base64 bytes from inner reader
        while this.buf_len < 4 {
            let mut read_slice = [0u8; 4];
            let needed = 4 - this.buf_len;
            let mut read_buf = ReadBuf::new(&mut read_slice[..needed]);
            match Pin::new(&mut this.reader).poll_read(cx, &mut read_buf) {
                Poll::Ready(Ok(())) => {
                    let n = read_buf.filled().len();
                    if n == 0 {
                        this.eof = true;
                        break;
                    }
                    this.buf[this.buf_len..this.buf_len + n].copy_from_slice(read_buf.filled());
                    this.buf_len += n;
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }

        if this.buf_len == 0 {
            return Poll::Ready(Ok(()));
        }

        let written = decode_to_slice(
            &this.buf[..this.buf_len],
            &mut this.decoded_buf,
            this.config,
        )
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        this.buf_len = 0;
        this.decoded_len = written;
        this.decoded_pos = 0;

        let take = written.min(buf.remaining());
        buf.put_slice(&this.decoded_buf[..take]);
        this.decoded_pos = take;

        Poll::Ready(Ok(()))
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use std::vec::Vec;

    #[test]
    fn test_encoder_writer_stream() {
        let mut dest = Vec::new();
        {
            let mut enc = EncoderWriter::new(&mut dest);
            enc.write_all(b"Hello ").unwrap();
            enc.write_all(b"World").unwrap();
            enc.finish().unwrap();
        }
        assert_eq!(core::str::from_utf8(&dest).unwrap(), "SGVsbG8gV29ybGQ=");
    }

    #[test]
    fn test_decoder_reader_stream() {
        let input = b"SGVsbG8gV29ybGQ=";
        let mut dec = DecoderReader::new(&input[..]);
        let mut out = Vec::new();
        dec.read_to_end(&mut out).unwrap();
        assert_eq!(&out, b"Hello World");
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_async_encoder_and_decoder_stream() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let mut dest = Vec::new();
        {
            let mut enc = AsyncEncoderWriter::new(&mut dest);
            enc.write_all(b"Tokio ").await.unwrap();
            enc.write_all(b"Async ").await.unwrap();
            enc.write_all(b"Stream").await.unwrap();
            enc.shutdown().await.unwrap();
        }

        let encoded_str = core::str::from_utf8(&dest).unwrap();
        assert_eq!(encoded_str, "VG9raW8gQXN5bmMgU3RyZWFt");

        let mut dec = AsyncDecoderReader::new(&dest[..]);
        let mut decoded = Vec::new();
        dec.read_to_end(&mut decoded).await.unwrap();
        assert_eq!(&decoded, b"Tokio Async Stream");
    }
}
