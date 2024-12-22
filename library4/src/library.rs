use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{BufReader, Error as IoError, Write},
};
use MediaType::*;

#[derive(Hash, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum MediaType {
    Book {
        isbn10: Option<u64>,
        isbn13: Option<u64>,
    },
    AudioBook {
        duration: u32,
        isbn10: Option<u64>,
        isbn13: Option<u64>,
    },
    Sculpture {
        height: u32,
        width: u32,
        depth: u32,
        weight: u32,
        material: Option<String>,
    },
    Paiting {
        height: u32,
        width: u32,
        material: Option<String>,
    },
}

#[derive(Hash, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Media {
    pub id: u64,
    pub title: String,
    pub author: String,
    pub year: Option<u16>,
    pub available: bool,
    pub media_type: MediaType,
    pub keywords: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Library {
    pub name: String,
    pub file_path: String,
    pub catalogue: HashMap<u64, Media>,
}

pub enum ErrorKind {
    Io(IoError),
    MediaNotFound(u64),
    IsbnNotFound(u64),
    MediaNotAvailable(u64),
    MediaAlreadyAvailable(u64),
    IdAlreadyExists(u64),
    BookIsbnAlreadyExists,
    AudioBookIsbnAlreadyExists,
    MediaAlreadyExists(String),
    MediaDoesntHaveIsbn(String),
    MediaDoesntHaveDuration(String),
    MediaDoesntHaveDimensions(String),
    TitleNotFound(Vec<String>),
    AuthorNotFound(Vec<String>),
    KeywordNotFound(Vec<String>),
    TitleByAuthorNotFound(String, String),
}

impl ErrorKind {
    pub(crate) fn details(&self) -> String {
        use ErrorKind::*;
        match self {
            Io(e) => e.to_string(),
            MediaNotFound(id) => format!("Media with ID {} not found", id),
            IsbnNotFound(isbn) => format!("Media with ISBN {} not found", isbn),
            MediaNotAvailable(id) => format!("Media with ID {} is not available", id),
            MediaAlreadyAvailable(id) => format!("Media with ID {} is already available", id),
            IdAlreadyExists(id) => format!("Media with ID {} already exists", id),
            BookIsbnAlreadyExists => "Book with that ISBN already exists".to_string(),
            AudioBookIsbnAlreadyExists => format!("Audio {}", BookIsbnAlreadyExists.details()),
            TitleNotFound(title) => format!("Title with term(s) {:?} not found", title),
            AuthorNotFound(author) => format!("Author with term(s) {:?} not found", author),
            KeywordNotFound(keyword) => format!("Keyword(s) {:?} not found", keyword),
            MediaDoesntHaveIsbn(media) => format!("{}s don't have ISBN", media),
            MediaDoesntHaveDuration(media) => format!("{}s don't have duration", media),
            MediaDoesntHaveDimensions(media) => format!("{}s don't have dimensions registered", media),
            MediaAlreadyExists(media_type) => {
                format!("{} with this title and author already exists", media_type)
            }
            TitleByAuthorNotFound(title, author) => {
                format!("{} by {} not found", title, author)
            }
        }
    }
}

