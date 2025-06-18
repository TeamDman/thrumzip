use crate::PathInsideZip;
use crate::PathToZip;
use eyre::OptionExt;
use std::path::Path;
use std::path::PathBuf;

pub enum DisambiguationStrategy {
    None,
    Some { path_to_zip: PathToZip },
}

/// consider /a/b/c.zip/d/e/f.txt
/// Destination= /dest, disambiguate=true
/// Splat path = /dest/d/e/c.zip/f.txt
/// Destination= /dest, disambiguate=false
/// Splat path = /dest/d/e/f.txt
pub fn get_splat_path(
    path_inside_zip: &PathInsideZip,
    dest_dir: &Path,
    disambiguate: DisambiguationStrategy,
) -> eyre::Result<PathBuf> {
    let file_name = <PathInsideZip as AsRef<std::path::Path>>::as_ref(path_inside_zip)
        .file_name()
        .ok_or_eyre(eyre::eyre!(
            "Entry {} in zip has no file name, cannot process it.",
            path_inside_zip.display(),
        ))?;
    let splatted = match disambiguate {
        DisambiguationStrategy::None => path_inside_zip.to_path_buf(),
        DisambiguationStrategy::Some { path_to_zip } => match path_inside_zip.parent() {
            Some(parent) => {
                let zip_file_name = path_to_zip
                    .file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new("unknown_zip"));
                parent.join(zip_file_name).join(file_name).to_path_buf()
            }
            None => PathBuf::from(file_name),
        },
    };
    Ok(dest_dir.join(splatted))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn create_path_inside_zip(path: &str) -> PathInsideZip {
        PathInsideZip {
            inner: Arc::new(PathBuf::from(path)),
        }
    }

    fn create_path_to_zip(path: &str) -> PathToZip {
        PathToZip {
            inner: Arc::new(PathBuf::from(path)),
        }
    }

    #[test]
    fn test_get_splat_path_no_disambiguate() {
        let path_inside_zip = create_path_inside_zip("d/e/f.txt");
        let dest_dir = Path::new("/dest");

        let result =
            get_splat_path(&path_inside_zip, dest_dir, DisambiguationStrategy::None).unwrap();

        assert_eq!(result, PathBuf::from("/dest/d/e/f.txt"));
    }

    #[test]
    fn test_get_splat_path_with_disambiguate() {
        let path_inside_zip = create_path_inside_zip("d/e/f.txt");
        let path_to_zip = create_path_to_zip("/a/b/c.zip");
        let dest_dir = Path::new("/dest");

        let result = get_splat_path(
            &path_inside_zip,
            dest_dir,
            DisambiguationStrategy::Some { path_to_zip },
        )
        .unwrap();

        assert_eq!(result, PathBuf::from("/dest/d/e/c.zip/f.txt"));
    }

    #[test]
    fn test_get_splat_path_no_parent_directory() {
        let path_inside_zip = create_path_inside_zip("f.txt");
        let path_to_zip = create_path_to_zip("/a/b/c.zip");
        let dest_dir = Path::new("/dest");

        let result =
            get_splat_path(&path_inside_zip, dest_dir, DisambiguationStrategy::None).unwrap();
        assert_eq!(result, PathBuf::from("/dest/f.txt"));

        let result = get_splat_path(
            &path_inside_zip,
            dest_dir,
            DisambiguationStrategy::Some { path_to_zip },
        )
        .unwrap();
        assert_eq!(result, PathBuf::from("/dest/c.zip/f.txt"));
    }

    #[test]
    fn test_get_splat_path_deeply_nested() {
        let path_inside_zip = create_path_inside_zip("a/b/c/d/e/f/g.txt");
        let path_to_zip = create_path_to_zip("/source/backup.zip");
        let dest_dir = Path::new("/destination");

        let result =
            get_splat_path(&path_inside_zip, dest_dir, DisambiguationStrategy::None).unwrap();
        assert_eq!(result, PathBuf::from("/destination/a/b/c/d/e/f/g.txt"));

        let result = get_splat_path(
            &path_inside_zip,
            dest_dir,
            DisambiguationStrategy::Some { path_to_zip },
        )
        .unwrap();
        assert_eq!(
            result,
            PathBuf::from("/destination/a/b/c/d/e/f/backup.zip/g.txt")
        );
    }

    #[test]
    fn test_get_splat_path_no_file_name() {
        let path_inside_zip = create_path_inside_zip("folder/");
        let dest_dir = Path::new("/dest");

        let result = get_splat_path(&path_inside_zip, dest_dir, DisambiguationStrategy::None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_splat_path_windows_paths() {
        let path_inside_zip = create_path_inside_zip("documents\\images\\photo.jpg");
        let path_to_zip = create_path_to_zip("C:\\backups\\photos.zip");
        let dest_dir = Path::new("C:\\extracted");

        let result =
            get_splat_path(&path_inside_zip, dest_dir, DisambiguationStrategy::None).unwrap();
        // Note: PathBuf normalizes path separators based on the OS
        let expected = if cfg!(windows) {
            PathBuf::from("C:\\extracted\\documents\\images\\photo.jpg")
        } else {
            PathBuf::from("C:\\extracted\\documents\\images\\photo.jpg")
        };
        assert_eq!(result, expected);

        let result = get_splat_path(
            &path_inside_zip,
            dest_dir,
            DisambiguationStrategy::Some { path_to_zip },
        )
        .unwrap();
        let expected = if cfg!(windows) {
            PathBuf::from("C:\\extracted\\documents\\images\\photos.zip\\photo.jpg")
        } else {
            PathBuf::from("C:\\extracted\\documents\\images\\photos.zip\\photo.jpg")
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_splat_path_zip_file_no_extension() {
        let path_inside_zip = create_path_inside_zip("data/file.txt");
        let path_to_zip = create_path_to_zip("/archives/backup");
        let dest_dir = Path::new("/output");

        let result = get_splat_path(
            &path_inside_zip,
            dest_dir,
            DisambiguationStrategy::Some { path_to_zip },
        )
        .unwrap();
        assert_eq!(result, PathBuf::from("/output/data/backup/file.txt"));
    }

    #[test]
    fn test_get_splat_path_special_characters() {
        let path_inside_zip = create_path_inside_zip("folder with spaces/file-name_test.txt");
        let path_to_zip = create_path_to_zip("/path/archive with spaces.zip");
        let dest_dir = Path::new("/dest");

        let result =
            get_splat_path(&path_inside_zip, dest_dir, DisambiguationStrategy::None).unwrap();
        assert_eq!(
            result,
            PathBuf::from("/dest/folder with spaces/file-name_test.txt")
        );

        let result = get_splat_path(
            &path_inside_zip,
            dest_dir,
            DisambiguationStrategy::Some { path_to_zip },
        )
        .unwrap();
        assert_eq!(
            result,
            PathBuf::from("/dest/folder with spaces/archive with spaces.zip/file-name_test.txt")
        );
    }

    #[test]
    fn test_get_splat_path_empty_parent() {
        let path_inside_zip = create_path_inside_zip("file.txt");
        let path_to_zip = create_path_to_zip("/archives/data.zip");
        let dest_dir = Path::new("/target");

        let result = get_splat_path(
            &path_inside_zip,
            dest_dir,
            DisambiguationStrategy::Some { path_to_zip },
        )
        .unwrap();
        // When there's no parent directory, the zip file name should be used as the parent
        assert_eq!(result, PathBuf::from("/target/data.zip/file.txt"));
    }

    #[test]
    fn test_get_splat_path_zip_without_path() {
        let path_inside_zip = create_path_inside_zip("subfolder/document.pdf");
        let path_to_zip = create_path_to_zip("simple.zip");
        let dest_dir = Path::new("/extract");

        let result = get_splat_path(
            &path_inside_zip,
            dest_dir,
            DisambiguationStrategy::Some { path_to_zip },
        )
        .unwrap();
        assert_eq!(
            result,
            PathBuf::from("/extract/subfolder/simple.zip/document.pdf")
        );
    }

    #[test]
    fn test_get_splat_path_unicode_characters() {
        let path_inside_zip = create_path_inside_zip("フォルダ/ファイル.txt");
        let path_to_zip = create_path_to_zip("/アーカイブ/データ.zip");
        let dest_dir = Path::new("/出力");

        let result =
            get_splat_path(&path_inside_zip, dest_dir, DisambiguationStrategy::None).unwrap();
        assert_eq!(result, PathBuf::from("/出力/フォルダ/ファイル.txt"));

        let result = get_splat_path(
            &path_inside_zip,
            dest_dir,
            DisambiguationStrategy::Some { path_to_zip },
        )
        .unwrap();
        assert_eq!(
            result,
            PathBuf::from("/出力/フォルダ/データ.zip/ファイル.txt")
        );
    }
}
