use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{BufReader, Error as IoError, Write},
};

#[derive(Hash, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Book {
    pub title: String,
    pub author: String,
    pub available: bool,
    pub isbn: u64,
    pub keywords: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Library {
    pub name: String,
    pub file_path: String,
    pub books: HashMap<u64, Book>,
}

pub enum ErrorKind {
    Io(IoError),
    BookNotFound(u64),
    BookNotAvailable(u64),
    BookAlreadyAvailable(u64),
    BookAlreadyExists(u64),
    TitleNotFound(Vec<String>),
    AuthorNotFound(Vec<String>),
    KeywordNotFound(Vec<String>),
}

impl ErrorKind {
    pub(crate) fn details(&self) -> String {
        use ErrorKind::*;
        match self {
            Io(e) => e.to_string(),
            BookNotFound(isbn) => format!("Book with ISBN {} not found", isbn),
            BookNotAvailable(isbn) => format!("Book with ISBN {} is not available", isbn),
            BookAlreadyAvailable(isbn) => format!("Book with ISBN {} is already available", isbn),
            BookAlreadyExists(isbn) => format!("Book with ISBN {} already exists", isbn),
            TitleNotFound(title) => format!("Title with terms {:?} not found", title),
            AuthorNotFound(author) => format!("Author with terms {:?} not found", author),
            KeywordNotFound(keyword) => format!("Keyword {:?} not found", keyword),
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ErrorKind::Io(e) => write!(f, "I/O error: {}", e),
            _ => write!(f, "Library error: {}", self.details().as_str()),
        }
    }
}

impl From<IoError> for ErrorKind {
    fn from(e: IoError) -> Self {
        ErrorKind::Io(e)
    }
}

impl Book {
    pub fn new(title: &str, author: &str, isbn: u64, keywords: Vec<String>) -> Book {
        Book {
            title: title.to_string(),
            author: author.to_string(),
            available: true,
            isbn,
            keywords: keywords
                .iter()
                .map(|k| k.to_lowercase())
                .collect::<Vec<String>>(),
        }
    }

    fn change_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    fn change_author(&mut self, author: &str) {
        self.author = author.to_string();
    }

    fn change_keywords(&mut self, keywords: Vec<String>) {
        let keywords: Vec<String> = keywords.iter().map(|k| k.to_lowercase()).collect();
        self.keywords = keywords;
    }

    fn add_keyword(&mut self, keyword: &str) {
        let keyword = keyword.to_lowercase();
        self.keywords.push(keyword.to_string());
    }

    fn remove_keyword(&mut self, keyword: &str) {
        let keyword = keyword.to_lowercase();
        self.keywords.retain(|k| k != &keyword);
    }

    fn toggle_availability(&mut self) {
        self.available = !self.available;
    }
}

impl Display for &Book {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let isbn = format_isbn(self.isbn);
        write!(
            f,
            "Title: {}\nAuthor: {}\n{}\nAvailable: {}\nKeywords: {:?}",
            self.title, self.author, isbn, self.available, self.keywords
        )
    }
}

impl Library {
    pub fn new(name: &str, file_path: &str) -> Self {
        Library {
            name: name.to_string(),
            books: HashMap::new(),
            file_path: file_path.to_string(),
        }
    }