impl From<IoError> for ErrorKind {
    fn from(e: IoError) -> Self {
        ErrorKind::Io(e)
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

impl Display for &Media {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(year) = self.year {
            write!(
                f,
                "ID: {}\nTitle: {}\nAuthor: {}\n{}Year: {}\nAvailable: {}\nKeywords: {:?}",
                self.id,
                self.title,
                self.author,
                &self.media_type,
                year,
                self.available,
                self.keywords
            )
        } else {
            write!(
                f,
                "ID: {}\nTitle: {}\nAuthor: {}\n{}Available: {}\nKeywords: {:?}",
                self.id, self.title, self.author, &self.media_type, self.available, self.keywords
            )
        }
    }
}

impl Display for &MediaType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            MediaType::Book { isbn10, isbn13 } => {
                let mut display_isbn = String::new();
                if let Some(isbn) = isbn10 {
                    let isbn = format!("ISBN-10: {}\n", format_isbn(*isbn));
                    display_isbn.push_str(&isbn);
                }
                if let Some(isbn) = isbn13 {
                    let isbn = format!("ISBN-13: {}\n", format_isbn(*isbn));
                    display_isbn.push_str(&isbn);
                }
                write!(f, "{}", display_isbn)
            }
            MediaType::AudioBook {
                duration,
                isbn10,
                isbn13,
            } => {
                let mut display_audio_book = format!("Duration: {}\n", format_duration(*duration));
                if let Some(isbn) = isbn10 {
                    let isbn = format!("ISBN-10: {}\n", format_isbn(*isbn));
                    display_audio_book.push_str(&isbn);
                }
                if let Some(isbn) = isbn13 {
                    let isbn = format!("ISBN-13: {}\n", format_isbn(*isbn));
                    display_audio_book.push_str(&isbn);
                }
                write!(f, "{}", display_audio_book)
            }
            MediaType::Sculpture {
                height,
                width,
                depth,
                weight,
                material,
            } => {
                let mut display_sculpture = format!(
                    "Height: {} cm\nWidth: {} cm\nDepth: {} cm\nWeight: {} kg\n",
                    height, width, depth, weight
                );
                if let Some(material) = material {
                    let material = format!("Material: {}\n", material);
                    display_sculpture.push_str(&material);
                }
                write!(f, "{}", display_sculpture)
            }
            MediaType::Paiting {
                height,
                width,
                material,
            } => {
                let mut display_painting = format!("Height: {} cm\nWidth: {} cm\n", height, width);
                if let Some(material) = material {
                    let material = format!("Material: {}\n", material);
                    display_painting.push_str(&material);
                }
                write!(f, "{}", display_painting)
            }
        }
    }
}

impl Media {
    pub fn new(
        id: u64,
        title: String,
        author: String,
        year: Option<u16>,
        media_type: MediaType,
        keywords: Vec<String>,
    ) -> Media {
        Media {
            id,
            title,
            author,
            available: true,
            year,
            keywords: keywords
                .iter()
                .map(|k| k.to_lowercase())
                .collect::<Vec<String>>(),
            media_type,
        }
    }

    fn change_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    fn change_author(&mut self, author: &str) {
        self.author = author.to_string();
    }

    fn change_year(&mut self, year: u16) {
        self.year = Some(year);
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

    pub fn dimensions(&self) -> Result<String, ErrorKind> {
        match &self.media_type {
            Sculpture {
                height,
                width,
                depth,
                weight,
                ..
            } => Ok(format!(
                "Height: {} cm\nWidth: {} cm\nDepth: {} cm\nWeight: {} g\n",
                height, width, depth, weight
            )),
            Paiting { height, width, .. } => {
                Ok(format!("Height: {} cm\nWidth: {} cm\n", height, width))
            }
            _ => Err(ErrorKind::MediaDoesntHaveDimensions(self.type_to_string())),
        }
    }

    pub fn duration(&self) -> Result<String, ErrorKind> {
        match &self.media_type {
            AudioBook { duration, .. } => {
                Ok(format!("Duration: {}\n", format_duration(*duration)))
            }
            _ => Err(ErrorKind::MediaDoesntHaveDuration(self.type_to_string()))
        }
    }

    pub fn type_as_str(&self) -> &str {
        self.media_type.as_str()
    }
    
    pub fn type_to_string(&self) -> String {
        self.media_type.type_to_string()
    }

    pub fn isbn(&self) -> Result<String, ErrorKind> {
        match &self.media_type {
            Book { isbn10, isbn13 } => {
                let mut display_isbn = String::new();
                if let Some(isbn) = isbn10 {
                    let isbn = format!("ISBN-10: {}\n", format_isbn(*isbn));
                    display_isbn.push_str(&isbn);
                }
                if let Some(isbn) = isbn13 {
                    let isbn = format!("ISBN-13: {}\n", format_isbn(*isbn));
                    display_isbn.push_str(&isbn);
                }
                Ok(display_isbn)
            }
            AudioBook { isbn10, isbn13, .. } => {
                let mut display_audio_book = String::new();
                if let Some(isbn) = isbn10 {
                    let isbn = format!("ISBN-10: {}\n", format_isbn(*isbn));
                    display_audio_book.push_str(&isbn);
                }
                if let Some(isbn) = isbn13 {
                    let isbn = format!("ISBN-13: {}\n", format_isbn(*isbn));
                    display_audio_book.push_str(&isbn);
                }
                Ok(display_audio_book)
            }
            _ => Err(ErrorKind::MediaDoesntHaveIsbn(self.type_to_string())),
        }
    }

    pub fn material(&self) -> Result<String, ErrorKind> {
        match &self.media_type {
            Sculpture { material, .. } => {
                if let Some(material) = material {
                    Ok(format!("Material: {}\n", material))
                } else {
                    Err(ErrorKind::MediaDoesntHaveDimensions(self.type_to_string()))
                }
            }
            Paiting { material, .. } => {
                if let Some(material) = material {
                    Ok(format!("Material: {}\n", material))
                } else {
                    Err(ErrorKind::MediaDoesntHaveDimensions(self.type_to_string()))
                }
            }
            _ => Err(ErrorKind::MediaDoesntHaveDimensions(self.type_to_string())),
        }
    }
}

impl MediaType {
    pub fn as_str(&self) -> &str {
        match self {
            Book { .. } => "Book",
            AudioBook { .. } => "Audio Book",
            Sculpture { .. } => "Sculpture",
            Paiting { .. } => "Painting",
        }
    }

