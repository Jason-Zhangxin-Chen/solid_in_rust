// Write/Read
// std::io::Write / Read
// Abstract I/O traits. Write enables write(), write_all(), writeln!().
// Read enables read(), read_to_string(). Accept any sink/source: File, Vec, TcpStream, BufWriter, etc.
use std::io::{self, Write, Read};
fn write_read() -> io::Result<()> {

    // Write to any sink — file, socket, Vec:
    fn write_greeting(mut w: impl Write) -> io::Result<()> {
        writeln!(w, "Hello, world!")?;
        w.flush()?;
        Ok(())
    }
    write_greeting(io::stdout())?;          // ✅ stdout
    write_greeting(std::fs::File::create("out.txt")?)?; // ✅ file
    let mut buf: Vec<u8> = Vec::new();
    write_greeting(&mut buf)?;              // ✅ in-memory

    return Ok(());

    // Read from any source:
    fn count_bytes(mut r: impl Read) -> io::Result<usize> {
        let mut buf = Vec::new();
        r.read_to_end(&mut buf)?;
        Ok(buf.len())
    }
}