    pub fn save(&self) -> Result<(), ErrorKind> {
        match File::create(&self.file_path) {
            Ok(mut file) => match serde_json::to_string_pretty(self) {
                Ok(json) => match file.write_all(json.as_bytes()) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(ErrorKind::Io(e)),
                },
                Err(e) => Err(ErrorKind::Io(e.into())),
            },
            Err(e) => Err(ErrorKind::Io(e)),
        }
    }

    pub fn load<'a>(file_path: &str, library: &'a mut Library) -> Result<&'a mut Library, ErrorKind> {
        let path = file_path;
        match File::open(path) {
            Ok(file) => {
                let reader = std::io::BufReader::new(file);
                match serde_json::from_reader::<BufReader<File>, Library>(reader) {
                    Ok(loaded) => {
                        library.name = loaded.name;
                        library.books = loaded.books;
                        library.file_path = loaded.file_path;
                        Ok(library)
                    },
                    Err(e) => Err(ErrorKind::Io(e.into())),
                }
            }
            Err(e) => Err(ErrorKind::Io(e)),
        }
    }

    pub fn add(&mut self, book: Book) -> Result<(), ErrorKind> {
        let isbn = book.isbn;
        if self.books.contains_key(&isbn) {
            return Err(ErrorKind::BookAlreadyExists(isbn));
        }
        self.books.insert(isbn, book);
        Ok(())
    }

    pub fn remove(&mut self, isbn: u64) -> Result<(), ErrorKind> {
        match self.books.remove(&isbn) {
            Some(_) => Ok(()),
            None => Err(ErrorKind::BookNotFound(isbn)),
        }
    }

    pub fn borrow(&mut self, isbn: u64) -> Result<(), ErrorKind> {
        match self.books.get_mut(&isbn) {
            Some(book) => {
                if book.available {
                    book.toggle_availability();
                    Ok(())
                } else {
                    Err(ErrorKind::BookNotAvailable(isbn))
                }
            }
            None => Err(ErrorKind::BookNotFound(isbn)),
        }
    }

    pub fn return_book(&mut self, isbn: u64) -> Result<(), ErrorKind> {
        match self.books.get_mut(&isbn) {
            Some(book) => {
                if book.available {
                    Err(ErrorKind::BookAlreadyAvailable(isbn))
                } else {
                    book.toggle_availability();
                    Ok(())
                }
            }
            None => Err(ErrorKind::BookNotFound(isbn)),
        }
    }

    pub fn search_author(&self, terms: Vec<String>) -> Result<Vec<&Book>, ErrorKind> {
        let mut books = Vec::new();
        let terms: Vec<String> = terms.iter().map(|term| term.to_lowercase()).collect();
        for book in self.books.values() {
            let author = book.author.to_lowercase();
            if terms.iter().all(|term| author.contains(term)) {
                books.push(book);
            }
        }
        if books.is_empty() {
            Err(ErrorKind::AuthorNotFound(terms))
        } else {
            Ok(books)
        }
    }

    pub fn search_title(&self, terms: Vec<String>) -> Result<Vec<&Book>, ErrorKind> {
        let mut books = Vec::new();
        let terms: Vec<String> = terms.iter().map(|term| term.to_lowercase()).collect();
        for book in self.books.values() {
            let title = book.title.to_lowercase();
            if terms.iter().all(|term| title.contains(term)) {
                books.push(book);
            }
        }
        if books.is_empty() {
            Err(ErrorKind::TitleNotFound(terms))
        } else {
            Ok(books)
        }
    }

    pub fn search_keywords(&self, keywords: Vec<String>) -> Result<Vec<&Book>, ErrorKind> {
        let mut books = Vec::new();
        let keywords: Vec<String> = keywords.iter().map(|k| k.to_lowercase()).collect();
        for book in self.books.values() {
            if keywords.iter().all(|keyword| book.keywords.contains(keyword)) {
                books.push(book);
            }
        }
        if books.is_empty() {
            Err(ErrorKind::KeywordNotFound(keywords))
        } else {
            Ok(books)
        }
    }

    pub fn list_books(&self) -> Vec<&Book> {
        self.books.values().collect()
    }

    pub fn list_available_books(&self) -> Vec<&Book> {
        self.books
            .values()
            .filter(|book| book.available)
            .collect()
    }

    pub fn list_borrowed_books(&self) -> Vec<&Book> {
        self.books
            .values()
            .filter(|book| !book.available)
            .collect()
    }

    pub fn change_book_title(&mut self, isbn: u64, title: &str) -> Result<(), ErrorKind> {
        match self.books.get_mut(&isbn) {
            Some(book) => {
                book.change_title(title);
                Ok(())
            }
            None => Err(ErrorKind::BookNotFound(isbn)),
        }
    }

    pub fn change_book_author(&mut self, isbn: u64, author: &str) -> Result<(), ErrorKind> {
        match self.books.get_mut(&isbn) {
            Some(book) => {
                book.change_author(author);
                Ok(())
            }
            None => Err(ErrorKind::BookNotFound(isbn)),
        }
    }

    pub fn change_book_keywords(
        &mut self,
        isbn: u64,
        keywords: Vec<String>,
    ) -> Result<(), ErrorKind> {
        match self.books.get_mut(&isbn) {
            Some(book) => {
                book.change_keywords(keywords);
                Ok(())
            }
            None => Err(ErrorKind::BookNotFound(isbn)),
        }
    }

    pub fn add_book_keyword(&mut self, isbn: u64, keyword: &str) -> Result<(), ErrorKind> {
        match self.books.get_mut(&isbn) {
            Some(book) => {
                book.add_keyword(keyword);
                Ok(())
            }
            None => Err(ErrorKind::BookNotFound(isbn)),
        }
    }

    pub fn remove_book_keyword(&mut self, isbn: u64, keyword: &str) -> Result<(), ErrorKind> {
        match self.books.get_mut(&isbn) {
            Some(book) => {
                book.remove_keyword(keyword);
                Ok(())
            }
            None => Err(ErrorKind::BookNotFound(isbn)),
        }
    }
}

impl Default for Library {
    fn default() -> Self {
        Library {
            name: "Library".to_string(),
            books: HashMap::new(),
            file_path: "library.json".to_string(),
        }
    }
}

fn format_isbn(isbn: u64) -> String {
    let isbn_str = isbn.to_string();
    if isbn.checked_ilog10() == Some(12) {
        format!(
            "ISBN-13: {}-{}-{}-{}-{}", 
            &isbn_str[0..3],
            &isbn_str[3..4],
            &isbn_str[4..9],
            &isbn_str[9..12],
            &isbn_str[12..13]
        )
    } else {
        format!(
            "ISBN-10: {}-{}-{}-{}", 
            &isbn_str[0..1],
            &isbn_str[1..5],
            &isbn_str[5..9],
            &isbn_str[9..10]
        )
    }
}
