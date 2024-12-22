use clap::{crate_name, ArgAction, Args, Parser, Subcommand};
use std::{
    fmt::{self, Display, Formatter},
    io::{stdin, stdout, Write},
    path::Path,
};

use crate::library::{ErrorKind as LibraryError, Library, Media, MediaType};
use ErrorKind::*;

#[derive(Parser, Debug)]
struct Repl {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Parser, Debug)]
pub struct Cli {
    library_path: Option<String>,
    #[command(subcommand)]
    cmd: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(subcommand_required = true, about = "Add a book to the library")]
    Add(AddCommands),
    #[command(
        arg_required_else_help = true,
        alias = "rm",
        about = "Remove a book from the library"
    )]
    Remove { id: u64 },
    #[command(subcommand_required = true, about = "Change a book's details")]
    Change(ChangeCommands),
    #[command(arg_required_else_help = true, about = "Borrow a book")]
    Borrow { id: u64 },
    #[command(arg_required_else_help = true, about = "Return a book")]
    Return { id: u64 },
    #[command(
        arg_required_else_help = true,
        alias = "addkeyword",
        alias = "addk",
        about = "Add a keyword to a book"
    )]
    AddKeyword { id: u64, keyword: String },
    #[command(
        arg_required_else_help = true,
        alias = "rmkeyword",
        alias = "rmk",
        about = "Remove a keyword from a book"
    )]
    RemoveKeyword { id: u64, keyword: String },
    #[command(
        subcommand_required = true,
        about = "Search for books based on a field"
    )]
    Search(SearchCommands),
    #[command(alias = "ls", about = "List books in the library")]
    List {
        #[arg(short, long, exclusive(true))]
        #[clap(
            default_missing_value("true"),
            default_value("true"),
            num_args(0..=1),
            action = ArgAction::Set
        )]
        available: Option<bool>,
        #[arg(short, long)]
        media_type: Option<String>,
    },
    #[command(subcommand_required = true, about = "Get information about an item")]
    Get(GetCommands),
    #[command(arg_required_else_help = true)]
    #[command(alias = "w", about = "Save the library (interactive mode only)")]
    Save { file_path: Option<String> },
    #[command(
        alias = "r",
        alias = "l",
        about = "Load the library (interactive mode only)"
    )]
    Load { file_path: String },
    #[command(alias = "q", about = "Save and exit (interactive mode only)")]
    Exit,
    #[command(
        alias = "q!",
        alias = "exit!",
        about = "Exit without saving (interactive mode only)"
    )]
    ForceExit,
}

pub enum ErrorKind {
    Library(LibraryError),
    InvalidCommand(String),
    InvalidDuration,
    InvalidIsbn,
    InvalidIsbn10,
    InvalidIsbn13,
    InvalidIsbnLength,
    InvalidQuoting,
    CouldNotReadLine,
    FileNotFound,
    InteractiveModeOnly,
    UsingTitleToGetTitle,
    UsingAuthorToGetAuthor,
    UsingIdToGetId,
    UsingIsbnToGetIsbn,
    MissingId,
    MissingArgs,
    YearNotFound,
    MediaDoesntHaveIsbn,
}

#[derive(Debug, Args)]
pub struct SearchCommands {
    #[command(subcommand, name = "field")]
    search_type: SearchField,
}

#[derive(Debug, Subcommand)]
enum SearchField {
    Title(SearchArgs),
    Author(SearchArgs),
    Keyword(SearchArgs),
}

#[derive(Debug, Args)]
struct SearchArgs {
    #[arg(required = true)]
    search_terms: Vec<String>,
}

#[derive(Debug, Args)]
pub struct ChangeCommands {
    #[command(subcommand, name = "field")]
    field: ChangeField,
}

#[derive(Debug, Subcommand)]
enum ChangeField {
    Title(ChangeArgs),
    Author(ChangeArgs),
    Keywords(ChangeArgs),
}

#[derive(Debug, Args)]
pub struct ChangeArgs {
    id: u64,
    substitution: Vec<String>,
}

#[derive(Debug, Args)]
#[clap(flatten_help=true)]
pub struct GetCommands {
    #[command(subcommand, name = "field")]
    get_field: GetField,
}

