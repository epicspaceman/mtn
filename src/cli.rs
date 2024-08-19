use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Read useful exif metadata for a given image
    Read(AddPath),

    /// Find files in given directory that match the tag and value
    Match(AddQueryParameters),

    // Group images with similar tag values into a new directory
    Group(AddDirectory),

    /// Render image in ASCII
    Render(AddPath),
}

#[derive(Args)]
pub struct AddPath {
    /// path to image
    pub path: String,
}

#[derive(Args)]
pub struct AddQueryParameters {
    /// tag to match
    pub tag: String,

    /// value to match
    pub value: String,
}

#[derive(Args)]
pub struct AddDirectory {
    pub tag: String,

    pub value: String,

    #[arg(trailing_var_arg = true)]
    pub directory_name: Vec<String>,
}