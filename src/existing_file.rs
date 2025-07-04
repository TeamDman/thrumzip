use crate::path_inside_zip::PathInsideZip;
use crate::size_of_thing::KnownCount;
use crate::size_of_thing::KnownSize;
use std::path::PathBuf;
use uom::si::f64::Information;
use uom::si::information::byte;

#[derive(Debug)]
pub enum ExistingFile {
    Unambiguous {
        path_inside_zip: PathInsideZip,
        path_on_disk: PathBuf,
        size: Information,
    },
    Ambiguous {
        path_inside_zip: PathInsideZip,
        zip_name: String,
        path_on_disk: PathBuf,
        size: Information,
    },
}
impl KnownSize for ExistingFile {
    fn size_in_bytes(&self) -> usize {
        match self {
            ExistingFile::Unambiguous { size, .. } => size.get::<byte>() as usize,
            ExistingFile::Ambiguous { size, .. } => size.get::<byte>() as usize,
        }
    }
    fn size_of(&self) -> Information {
        match self {
            ExistingFile::Unambiguous { size, .. } => *size,
            ExistingFile::Ambiguous { size, .. } => *size,
        }
    }
}
impl KnownCount for ExistingFile {
    fn count(&self) -> usize {
        1
    }
}

impl ExistingFile {
    pub fn path_inside_zip(&self) -> &PathInsideZip {
        match self {
            ExistingFile::Unambiguous {
                path_inside_zip, ..
            } => path_inside_zip,
            ExistingFile::Ambiguous {
                path_inside_zip, ..
            } => path_inside_zip,
        }
    }

    pub fn path_on_disk(&self) -> Option<&PathBuf> {
        match self {
            ExistingFile::Unambiguous { path_on_disk, .. } => Some(path_on_disk),
            ExistingFile::Ambiguous { .. } => None,
        }
    }

    pub fn is_ambiguous(&self) -> bool {
        matches!(self, ExistingFile::Ambiguous { .. })
    }
}