#[derive(Debug, Subcommand)]
#[command(arg_required_else_help = true)]
pub enum GetField {
    Title(GetTitleArgs),
    Author(GetAuthorArgs),
    Keywords(GetArgs),
    Duration(GetArgs),
    Isbn(GetIsbnArgs),
    Year(GetArgs),
    Material(GetArgs),
    Dimensions(GetArgs),
    Id(GetIdArgs),
}

#[derive(Debug, Args)]
pub struct GetArgs {
    #[arg(short, long, exclusive(true))]
    id: Option<u64>,
    #[arg(short, long, requires("author"))]
    title: Option<String>,
    #[arg(short, long, requires("title"))]
    author: Option<String>,
    #[arg(long, exclusive(true))]
    isbn: Option<String>,
}

#[derive(Debug, Args)]
pub struct GetTitleArgs {
    #[arg(short, long, exclusive(true))]
    id: Option<u64>,
    #[arg(long, exclusive(true))]
    isbn: Option<String>,
}

#[derive(Debug, Args)]
pub struct GetAuthorArgs {
    #[arg(short, long, exclusive(true))]
    id: Option<u64>,
    #[arg(long, exclusive(true))]
    isbn: Option<String>,
}

#[derive(Debug, Args)]
pub struct GetIdArgs {
    #[arg(short, long, requires("author"))]
    title: Option<String>,
    #[arg(short, long, requires("title"))]
    author: Option<String>,
    #[arg(short, long, exclusive(true))]
    isbn: Option<String>,
}

#[derive(Debug, Args)]
pub struct GetIsbnArgs {
    #[arg(short, long, exclusive(true))]
    id: Option<u64>,
    #[arg(short, long, requires("author"))]
    title: Option<String>,
    #[arg(short, long, requires("title"))]
    author: Option<String>,
}

#[derive(Debug, Args)]
pub struct AddCommands {
    #[command(subcommand, name = "media-type")]
    media_type: MediaField,
}

#[derive(Debug, Subcommand)]
enum MediaField {
    Book(BookArgs),
    Audiobook(AudiobookArgs),
    Sculpture(SculptureArgs),
    Painting(PaintingArgs),
}

#[derive(Debug, Args)]
pub struct BookArgs {
    title: String,
    author: String,
    #[arg(name = "ISBN-13", required = true)]
    isbn1: String,
    #[arg(name = "ISBN-10", required = false)]
    isbn2: Option<String>,
    year: Option<u16>,
    keywords: Vec<String>,
}

#[derive(Debug, Args)]
pub struct AudiobookArgs {
    title: String,
    author: String,
    #[arg(help = "Should be in the format HH:MM:SS")]
    duration: String,
    #[arg(name = "ISBN-13", required = true)]
    isbn1: String,
    #[arg(name = "ISBN-10", required = false)]
    isbn2: Option<String>,
    year: Option<u16>,
    keywords: Vec<String>,
}

#[derive(Debug, Args)]
pub struct SculptureArgs {
    title: String,
    author: String,
    #[arg(help = "Height in cm (integer)")]
    height: u32,
    #[arg(help = "Width in cm (integer)")]
    width: u32,
    #[arg(help = "Depth in cm (integer)")]
    depth: u32,
    #[arg(help = "Weight in grams (integer)")]
    weight: u32,
    year: Option<u16>,
    material: Option<String>,
    keywords: Vec<String>,
}

#[derive(Debug, Args)]
pub struct PaintingArgs {
    title: String,
    author: String,
    #[arg(help = "Height in cm (integer)")]
    height: u32,
    #[arg(help = "Width in cm (integer)")]
    width: u32,
    year: Option<u16>,
    material: Option<String>,
    keywords: Vec<String>,
}