    pub fn type_to_string(&self) -> String {
        self.as_str().to_string()
    }

    pub fn new_book(isbn1: Option<u64>, isbn2: Option<u64>) -> MediaType {
        let isbn10: Option<u64>;
        let isbn13: Option<u64>;
        match (isbn1, isbn2) {
            (Some(isbn), None) => {
                if is_isbn13(isbn) {
                    isbn10 = None;
                    isbn13 = Some(isbn);
                } else {
                    isbn10 = Some(isbn);
                    isbn13 = None;
                }
            }
            (None, Some(isbn)) => {
                if is_isbn13(isbn) {
                    isbn10 = None;
                    isbn13 = Some(isbn);
                } else {
                    isbn10 = Some(isbn);
                    isbn13 = None;
                }
            }
            (Some(isbn1), Some(isbn2)) => {
                if is_isbn13(isbn1) {
                    isbn10 = Some(isbn2);
                    isbn13 = Some(isbn1);
                } else {
                    isbn10 = Some(isbn1);
                    isbn13 = Some(isbn2);
                }
            }
            _ => {
                isbn10 = None;
                isbn13 = None;
            }
        }
        MediaType::Book { isbn10, isbn13 }
    }

    pub fn new_audio_book(duration: u32, isbn1: Option<u64>, isbn2: Option<u64>) -> MediaType {
        let isbn10: Option<u64>;
        let isbn13: Option<u64>;
        match (isbn1, isbn2) {
            (Some(isbn), None) => {
                if is_isbn13(isbn) {
                    isbn10 = None;
                    isbn13 = Some(isbn);
                } else {
                    isbn10 = Some(isbn);
                    isbn13 = None;
                }
            }
            (None, Some(isbn)) => {
                if is_isbn13(isbn) {
                    isbn10 = None;
                    isbn13 = Some(isbn);
                } else {
                    isbn10 = Some(isbn);
                    isbn13 = None;
                }
            }
            (Some(isbn1), Some(isbn2)) => {
                if is_isbn13(isbn1) {
                    isbn10 = Some(isbn2);
                    isbn13 = Some(isbn1);
                } else {
                    isbn10 = Some(isbn1);
                    isbn13 = Some(isbn2);
                }
            }
            _ => {
                isbn10 = None;
                isbn13 = None;
            }
        }
        MediaType::AudioBook {
            duration,
            isbn10,
            isbn13,
        }
    }

    pub fn new_sculpture(
        height: u32,
        width: u32,
        depth: u32,
        weight: u32,
        material: Option<String>,
    ) -> MediaType {
        MediaType::Sculpture {
            height,
            width,
            depth,
            weight,
            material,
        }
    }

