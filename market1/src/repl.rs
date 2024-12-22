use {
    crate::{
        inventory::{ErrorKind as InventoryError, Storage},
        product::ProductList,
        warehouse::Warehouse,
    },
    chrono::NaiveDate,
    std::{
        error::Error,
        fmt::{self, Display, Formatter},
        io::{stdin, stdout, Write},
        path::Path,
    },
    // clap::{Parser as CliParser,Subcommand},
    ErrorKind::*,
};

struct Prompt;

struct Parser;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ErrorKind {
    InvalidCommand,
    InvalidId,
    InvalidIdOrName,
    InvalidQuantity,
    InvalidPrice,
    InvalidDate,
    InvalidNumber,
    InvalidFile,
    CouldNotSaveStorage,
    CouldNotCreateStorage,
    CouldNotLoadStorage,
    InvalidArguments(Usage),
    StorageError(InventoryError),
}

#[derive(Debug)]
pub enum Usage {
    AddProduct,
    DeleteProduct,
    RestockProduct,
    RemoveStock,
    EmptyStock,
    Storage,
}

impl ErrorKind {
    pub(crate) fn as_str(&self) -> &'static str {
        match *self {
            InvalidCommand => "Invalid command",
            InvalidId => "Invalid ID",
            InvalidIdOrName => "Invalid ID or Name",
            InvalidQuantity => "Invalid quantity",
            InvalidPrice => "Invalid price",
            InvalidDate => "Invalid date",
            InvalidNumber => "Invalid number",
            InvalidFile => "Invalid file",
            CouldNotSaveStorage => "Could not save storage",
            CouldNotCreateStorage => "Could not create storage",
            CouldNotLoadStorage => "Could not load storage",
            StorageError(_) => "Storage error",
            InvalidArguments(_) => "Invalid arguments",
        }
    }

    pub(crate) fn detailed_msg(&self) -> String {
        match self {
            StorageError(e) => format!("{}", e),
            InvalidArguments(u) => format!("Invalid arguments. \nUsage: {}", u),
            _ => self.as_str().to_string(),
        }
    }
}

impl Usage {
    pub(crate) fn as_str(&self) -> &'static str {
        use Usage::*;

        match *self {
            AddProduct => "add_product [<name> <price>]",
            DeleteProduct => "delete_product [id or name]",
            RestockProduct => "restock_product [id or name] [quantity] [expiration_date]",
            RemoveStock => "remove_stock [id or name] [quantity]",
            EmptyStock => "empty_stock [id or name]",
            Storage => "storage [create | load <file_path>]",
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let message = self.detailed_msg();
        match self {
            StorageError(_) => write!(f, "{}", message),
            _ => write!(f, "REPL error: {}", message),
        }
    }
}

impl Display for Usage {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Error for ErrorKind {}

fn read_number() -> Result<u32, ErrorKind> {
    let mut input = String::new();
    match stdin().read_line(&mut input) {
        Ok(_) => match input.trim().parse::<u32>() {
            Ok(number) => Ok(number),
            Err(_) => Err(InvalidNumber),
        },
        Err(_) => Err(InvalidNumber),
    }
}

#[allow(dead_code)]
impl Parser {
    fn price(price: &str) -> Result<u64, ErrorKind> {
        let normalized_price = price.replace(",", ".");

        match normalized_price.parse::<f64>() {
            Ok(parsed_price) => {
                let price_in_cents = (parsed_price * 100.0).round() as u64;
                Ok(price_in_cents)
            }
            Err(_) => Err(InvalidPrice),
        }
    }

    fn optional_date(date_str: &str) -> Option<NaiveDate> {
        let formats = ["%Y-%m-%d", "%Y/%m/%d", "%Y.%m.%d", "%Y %m %d", "%Y%m%d", "%d-%m-%Y", "%d/%m/%Y", "%d.%m.%Y", "%d %m %Y", "%d%m%Y"];

        for format in formats.iter() {
            if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
                return Some(date);
            }
        }

        None
    }

    fn handle_args(args: Vec<String>, expected_args: usize) -> Result<Vec<String>, &'static str> {
        if args.is_empty() {
            return Err("No arguments provided.");
        }

