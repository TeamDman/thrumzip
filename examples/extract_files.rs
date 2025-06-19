use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::FzfArgs;
use cloud_terrastodon_user_input::pick_many;
use eyre::Context;
use eyre::OptionExt;
use eyre::Result;
use eyre::eyre;
use positioned_io::RandomAccessFile;
use rc_zip_tokio::ReadZip;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use thrumzip::get_zips::get_zips;
use thrumzip::path_inside_zip::PathInsideZip;
use thrumzip::path_to_zip::PathToZip;
use thrumzip::state::profiles::Profile;
use tokio::fs as async_fs;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install().wrap_err("Failed to install color_eyre")?;
    thrumzip::init_tracing::init_tracing(Level::INFO);
    // let profile = thrumzip::state::profiles::Profiles::load_and_get_active_profile().await?;
    let profile = Profile::new_example();

    // Collect zip files from both directories
    let (zip_paths, _) = get_zips(&profile.sources).await?;
    if zip_paths.is_empty() {
        eyre::bail!("No zip files found in {:?}", profile.sources);
    }

    // Map each entry name to the set of zip archives containing it
    let mut entry_map: HashMap<PathInsideZip, HashSet<PathToZip>> = HashMap::new();
    for zip in &zip_paths {
        let f = Arc::new(RandomAccessFile::open(zip)?);
        let archive = f.read_zip().await?;
        for entry in archive.entries() {
            let name_buf = entry
                .sanitized_name()
                .ok_or_eyre(eyre!("Invalid entry name"))?;
            let name = PathInsideZip::new(Arc::new(PathBuf::from(name_buf)));
            entry_map.entry(name).or_default().insert(zip.clone());
        }
    }

    // Build choices for fzf
    let mut choices: Vec<Choice<(PathInsideZip, HashSet<PathToZip>)>> = Vec::new();
    for (name, zips) in &entry_map {
        choices.push(Choice {
            key: format!("{} ({})", name.display(), zips.len()),
            value: (name.clone(), zips.clone()),
        });
    }

    let selected = pick_many(FzfArgs {
        choices,
        header: Some("Pick the files to extract".to_string()),
        ..Default::default()
    })?;

    // Prepare output dir
    let out_dir = PathBuf::from("extracted");
    async_fs::create_dir_all(&out_dir).await?;

    // Find the next available n for output dirs
    let mut max_n = 0;
    let mut read_dir = async_fs::read_dir(&out_dir).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        if let Some(fname) = entry.file_name().to_str() {
            if let Some(nstr) = fname.split('_').next() {
                if let Ok(n) = nstr.parse::<u32>() {
                    if n > max_n {
                        max_n = n;
                    }
                }
            }
        }
    }
    let mut next_n = max_n + 1;

    // For each selected entry, extract from all zips containing it
    for choice in selected.into_iter() {
        let (entry_name_path, zips) = choice.value;
        let entry_filename = entry_name_path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("");
        let entry_dir = out_dir.join(format!("{next_n:04}_{entry_filename}"));
        async_fs::create_dir_all(&entry_dir).await?;
        let mut provenance = String::new();
        for (k, zip) in zips.iter().enumerate() {
            let f = Arc::new(RandomAccessFile::open(zip)?);
            let archive = f.read_zip().await?;
            let entry = archive
                .entries()
                .find(|e| {
                    e.sanitized_name()
                        .map(|n| PathBuf::from(n) == **entry_name_path)
                        .unwrap_or(false)
                })
                .ok_or_eyre(eyre!("Entry not found in zip"))?;
            let data = entry.bytes().await?;
            let out_path = entry_dir.join(format!("{:04}/{}", k + 1, entry_filename));
            println!("Preparing to write to {}", out_path.display());
            if let Some(parent) = out_path.parent() {
                async_fs::create_dir_all(parent).await?;
            }
            println!("Writing {} bytes to {}", data.len(), out_path.display());
            async_fs::write(&out_path, &data).await?;
            provenance.push_str(&format!(
                "{:04}_{} <- {}\n",
                k + 1,
                entry_filename,
                zip.display()
            ));
            println!("Extracted {} to {}", entry_filename, out_path.display());
        }
        // Write provenance.txt
        let prov_path = entry_dir.join("provenance.txt");
        async_fs::write(&prov_path, provenance).await?;
        next_n += 1;
    }

    println!("All selected files extracted.");
    Ok(())
}
