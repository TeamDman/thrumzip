use eyre::Result;
use color_eyre::install;
use duckdb::{Connection, params};
use image::{DynamicImage, ImageOutputFormat, RgbaImage, Rgba};
use std::fs;

fn main() -> Result<()> {
    // Initialize error reporting
    install()?;
    // Open or create the DuckDB database file
    let conn = Connection::open("images.db")?;

    // Create a table for images
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS images(name TEXT, data BLOB);"
    )?;

    // Generate a 50x50 solid yellow image
    let width = 50;
    let height = 50;
    let yellow = Rgba([255, 255, 0, 255]);
    let img: RgbaImage = RgbaImage::from_pixel(width, height, yellow);
    let dyn_img = DynamicImage::ImageRgba8(img);

    // Encode image to PNG in memory
    let mut buf: Vec<u8> = Vec::new();
    dyn_img.write_to(&mut buf, ImageOutputFormat::Png)?;

    // Insert the image blob into the database
    conn.execute(
        "INSERT INTO images (name, data) VALUES (?, ?)",
        params!["yellow", buf]
    )?;

    // Query back the images and save to files
    let mut stmt = conn.prepare("SELECT name, data FROM images")?;
    let rows = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        let data: Vec<u8> = row.get(1)?;
        Ok((name, data))
    })?;
    for result in rows {
        let (name, data) = result?;
        println!("Retrieved image '{}' ({} bytes)", name, data.len());
        let filename = format!("{}.png", name);
        fs::write(&filename, &data)?;
        println!("Saved image to {}", filename);
    }

    Ok(())
}