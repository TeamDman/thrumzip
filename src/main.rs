use std::path::Path;
use std::path::PathBuf;

use holda::Holda;
#[derive(Holda)]
#[holda(NoDisplay)]
pub struct PathToZip {
    inner: PathBuf,
}
impl AsRef<Path> for PathToZip {
    fn as_ref(&self) -> &Path {
        self.inner.as_ref()
    }
}

#[derive(Holda)]
#[holda(NoDisplay)]
pub struct PathInsideZip {
    inner: PathBuf,
}
impl AsRef<Path> for PathInsideZip {
    fn as_ref(&self) -> &Path {
        self.inner.as_ref()
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    println!("All entries processed successfully");
    Ok(())
}
