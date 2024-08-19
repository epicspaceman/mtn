use std::{ffi::OsStr, path::Path};
use std::{env, fs};
use std::path::PathBuf;
use anyhow::{Context, Error, Ok, Result};
use cli::{AddDirectory, AddPath, AddQueryParameters};
use::exif;
use exif::Tag;
use clap::Parser;

use image::Pixel;

mod cli;
use crate::cli::Cli;
use crate::cli::Commands;

static IMAGE_FILE_TYPES: [&str; 13] = ["TIFF", "JPEG", "HEIF", "PNG", "WebP", "JPG", "tiff", "jpg", "jpeg", "heif", "png", "WEBP", "webp"];

static LUMA_MAX_VALUE: usize = 255;

static ID_TAGS: [Tag; 15]  = [Tag::Model, Tag::DateTime, Tag::ExposureTime, Tag::FNumber, Tag::ShutterSpeedValue, Tag::ApertureValue, Tag::ExposureBiasValue, Tag::Flash, Tag::FocalLength, Tag::ColorSpace, Tag::ExposureMode, Tag::WhiteBalance, Tag::CameraOwnerName, Tag::LensModel, Tag::ImageDescription];

fn get_tag(key: &str) -> Option<Tag> {
    for t in ID_TAGS {
        if t.to_string().trim() == key {
            return Some(t);
        }
    }
    None
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let current_working_directory = env::current_dir().unwrap();
    let default_path = current_working_directory.into_os_string().into_string().unwrap();

    match &args.command {
        Commands::View(added_path) => show_exif(added_path)?,
        Commands::Match(added_query_params) => match_exif(added_query_params, default_path)?,
        Commands::Group(added_dir) => group_images(added_dir, default_path)?,
        Commands::Render(added_path) => render_image(added_path)?,
    }

    Ok(())
}

fn show_exif(added_path: &AddPath) -> Result<(), Error> {
    let path_as_string = &added_path.path;
    let path = Path::new(&path_as_string);

    if !path.exists() || !IMAGE_FILE_TYPES.contains(&path.extension().and_then(OsStr::to_str).unwrap()) {
        println!("Not an image");
        return Ok(());
    }

    let exif = get_exif(path)
        .with_context(|| format!("Could not get exif from path: {:?}", &path))?;
    
    if exif.fields().len() == 0 {
        println!("Could not find any exif data");
        return Ok(());
    }

    for field in exif.fields() {
        // TODO add option to display all tags
        if ID_TAGS.contains(&field.tag) {
            if field.tag == Tag(exif::Context::Exif, 0) {
                println!("Custom Tag: {}",
                    field.display_value().with_unit(&exif));
            }
            println!("{}: {}",
                field.tag, field.display_value().with_unit(&exif));
        }
    }
    
    Ok(())
}

fn match_exif(added_query_params: &AddQueryParameters, default_path: String) -> Result<(), Error> {
    let directory = Path::new(&default_path);

    if !directory.is_dir() || !directory.exists() {
        println!("Invalid directory");
        return Ok(());
    }
    
    let tag_option = get_tag(&added_query_params.tag);

    if tag_option.is_none() {
        println!("Invalid tag");
        return Ok(());
    }

    let value = &added_query_params.value;

    let found_images = search_dir(directory, tag_option.unwrap(), &value)?;

    for image in found_images {
        println!("{:?}", image);
    }

    Ok(())
}

fn group_images(added_directory: &AddDirectory, default_path: String) -> Result<(), Error> {
    let directory_name_as_str = &added_directory.directory_name.join(" ");

    let directory = Path::new(&default_path);

    if !directory.is_dir() || !directory.exists() {
        println!("Invalid directory");
        return Ok(());
    }
    
    let tag_option = get_tag(&added_directory.tag);

    if tag_option.is_none() {
        println!("Invalid tag");
        return Ok(());
    }
    
    let value = &added_directory.value;
    let found_image_paths = search_dir(directory, tag_option.unwrap(), &value)?;

    let mut new_directory = PathBuf::from(directory);
    new_directory.push(directory_name_as_str);

    fs::create_dir(&new_directory)?;

    for image_path in found_image_paths {
        let mut new_image_path = new_directory.clone();
        new_image_path.push(image_path.file_name().and_then(OsStr::to_str).unwrap());

        fs::rename(image_path, new_image_path)?;
    }

    Ok(())
}

fn render_image(added_path: &AddPath) -> Result<(), Error> {
    let path_as_str = &added_path.path;
    let img = image::open(path_as_str).unwrap();
    let gray_img = img.to_luma8();

    let rendered_img_width = 50;

    let width_step = img.width() / rendered_img_width;
    let height_step = img.height() / rendered_img_width;

    let mut ascii_img = String::from("");
    
    // ascii chars ranked on darkness
    let mut ascii_chars_list: Vec<_> = "$@B%8&WM#*oahkbdpqwmZO0QLCJUYXzcvunxrjft/|()1{}[]?-_+~<>i!lI;:,^`'.".chars().collect();

    // since cli bg is black we'll want the darker ascii chars to act as lighter ones
    ascii_chars_list.reverse();

    for i in (0..img.height()).step_by(height_step as usize) {
        for j in (0..img.width()).step_by(width_step as usize) {
            // grab brightness for pixel at j, i
            let luma_val = gray_img.get_pixel(j, i).channels().first().unwrap();

            // scale brightness value down into range of ascii char list and then get the char at the corresponding index
            let ascii_char = *ascii_chars_list.get(*luma_val as usize / (LUMA_MAX_VALUE / ascii_chars_list.len())).unwrap_or(&'@');

            // push three times since ascii chars are tall
            ascii_img.push(ascii_char);
            ascii_img.push(ascii_char);
            ascii_img.push(ascii_char);
        }
        ascii_img.push_str("\n");
    }

    println!("{}", ascii_img);

    Ok(())
}

fn get_exif(file_path: &Path) -> Result<exif::Exif, exif::Error> {
    let file = std::fs::File::open(file_path).unwrap();

    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    return exifreader.read_from_container(&mut bufreader);
}

fn search_dir(directory: &Path, tag: Tag, value: &String) -> Result<Vec<PathBuf>, Error> {
    let mut found_images = Vec::new();

    for entry in std::fs::read_dir(directory)? {
        let entry = &entry?.path();
        if entry.is_dir() {
            let mut images_found_in_directory = search_dir(entry, tag, value)?;
            found_images.append(&mut images_found_in_directory);
        } else if !entry.extension().is_none() && IMAGE_FILE_TYPES.contains(&entry.extension().and_then(OsStr::to_str).unwrap()) {
            let exif_result = get_exif(entry);

            if !exif_result.is_err() {
                let exif = exif_result.unwrap();
                for field in exif.fields() {
                    if field.tag == tag && &field.display_value().with_unit(&exif).to_string() == value {
                        found_images.push(entry.clone());
                    }
                }
            } else {
                println!("Could not read exif for image at {:?}", entry);
            }
        }
    }
    return Ok(found_images);
}

