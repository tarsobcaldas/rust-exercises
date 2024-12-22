use std::{
    collections::HashSet,
    fs::{File, OpenOptions},
    io::{prelude::*, stdin, stdout, BufReader, Write},
};

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
struct Book {
    title: String,
    author: String,
    available: bool,
}

impl Book {
    fn new(title: &str, author: &str) -> Book {
        Book {
            title: title.to_string(),
            author: author.to_string(),
            available: true,
        }
    }
}

struct Library {
    name: String,
    books: HashSet<Book>,
    file_path: String,
}

impl Library {
    fn new(name: &str, path: Option<&str>) -> Library {
        let default_path = "library-".to_owned() + name + ".txt";
        Library {
            name: name.to_string(),
            books: HashSet::new(),
            file_path: path.unwrap_or(&default_path).to_string(),
        }
    }

    fn add_book(&mut self, book: Book) {
        let title = &book.title;
        let author = &book.author;
        if self.find_book(title, author).is_some() {
            println!("Book already exists");
            return;
        }
        self.books.insert(book);
    }

    fn remove_book(&mut self, book: Book) {
        let title = &book.title;
        let author = &book.author;
        match self.find_book(title, author) {
            Some(_book) => {
                let tmp_book = book.clone();
                self.books.remove(&tmp_book);
                println!("Removed book {} by {}", title, author);
            }
            None => println!("Book not found"),
        }
    }

    fn rename_entry(&mut self, book: Book) {
        let title = &book.title;
        let author = &book.author;
        println!("Enter new title and author for book {} by {}", title, author);
        let new_book = input_book();
        match self.find_book(title, author) {
            Some(_book) => {
                let tmp_book = book.clone();
                self.books.remove(&tmp_book);
                self.books.insert(new_book);
                println!("Removed book {} by {}", title, author);
            }
            None => println!("Book not found"),
        }
    }


    fn list_books(&self) {
        for book in &self.books {
            println!(
                "Title: {}, Author: {}, Available: {}",
                book.title, book.author, book.available
            );
        }
    }

    fn find_book(&self, title: &str, author: &str) -> Option<&Book> {
        let book = self
            .books
            .iter()
            .find(|book| book.title == title && book.author == author);
        if let Some(book) = book {
            return Some(book);
        } else {
            return None;
        }
    }

    fn borrow_book(&mut self, book: Book) {
        if self.books.is_empty() {
            println!("No books in the library");
            return;
        } else {
            let title = &book.title;
            let author = &book.author;
            let book = self.find_book(title, author);
            if let Some(book) = book {
                if book.available == true {
                    let mut book = book.clone();
                    book.available = false;
                    self.books.replace(book.clone());
                    println!("Borrowed book {} from {}", book.title, book.author);
                    return;
                } else {
                    println!("Book is not available");
                }
            } else {
                println!("Book not found");
            }
        }
    }

    fn return_book(&mut self, book: Book) {
        if self.books.is_empty() {
            println!("No books in the library");
            return;
        } else {
            let title = &book.title;
            let author = &book.author;
            let book = self.find_book(title, author);
            if let Some(book) = book {
                if book.available == false {
                    let mut book = book.clone();
                    book.available = true;
                    self.books.replace(book.clone());
                    println!("Returned book {} to {}", book.title, book.author);
                    return;
                } else {
                    println!("Book is already available");
                }
            } else {
                println!("Book not found");
            }
        }
    }
}

fn save_library(library: &Library) {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&library.file_path)
        .unwrap();

    for book in &library.books {
        let line = format!("{};{};{}\n", book.title, book.author, book.available);
        file.write_all(line.as_bytes()).unwrap();
    }
}

fn load_library(library: &mut Library) {
    let file = File::open(&library.file_path).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        let parts: Vec<&str> = line.split(';').collect();
        let book = Book {
            title: parts[0].to_string(),
            author: parts[1].to_string(),
            available: parts[2].parse().unwrap(),
        };
        library.add_book(book);
    }
    println!("Library {} at {}", library.name, library.file_path);
}

