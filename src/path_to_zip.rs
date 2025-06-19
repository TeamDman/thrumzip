use crate::size_of_thing::KnownCount;
use crate::size_of_thing::KnownSize;
use holda::Holda;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Holda)]
#[holda(NoDisplay)]
pub struct PathToZip {
    inner: Arc<PathBuf>,
}
impl AsRef<Path> for PathToZip {
    fn as_ref(&self) -> &Path {
        self.inner.as_ref()
    }
}
impl KnownSize for PathToZip {
    fn size_in_bytes(&self) -> usize {
        self.inner.size_in_bytes()
    }
}
impl KnownCount for PathToZip {
    fn count(&self) -> usize {
        1
    }
}
