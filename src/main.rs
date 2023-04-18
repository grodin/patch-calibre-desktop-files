use clap::Parser;
use color_eyre::eyre;
use color_eyre::eyre::eyre;
use freedesktop_entry_parser::Entry;
use itertools::Itertools;
use mediatype::names::{APPLICATION, TEXT};
use mediatype::{media_type, MediaType, MediaTypeError, Name};
use std::path::PathBuf;

static DESKTOP_ENTRY_SECTION: &str = "Desktop Entry";

static MIME_TYPES_KEY: &str = "MimeType";

fn main() -> eyre::Result<()> {
    let args = Cli::parse();
    let desktop_file = Entry::parse_file(&args.input_file)?;

    if desktop_file.sections().count() > 1 {
        eprintln!("Can't process files with more than one section");
        std::process::exit(1)
    }

    if !desktop_file.has_section(DESKTOP_ENTRY_SECTION) {
        eprintln!(
            "No [Desktop Entry] section found in file {}",
            args.input_file.to_string_lossy()
        );
        std::process::exit(2)
    }

    let section = desktop_file.section(DESKTOP_ENTRY_SECTION);

    let mime_types = section
        .attr(MIME_TYPES_KEY)
        .ok_or_else(|| {
            eyre!(
                "Can't find MimeTypes entry in file {}",
                args.input_file.to_string_lossy()
            )
        })?
        .split_terminator(";")
        .map(MediaType::parse)
        .collect::<Result<Vec<_>, MediaTypeError>>()?;

    let disallowed_media_types = [
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

    let filtered_mime_types = mime_types
        .into_iter()
        .filter(|m| !disallowed_media_types.contains(m))
        .join(";");

    println!("[{}]", section.name());
    for key in KEYS {
        if let Some(val) = section.attr(key) {
            println!("{}={}", key, val);
        }
    }

    println!("{MIME_TYPES_KEY}={filtered_mime_types}");

    Ok(())
}

static X_RUBY: Name = Name::new_unchecked("x-ruby");
static X_MARKDOWN: Name = Name::new_unchecked("x-markdown");

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
    /// .desktop file to process
    input_file: PathBuf,
}