        if expected_args >= 2 {
            match args[0].parse::<u32>() {
                Ok(_) => {
                    if args.len() >= expected_args {
                        return Ok(args);
                    } else {
                        return Err("Not enough arguments provided.");
                    }
                }
                Err(_) => {
                    if args.len() >= expected_args {
                        return Ok(args);
                    } else {
                        return Err("Not enough arguments provided.");
                    }
                }
            }
        }
        Err("Invalid command or arguments.")
    }
}

#[allow(dead_code)]
impl Prompt {
    fn id() -> Result<u32, ErrorKind> {
        println!("Enter the ID of the product:");
        read_number()
    }

    fn name() -> String {
        println!("Enter the name of the product:");
        let mut name = String::new();
        stdin().read_line(&mut name).unwrap();
        name.trim().to_string()
    }

    fn id_or_name() -> Result<String, ErrorKind> {
        println!("Enter the ID or name of the product:");
        let mut id_or_name = String::new();
        match stdin().read_line(&mut id_or_name) {
            Ok(_) => Ok(id_or_name.trim().to_string()),
            Err(_) => Err(InvalidIdOrName),
        }
    }

    fn quantity() -> Result<usize, ErrorKind> {
        println!("Enter the quantity of the product:");
        let mut quantity = String::new();
        match stdin().read_line(&mut quantity) {
            Ok(_) => match quantity.trim().parse::<usize>() {
                Ok(quantity) => Ok(quantity),
                Err(_) => Err(InvalidQuantity),
            },
            Err(_) => Err(InvalidQuantity),
        }
    }

    fn price() -> Result<u64, ErrorKind> {
        println!("Enter the price of the product:");
        let mut price = String::new();
        match stdin().read_line(&mut price) {
            Ok(_) => match Parser::price(price.trim()) {
                Ok(price) => Ok(price),
                Err(e) => Err(e),
            },
            Err(_) => Err(InvalidPrice),
        }
    }

    fn expiration_date() -> Option<NaiveDate> {
        println!("Enter the expiration date of the product (optional):");
        let mut expiration_date = String::new();
        match stdin().read_line(&mut expiration_date) {
            Ok(_) => Parser::optional_date(expiration_date.trim()),
            Err(_) => None,
        }
    }

    fn file_path() -> Option<String> {
        println!("Enter the file path for the storage (default: ./storage-<name>.json):");
        let mut file_path = String::new();
        if stdin().read_line(&mut file_path).is_ok() {
            if file_path.trim().is_empty() {
                None
            } else {
                Some(file_path.trim().to_string())
            }
        } else {
            None
        }
    }

    fn warehouse_creation(mut warehouse: Warehouse) -> Result<Warehouse, ErrorKind> {
        print!("Enter the number of rows in the warehouse:");
        stdout().flush().unwrap();
        let rows: usize = match read_number() {
            Ok(number) => number as usize,
            Err(_) => return Err(InvalidNumber),
        };

        print!("Enter the number of columns in each row of the warehouse:");
        stdout().flush().unwrap();
        let columns: usize = match read_number() {
            Ok(number) => number as usize,
            Err(_) => return Err(InvalidNumber),
        };

        print!("Enter the number of zones in each column of the warehouse:");
        stdout().flush().unwrap();
        let zones: usize = match read_number() {
            Ok(number) => number as usize,
            Err(_) => return Err(InvalidNumber),
        };

        warehouse.initialize_rows(rows, columns, zones);
        Ok(warehouse)
    }

    fn storage_load(storage: &mut Storage) -> Result<&mut Storage, ErrorKind> {
        match Prompt::file_path() {
            Some(file_path) => {
                let default_path_name = format!("./storage-{}.json", &file_path);
                let default_path = Path::new(&default_path_name);
                if !default_path.exists() {
                    match Storage::load(&file_path, storage) {
                        Ok(loaded) => Ok(loaded),
                        Err(e) => Err(ErrorKind::StorageError(e)),
                    }
                } else {
                    match Storage::load(&default_path_name, storage) {
                        Ok(loaded) => Ok(loaded),
                        Err(e) => Err(ErrorKind::StorageError(e)),
                    }
                }
            }
            None => Err(InvalidFile),
        }
    }

    fn storage_creation(storage: &mut Storage) -> Result<&mut Storage, ErrorKind> {
        println!("Enter the name of the storage:");
        let mut name = String::new();
        if stdin().read_line(&mut name).is_ok() {
            name = name.trim().to_string();
        }
        let file_path = Prompt::file_path();

        let warehouse = Warehouse::new();
        storage.file_path = file_path.unwrap_or(format!("./storage-{}.json", name));
        storage.name = name;
        storage.product_list = ProductList::new();
        match Prompt::warehouse_creation(warehouse) {
            Ok(warehouse) => {
                storage.warehouse = warehouse;
                Ok(storage)
            }
            Err(e) => Err(e),
        }
    }