fn input_book() -> Book {
    let mut title = String::new();
    let mut author = String::new();
    print!("Enter title: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut title).unwrap();
    print!("Enter author: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut author).unwrap();
    Book::new(title.trim(), author.trim())
}

fn process_book(str: &str) -> Book {
    let prefix = match str.split_whitespace().next() {
        Some(prefix) => prefix,
        None => "",
    }; 
    if let Some(args) =
        str.strip_prefix(prefix)
            .and_then(|a| if !a.is_empty() { Some(a) } else { None })
    {
        let split_args: Vec<&str> = args.split(';').collect();
        if split_args.len() == 2 {
            let title = split_args[0].trim();
            let author = split_args[1].trim();
            Book::new(title, author)
        } else {
            println!("Invalid arguments. Use: <title>;<author>");
            input_book()
        }
    } else {
        input_book()
    }
}

fn run_repl(library: &mut Library) {
    loop {
        print!("> ");
        stdout().flush().unwrap();
        let mut action = String::new();
        stdin().read_line(&mut action).unwrap();
        let action = action.trim();

        match action {
            ref str if str.starts_with("add") => {
                let book = process_book(str);
                library.add_book(book);
                println!("Added book {} by {}", title, author);
                save_library(&library);
            }
            ref str if str.starts_with("borrow") => {
                let book = process_book(str);
                library.borrow_book(book);
                save_library(&library);
            }
            ref str if str.starts_with("remove") => {
                let book = process_book(str);
                library.remove_book(book);
                save_library(&library);
            }
            ref str if str.starts_with("return") => {
                let book = process_book(str);
                library.return_book(book);
                save_library(&library);
            }
            ref str if str.starts_with("rename") => {
                let book = process_book(str);
                library.rename_entry(book);
                save_library(&library);
            }
            ref str if str.starts_with("load") => {
                let parts: Vec<&str> = str.split_whitespace().collect();
                let libname = String::from(parts[1]);
                let libpath = String::from(parts[2]);
                let mut next_library = Library::new(&libname, Some(&libpath));
                load_library(&mut next_library);
            }
            "list" | "ls" => {
                library.list_books();
            }
            "exit" | "q" => {
                print!("Cancel library edits? y (default)/n: ");
                stdout().flush().unwrap();
                let mut save_bool = String::new();
                stdin().read_line(&mut save_bool).unwrap();
                match save_bool.trim() {
                    "n" => break,
                    _ => {
                        save_library(&library);
                        break;
                    }
                }
            }
            "save" | "s" => {
                save_library(&library);
            }
            "help" | "h" => {
                println!("Commands:");
                println!("add <title>,<author> - Add a book to the library");
                println!("borrow <title> - Borrow a book from the library");
                println!("return <title> - Return a book to the library");
                println!("list - List all books in the library");
                println!("load - Load the library from the file");
                println!("save - Save the library to the file");
                println!("exit - Exit library");
            }
            _ => {
                println!("Invalid command");
            }
        }
    }
}

fn file_exists(file_path: &str) -> bool {
    match File::open(file_path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn main() {
    loop {
        let mut input = String::new();
        print!("Enter the name of the library and path (optional): ");
        stdout().flush().unwrap();
        stdin().read_line(&mut input).unwrap();
        match input.trim() {
            "" | "exit" | "q" | "break" => break,
            _ => (),
        }
        let args: Vec<&str> = input.split_whitespace().collect();
        let mut library: Library = match args.len() {
            1 => Library::new(args[0], None),
            2 => Library::new(args[0], Some(args[1])),
            _ => Library::new("library", None),
        };
        if !file_exists(&library.file_path) {
            println!("Library file not found, creating new library");
            run_repl(&mut library);
        } else {
            println!("Library file found, loading library");
            load_library(&mut library);
            run_repl(&mut library);
        }
    }
}
