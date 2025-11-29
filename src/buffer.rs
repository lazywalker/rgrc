//! # buffer.rs - Buffered writers for rgrc
//!
//! This module provides specialized buffered writers for handling output
//! with different buffering strategies.

/// Line-buffered writer that flushes after each newline
/// This ensures real-time output for commands like ping
pub struct LineBufferedWriter<W: std::io::Write> {
    inner: W,
}

impl<W: std::io::Write> LineBufferedWriter<W> {
    /// Create a new `LineBufferedWriter` wrapping `inner`.
    ///
    /// The returned writer will delegate write and flush calls to `inner`,
    /// but will also flush `inner` whenever a newline (`\n`) byte is written
    /// to ensure near-real-time line output for interactive commands.
    pub fn new(inner: W) -> Self {
        Self { inner }
    }
}

impl<W: std::io::Write> std::io::Write for LineBufferedWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let written = self.inner.write(buf)?;
        // Flush after each newline to ensure real-time output
        if buf.contains(&b'\n') {
            self.inner.flush()?;
        }
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // Flush the underlying writer.
        //
        // This forwards to the wrapped writer's `flush` implementation.
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};

    #[test]
    fn test_line_buffered_writer() {
        // Test basic functionality with Cursor<Vec<u8>> as underlying writer
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut writer = LineBufferedWriter::new(cursor);

        // Test writing data without newlines - should write but not flush
        writer.write_all(b"hello").unwrap();
        // Data should be written to buffer immediately
        let data = writer.inner.get_ref();
        assert_eq!(
            data, b"hello",
            "Buffer should contain written data immediately"
        );

        // Test writing data with newline - should write and flush
        writer.write_all(b" world\n").unwrap();
        let data = writer.inner.get_ref();
        assert_eq!(
            data, b"hello world\n",
            "Buffer should contain all written data"
        );

        // Test writing more data without newline
        writer.write_all(b"more data").unwrap();
        let data = writer.inner.get_ref();
        assert_eq!(
            data, b"hello world\nmore data",
            "Buffer should contain all written data"
        );

        // Test explicit flush (should be no-op since data is already written)
        writer.flush().unwrap();
        let data = writer.inner.get_ref();
        assert_eq!(
            data, b"hello world\nmore data",
            "Buffer should remain unchanged after flush"
        );
    }

    #[test]
    fn test_line_buffered_writer_empty_writes() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut writer = LineBufferedWriter::new(cursor);

        // Test empty write
        writer.write_all(b"").unwrap();
        let data = writer.inner.get_ref();
        assert!(data.is_empty(), "Empty write should not affect buffer");

        // Test write with only newline
        writer.write_all(b"\n").unwrap();
        let data = writer.inner.get_ref();
        assert_eq!(
            data, b"\n",
            "Write with only newline should flush immediately"
        );

        // Test multiple empty writes
        writer.write_all(b"").unwrap();
        writer.write_all(b"").unwrap();
        let data = writer.inner.get_ref();
        assert_eq!(
            data, b"\n",
            "Multiple empty writes should not affect buffer"
        );
    }

    #[test]
    fn test_line_buffered_writer_partial_writes() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut writer = LineBufferedWriter::new(cursor);

        // Test partial writes that together form a line
        let result1 = writer.write(b"hello ").unwrap();
        assert_eq!(result1, 6);
        let data = writer.inner.get_ref();
        assert_eq!(
            data, b"hello ",
            "Partial write should be written immediately"
        );

        let result2 = writer.write(b"world\n").unwrap();
        assert_eq!(result2, 6);
        let data = writer.inner.get_ref();
        assert_eq!(
            data, b"hello world\n",
            "Write with newline should be written immediately"
        );

        // Test write method with data containing newlines
        let result3 = writer.write(b"test\nmore").unwrap();
        assert_eq!(result3, 9);
        let data = writer.inner.get_ref();
        assert_eq!(
            data, b"hello world\ntest\nmore",
            "Write with newline should write all data immediately"
        );

        writer.flush().unwrap();
        let data = writer.inner.get_ref();
        assert_eq!(
            data, b"hello world\ntest\nmore",
            "Final flush should ensure all data is written"
        );
    }

    #[test]
    fn test_line_buffered_writer_error_handling() {
        use std::io::Error;

        // Create a writer that always fails
        struct FailingWriter;
        impl std::io::Write for FailingWriter {
            fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
                Err(Error::other("Simulated write error"))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Err(Error::other("Simulated flush error"))
            }
        }

        let failing_writer = FailingWriter;
        let mut writer = LineBufferedWriter::new(failing_writer);

        // Test that write errors are propagated
        let result = writer.write(b"test");
        assert!(result.is_err(), "Write error should be propagated");

        // Test that flush errors are propagated
        let result = writer.flush();
        assert!(result.is_err(), "Flush error should be propagated");
    }
}
