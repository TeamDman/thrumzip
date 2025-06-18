use chrono::DateTime;
use chrono::Local;
use chrono::Utc;
use clap::builder::OsStr;
use positioned_io::RandomAccessFile;
use rc_zip::parse::Entry;
use std::path::PathBuf;
use std::sync::Arc;
use uom::si::f64::Information;
use uom::si::information::byte;

pub trait KnownSize: Sized {
    /// Returns the size of the type in bytes.
    fn size_in_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
    fn size_of(&self) -> Information {
        Information::new::<byte>(self.size_in_bytes() as f64)
    }
}

impl KnownSize for DateTime<Utc> {}
impl KnownSize for DateTime<Local> {}
impl KnownSize for Arc<RandomAccessFile> {}
impl KnownSize for OsStr {
    fn size_in_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.len() as usize
    }
}
impl KnownSize for PathBuf {
    fn size_in_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.as_os_str().len() as usize
    }
}
impl KnownSize for &str {
    fn size_in_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.len() as usize
    }
}
impl KnownSize for Entry {
    fn size_in_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.uncompressed_size as usize
            + self.name.len()
            + self.comment.len()
    }
}
impl<T> KnownSize for &T
where
    T: KnownSize,
{
    fn size_in_bytes(&self) -> usize {
        (*self).size_in_bytes()
    }
}

impl<L, R> KnownSize for (L, R)
where
    L: KnownSize,
    R: KnownSize,
{
    fn size_in_bytes(&self) -> usize {
        self.0.size_in_bytes() + self.1.size_in_bytes()
    }
}
