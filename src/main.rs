use clap::Parser;
use color_eyre::eyre;
use color_eyre::eyre::{ensure, eyre, WrapErr};
use freedesktop_entry_parser::Entry;
use itertools::Itertools;
use mediatype::names::{APPLICATION, TEXT};
use mediatype::{media_type, MediaType, MediaTypeError, Name};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let args = Cli::parse();

    for input_file in &args.input_files {
        let desktop_file = Entry::parse_file(input_file)
            .wrap_err_with(|| format!("Error parsing file {}", input_file.to_string_lossy()))?;

        let buf = process_file(&desktop_file)
            .wrap_err_with(|| format!("Error processing file {}", input_file.to_string_lossy()))?;

        fs::write(input_file, &buf)
            .wrap_err_with(|| format!("Error writing to file {}", input_file.to_string_lossy()))?;
    }
    Ok(())
}

fn process_file(desktop_file: &Entry) -> eyre::Result<Vec<u8>> {
    ensure!(
        desktop_file.sections().count() == 1,
        "Can't process files with more than one section"
    );

    ensure!(
        desktop_file.has_section(&DESKTOP_ENTRY_SECTION),
        "No [Desktop Entry] section found in file"
    );

    let section = desktop_file.section(DESKTOP_ENTRY_SECTION);

    let mime_types = section
        .attr(MIME_TYPES_KEY)
        .ok_or_else(|| eyre!("Can't find MimeTypes entry in file"))?
        .split_terminator(";")
        .map(MediaType::parse)
        .collect::<Result<Vec<_>, MediaTypeError>>()?;

    let filtered_mime_types = mime_types
        .iter()
        .filter(|&m| DISALLOWED_MEDIA_TYPES.iter().any(|e| e != m))
        .join(";");

    let mut buf = Vec::new();

    writeln!(buf, "[{}]", section.name())?;
    for key in KEYS {
        if let Some(val) = section.attr(key) {
            writeln!(buf, "{key}={val}")?;
        }
    }

    writeln!(buf, "{MIME_TYPES_KEY}={filtered_mime_types}")?;
    Ok(buf)
}

const DISALLOWED_MEDIA_TYPES: [MediaType; 10] = [
    media_type!(TEXT / HTML),
    media_type!(TEXT / RTF),
    media_type!(APPLICATION / vnd::MS_WORD_DOCUMENT_MACRO_ENABLED_12),
    media_type!(TEXT / PLAIN),
    media_type!(APPLICATION / vnd::OPENXMLFORMATS_OFFICEDOCUMENT_WORDPROCESSINGML_DOCUMENT),
    MediaType::new(TEXT, X_MARKDOWN),
    media_type!(APPLICATION / vnd::OASIS_OPENDOCUMENT_TEXT),
    media_type!(APPLICATION / PDF),
    media_type!(APPLICATION / XHTML + XML),
    MediaType::new(APPLICATION, X_RUBY),
];

const X_RUBY: Name = Name::new_unchecked("x-ruby");
const X_MARKDOWN: Name = Name::new_unchecked("x-markdown");

static MIME_TYPES_KEY: &str = "MimeType";
static DESKTOP_ENTRY_SECTION: &str = "Desktop Entry";

static KEYS: [&str; 10] = [
    "Version",
    "Type",
    "Name",
    "GenericName",
    "Comment",
    "TryExec",
    "Exec",
    "Icon",
    "Categories",
    "X-GNOME-UsesNotifications",
];

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    /// .desktop files to process
    #[arg(required = true)]
    input_files: Vec<PathBuf>,
}
