use chrono::NaiveDate;
use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufReader, Write},
};

use crate::{
    product::{ErrorKind as ProductError, Product, ProductList},
    warehouse::{ErrorKind as WarehouseError, Warehouse},
};
use serde::{Deserialize, Serialize};
use serde_json;

use ErrorKind::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Storage {
    pub name: String,
    pub product_list: ProductList,
    pub file_path: String,
    pub warehouse: Warehouse,
}

#[derive(Debug)]
pub enum ErrorKind {
    Io(io::Error),
    ProductNotFound,
    HasStock,
    WarehouseError(WarehouseError),
    ProductError(ProductError),
}

impl From<ErrorKind> for std::io::Error {
    fn from(e: ErrorKind) -> Self {
        match e {
            Io(e) => Io(e).into(),
            ProductNotFound => ProductNotFound.into(),
            HasStock => HasStock.into(),
            WarehouseError(e) => WarehouseError(e).into(),
            ProductError(e) => ProductError(e).into(),
        }
    }
}

impl From<std::io::Error> for ErrorKind {
    fn from(e: std::io::Error) -> Self {
        Io(e)
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let message = self.detailed_message();
        match self {
            WarehouseError(_) => write!(f, "Warehouse error:{}", message),
            ProductError(_) => write!(f, "Product error: {}", message),
            Io(_) => write!(f, "I/O error: {}", message),
            _ => write!(f, "Storage error: {}", message),
        }
    }
}

impl ErrorKind {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Io(_) => "I/O Error",
            ProductNotFound => "Product Not Found",
            HasStock => "Product has stock",
            WarehouseError(_) => "Warehouse Error",
            ProductError(_) => "Product Error",
        }
    }

    pub(crate) fn detailed_message(&self) -> String {
        match self {
            Io(e) => format!("{}", e),
            WarehouseError(e) => format!("{}", e),
            ProductError(e) => format!("{}", e),
            _ => self.as_str().to_string(),
        }
    }
}

#[allow(dead_code)]
impl Storage {
    pub fn new(name: String, file_path: Option<String>) -> Self {
        let default_path = format!("./storage_{}.json", name);
        Storage {
            name,
            product_list: ProductList::new(),
            warehouse: Warehouse::new(),
            file_path: file_path.unwrap_or(default_path),
        }
    }

    pub fn save(&self) -> io::Result<()> {
        match File::create(&self.file_path) {
            Ok(mut file) => match serde_json::to_string_pretty(self) {
                    Ok(json) => file.write_all(json.as_bytes()),
                    Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
                },
            Err(e) => Err(e),
        }
    }