    fn new_product(storage: &mut Storage) -> Result<(), ErrorKind> {
        let name = Prompt::name();
        match Prompt::price() {
            Ok(price) => match storage.new_product(name, price) {
                Ok(_) => Ok(()),
                Err(e) => Err(StorageError(e)),
            },
            Err(e) => Err(e),
        }
    }

    fn delete_product(storage: &mut Storage) -> Result<(), ErrorKind> {
        match Prompt::id_or_name() {
            Ok(id_or_name) => match id_or_name.parse::<u32>() {
                Ok(id) => match storage.delete_product_by_id(id) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(StorageError(e)),
                },
                Err(_) => match storage.delete_product_by_name(&id_or_name) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(StorageError(e)),
                },
            },
            Err(e) => Err(e),
        }
    }

    fn price_change(storage: &mut Storage) -> Result<(), ErrorKind> {
        match Prompt::id() {
            Ok(id) => match Prompt::price() {
                Ok(price) => match storage.change_price(id, price) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(StorageError(e)),
                },
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }

    fn restock_product(storage: &mut Storage) -> Result<(), ErrorKind> {
        match Prompt::id_or_name() {
            Ok(id_or_name) => match id_or_name.parse::<u32>() {
                Ok(id) => match Prompt::quantity() {
                    Ok(quantity) => match Prompt::expiration_date() {
                        Some(expiry) => match storage.restock_product(id, quantity, Some(expiry)) {
                            Ok(_) => Ok(()),
                            Err(e) => Err(StorageError(e)),
                        },
                        None => match storage.restock_product(id, quantity, None) {
                            Ok(_) => Ok(()),
                            Err(e) => Err(StorageError(e)),
                        },
                    },
                    Err(e) => Err(e),
                },
                Err(_) => match Prompt::quantity() {
                    Ok(quantity) => match Prompt::expiration_date() {
                        Some(expiry) => {
                            match storage.restock_by_name(&id_or_name, quantity, Some(expiry)) {
                                Ok(_) => Ok(()),
                                Err(e) => Err(StorageError(e)),
                            }
                        }
                        None => match storage.restock_by_name(&id_or_name, quantity, None) {
                            Ok(_) => Ok(()),
                            Err(e) => Err(StorageError(e)),
                        },
                    },
                    Err(e) => Err(e),
                },
            },
            Err(e) => Err(e),
        }
    }

    fn remove_stock(storage: &mut Storage) -> Result<(), ErrorKind> {
        match Prompt::id_or_name() {
            Ok(id_or_name) => match id_or_name.parse::<u32>() {
                Ok(id) => match Prompt::quantity() {
                    Ok(quantity) => match storage.remove_stock(id, quantity) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(StorageError(e)),
                    },
                    Err(e) => Err(e),
                },
                Err(_) => match Prompt::quantity() {
                    Ok(quantity) => match storage.remove_stock_by_name(&id_or_name, quantity) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(StorageError(e)),
                    },
                    Err(e) => Err(e),
                },
            },
            Err(e) => Err(e),
        }
    }

    fn empty_stock(storage: &mut Storage) -> Result<(), ErrorKind> {
        match Prompt::id_or_name() {
            Ok(id_or_name) => match id_or_name.parse::<u32>() {
                Ok(id) => match storage.empty_stock(id) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(StorageError(e)),
                },
                Err(_) => match storage.empty_stock_by_name(&id_or_name) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(StorageError(e)),
                },
            },
            Err(e) => Err(e),
        }
    }
}

fn add_product(storage: &mut Storage, args: &[String]) -> Result<(), ErrorKind> {
    match args.len() {
        2 => {
            let name = &args[0];
            match Parser::price(&args[1]) {
                Ok(price) => match storage.new_product(name.to_string(), price) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(StorageError(e)),
                },
                Err(_) => Err(InvalidPrice),
            }
        }
        0 => match Prompt::new_product(storage) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        },
        _ => Err(InvalidArguments(Usage::AddProduct)),
    }
}

