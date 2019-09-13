use std::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};

pub use self::inner::*;

// Last-modified time of a file, in milliseconds since the UNIX epoch.
pub fn file_last_modified(file: &File) -> u128 {
    file.metadata()
        .and_then(|md| {
            md.modified()
                .map(|t| t.duration_since(UNIX_EPOCH).unwrap().as_millis())
        })
        .unwrap_or(0)
}

// Current system time, in milliseconds since the UNIX epoch.
pub fn current_time() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

#[cfg(unix)]
mod inner {
    pub use super::*;

    use std::io;
    use std::os::unix::ffi::OsStrExt;
    use std::path::{Path, PathBuf};

    pub fn write_path<W>(w: &mut W, path: &Path) -> io::Result<()>
    where
        W: io::Write,
    {
        w.write_all(path.as_os_str().as_bytes())
    }

    pub fn read_path<R>(r: &mut R) -> io::Result<PathBuf>
    where
        R: io::Read,
    {
        use std::ffi::OsStr;
        let mut buf = vec![];
        r.read_to_end(&mut buf)?;
        Ok(OsStr::from_bytes(&buf).into())
    }

    /**
    Returns `true` if `cargo-eval` should force Cargo to use coloured output.

    This depends on whether `cargo-eval`'s STDERR is connected to a TTY or not.
    */
    pub fn force_cargo_color() -> bool {
        atty::is(atty::Stream::Stderr)
    }
}

#[cfg(windows)]
pub mod inner {
    #![allow(non_snake_case)]

    pub use super::*;

    use std::ffi::OsString;
    use std::io;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use std::path::{Path, PathBuf};

    pub fn write_path<W>(w: &mut W, path: &Path) -> io::Result<()>
    where
        W: io::Write,
    {
        for word in path.as_os_str().encode_wide() {
            let lo = (word & 0xff) as u8;
            let hi = (word >> 8) as u8;
            w.write_all(&[lo, hi])?;
        }
        Ok(())
    }

    pub fn read_path<R>(r: &mut R) -> io::Result<PathBuf>
    where
        R: io::Read,
    {
        let mut buf = vec![];
        r.read_to_end(&mut buf)?;

        let mut words = Vec::with_capacity(buf.len() / 2);
        let mut it = buf.iter().cloned();
        while let Some(lo) = it.next() {
            let hi = it.next().unwrap();
            words.push(u16::from(lo) | (u16::from(hi) << 8));
        }

        Ok(OsString::from_wide(&words).into())
    }

    /**
    Returns `true` if `cargo-eval` should force Cargo to use coloured output.

    Always returns `false` on Windows because colour is communicated over a side-channel.
    */
    pub fn force_cargo_color() -> bool {
        false
    }
}