    pub fn load<'a>(
        file_path: &str,
        storage: &'a mut Storage,
    ) -> Result<&'a mut Storage, ErrorKind> {
        let path = file_path;
        match File::open(path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                match serde_json::from_reader::<BufReader::<File>, Storage>(reader) {
                    Ok(new_storage) => {
                        storage.name = new_storage.name;
                        storage.product_list = new_storage.product_list;
                        storage.warehouse = new_storage.warehouse;
                        storage.file_path = new_storage.file_path;

                        Ok(storage)
                    }
                    Err(e) => Err(Io(e.into())),
                }
            }
            Err(e) => Err(Io(e)),
        }
    }

    pub fn check_capacity(&self) -> usize {
        self.warehouse.capacity
    }

    pub fn check_available_space(&self) -> usize {
        self.warehouse.available_space
    }

    pub fn list_products(&self) {
        for product in self.product_list.products.values() {
            println!("{}", product);
        }
    }

    pub fn new_product(&mut self, name: String, price: u64) -> Result<(), ErrorKind> {
        let id = self.product_list.products.len() as u32 + 1;
        let product = Product::new(id, name.clone(), price, 0);
        match self.product_list.add(product) {
            Ok(_) => Ok(()),
            Err(e) => Err(ProductError(e)),
        }
    }

    pub fn find_product_id(&self, name: &str) -> Option<u32> {
        for product in self.product_list.products.values() {
            if product.name == name {
                return Some(product.id);
            }
        }
        None
    }

    pub fn get_product_by_id(&self, id: u32) -> Option<&str> {
        match self.product_list.products.get(&id) {
            Some(product) => Some(product.name.as_str()),
            None => None,
        }
    }

    pub fn delete_product_by_id(&mut self, id: u32) -> Result<(), ErrorKind> {
        let product = self.product_list.products.get(&id);
        if let Some(product) = product {
            if product.quantity > 0 {
                Err(HasStock)
            } else {
                self.product_list.products.remove(&id);
                println!("Product {} removed", id);
                Ok(())
            }
        } else {
            Err(ProductNotFound)
        }
    }

    pub fn delete_product_by_name(&mut self, name: &str) -> Result<(), ErrorKind> {
        match self.find_product_id(name) {
            Some(id) => match self.delete_product_by_id(id) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            None => Err(ProductNotFound),
        }
    }

    pub fn restock_product(
        &mut self,
        id: u32,
        quantity: usize,
        expiration_date: Option<NaiveDate>,
    ) -> Result<(), ErrorKind> {
        if self.product_list.products.contains_key(&id) {
            match self
                .warehouse
                .add_items_by_qty(id, quantity, expiration_date)
            {
                Ok(_) => match self.product_list.products.get_mut(&id) {
                    Some(product) => {
                        product.quantity += quantity;
                        Ok(())
                    }
                    None => Err(ProductNotFound),
                },
                Err(e) => Err(WarehouseError(e)),
            }
        } else {
            Err(ProductNotFound)
        }
    }

    pub fn restock_by_name(
        &mut self,
        name: &str,
        quantity: usize,
        expiration_date: Option<NaiveDate>,
    ) -> Result<(), ErrorKind> {
        match self.find_product_id(name) {
            Some(id) => match self.restock_product(id, quantity, expiration_date) {
                Ok(_) => match self.product_list.products.get_mut(&id) {
                    Some(product) => {
                        product.quantity += quantity;
                        Ok(())
                    }
                    None => Err(ProductNotFound),
                },
                Err(e) => Err(e),
            },
            None => Err(ProductNotFound),
        }
    }

    pub fn change_price(&mut self, id: u32, price: u64) -> Result<(), ErrorKind> {
        let current_price = self.product_list.products.get(&id).unwrap().price;
        if let Some(product) = self.product_list.products.get_mut(&id) {
            product.set_price(price);
            println!(
                "Price for product {} changed from {} to {}",
                id, current_price, price
            );
            Ok(())
        } else {
            Err(ProductNotFound)
        }
    }

    pub fn change_price_by_name(&mut self, name: &str, price: u64) -> Result<(), ErrorKind> {
        match self.find_product_id(name) {
            Some(id) => self.change_price(id, price),
            None => Err(ProductNotFound),
        }
    }

    pub fn remove_stock(&mut self, id: u32, quantity: usize) -> Result<(), ErrorKind> {
        match self.product_list.products.get(&id) {
            Some(_) => match self.warehouse.remove_item_by_qty(id, quantity) {
                Ok(_) => match self.product_list.products.get_mut(&id) {
                    Some(product) => {
                        product.quantity -= quantity;
                        Ok(())
                    }
                    None => Err(ProductNotFound),
                },
                Err(e) => Err(WarehouseError(e)),
            },
            None => Err(ProductNotFound),
        }
    }

    pub fn remove_stock_by_name(&mut self, name: &str, quantity: usize) -> Result<(), ErrorKind> {
        match self.find_product_id(name) {
            Some(id) => self.remove_stock(id, quantity),
            None => Err(ProductNotFound),
        }
    }

    pub fn empty_stock(&mut self, id: u32) -> Result<(), ErrorKind> {
        match self.product_list.products.get(&id) {
            Some(_) => match self.warehouse.remove_all_items(id) {
                Ok(_) => match self.product_list.products.get_mut(&id) {
                    Some(product) => {
                        product.quantity = 0;
                        Ok(())
                    }
                    None => Err(ProductNotFound),
                },
                Err(e) => Err(WarehouseError(e)),
            },
            None => Err(ProductNotFound),
        }
    }

    pub fn empty_stock_by_name(&mut self, name: &str) -> Result<(), ErrorKind> {
        match self.find_product_id(name) {
            Some(id) => match self.empty_stock(id) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            None => Err(ProductNotFound),
        }
    }
}
