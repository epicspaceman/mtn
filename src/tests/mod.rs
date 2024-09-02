use super::*;
use anyhow::{Result, Error};

#[test]
fn test_show_exif() -> Result<(), Error> {
    let mock_added_path = AddPathForExif {
        all: false,
        path: vec!["src/tests/test_img.jpg".to_string()],
    };

    let mock_show_exif_command = Commands::View(mock_added_path);

    match_command(&mock_show_exif_command)?;

    Ok(())
}

#[test]
fn test_match_exif() -> Result<(), Error> {
    let mock_added_query_parameters = AddQueryParameters {
        tag: "ColorSpace".to_string(),
        value: vec!["sRGB".to_string()],
    };

    let mock_show_exif_command = Commands::Match(mock_added_query_parameters);

    match_command(&mock_show_exif_command)?;

    Ok(())
}

#[test]
fn test_group_exif() -> Result<(), Error> {
    let mock_added_directory = AddDirectory {
        tag: "ColorSpace".to_string(),
        value: "sRGB".to_string(),
        directory_name: vec!["test".to_string()],
    };

    let mock_show_exif_command = Commands::Group(mock_added_directory);

    match_command(&mock_show_exif_command)?;

    fs::rename("test/test_img.jpg", "src/tests/test_img.jpg")?;
    fs::remove_dir("test")?;

    Ok(())
}

#[test]
fn test_render_exif() -> Result<(), Error> {
    let mock_added_path = AddPath {
        path: vec!["src/tests/test_img.jpg".to_string()],
    };

    let mock_show_exif_command = Commands::Render(mock_added_path);

    match_command(&mock_show_exif_command)?;

    Ok(())
}

#[test]
fn test_move_exif() -> Result<(), Error> {
    let mock_added_directory = AddDirectory {
        tag: "ColorSpace".to_string(),
        value: "sRGB".to_string(),
        directory_name: vec!["src".to_string()],
    };

    let mock_show_exif_command = Commands::Move(mock_added_directory);

    match_command(&mock_show_exif_command)?;

    fs::rename("src/test_img.jpg", "src/tests/test_img.jpg")?;

    Ok(())
}