fn delete_product(storage: &mut Storage, args: &[String]) -> Result<(), ErrorKind> {
    match args.len() {
        1 => match args[0].parse::<u32>() {
            Ok(id) => match storage.delete_product_by_id(id) {
                Ok(_) => Ok(()),
                Err(e) => Err(StorageError(e)),
            },
            Err(_) => match storage.delete_product_by_name(&args[0]) {
                Ok(_) => Ok(()),
                Err(e) => Err(StorageError(e)),
            },
        },
        0 => match Prompt::delete_product(storage) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        },
        _ => Err(InvalidArguments(Usage::DeleteProduct)),
    }
}

fn restock_product(storage: &mut Storage, args: &[String]) -> Result<(), ErrorKind> {
    match args.len() {
        2 => match args[0].parse::<u32>() {
            Ok(id) => match args[1].parse::<usize>() {
                Ok(quantity) => match storage.restock_product(id, quantity, None) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(StorageError(e)),
                },
                Err(_) => Err(InvalidQuantity),
            },
            Err(_) => match storage.restock_by_name(&args[0], 1, None) {
                Ok(_) => Ok(()),
                Err(e) => Err(StorageError(e)),
            },
        },
        3 => match args[0].parse::<u32>() {
            Ok(id) => match args[1].parse::<usize>() {
                Ok(quantity) => match Parser::optional_date(&args[2]) {
                    Some(expiry) => match storage.restock_product(id, quantity, Some(expiry)) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(StorageError(e)),
                    },
                    None => Err(InvalidDate),
                },
                Err(_) => Err(InvalidQuantity),
            },
            Err(_) => match args[2].parse::<usize>() {
                Ok(quantity) => match Parser::optional_date(&args[1]) {
                    Some(expiry) => {
                        match storage.restock_by_name(&args[0], quantity, Some(expiry)) {
                            Ok(_) => Ok(()),
                            Err(e) => Err(StorageError(e)),
                        }
                    }
                    None => Err(InvalidDate),
                },
                Err(_) => Err(InvalidQuantity),
            },
        },
        0 => match Prompt::restock_product(storage) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        },
        _ => Err(InvalidArguments(Usage::RestockProduct)),
    }
}

fn remove_stock(storage: &mut Storage, args: &[String]) -> Result<(), ErrorKind> {
    match args.len() {
        1 => match args[0].parse::<u32>() {
            Ok(id) => match storage.remove_stock(id, 1) {
                Ok(_) => Ok(()),
                Err(e) => Err(StorageError(e)),
            },
            Err(_) => match storage.remove_stock_by_name(&args[0], 1) {
                Ok(_) => Ok(()),
                Err(e) => Err(StorageError(e)),
            },
        },
        2 => match args[0].parse::<u32>() {
            Ok(id) => match args[1].parse::<usize>() {
                Ok(quantity) => match storage.remove_stock(id, quantity) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(StorageError(e)),
                },
                Err(_) => Err(InvalidQuantity),
            },
            Err(_) => match args[1].parse::<usize>() {
                Ok(quantity) => match storage.remove_stock_by_name(&args[0], quantity) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(StorageError(e)),
                },
                Err(_) => Err(InvalidQuantity),
            },
        },
        0 => match Prompt::remove_stock(storage) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        },
        _ => Err(InvalidArguments(Usage::RemoveStock)),
    }
}

fn empty_stock(storage: &mut Storage, args: &[String]) -> Result<(), ErrorKind> {
    match args.len() {
        1 => match args[0].parse::<u32>() {
            Ok(id) => match storage.empty_stock(id) {
                Ok(_) => Ok(()),
                Err(e) => Err(StorageError(e)),
            },
            Err(_) => match storage.empty_stock_by_name(&args[0]) {
                Ok(_) => Ok(()),
                Err(e) => Err(StorageError(e)),
            },
        },
        0 => match Prompt::empty_stock(storage) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        },
        _ => Err(InvalidArguments(Usage::EmptyStock)),
    }
}

fn save_storage(storage: &Storage) -> Result<(), ErrorKind> {
    match storage.save() {
        Ok(_) => Ok(()),
        Err(_) => Err(CouldNotSaveStorage),
    }
}

fn resolve_storage_command(command: &str) -> &str {
    match command {
        "add" => "add_product",
        "delete" => "delete_product",
        "del" => "delete_product",
        "restock" => "restock_product",
        "remove" => "remove_stock",
        "empty" => "empty_stock",
        "ls" => "list_products",
        _ => command,
    }
}