impl ErrorKind {
    pub(crate) fn as_str(&self) -> &str {
        match *self {
            Library(_) => "Library error",
            InvalidCommand(_) => "Invalid command",
            InvalidDuration => "Invalid duration",
            InvalidIsbn => "Invalid ISBN",
            InvalidIsbn10 => "Invalid ISBN-10",
            InvalidIsbn13 => "Invalid ISBN-13",
            InvalidIsbnLength => "Invalid ISBN length",
            InvalidQuoting => "Invalid quoting",
            CouldNotReadLine => "Could not read line",
            FileNotFound => "Library file not found",
            InteractiveModeOnly => "Command not allowed in non-interactive mode",
            UsingTitleToGetTitle => "Using title to get title?",
            UsingAuthorToGetAuthor => "Using author to get author?",
            UsingIdToGetId => "Using ID to get ID?",
            UsingIsbnToGetIsbn => "Using ISBN to get ISBN?",
            MissingId => "Missing ID",
            MissingArgs => "Missing arguments",
            YearNotFound => "Year not found",
            MediaDoesntHaveIsbn => "Media doesn't have an ISBN",
        }
    }

    pub(crate) fn detailed_message(&self) -> String {
        use ErrorKind::*;
        match self {
            Library(e) => format!("{}", e),
            InvalidCommand(e) => e.to_string(),
            _ => self.as_str().to_string(),
        }
    }
}

impl From<clap::Error> for ErrorKind {
    fn from(e: clap::Error) -> Self {
        InvalidCommand(e.to_string())
    }
}

impl From<LibraryError> for ErrorKind {
    fn from(e: LibraryError) -> Self {
        Library(e)
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use ErrorKind::*;
        let message = self.detailed_message();
        match self {
            Library(_) => write!(f, "{}", message),
            InvalidCommand(_) => write!(f, "{}", message),
            _ => write!(f, "REPL error: {}", self.as_str()),
        }
    }
}

fn generate_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).unwrap();
    since_the_epoch.as_secs()
}

fn parse_duration(duration: &str) -> Result<u32, ErrorKind> {
    let time: Vec<&str> = duration.split(':').collect();
    if time.len() == 2 {
        let hours = time[0].parse::<u32>().map_err(|_| InvalidDuration)?;
        let minutes = time[1].parse::<u32>().map_err(|_| InvalidDuration)?;
        return Ok(hours * 3600 + minutes * 60);
    } else if time.len() == 3 {
        let hours = time[0].parse::<u32>().map_err(|_| InvalidDuration)?;
        let minutes = time[1].parse::<u32>().map_err(|_| InvalidDuration)?;
        let seconds = time[2].parse::<u32>().map_err(|_| InvalidDuration)?;
        return Ok(hours * 3600 + minutes * 60 + seconds);
    }
    Err(InvalidCommand("Invalid duration".to_string()))
}

fn parse_isbn(isbn: &str) -> Result<u64, ErrorKind> {
    let clean_isbn = isbn.replace("-", "");
    if clean_isbn.len() == 10 {
        let sum: i32 = clean_isbn
            .chars()
            .enumerate()
            .map(|(i, c)| match c {
                'X' => {
                    if i == 9 {
                        10
                    } else {
                        0
                    }
                }
                c if c.is_ascii_digit() => c.to_digit(10).unwrap() as i32,
                _ => 0,
            })
            .sum();

        if sum % 11 == 0 {
            let num_isbn: u64 = clean_isbn.parse::<u64>().map_err(|_| InvalidIsbn10)?;
            Ok(num_isbn)
        } else {
            Err(InvalidIsbn10)
        }
    } else if clean_isbn.len() == 13 {
        let sum: i32 = clean_isbn
            .chars()
            .enumerate()
            .map(|(i, c)| match c {
                c if c.is_ascii_digit() => {
                    c.to_digit(10).unwrap() as i32 * {
                        if i % 2 == 0 {
                            1
                        } else {
                            3
                        }
                    }
                }
                _ => 0,
            })
            .sum();

        if sum % 10 == 0 {
            let num_isbn: u64 = clean_isbn.parse::<u64>().map_err(|_| InvalidIsbn13)?;
            Ok(num_isbn)
        } else {
            Err(InvalidIsbn13)
        }
    } else {
        Err(InvalidIsbnLength)
    }
}

fn readline() -> Result<String, ErrorKind> {
    print!("> ");
    stdout().flush().unwrap();
    let mut buffer = String::new();
    match stdin().read_line(&mut buffer) {
        Ok(_) => {
            let trimmed_input = buffer.trim().to_string();
            let line = format!("{} {}", crate_name!(), trimmed_input);
            Ok(line)
        }
        Err(_) => Err(CouldNotReadLine),
    }
}

