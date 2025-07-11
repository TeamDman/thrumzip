use crate::size_of_thing::KnownCount;
use crate::size_of_thing::KnownSize;
use holda::Holda;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Holda)]
#[holda(NoDisplay)]
pub struct PathInsideZip {
    inner: Arc<PathBuf>,
}
impl AsRef<Path> for PathInsideZip {
    fn as_ref(&self) -> &Path {
        self.inner.as_ref()
    }
}
impl KnownSize for PathInsideZip {
    fn size_in_bytes(&self) -> usize {
        self.inner.size_in_bytes()
    }
}
impl KnownCount for PathInsideZip {
    fn count(&self) -> usize {
        1
    }
}

#[cfg(test)]
mod test {
    use crate::path_inside_zip::PathInsideZip;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn it_works() -> eyre::Result<()> {
        let left = PathInsideZip::from(Arc::new(PathBuf::from("a/b/c.txt")));
        let right = PathInsideZip::from(Arc::new(PathBuf::from(r"a\b\c.txt")));
        assert_eq!(left, right);
        Ok(())
    }
}