fn resolve_intro_command(command: &str) -> &str {
    match command {
        "create" => "create_storage",
        "load" => "load_storage",
        _ => command,
    }
}

fn confirm_exit() -> bool {
    println!("Are you sure you want to exit? (y/n)");
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    input.trim().eq_ignore_ascii_case("y")
}

fn intro_repl() -> Result<(), ErrorKind> {
    println!("Welcome to the storage management system");
    loop {
        print!("> ");
        stdout().flush().unwrap();
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let parts = input.trim().split(' ').collect::<Vec<&str>>();
        let command = parts[0];
        let mut storage = Storage::new("default".to_string(), None);

        match resolve_intro_command(command) {
            "load_storage" => match Prompt::storage_load(&mut storage) {
                Ok(loaded) => return storage_repl(loaded),
                Err(e) => {
                    eprintln!("{}", e);
                    continue;
                }
            },
            "create_storage" => match Prompt::storage_creation(&mut storage) {
                Ok(created) => return storage_repl(created),
                Err(e) => {
                    eprintln!("{}", e);
                    continue;
                }
            },
            "exit" => break,
            _ => println!("Invalid command"),
        }
    }
    Ok(())
}

fn read_user_input() -> (String, Vec<String>) {
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    let parts = input.trim().split(' ').collect::<Vec<&str>>();
    let command = parts[0].to_string();
    let args = parts[1..].iter().map(|s| s.to_string()).collect();
    (command, args)
}

fn storage_repl(storage: &mut Storage) -> Result<(), ErrorKind> {
    loop {
        print!("{} > ", storage.name);
        stdout().flush().unwrap();
        let (command, args) = read_user_input();
        match resolve_storage_command(&command) {
            "add_product" => match add_product(storage, &args) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    continue;
                }
            },
            "delete_product" => match delete_product(storage, &args) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    continue;
                }
            },
            "restock_product" => match restock_product(storage, &args) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    continue;
                }
            },
            "remove_stock" => match remove_stock(storage, &args) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    continue;
                }
            },
            "empty_stock" => match empty_stock(storage, &args) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    continue;
                }
            },
            "save" => match save_storage(storage) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    continue;
                }
            },
            "list_products" => storage.list_products(),
            "help" => print_storage_help(),
            "exit" => {
                if confirm_exit() {
                    match save_storage(storage) {
                        Ok(_) => break,
                        Err(e) => {
                            eprintln!("{}", e);
                            continue;
                        }
                    }
                }
            }
            "force_exit" => {
                if confirm_exit() {
                    break;
                }
            }
            _ => println!("Invalid command"),
        }
    }
    Ok(())
}

fn print_storage_help() {
    println!("Commands:");
    println!("  add_product <name> <price>");
    println!("  delete_product <id>");
    println!("  restock_product <id> <quantity> [expiration_date]");
    println!("  remove_stock <id> [quantity]");
    println!("  empty_stock <id>");
    println!("  list_products");
    println!("  save");
    println!("  exit (save and exit)");
    println!("  force_exit (exit without saving)");
}

#[allow(dead_code)]
fn print_intro_help() {
    println!("Commands:");
    println!("  create_storage");
    println!("  load_storage");
    println!("  exit");
}

fn resolve_arg(arg: &str) -> &str {
    match arg {
        "load" => "load_storage",
        "add" => "add_product",
        "delete" => "delete_product",
        "del" => "delete_product",
        "restock" => "restock_product",
        "remove" => "remove_stock",
        "empty" => "empty_stock",
        "ls" => "list_products",
        _ => arg,
    }
}

pub fn run(args: Vec<String>) -> Result<(), ErrorKind> {
    let mut storage = Storage::new("default".to_string(), None);

    if args.len() > 1 {
        let command = &args[1];
        let load = {
            let path: &str = &args[2];
            match Storage::load(path, &mut storage) {
                Ok(loaded) => Ok(loaded),
                Err(_) => Err(CouldNotLoadStorage),
            }
        };
        match resolve_arg(command.as_str()) {
            "load_storage" => match args.len() {
                3 => match load {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                },
                _ => Err(InvalidArguments(Usage::Storage)),
            },
            "create_storage" => match Prompt::storage_creation(&mut storage) {
                Ok(created) => match storage_repl(created) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            },
            _ => Err(InvalidCommand),
        }
    } else {
        match intro_repl() {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