fn resolve_cmd(cmd: Commands, library: &mut Library) -> Result<bool, ErrorKind> {
    use Commands::*;
    match cmd {
        Add(args) => {
            use MediaField::*;
            let id = generate_id();
            let media = match args.media_type {
                Book(BookArgs {
                    title,
                    author,
                    year,
                    isbn1,
                    isbn2,
                    keywords,
                }) => {
                    let isbn1 = parse_isbn(isbn1.as_str())?;
                    if let Some(isbn2) = isbn2 {
                        let isbn2 = parse_isbn(isbn2.as_str())?;
                        let book = MediaType::new_book(Some(isbn1), Some(isbn2));
                        let media = Media::new(id, title, author, year, book, keywords);
                        library.add(media)?;
                        return Ok(false);
                    }
                    let book = MediaType::new_book(Some(isbn1), None);
                    Media::new(id, title, author, year, book, keywords)
                }
                Audiobook(AudiobookArgs {
                    title,
                    author,
                    year,
                    isbn1,
                    isbn2,
                    keywords,
                    duration,
                }) => {
                    let isbn1 = parse_isbn(isbn1.as_str())?;
                    let duration = parse_duration(duration.as_str())?;
                    if let Some(isbn2) = isbn2 {
                        let isbn2 = parse_isbn(isbn2.as_str())?;
                        let audio_book =
                            MediaType::new_audio_book(duration, Some(isbn1), Some(isbn2));
                        let media = Media::new(id, title, author, year, audio_book, keywords);
                        library.add(media)?;
                        return Ok(false);
                    }
                    let audio_book = MediaType::new_audio_book(duration, Some(isbn1), None);
                    Media::new(id, title, author, year, audio_book, keywords)
                }
                Sculpture(SculptureArgs {
                    title,
                    author,
                    height,
                    width,
                    depth,
                    weight,
                    year,
                    material,
                    keywords,
                }) => {
                    let statue = MediaType::new_sculpture(height, width, depth, weight, material);
                    Media::new(id, title, author, year, statue, keywords)
                }
                Painting(PaintingArgs {
                    title,
                    author,
                    height,
                    width,
                    year,
                    material,
                    keywords,
                }) => {
                    let painting = MediaType::new_painting(height, width, material);
                    Media::new(id, title, author, year, painting, keywords)
                }
            };
            match library.add(media) {
                Ok(_) => {
                    println!("Media added with ID: {}", id);
                    Ok(false)
                }
                Err(e) => Err(Library(e)),
            }
        }
        Remove { id } => {
            library.remove(id)?;
            Ok(false)
        }
        Change(args) => match args.field {
            ChangeField::Title(ChangeArgs { id, substitution }) => {
                let title = substitution.join(" ");
                library.change_title(id, &title)?;
                Ok(false)
            }
            ChangeField::Author(ChangeArgs { id, substitution }) => {
                let author = substitution.join(" ");
                library.change_author(id, &author)?;
                Ok(false)
            }
            ChangeField::Keywords(ChangeArgs { id, substitution }) => {
                library.change_keywords(id, substitution)?;
                Ok(false)
            }
        },
        Get(args) => {
            use GetField::*;
            match args.get_field {
                Title(GetTitleArgs { id, isbn }) => {
                    if let Some(id) = id {
                        match library.get(id) {
                            Ok(media) => {
                                println!("{}", media.title);
                                Ok(false)
                            }
                            Err(e) => Err(Library(e)),
                        }
                    } else if let Some(isbn) = isbn {
                        let isbn = parse_isbn(isbn.as_str())?;
                        match library.get_by_isbn(isbn) {
                            Ok(media) => {
                                println!("{}", media.title);
                                Ok(false)
                            }
                            Err(e) => Err(Library(e)),
                        }
                    } else {
                        Err(MissingId)
                    }
                }
                Author(GetAuthorArgs { id, isbn }) => {
                    if let Some(id) = id {
                        match library.get(id) {
                            Ok(media) => {
                                println!("{}", media.author);
                                Ok(false)
                            }
                            Err(e) => Err(Library(e)),
                        }
                    } else if let Some(isbn) = isbn {
                        let isbn = parse_isbn(isbn.as_str())?;
                        match library.get_by_isbn(isbn) {
                            Ok(media) => {
                                println!("{}", media.author);
                                Ok(false)
                            }
                            Err(e) => Err(Library(e)),
                        }
                    } else {
                        Err(MissingId)
                    }
                }
                Id(GetIdArgs { author, title, isbn }) => {
                    match (title, author) {
                        (Some(title), Some(author)) => {
                            match library.get_by_title(&title, &author) {
                                Ok(media) => {
                                    println!("{}", media.id);
                                    Ok(false)
                                }
                                Err(e) => Err(Library(e)),
                            }
                        }
                        (None, None) => {
                            if let Some(isbn) = isbn {
                                let isbn = parse_isbn(isbn.as_str())?;
                                match library.get_by_isbn(isbn) {
                                    Ok(media) => {
                                        println!("{}", media.id);
                                        Ok(false)
                                    }
                                    Err(e) => Err(Library(e)),
                                }
                            } else {
                                Err(MissingArgs)
                            }
                        }
                        _ => Err(MissingArgs),
                    }
                }
                Isbn(GetIsbnArgs { id, title, author, }) => {
                    if let Some(id) = id {
                        match library.get(id) {
                            Ok(media) => {
                                match media.isbn() {
                                    Ok(isbn) => {
                                        println!("{}", isbn);
                                        Ok(false)
                                    }
                                    Err(e) => Err(Library(e)),
                                }
                            }
                            Err(e) => Err(Library(e)),
                        }
                    } else if let (Some(title), Some(author)) = (title, author) {
                        match library.get_by_title(&title, &author) {
                            Ok(media) => {
                                match media.isbn() {
                                    Ok(isbn) => {
                                        println!("{}", isbn);
                                        Ok(false)
                                    }
                                    Err(e) => Err(Library(e)),
                                }
                            }
                            Err(e) => Err(Library(e)),
                        }
                    } else {
                        Err(MissingId)
                    }
                }
                Duration(GetArgs { id, title, author, isbn }) => {
                    if let Some(id) = id {
                        match library.get(id) {
                            Ok(media) => {
                                match media.duration() {
                                    Ok(duration) => {
                                        println!("{}", duration);
                                        Ok(false)
                                    }
                                    Err(e) => Err(Library(e)),
                                }
                            }
                            Err(e) => Err(Library(e)),
                        }
                    } else if let (Some(title), Some(author)) = (title, author) {
                        match library.get_by_title(&title, &author) {
                            Ok(media) => {
                                match media.duration() {
                                    Ok(duration) => {
                                        println!("{}", duration);
                                        Ok(false)
                                    }
                                    Err(e) => Err(Library(e)),
                                }
                            }
                            Err(e) => Err(Library(e)),
                        }
                    } else if let Some(isbn) = isbn {
                        let isbn = parse_isbn(isbn.as_str())?;
                        match library.get_by_isbn(isbn) {
                            Ok(media) => {
                                match media.duration() {
                                    Ok(duration) => {
                                        println!("{}", duration);
                                        Ok(false)
                                    }
                                    Err(e) => Err(Library(e)),
                                }
                            }
                            Err(e) => Err(Library(e)),
                        }
                    } else {
                        Err(MissingId)
                    }
                }
                Material(GetArgs { id, title, author, .. }) => {
                    if let Some(id) = id {
                        match library.get(id) {
                            Ok(media) => {
                                match media.material() {
                                    Ok(material) => {
                                        println!("{}", material);
                                        Ok(false)
                                    }
                                    Err(e) => Err(Library(e)),
                                }
                            }
                            Err(e) => Err(Library(e)),
                        }
                    } else if let (Some(title), Some(author)) = (title, author) {
                        match library.get_by_title(&title, &author) {
                            Ok(media) => {
                                match media.material() {
                                    Ok(material) => {
                                        println!("{}", material);
                                        Ok(false)
                                    }
                                    Err(e) => Err(Library(e)),
                                }
                            }
                            Err(e) => Err(Library(e)),
                        }
                    } else {
                        Err(MissingId)
                    }
                }
                Dimensions(GetArgs { id, title, author, isbn }) => {
                    if let Some(id) = id {
                        match library.get(id) {
                            Ok(media) => {
                                match media.dimensions() {
                                    Ok(dimensions) => {
                                        println!("{}", dimensions);
                                        Ok(false)
                                    }
                                    Err(e) => Err(Library(e)),
                                }
                            }
                            Err(e) => Err(Library(e)),
                        }
                    } else if let (Some(title), Some(author)) = (title, author) {
                        match library.get_by_title(&title, &author) {
                            Ok(media) => {
                                match media.dimensions() {
                                    Ok(dimensions) => {
                                        println!("{}", dimensions);
                                        Ok(false)
                                    }
                                    Err(e) => Err(Library(e)),
                                }
                            }
                            Err(e) => Err(Library(e)),
                        }
                    } else if isbn.is_some() {
                        Err(MediaDoesntHaveIsbn)
                    } else {
                        Err(MissingArgs)
                    }
                }
                Keywords(GetArgs { id, title, author, isbn }) => {
                    if let Some(id) = id {
                        match library.get(id) {
                            Ok(media) => {
                                println!("{:?}", media.keywords);
                                Ok(false)
                            }
                            Err(e) => Err(Library(e)),
                        }
                    } else if let (Some(title), Some(author)) = (title, author) {
                        match library.get_by_title(&title, &author) {
                            Ok(media) => {
                                println!("{:?}", media.keywords);
                                Ok(false)
                            }
                            Err(e) => Err(Library(e)),
                        }
                    } else if let Some(isbn) = isbn {
                        let isbn = parse_isbn(isbn.as_str())?;
                        match library.get_by_isbn(isbn) {
                            Ok(media) => {
                                println!("{:?}", media.keywords);
                                Ok(false)
                            }
                            Err(e) => Err(Library(e)),
                        }
                    } else {
                        Err(MissingArgs)
                    }
                }
                Year(GetArgs { id, title, author, isbn }) => {
                    if let Some(id) = id {
                        match library.get(id) {
                            Ok(media) => {
                                match media.year {
                                    Some(year) => {
                                        println!("{}", year);
                                        Ok(false)
                                    }
                                    None => Err(YearNotFound),
                                }
                            }
                            Err(e) => Err(Library(e)),
                        }
                    } else if let (Some(title), Some(author)) = (title, author) {
                        match library.get_by_title(&title, &author) {
                            Ok(media) => {
                                match media.year {
                                    Some(year) => {
                                        println!("{}", year);
                                        Ok(false)
                                    }
                                    None => Err(YearNotFound),
                                }
                            }
                            Err(e) => Err(Library(e)),
                        }
                    } else if let Some(isbn) = isbn {
                        let isbn = parse_isbn(isbn.as_str())?;
                        match library.get_by_isbn(isbn) {
                            Ok(media) => {
                                match media.year {
                                    Some(year) => {
                                        println!("{}", year);
                                        Ok(false)
                                    }
                                    None => Err(YearNotFound),
                                }
                            }
                            Err(e) => Err(Library(e)),
                        }
                    } else {
                        Err(MissingArgs)
                    }
                }
            }
        }
        Borrow { id } => {
            library.borrow(id)?;
            Ok(false)
        }
        Return { id } => {
            library.return_media(id)?;
            Ok(false)
        }
        AddKeyword { id, keyword } => {
            library.add_keyword(id, keyword.as_str())?;
            Ok(false)
        }
        RemoveKeyword { id, keyword } => {
            library.remove_keyword(id, keyword.as_str())?;
            Ok(false)
        }
        Search(args) => {
            match args.search_type {
                SearchField::Title(SearchArgs { search_terms }) => {
                    match library.search_title(search_terms) {
                        Ok(books) => {
                            for book in books {
                                println!("{}\n", book);
                            }
                        }
                        Err(e) => return Err(Library(e)),
                    }
                }
                SearchField::Author(SearchArgs { search_terms }) => {
                    match library.search_author(search_terms) {
                        Ok(books) => {
                            for book in books {
                                println!("{}\n", book);
                            }
                        }
                        Err(e) => return Err(Library(e)),
                    }
                }
                SearchField::Keyword(SearchArgs { search_terms }) => {
                    match library.search_keywords(search_terms) {
                        Ok(books) => {
                            for book in books {
                                println!("{}\n", book);
                            }
                        }
                        Err(e) => return Err(Library(e)),
                    }
                }
            }
            Ok(false)
        }
        List {
            available,
            media_type,
        } => {
            if let Some(media_type) = media_type {
                match available {
                    Some(true) => {
                        let media_list = library.list_available_from_type(&media_type);
                        for media in media_list {
                            println!("{}\n", media);
                        }
                    }
                    Some(false) => {
                        let media_list = library.list_borrowed_from_type(&media_type);
                        for media in media_list {
                            println!("{}\n", media);
                        }
                    }
                    _ => {
                        let media_list = library.list_media_type(&media_type);
                        for media in media_list {
                            println!("{}\n", media);
                        }
                    }
                }
                return Ok(false);
            }

            match available {
                Some(true) => {
                    let media_list = library.list_available();
                    for media in media_list {
                        println!("{}\n", media);
                    }
                }
                Some(false) => {
                    let media_list = library.list_borrowed();
                    for media in media_list {
                        println!("{}\n", media);
                    }
                }
                _ => {
                    let media_list = library.list();
                    for media in media_list {
                        println!("{}\n", media);
                    }
                }
            }
            Ok(false)
        }
        Load { file_path } => {
            Library::load(file_path.as_str(), library).map_err(Library)?;
            Ok(false)
        }
        Save { file_path } => {
            if let Some(file_path) = file_path {
                library.file_path = file_path;
            }
            library.save().map_err(Library)?;
            Ok(false)
        }
        Exit => match confirm_exit() {
            Ok(true) => {
                library.save().map_err(Library)?;
                Ok(true)
            }
            not_confirmed => not_confirmed,
        },
        ForceExit => Ok(true),
    }
}