    pub fn new_painting(height: u32, width: u32, material: Option<String>) -> MediaType {
        MediaType::Paiting {
            height,
            width,
            material,
        }
    }

    pub fn change_isbn10(&mut self, isbn: u64) -> Result<(), ErrorKind> {
        match self {
            MediaType::Book { isbn10, .. } => {
                *isbn10 = Some(isbn);
                Ok(())
            }
            MediaType::AudioBook { isbn10, .. } => {
                *isbn10 = Some(isbn);
                Ok(())
            }
            media_type => Err(ErrorKind::MediaDoesntHaveIsbn(media_type.type_to_string())),
        }
    }

    pub fn change_isbn13(&mut self, isbn: u64) -> Result<(), ErrorKind> {
        match self {
            MediaType::Book { isbn13, .. } => {
                *isbn13 = Some(isbn);
                Ok(())
            }
            MediaType::AudioBook { isbn13, .. } => {
                *isbn13 = Some(isbn);
                Ok(())
            }
            media_type => Err(ErrorKind::MediaDoesntHaveIsbn(media_type.type_to_string())),
        }
    }

    pub fn check_isbn(&self, isbn: u64) -> bool {
        match self {
            MediaType::Book { isbn10, isbn13 } => {
                if let Some(isbn10) = isbn10 {
                    isbn == *isbn10
                } else if let Some(isbn13) = isbn13 {
                    isbn == *isbn13
                } else {
                    false
                }
            }
            MediaType::AudioBook { isbn10, isbn13, .. } => {
                if let Some(isbn10) = isbn10 {
                    isbn == *isbn10
                } else if let Some(isbn13) = isbn13 {
                    isbn == *isbn13
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

impl Library {
    pub fn new(name: &str, file_path: &str) -> Self {
        Library {
            name: name.to_string(),
            catalogue: HashMap::new(),
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

    pub fn load<'a>(
        file_path: &str,
        library: &'a mut Library,
    ) -> Result<&'a mut Library, ErrorKind> {
        let path = file_path;
        match File::open(path) {
            Ok(file) => {
                let reader = std::io::BufReader::new(file);
                match serde_json::from_reader::<BufReader<File>, Library>(reader) {
                    Ok(loaded) => {
                        library.name = loaded.name;
                        library.catalogue = loaded.catalogue;
                        library.file_path = loaded.file_path;
                        Ok(library)
                    }
                    Err(e) => Err(ErrorKind::Io(e.into())),
                }
            }
            Err(e) => Err(ErrorKind::Io(e)),
        }
    }

    pub fn add(&mut self, media: Media) -> Result<(), ErrorKind> {
        if self.contains(&media) {
            match media.media_type {
                Book { .. } => return Err(ErrorKind::BookIsbnAlreadyExists),
                AudioBook { .. } => return Err(ErrorKind::BookIsbnAlreadyExists),
                _ => {
                    return Err(ErrorKind::MediaAlreadyExists(
                        media.media_type.type_to_string(),
                    ))
                }
            }
        }
        self.catalogue.insert(media.id, media);
        Ok(())
    }

    pub fn remove(&mut self, id: u64) -> Result<(), ErrorKind> {
        match self.catalogue.remove(&id) {
            Some(_) => Ok(()),
            None => Err(ErrorKind::MediaNotFound(id)),
        }
    }

    pub fn list(&self) -> Vec<&Media> {
        self.catalogue.values().collect()
    }

    pub fn list_media_type(&self, media_type: &str) -> Vec<&Media> {
        self.catalogue
            .values()
            .filter(|media| {
                media.media_type.as_str().replace(" ", "").to_lowercase()
                    == media_type.replace(" ", "").to_lowercase()
            })
            .collect()
    }

    pub fn list_available(&self) -> Vec<&Media> {
        self.catalogue
            .values()
            .filter(|media| media.available)
            .collect()
    }

    pub fn list_available_from_type(&self, media_type: &str) -> Vec<&Media> {
        self.catalogue
            .values()
            .filter(|media| {
                media.available
                    && media.media_type.as_str().to_lowercase() == media_type.to_lowercase()
            })
            .collect()
    }

    pub fn list_borrowed(&self) -> Vec<&Media> {
        self.catalogue
            .values()
            .filter(|media| !media.available)
            .collect()
    }

    pub fn list_borrowed_from_type(&self, media_type: &str) -> Vec<&Media> {
        self.catalogue
            .values()
            .filter(|media| !media.available && media.media_type.as_str() == media_type)
            .collect()
    }

    pub fn contains(&self, media: &Media) -> bool {
        match media.media_type {
            Book { isbn10, isbn13 } => {
                let books = self.list_media_type("Book");
                if let Some(isbn) = isbn10 {
                    books.iter().any(|book| book.media_type.check_isbn(isbn))
                } else if let Some(isbn) = isbn13 {
                    books.iter().any(|book| book.media_type.check_isbn(isbn))
                } else {
                    false
                }
            }
            AudioBook { isbn10, isbn13, .. } => {
                let audio_books = self.list_media_type("Audio Book");
                if let Some(isbn) = isbn10 {
                    audio_books
                        .iter()
                        .any(|audio_book| audio_book.media_type.check_isbn(isbn))
                } else if let Some(isbn) = isbn13 {
                    audio_books
                        .iter()
                        .any(|audio_book| audio_book.media_type.check_isbn(isbn))
                } else {
                    false
                }
            }
            _ => self
                .catalogue
                .values()
                .any(|m| m.title == media.title && m.author == media.author),
        }
    }

    pub fn borrow(&mut self, id: u64) -> Result<(), ErrorKind> {
        match self.catalogue.get_mut(&id) {
            Some(book) => {
                if book.available {
                    book.toggle_availability();
                    Ok(())
                } else {
                    Err(ErrorKind::MediaNotAvailable(id))
                }
            }
            None => Err(ErrorKind::MediaNotFound(id)),
        }
    }

    pub fn return_media(&mut self, id: u64) -> Result<(), ErrorKind> {
        match self.catalogue.get_mut(&id) {
            Some(book) => {
                if book.available {
                    Err(ErrorKind::MediaAlreadyAvailable(id))
                } else {
                    book.toggle_availability();
                    Ok(())
                }
            }
            None => Err(ErrorKind::MediaNotFound(id)),
        }
    }

    pub fn search_author(&self, terms: Vec<String>) -> Result<Vec<&Media>, ErrorKind> {
        let mut media_items = Vec::new();
        let terms: Vec<String> = terms.iter().map(|term| term.to_lowercase()).collect();
        for media in self.catalogue.values() {
            let author = media.author.to_lowercase();
            if terms.iter().all(|term| author.contains(term)) {
                media_items.push(media);
            }
        }
        if media_items.is_empty() {
            Err(ErrorKind::AuthorNotFound(terms))
        } else {
            Ok(media_items)
        }
    }

    pub fn search_title(&self, terms: Vec<String>) -> Result<Vec<&Media>, ErrorKind> {
        let mut media_items = Vec::new();
        let terms: Vec<String> = terms.iter().map(|term| term.to_lowercase()).collect();
        for media in self.catalogue.values() {
            let title = media.title.to_lowercase();
            if terms.iter().all(|term| title.contains(term)) {
                media_items.push(media);
            }
        }
        if media_items.is_empty() {
            Err(ErrorKind::TitleNotFound(terms))
        } else {
            Ok(media_items)
        }
    }

    pub fn search_keywords(&self, keywords: Vec<String>) -> Result<Vec<&Media>, ErrorKind> {
        let mut media_items = Vec::new();
        let keywords: Vec<String> = keywords.iter().map(|k| k.to_lowercase()).collect();
        for media in self.catalogue.values() {
            if keywords
                .iter()
                .all(|keyword| media.keywords.contains(keyword))
            {
                media_items.push(media);
            }
        }
        if media_items.is_empty() {
            Err(ErrorKind::KeywordNotFound(keywords))
        } else {
            Ok(media_items)
        }
    }

    pub fn change_title(&mut self, id: u64, title: &str) -> Result<(), ErrorKind> {
        match self.catalogue.get_mut(&id) {
            Some(media) => {
                media.change_title(title);
                Ok(())
            }
            None => Err(ErrorKind::MediaNotFound(id)),
        }
    }

    pub fn change_author(&mut self, id: u64, author: &str) -> Result<(), ErrorKind> {
        match self.catalogue.get_mut(&id) {
            Some(media) => {
                media.change_author(author);
                Ok(())
            }
            None => Err(ErrorKind::MediaNotFound(id)),
        }
    }

    pub fn change_keywords(&mut self, id: u64, keywords: Vec<String>) -> Result<(), ErrorKind> {
        match self.catalogue.get_mut(&id) {
            Some(media) => {
                media.change_keywords(keywords);
                Ok(())
            }
            None => Err(ErrorKind::MediaNotFound(id)),
        }
    }

    pub fn change_year(&mut self, id: u64, year: u16) -> Result<(), ErrorKind> {
        match self.catalogue.get_mut(&id) {
            Some(media) => {
                media.change_year(year);
                Ok(())
            }
            None => Err(ErrorKind::MediaNotFound(id)),
        }
    }

    pub fn change_isbn(&mut self, id: u64, isbn: u64) -> Result<(), ErrorKind> {
        match self.catalogue.get_mut(&id) {
            Some(media) => {
                if is_isbn13(isbn) {
                    match media.media_type.change_isbn13(isbn) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e),
                    }
                } else {
                    match media.media_type.change_isbn10(isbn) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e),
                    }
                }
            }
            None => Err(ErrorKind::MediaNotFound(id)),
        }
    }

    pub fn add_keyword(&mut self, id: u64, keyword: &str) -> Result<(), ErrorKind> {
        match self.catalogue.get_mut(&id) {
            Some(media) => {
                media.add_keyword(keyword);
                Ok(())
            }
            None => Err(ErrorKind::MediaNotFound(id)),
        }
    }

    pub fn remove_keyword(&mut self, id: u64, keyword: &str) -> Result<(), ErrorKind> {
        match self.catalogue.get_mut(&id) {
            Some(book) => {
                book.remove_keyword(keyword);
                Ok(())
            }
            None => Err(ErrorKind::MediaNotFound(id)),
        }
    }

    pub fn get(&self, id: u64) -> Result<&Media, ErrorKind> {
        match self.catalogue.get(&id) {
            Some(media) => Ok(media),
            None => Err(ErrorKind::MediaNotFound(id)),
        }
    }

    pub fn get_by_isbn(&self, isbn: u64) -> Result<&Media, ErrorKind> {
        for media in self.catalogue.values() {
            match media.type_as_str() {
                "Book"  => {
                    if media.media_type.check_isbn(isbn) {
                        return Ok(media);
                    }
                }
                "Audo Book"  => {
                    if media.media_type.check_isbn(isbn) {
                        return Ok(media);
                    }
                }
                _ => return Err(ErrorKind::MediaDoesntHaveIsbn(media.type_to_string())),
            }
        }
        Err(ErrorKind::MediaNotFound(isbn))
    }

    pub fn get_by_title(&self, title: &str, author: &str) -> Result<&Media, ErrorKind> {
        for media in self.catalogue.values() {
            if media.title.to_lowercase() == title.to_lowercase() && media.author.to_lowercase() == author.to_lowercase() {
                return Ok(media);
            }
        }
        Err(ErrorKind::TitleByAuthorNotFound(title.to_string(), author.to_string()))
    }
}

impl Default for Library {
    fn default() -> Self {
        Library {
            name: "Library".to_string(),
            catalogue: HashMap::new(),
            file_path: "library.json".to_string(),
        }
    }
}

pub fn is_isbn13(isbn: u64) -> bool {
    isbn.checked_ilog10() == Some(12)
}

fn format_isbn(isbn: u64) -> String {
    let isbn_str = isbn.to_string();
    if is_isbn13(isbn) {
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

fn format_duration(duration: u32) -> String {
    let hours = duration / 3600;
    let minutes = (duration % 3600) / 60;
    let seconds = duration % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
