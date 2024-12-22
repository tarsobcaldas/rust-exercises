use clap::{crate_name, ArgAction, Args, CommandFactory, Parser, Subcommand};
use std::{
    fmt::{self, Display, Formatter},
    io::{stdin, stdout, Write},
    path::Path,
};

use crate::library::{Book, ErrorKind as LibraryError, Library};
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
    #[command(
        arg_required_else_help = true,
        alias = "add",
        about = "Add a book to the library"
    )]
    AddBook {
        #[arg(required = true)]
        title: String,
        #[arg(required = true)]
        author: String,
        #[arg(required = true)]
        isbn: String,
        #[arg(required = false)]
        keywords: Vec<String>,
    },
    #[command(
        arg_required_else_help = true,
        alias = "rm",
        alias = "remove",
        about = "Remove a book from the library"
    )]
    RemoveBook { isbn: String },
    #[command(arg_required_else_help = true, about = "Change a book's details")]
    Change(ChangeCommands),
    #[command(
        arg_required_else_help = true,
        alias = "borrow",
        about = "Borrow a book"
    )]
    BorrowBook { isbn: String },
    #[command(
        arg_required_else_help = true,
        alias = "return",
        about = "Return a book"
    )]
    ReturnBook { isbn: String },
    #[command(
        arg_required_else_help = true,
        alias = "addkeyword",
        alias = "addk",
        about = "Add a keyword to a book"
    )]
    AddKeyword { isbn: String, keyword: String },
    #[command(arg_required_else_help = true, alias = "rmkeyword", alias = "rmk", about = "Remove a keyword from a book")]
    RemoveKeyword { isbn: String, keyword: String },
    #[command(arg_required_else_help = true, about = "Search for books based on a field")]
    Search(SearchCommands),
    #[command(alias = "ls", alias = "list", about = "List books in the library")]
    ListBooks {
        #[arg(short, long, exclusive(true))]
        #[clap(
            default_missing_value("true"),
            default_value("true"),
            num_args(0..=1),
            action = ArgAction::Set
        )]
        available: Option<bool>,
    },
    #[command(arg_required_else_help = true)]
    #[command(alias = "w", about = "Save the library")]
    Save { file_path: Option<String> },
    #[command(alias = "r", alias = "l", about = "Load the library")]
    Load { file_path: String },
    #[command(alias = "q", about = "Save and exit")]
    Exit,
    #[command(alias = "q!", alias = "exit!", about = "Exit without saving")]
    ForceExit,
}

