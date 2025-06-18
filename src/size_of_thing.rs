use chrono::DateTime;
use chrono::Local;
use chrono::Utc;
use clap::builder::OsStr;
use positioned_io::RandomAccessFile;
use rc_zip::parse::Entry;
use std::path::PathBuf;
use std::sync::Arc;
use uom::si::f64::Information;
use uom::si::f64::InformationRate;
use uom::si::information::byte;
use uom::si::information_rate::byte_per_second;

/// Convenience trait for displaying the size of things.
pub trait KnownSize: Sized {
    /// Returns the size of the type in bytes.
    fn size_in_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
    fn size_of(&self) -> Information {
        Information::new::<byte>(self.size_in_bytes() as f64)
    }
    fn human_size(&self) -> String {
        humansize::format_size_i(self.size_in_bytes() as u64, humansize::DECIMAL)
    }
}

impl KnownSize for Information {
    fn size_in_bytes(&self) -> usize {
        self.get::<byte>() as usize
    }
    fn size_of(&self) -> Information {
        *self
    }
}

/// This is mainly for the human_size helper, treating the size of the rate as the size of one second of data.
impl KnownSize for InformationRate {
    fn size_in_bytes(&self) -> usize {
        self.get::<byte_per_second>() as usize
    }
}

impl<T> KnownSize for Vec<T>
where
    T: KnownSize,
{
    fn size_in_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.iter().map(|item| item.size_in_bytes()).sum::<usize>()
    }
}
impl<K, V> KnownSize for std::collections::HashMap<K, V>
where
    K: KnownSize,
    V: KnownSize,
{
    fn size_in_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.keys().map(|k| k.size_in_bytes()).sum::<usize>()
            + self.values().map(|v| v.size_in_bytes()).sum::<usize>()
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

impl<A, B, C> KnownSize for (A, B, C)
where
    A: KnownSize,
    B: KnownSize,
    C: KnownSize,
{
    fn size_in_bytes(&self) -> usize {
        self.0.size_in_bytes() + self.1.size_in_bytes() + self.2.size_in_bytes()
    }
}

impl<A, B, C, D> KnownSize for (A, B, C, D)
where
    A: KnownSize,
    B: KnownSize,
    C: KnownSize,
    D: KnownSize,
{
    fn size_in_bytes(&self) -> usize {
        self.0.size_in_bytes()
            + self.1.size_in_bytes()
            + self.2.size_in_bytes()
            + self.3.size_in_bytes()
    }
}

impl KnownSize for usize {
    fn size_in_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

pub trait KnownCount: Sized {
    /// Returns the length of the type.
    fn count(&self) -> usize;
}
impl<T: KnownCount> KnownCount for Vec<T> {
    fn count(&self) -> usize {
        self.iter().map(|item| item.count()).sum()
    }
}
impl<K: KnownCount, V: KnownCount> KnownCount for std::collections::HashMap<K, V> {
    fn count(&self) -> usize {
        self.values().map(|v| v.count()).sum()
    }
}
impl<A: KnownCount, B: KnownCount, C: KnownCount> KnownCount for (A, B, C) {
    fn count(&self) -> usize {
        self.0.count() + self.1.count() + self.2.count()
    }
}
