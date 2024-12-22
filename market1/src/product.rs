use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Display, Formatter},
    io,
};
use ErrorKind::*;

#[derive(Debug, Clone, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Product {
    pub id: u32,
    pub name: String,
    pub price: u64,
    pub quantity: usize,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProductItem {
    pub id: u32,
    #[serde(skip_deserializing)]
    pub row: usize,
    #[serde(skip_deserializing)]
    pub column: usize,
    #[serde(skip_deserializing)]
    pub zone: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductList {
    pub products: HashMap<u32, Product>,
}

impl Display for Product {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let price = format_price(self.price);
        write!(
            f,
            "Product: {}\n ID: {}, Price: {}, Quantity: {}",
            self.name, self.id, price, self.quantity,
        )
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    NotEnoughQuantity,
    ProductNotFound,
    IDExists,
    NameExists,
    InvalidInput,
}

impl ErrorKind {
    pub fn as_str(&self) -> &str {
        match self {
            NotEnoughQuantity => "Not enough quantity",
            ProductNotFound => "Product not found",
            IDExists => "ID already exists",
            NameExists => "Product with this name already exists",
            InvalidInput => "Invalid input",
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<ErrorKind> for io::Error {
    fn from(e: ErrorKind) -> Self {
        match e {
            NotEnoughQuantity => NotEnoughQuantity.into(),
            ProductNotFound => ProductNotFound.into(),
            IDExists => IDExists.into(),
            NameExists => NameExists.into(),
            InvalidInput => InvalidInput.into(),
        }
    }
}

impl Error for ErrorKind {}



fn format_price(price: u64) -> String {
    let numeral = price / 100;
    let decimal = price % 100;

    format!("${}.{}", numeral, format_args!("{:02}", decimal))
}

#[allow(dead_code)]
impl Product {
    pub fn new(id: u32, name: String, price: u64, quantity: usize) -> Self {
        Product {
            id,
            name,
            price,
            quantity,
        }
    }

    pub fn add_quantity(&mut self, quantity: usize) {
        self.quantity += quantity;
    }

    pub fn remove_quantity(&mut self, quantity: usize) -> Result<(), ErrorKind> {
        match self.quantity >= quantity {
            true => {
                self.quantity -= quantity;
                Ok(())
            }
            false => Err(NotEnoughQuantity),
        }
    }

    pub fn set_price(&mut self, price: u64) {
        self.price = price;
    }

    pub fn print_price(&self) {
        println!("Price: {}", format_price(self.price));
    }
}

impl Display for ProductItem {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let date = match self.expiry_date {
            Some(date) => date.to_string(),
            None => "N/A".to_string(),
        };
        write!(f, "Product ID: {}, Expiry Date: {}", self.id, date)
    }
}

#[allow(dead_code)]
impl ProductItem {
    pub fn new(
        product_id: u32,
        row: usize,
        column: usize,
        zone: usize,
        expiry_date: Option<NaiveDate>,
    ) -> Self {
        ProductItem {
            id: product_id,
            row,
            column,
            zone,
            expiry_date,
        }
    }

    pub fn position(&self) -> (usize, usize, usize) {
        (self.row, self.column, self.zone)
    }

    pub fn expiry(&self) -> Option<NaiveDate> {
        self.expiry_date
    }

    pub fn place(&mut self, row: usize, column: usize, zone: usize) {
        self.row = row;
        self.column = column;
        self.zone = zone;
    }

    pub fn set_expiration(&mut self, expiry_date: Option<NaiveDate>) {
        self.expiry_date = expiry_date;
    }

    pub fn copy_at_zone(&self, row: usize, column: usize, zone: usize) -> ProductItem {
        let expry_date_copy = self.expiry_date;
        ProductItem {
            id: self.id,
            row,
            column,
            zone,
            expiry_date: expry_date_copy,
        }
    }
}

#[allow(dead_code)]
impl ProductList {
    pub fn new() -> Self {
        ProductList {
            products: HashMap::new(),
        }
    }

    pub fn add(&mut self, product: Product) -> Result<(), ErrorKind> {
        let id = &product.id;
        if self.products.contains_key(id) {
            return Err(IDExists);
        }
        if self.products.values().any(|p| p.name == product.name) {
            return Err(NameExists);
        }
        println!("Product {} added", id);
        self.products.insert(product.id, product);
        Ok(())
    }

    pub fn remove_by_id(&mut self, id: u32) -> Result<(), ErrorKind> {
        if self.products.remove(&id).is_some() {
            println!("Product {} removed", id);
            Ok(())
        } else {
            Err(ProductNotFound)
        }
    }

    pub fn remove_by_name(&mut self, name: &str) -> Result<(), ErrorKind> {
        let id = match self.products.values().find(|p| p.name == name) {
            Some(product) => product.id,
            None => return Err(ProductNotFound),
        };
        match self.remove_by_id(id) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn get(&self, id: u32) -> Option<&Product> {
        self.products.get(&id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut Product> {
        self.products.get_mut(&id)
    }

    pub fn list(&self) {
        for product in self.products.values() {
            println!("{}", product);
        }
    }
}

impl Default for ProductList {
    fn default() -> Self {
        Self::new()
    }
}