pub enum ErrorKind {
    Library(LibraryError),
    InvalidCommand(String),
    InvalidIsbn,
    InvalidIsbn10,
    InvalidIsbn13,
    InvalidIsbnLength,
    InvalidQuoting,
    CouldNotReadLine,
    FileNotFound,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct SearchCommands {
    #[command(subcommand)]
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
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct ChangeCommands {
    #[command(subcommand)]
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
    isbn: String,
    substitution: Vec<String>,
}

impl ErrorKind {
    pub(crate) fn as_str(&self) -> &str {
        match *self {
            Library(_) => "Library error",
            InvalidCommand(_) => "Invalid command",
            InvalidIsbn => "Invalid ISBN",
            InvalidIsbn10 => "Invalid ISBN-10",
            InvalidIsbn13 => "Invalid ISBN-13",
            InvalidIsbnLength => "Invalid ISBN length",
            InvalidQuoting => "Invalid quoting",
            CouldNotReadLine => "Could not read line",
            FileNotFound => "Library file not found",
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


fn respond(line: &str, library: &mut Library) -> Result<bool, ErrorKind> {
    use Commands::*;
    let args = shlex::split(line).ok_or(InvalidQuoting)?;
    let cli = Repl::try_parse_from(args)?;
    match cli.cmd {
        AddBook {
            title,
            author,
            isbn,
            keywords,
        } => {
            let isbn = parse_isbn(isbn.as_str())?;
            library.add(Book::new(title.as_str(), author.as_str(), isbn, keywords))?;
            Ok(false)
        }
        RemoveBook { isbn } => {
            let isbn = parse_isbn(isbn.as_str())?;
            library.remove(isbn)?;
            Ok(false)
        }
        Change(args) => match args.field {
            ChangeField::Title(ChangeArgs { isbn, substitution }) => {
                let isbn_num = parse_isbn(isbn.as_str())?;
                let title = substitution.join(" ");
                library.change_book_title(isbn_num, &title)?;
                Ok(false)
            }
            ChangeField::Author(ChangeArgs { isbn, substitution }) => {
                let isbn_num = parse_isbn(isbn.as_str())?;
                let author = substitution.join(" ");
                library.change_book_author(isbn_num, &author)?;
                Ok(false)
            }
            ChangeField::Keywords(ChangeArgs { isbn, substitution }) => {
                let isbn_num = parse_isbn(isbn.as_str())?;
                library.change_book_keywords(isbn_num, substitution)?;
                Ok(false)
            }
        },
        BorrowBook { isbn } => {
            let isbn = parse_isbn(isbn.as_str())?;
            library.borrow(isbn)?;
            Ok(false)
        }
        ReturnBook { isbn } => {
            let isbn = parse_isbn(isbn.as_str())?;
            library.return_book(isbn)?;
            Ok(false)
        }
        AddKeyword { isbn, keyword } => {
            let isbn = parse_isbn(isbn.as_str())?;
            library.add_book_keyword(isbn, keyword.as_str())?;
            Ok(false)
        }
        RemoveKeyword { isbn, keyword } => {
            let isbn = parse_isbn(isbn.as_str())?;
            library.remove_book_keyword(isbn, keyword.as_str())?;
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
        ListBooks { available } => {
            match available {
                Some(true) => {
                    let books = library.list_available_books();
                    for book in books {
                        println!("{}", book);
                    }
                }
                Some(false) => {
                    let books = library.list_borrowed_books();
                    for book in books {
                        println!("{}", book);
                    }
                }
                _ => {
                    let books = library.list_books();
                    for book in books {
                        println!("{}", book);
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
        } else if Path::new(file_path.as_str()).exists() {
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
                AddBook {
                    title,
                    author,
                    isbn,
                    keywords,
                } => {
                    let isbn = parse_isbn(isbn.as_str()).map_err(|_| InvalidIsbn)?;
                    library
                        .add(Book::new(title.as_str(), author.as_str(), isbn, keywords))
                        .map_err(Library)?;
                    library.save().map_err(Library)?;
                    Ok(())
                }
                RemoveBook { isbn } => {
                    let isbn = parse_isbn(isbn.as_str()).map_err(|_| InvalidIsbn)?;
                    library.remove(isbn).map_err(Library)?;
                    library.save().map_err(Library)?;
                    Ok(())
                }
                Change(args) => match args.field {
                    ChangeField::Title(ChangeArgs { isbn, substitution }) => {
                        let isbn_num = parse_isbn(isbn.as_str())?;
                        let title = substitution.join(" ");
                        library
                            .change_book_title(isbn_num, &title)
                            .map_err(Library)?;
                        Ok(())
                    }
                    ChangeField::Author(ChangeArgs { isbn, substitution }) => {
                        let isbn_num = parse_isbn(isbn.as_str())?;
                        let author = substitution.join(" ");
                        library
                            .change_book_author(isbn_num, &author)
                            .map_err(Library)?;
                        Ok(())
                    }
                    ChangeField::Keywords(ChangeArgs { isbn, substitution }) => {
                        let isbn_num = parse_isbn(isbn.as_str())?;
                        library
                            .change_book_keywords(isbn_num, substitution)
                            .map_err(Library)?;
                        Ok(())
                    }
                },
                BorrowBook { isbn } => {
                    let isbn = parse_isbn(isbn.as_str())?;
                    library.borrow(isbn).map_err(Library)?;
                    library.save().map_err(Library)?;
                    Ok(())
                }
                ReturnBook { isbn } => {
                    let isbn = parse_isbn(isbn.as_str())?;
                    library.return_book(isbn).map_err(Library)?;
                    library.save().map_err(Library)?;
                    Ok(())
                }
                AddKeyword { isbn, keyword } => {
                    let isbn = parse_isbn(isbn.as_str())?;
                    library
                        .add_book_keyword(isbn, keyword.as_str())
                        .map_err(Library)?;
                    library.save().map_err(Library)?;
                    Ok(())
                }
                RemoveKeyword { isbn, keyword } => {
                    let isbn = parse_isbn(isbn.as_str())?;
                    library
                        .remove_book_keyword(isbn, keyword.as_str())
                        .map_err(Library)?;
                    library.save().map_err(Library)?;
                    Ok(())
                }
                Search(args) => {
                    match args.search_type {
                        SearchField::Title(SearchArgs { search_terms }) => {
                            if let Ok(books) = library.search_title(search_terms) {
                                for book in books {
                                    println!("{}\n", book);
                                }
                            } else {
                                println!("No books found");
                            }
                        }
                        SearchField::Author(SearchArgs { search_terms }) => {
                            if let Ok(books) = library.search_author(search_terms) {
                                for book in books {
                                    println!("{}\n", book);
                                }
                            } else {
                                println!("No books found");
                            }
                        }
                        SearchField::Keyword(SearchArgs { search_terms }) => {
                            if let Ok(books) = library.search_keywords(search_terms) {
                                for book in books {
                                    println!("{}\n", book);
                                }
                            } else {
                                println!("No books found");
                            }
                        }
                    }
                    Ok(())
                }
                ListBooks { available } => {
                    match available {
                        Some(true) => {
                            let books = library.list_available_books();
                            for book in books {
                                println!("{}\n", book);
                            }
                        }
                        Some(false) => {
                            let books = library.list_borrowed_books();
                            for book in books {
                                println!("{}\n", book);
                            }
                        }
                        _ => {
                            let books = library.list_books();
                            for book in books {
                                println!("{}\n", book);
                            }
                        }
                    }
                    Ok(())
                }
                _ => {
                    let mut usage_msg = Vec::new();
                    Cli::command().write_help(&mut usage_msg).unwrap();
                    let usage = String::from_utf8(usage_msg).unwrap();
                    Err(InvalidCommand(usage))
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