fn respond(line: &str, library: &mut Library) -> Result<bool, ErrorKind> {
    let args = shlex::split(line).ok_or(InvalidQuoting)?;
    let cli = Repl::try_parse_from(args)?;
    resolve_cmd(cli.cmd, library)
}

fn confirm_exit() -> Result<bool, ErrorKind> {
    print!("Are you sure you want to exit? (y/n): ");
    stdout().flush().unwrap();
    let mut buffer = String::new();
    match stdin().read_line(&mut buffer) {
        Ok(_) => match buffer.trim() {
            "y" => Ok(true),
            "n" => Ok(false),
            _ => confirm_exit(),
        },
        Err(_) => Err(CouldNotReadLine),
    }
}

fn run_repl(library: &mut Library) -> Result<(), ErrorKind> {
    loop {
        let line = readline()?;
        if line.is_empty() {
            continue;
        }

        match respond(&line, library) {
            Ok(true) => break,
            Ok(false) => continue,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        }
    }
    Ok(())
}

pub fn run(args: Cli) -> Result<(), ErrorKind> {
    use Commands::*;
    let mut library = Library::default();

    if let Some(file_path) = &args.library_path {
        let name = file_path.as_str();
        let default_path_name = name.to_owned() + "-library.json";
        let default_path = Path::new(&default_path_name);
        if default_path.exists() {
            library.file_path = String::from(&default_path_name);
            match Library::load(&default_path_name, &mut library) {
                Ok(_) => {}
                Err(e) => return Err(Library(e)),
            }
        } else if Path::new(name).exists() {
            library.file_path = String::from(file_path);
            match Library::load(file_path.as_str(), &mut library) {
                Ok(_) => {}
                Err(e) => return Err(Library(e)),
            }
        } else {
            return Err(FileNotFound);
        }

        if let Some(cli) = args.cmd {
            match cli {
                Save { .. } => Err(InteractiveModeOnly),
                Load { .. } => Err(InteractiveModeOnly),
                Exit => Err(InteractiveModeOnly),
                ForceExit => Err(InteractiveModeOnly),
                _ => {
                    resolve_cmd(cli, &mut library)?;
                    Ok(())
                }
            }
        } else {
            run_repl(&mut library)?;
            Ok(())
        }
    } else {
        let default_path = Path::new("library.json");
        if default_path.exists() {
            library.file_path = String::from("library.json");
            match Library::load("library.json", &mut library) {
                Ok(_) => {}
                Err(e) => return Err(Library(e)),
            }
        }
        run_repl(&mut library)?;
        Ok(())
    }
}
