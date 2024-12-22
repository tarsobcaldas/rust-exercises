use crate::product::ProductItem;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};
use ErrorKind::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    pub zone_number: usize,
    pub column_number: usize,
    pub row_number: usize,
    pub item: Option<ProductItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub column_number: usize,
    pub row_number: usize,
    pub capacity: usize,
    pub available_space: usize,
    pub zones: Vec<Zone>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub row_number: usize,
    pub column_count: usize,
    pub capacity: usize,
    pub available_space: usize,
    pub columns: Vec<Column>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warehouse {
    pub row_count: usize,
    pub column_count: usize,
    pub capacity: usize,
    pub available_space: usize,
    pub rows: Vec<Row>,
}

#[derive(Debug)]
pub enum ErrorKind {
    InsufficientSpace,
    NoContiguousSpace,
    NoProductFound,
    ZoneNotFound((usize, usize, usize)),
    ZoneOccupied((usize, usize, usize)),
    ZoneEmpty((usize, usize, usize)),
    ColumnNotFound((usize, usize)),
    RowNotFound(usize),
}

impl ErrorKind {
    pub(crate) fn as_str(&self) -> &'static str {
        match *self {
            InsufficientSpace => "Insufficient space",
            NoContiguousSpace => "No contiguous space available to add in bulk. Please organize items first, or add them individually.",
            ZoneNotFound(_) => "Zone not found",
            ZoneOccupied(_) => "Zone is already occupied",
            ZoneEmpty(_) => "Zone is empty",
            ColumnNotFound(_) => "Column not found",
            RowNotFound(_) => "Row not found",
            NoProductFound => "No product found",
        }
    }

    pub(crate) fn detailed_message(&self) -> String {
        match *self {
            ZoneNotFound((r, c, z)) => format!("Zone {} not found in column {} of row {}", z, c, r),
            ZoneOccupied((r, c, z)) => format!("Zone {} in column {} of row {} is already occupied", z, c, r),
            ZoneEmpty((r, c, z)) => format!("Zone {} in column {} of row {} is empty", z, c, r),
            ColumnNotFound((r, c)) => format!("Column {} in row {} not found", c, r),
            RowNotFound(r) => format!("Row {} not found", r),
            _ => self.as_str().to_string(),
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.detailed_message())
    }
}

#[allow(unused_must_use)]
impl Display for Zone {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let zone_id = format!(
            "{}.{}.{}",
            self.column_number, self.row_number, self.zone_number
        );
        if let Some(product) = &self.item {
            if product.expiry_date.is_some() {
                let date = product.expiry_date.unwrap().to_string();
                write!(
                    f,
                    "Zone: {}, Product ID: {}, Expiry Date: {}",
                    zone_id, product.id, date
                )
            } else {
                write!(
                    f,
                    "Zone: {}, Product ID: {}, Expiry Date: N/A",
                    zone_id, product.id
                )
            }
        } else {
            write!(f, "Zone: {}, Empty", zone_id)
        }
    }
}

#[allow(unused_must_use)]
impl Display for Column {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let col_id = format!("{}.{}", self.row_number, self.column_number);
        writeln!(
            f,
            "Column {}, Available Space: {}/{}",
            col_id, self.available_space, self.capacity
        );
        for zone in &self.zones {
            writeln!(f, "\t{}", zone);
        }
        Ok(())
    }
}

#[allow(unused_must_use)]
impl Display for Row {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(
            f,
            "Row {}, Available Space: {}/{}",
            self.row_number, self.available_space, self.capacity
        );
        for column in &self.columns {
            writeln!(f, "\t{}", column);
        }
        Ok(())
    }
}

impl Zone {
    pub fn new(
        zone_number: usize,
        column_number: usize,
        row_number: usize,
        item: Option<ProductItem>,
    ) -> Self {
        Zone {
            column_number,
            row_number,
            zone_number,
            item,
        }
    }

    pub fn add(&mut self, item: ProductItem) -> Result<(), ErrorKind> {
        if self.item.is_some() {
            return Err(ZoneOccupied((
                self.row_number,
                self.column_number,
                self.zone_number,
            )));
        }
        self.item = Some(item);
        Ok(())
    }

    pub fn remove(&mut self) -> Result<(), ErrorKind> {
        if self.item.is_none() {
            return Err(ZoneEmpty((
                self.row_number,
                self.column_number,
                self.zone_number,
            )));
        }
        self.item = None;
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.item.is_none()
    }

    pub fn pos(&self) -> (usize, usize, usize) {
        (self.row_number, self.column_number, self.zone_number)
    }
}

#[allow(dead_code)]
impl Column {
    pub fn new(column_number: usize, row_number: usize) -> Self {
        Column {
            column_number,
            row_number,
            capacity: 0,
            zones: Vec::new(),
            available_space: 0,
        }
    }

    pub fn add_zone(&mut self, zone: Zone) {
        self.zones.push(zone);
        self.available_space += 1;
        self.capacity += 1;
    }

    pub fn remove_zone(&mut self, zone_number: usize) -> Result<(), ErrorKind> {
        if let Some(zone) = self
            .zones
            .iter()
            .position(|zone| zone.zone_number == zone_number)
        {
            self.zones.remove(zone);
            self.available_space -= 1;
            self.capacity -= 1;
            Ok(())
        } else {
            Err(ZoneNotFound((
                self.row_number,
                self.column_number,
                zone_number,
            )))
        }
    }

    pub fn zone(&self, zone_number: usize) -> Option<&Zone> {
        self.zones
            .iter()
            .find(|zone| zone.zone_number == zone_number)
    }

    pub fn zone_mut(&mut self, zone_number: usize) -> Option<&mut Zone> {
        self.zones
            .iter_mut()
            .find(|zone| zone.zone_number == zone_number)
    }

    pub fn empty_zones(&self) -> Vec<&Zone> {
        self.zones
            .iter()
            .filter(|zone| zone.item.is_none())
            .collect()
    }

    pub fn empty_zones_mut(&mut self) -> Vec<&mut Zone> {
        self.zones
            .iter_mut()
            .filter(|zone| zone.item.is_none())
            .collect()
    }

    pub fn occupied_zones(&self) -> Vec<&Zone> {
        self.zones
            .iter()
            .filter(|zone| zone.item.is_some())
            .collect()
    }

    pub fn occupied_zones_mut(&mut self) -> Vec<&mut Zone> {
        self.zones
            .iter_mut()
            .filter(|zone| zone.item.is_some())
            .collect()
    }

    pub fn flat_map(&self) -> String {
        self.zones
            .iter()
            .map(|zone| if zone.is_empty() { "0" } else { "1" })
            .collect()
    }

    pub fn list(&self) {
        for zone in &self.zones {
            println!("{}", zone);
        }
    }

    pub fn is_full(&self) -> bool {
        self.available_space == 0
    }

    pub fn initialize_zones(&mut self, zone_count: usize) {
        for i in 1..=zone_count {
            let zone = Zone::new(i, self.column_number, self.row_number, None);
            self.add_zone(zone);
        }
    }

    pub fn add_item(&mut self, zone_number: usize, item: ProductItem) -> Result<(), ErrorKind> {
        if let Some(zone) = self.zone_mut(zone_number) {
            match zone.add(item) {
                Ok(_) => {
                    self.available_space -= 1;
                    Ok(())
                }
                Err(e) => Err(e),
            }
        } else {
            Err(ZoneNotFound((
                self.row_number,
                self.column_number,
                zone_number,
            )))
        }
    }

    pub fn remove_item(&mut self, zone_number: usize) -> Result<(), ErrorKind> {
        if let Some(zone) = self.zone_mut(zone_number) {
            match zone.remove() {
                Ok(_) => {
                    self.available_space += 1;
                    Ok(())
                }
                Err(e) => Err(e),
            }
        } else {
            Err(ZoneNotFound((
                self.row_number,
                self.column_number,
                zone_number,
            )))
        }
    }

    pub fn get_item(&self, zone_number: usize) -> Option<&ProductItem> {
        if let Some(zone) = self.zone(zone_number) {
            return zone.item.as_ref();
        }
        None
    }

    pub fn get_item_mut(&mut self, zone_number: usize) -> Option<&mut ProductItem> {
        if let Some(zone) = self.zone_mut(zone_number) {
            return zone.item.as_mut();
        }
        None
    }

    pub fn contains_product(&self, product_id: u32) -> bool {
        self.zones.iter().any(|zone| {
            if let Some(item) = &zone.item {
                item.id == product_id
            } else {
                false
            }
        })
    }

    pub fn find_item(&self, product_id: u32) -> Option<usize> {
        self.zones.iter().position(|zone| {
            if let Some(item) = &zone.item {
                item.id == product_id
            } else {
                false
            }
        })
    }

    pub fn find_last_item_occurrence_index(&self, product_id: u32) -> Option<usize> {
        self.zones.iter().rposition(|zone| {
            if let Some(item) = &zone.item {
                item.id == product_id
            } else {
                false
            }
        })
    }

    pub fn find_all_item_occurences(&self, product_id: u32) -> Vec<usize> {
        self.zones
            .iter()
            .enumerate()
            .filter_map(|(i, zone)| {
                if let Some(item) = &zone.item {
                    if item.id == product_id {
                        Some(i)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }
}

#[allow(dead_code)]
impl Row {
    pub fn new(row_number: usize) -> Self {
        Row {
            row_number,
            column_count: 0,
            capacity: 0,
            available_space: 0,
            columns: Vec::new(),
        }
    }

    pub fn add_column(&mut self, column: Column) {
        self.column_count += 1;
        self.capacity += column.capacity;
        self.available_space += column.available_space;
        self.columns.push(column);
    }

    pub fn remove_column(&mut self, column_number: usize) -> Result<(), ErrorKind> {
        if let Some(column) = self
            .columns
            .iter()
            .position(|col| col.column_number == column_number)
        {
            self.columns.remove(column);
            self.column_count -= 1;
            self.capacity -= self.columns[column].capacity;
            self.available_space -= self.columns[column].available_space;
            Ok(())
        } else {
            Err(ColumnNotFound((
                self.row_number,
                column_number,
            )))
        }
    }

    pub fn zone(&self, column_number: usize, zone_number: usize) -> Option<&Zone> {
        if let Some(column) = self.column(column_number) {
            return column.zone(zone_number);
        }
        None
    }

    pub fn zone_mut(&mut self, column_number: usize, zone_number: usize) -> Option<&mut Zone> {
        if let Some(column) = self.column_mut(column_number) {
            return column.zone_mut(zone_number);
        }
        None
    }

    pub fn column(&self, column_number: usize) -> Option<&Column> {
        self.columns
            .iter()
            .find(|col| col.column_number == column_number)
    }

    pub fn column_mut(&mut self, column_number: usize) -> Option<&mut Column> {
        self.columns
            .iter_mut()
            .find(|col| col.column_number == column_number)
    }

    pub fn empty_columns(&self) -> Vec<&Column> {
        self.columns
            .iter()
            .filter(|col| col.available_space > 0)
            .collect()
    }

    pub fn empty_columns_mut(&mut self) -> Vec<&mut Column> {
        self.columns
            .iter_mut()
            .filter(|col| col.available_space > 0)
            .collect()
    }

    pub fn occupied_columns_mut(&mut self) -> Vec<(&mut Column, usize)> {
        self.columns
            .iter_mut()
            .filter(|col| col.available_space == 0)
            .map(|col| {
                let available_space = col.available_space;
                (col, available_space)
            })
            .collect()
    }

    pub fn flat_map(&self) -> String {
        self.columns
            .iter()
            .map(|col| col.flat_map())
            .collect::<Vec<String>>()
            .join("")
    }

    pub fn contains_product(&self, product_id: u32) -> bool {
        self.columns
            .iter()
            .any(|col| col.contains_product(product_id))
    }

    pub fn is_full(&self) -> bool {
        self.available_space == 0
    }

    pub fn initialize_columns(&mut self, column_count: usize, zone_per_col: usize) {
        for i in 1..=column_count {
            let mut column = Column::new(i, self.row_number);
            column.initialize_zones(zone_per_col);
            self.add_column(column);
        }
    }

    pub fn add_item(
        &mut self,
        column_number: usize,
        zone_number: usize,
        item: ProductItem,
    ) -> Result<(), ErrorKind> {
        if let Some(column) = self.column_mut(column_number) {
            match column.add_item(zone_number, item) {
                Ok(_) => {
                    self.available_space -= 1;
                    Ok(())
                }
                Err(e) => Err(e),
            }
        } else {
            Err(ColumnNotFound((
                self.row_number,
                column_number,
            )))
        }
    }

    pub fn remove_item(
        &mut self,
        column_number: usize,
        zone_number: usize,
    ) -> Result<(), ErrorKind> {
        if let Some(column) = self.column_mut(column_number) {
            match column.remove_item(zone_number) {
                Ok(_) => {
                    self.available_space += 1;
                    Ok(())
                }
                Err(e) => Err(e),
            }
        } else {
            Err(ColumnNotFound((
                self.row_number,
                column_number,
            )))
        }
    }

    pub fn item(&self, column_number: usize, zone_number: usize) -> Option<&ProductItem> {
        if let Some(column) = self.column(column_number) {
            return column.get_item(zone_number);
        }
        None
    }

    pub fn item_mut(
        &mut self,
        column_number: usize,
        zone_number: usize,
    ) -> Option<&mut ProductItem> {
        if let Some(column) = self.column_mut(column_number) {
            return column.get_item_mut(zone_number);
        }
        None
    }

    pub fn find_item(&self, product_id: u32) -> Option<(usize, usize)> {
        if let Some(column) = self
            .columns
            .iter()
            .find(|col| col.contains_product(product_id))
        {
            let col_index = column.column_number;
            if let Some(zone_index) = column.find_item(product_id) {
                return Some((col_index, zone_index));
            }
        }
        None
    }

    pub fn find_last_item_occurrence(&self, product_id: u32) -> Option<(usize, usize)> {
        if let Some(column) = self
            .columns
            .iter()
            .find(|col| col.contains_product(product_id))
        {
            let col_index = column.column_number;
            if let Some(zone_index) = column.find_last_item_occurrence_index(product_id) {
                return Some((col_index, zone_index));
            }
        }
        None
    }

    pub fn find_all_item_occurences(&self, product_id: u32) -> Vec<(usize, usize)> {
        let mut items = Vec::new();
        for (col_index, column) in self.columns.iter().enumerate() {
            items.extend(
                column
                    .find_all_item_occurences(product_id)
                    .iter()
                    .map(|zone_index| (col_index, *zone_index)),
            );
        }
        items
    }
}

#[allow(dead_code)]
impl Warehouse {
    pub fn new() -> Self {
        Warehouse {
            row_count: 0,
            column_count: 0,
            capacity: 0,
            available_space: 0,
            rows: Vec::new(),
        }
    }

    pub fn add_row(&mut self, row: Row) {
        self.row_count += 1;
        self.capacity += row.capacity;
        self.available_space += row.available_space;
        self.column_count += row.columns.len();
        self.rows.push(row);
    }

    pub fn remove_row(&mut self, row_number: usize) -> Result<(), ErrorKind> {
        if let Some(row_index) = self.rows.iter().position(|r| r.row_number == row_number) {
            let row = &self.rows[row_index];
            self.capacity -= row.capacity;
            self.available_space -= row.available_space;
            self.rows.remove(row_index);
            self.row_count -= 1;
            Ok(())
        } else {
            Err(RowNotFound(row_number))
        }
    }

    pub fn zone(
        &self,
        row_number: usize,
        column_number: usize,
        zone_number: usize,
    ) -> Option<&Zone> {
        if let Some(row) = self.row(row_number) {
            return row.zone(column_number, zone_number);
        }
        None
    }

    pub fn zone_mut(
        &mut self,
        row_number: usize,
        column_number: usize,
        zone_number: usize,
    ) -> Option<&mut Zone> {
        if let Some(row) = self.row_mut(row_number) {
            return row.zone_mut(column_number, zone_number);
        }
        None
    }

    pub fn row(&self, row_number: usize) -> Option<&Row> {
        self.rows.iter().find(|r| r.row_number == row_number)
    }

    pub fn row_mut(&mut self, row_number: usize) -> Option<&mut Row> {
        self.rows.iter_mut().find(|r| r.row_number == row_number)
    }

    pub fn empty_rows(&self) -> Vec<&Row> {
        self.rows.iter().filter(|r| r.available_space > 0).collect()
    }

    pub fn empty_rows_mut(&mut self) -> Vec<&mut Row> {
        self.rows
            .iter_mut()
            .filter(|r| r.available_space > 0)
            .collect()
    }

    pub fn is_full(&self) -> bool {
        self.available_space == 0
    }

    pub fn flat_map(&self) -> String {
        self.rows
            .iter()
            .map(|row| row.flat_map())
            .collect::<Vec<String>>()
            .join("")
    }

    pub fn flat_map_position_to_zone(&self, position: usize) -> Option<(usize, usize, usize)> {
        let mut cumulative_capacity = 0;

        for (row_index, row) in self.rows.iter().enumerate() {
            let row_zone_count: usize = row.columns.iter().map(|col| col.capacity).sum();

            if position < cumulative_capacity + row_zone_count {
                let relative_position = position - cumulative_capacity;

                let mut column_cumulative_zones = 0;
                for (column_index, column) in row.columns.iter().enumerate() {
                    if relative_position < column_cumulative_zones + column.capacity {
                        let zone_index = relative_position - column_cumulative_zones;
                        return Some((row_index + 1, column_index + 1, zone_index + 1));
                    }

                    column_cumulative_zones += column.capacity;
                }
            }

            cumulative_capacity += row_zone_count;
        }
        None
    }

    pub fn initialize_rows(&mut self, row_count: usize, col_per_row: usize, zone_per_col: usize) {
        for i in 1..=row_count {
            let mut row = Row::new(i);
            row.initialize_columns(col_per_row, zone_per_col);
            self.add_row(row);
        }
    }

    pub fn contains_product(&self, product_id: u32) -> bool {
        self.rows.iter().any(|row| row.contains_product(product_id))
    }

    pub fn add_item(
        &mut self,
        row_number: usize,
        column_number: usize,
        zone_number: usize,
        item: ProductItem,
    ) -> Result<(), ErrorKind> {
        if let Some(row) = self.row_mut(row_number) {
            match row.add_item(column_number, zone_number, item) {
                Ok(_) => {
                    self.available_space -= 1;
                    Ok(())
                }
                Err(e) => Err(e),
            }
        } else {
            Err(RowNotFound(row_number))
        }
    }

    pub fn remove_item(
        &mut self,
        row_number: usize,
        column_number: usize,
        zone_number: usize,
    ) -> Result<(), ErrorKind> {
        if let Some(row) = self.row_mut(row_number) {
            match row.remove_item(column_number, zone_number) {
                Ok(_) => {
                    self.available_space += 1;
                    Ok(())
                }
                Err(e) => Err(e),
            }
        } else {
            Err(RowNotFound(row_number))
        }
    }

    pub fn get_item(
        &self,
        row_number: usize,
        column_number: usize,
        zone_number: usize,
    ) -> Option<&ProductItem> {
        if let Some(row) = self.row(row_number) {
            return row.item(column_number, zone_number);
        }
        None
    }

    pub fn get_item_mut(
        &mut self,
        row_number: usize,
        column_number: usize,
        zone_number: usize,
    ) -> Option<&mut ProductItem> {
        if let Some(row) = self.row_mut(row_number) {
            return row.item_mut(column_number, zone_number);
        }
        None
    }

    pub fn find_item(&self, product_id: u32) -> Option<(usize, usize, usize)> {
        for row in &self.rows {
            let row_number = row.row_number;
            if let Some((column_number, zone_number)) = row.find_item(product_id) {
                return Some((row_number, column_number, zone_number));
            }
        }
        None
    }

    pub fn find_last_item_occurrence(&self, product_id: u32) -> Option<(usize, usize, usize)> {
        for row in &self.rows {
            let row_number = row.row_number;
            if let Some((column_number, zone_number)) = row.find_last_item_occurrence(product_id) {
                return Some((row_number, column_number, zone_number));
            }
        }
        None
    }

    pub fn find_all_item_occurences(&self, product_id: u32) -> Vec<(usize, usize, usize)> {
        let mut items = Vec::new();
        for row in &self.rows {
            let row_number = row.row_number;
            items.extend(
                row.find_all_item_occurences(product_id)
                    .iter()
                    .map(|(column_number, zone_number)| (row_number, *column_number, *zone_number)),
            );
        }
        items
    }

    pub fn is_product_stored_contiguously(&self, product_id: u32) -> bool {
        let item_list = self.find_all_item_occurences(product_id);

        if item_list.is_empty() {
            return false;
        }
        if item_list.len() == 1 {
            return true;
        }

        for i in 1..item_list.len() {
            if item_list[i].0 != item_list[i - 1].0 {
                return false;
            }
        }

        for i in 1..item_list.len() {
            if item_list[i].1 != item_list[i - 1].1 {
                return false;
            }
        }

        for i in 1..item_list.len() {
            if item_list[i].2 != item_list[i - 1].2 + 1 {
                return false;
            }
        }

        true
    }

    pub fn add_items_by_qty(
        &mut self,
        product_id: u32,
        qty: usize,
        expiry_date: Option<NaiveDate>,
    ) -> Result<(), ErrorKind> {
        let mut qty_added = 0;

        if qty > self.available_space {
            return Err(InsufficientSpace);
        }

        if !self.contains_product(product_id) {
            println!("Product not found in warehouse, adding new items...");
            match self.find_contiguous_space(qty) {
                Ok(new_position) => {
                    println!("Contiguous space at {:?}", new_position);
                    let first_position = new_position;
                    let (mut r, mut c, mut z) = &new_position;
                    while qty_added < qty {
                        match self.add_item(
                            r,
                            c,
                            z,
                            ProductItem::new(product_id, r, c, z, expiry_date),
                        ) {
                            Ok(_) => {
                                qty_added += 1;
                                println!("Added item at {:?}", (r, c, z));
                                self.available_space -= 1;
                            }
                            Err(e) => return Err(e),
                        }
                        z += 1;
                        if z == self.rows[r-1].columns[c-1].zones.len() + 1 {
                            z = 1;
                            c += 1;
                            if c == self.rows[r-1].columns.len() + 1 {
                                c = 1;
                                r += 1;
                            }
                        }
                    }
                    let last_position = (r - 1, c - 1, z - 1);
                    println!(
                        "Added {} items from {:?} to {:?}",
                        qty, first_position, last_position
                    );
                    Ok(())
                }
                Err(e) => Err(e),
            }
        } else if self.is_product_stored_contiguously(product_id) {
            println!("Product is stored contiguously, adding new items...");
            let items = self.find_all_item_occurences(product_id);
            let last_item_position = items.last().unwrap();

            let (mut r, mut c, mut z) = *last_item_position;
            let first_position = *last_item_position;
            while qty_added < qty {
                if let Some(zone) = self.zone_mut(r, c, z) {
                    if zone.is_empty() {
                        let new_item = ProductItem::new(product_id, r, c, z, expiry_date);
                        match zone.add(new_item) {
                            Ok(_) => {
                                qty_added += 1;
                                self.available_space -= 1;
                            }
                            Err(e) => return Err(e),
                        }
                    }

                    z += 1;
                    if z == self.rows[r].columns[c].zones.len() + 1 {
                        z = 1;
                        c += 1;
                        if c == self.rows[r].columns.len() + 1 {
                            c = 1;
                            r += 1;
                        }
                    }
                }
            }
            let last_position = (r - 1, c - 1, z - 1);
            println!(
                "Added {} items from {:?} to {:?}",
                qty, first_position, last_position
            );
            Ok(())
        } else {
            println!("Product is not stored contiguously, moving items and adding new items...");
            let existing_items = self.find_all_item_occurences(product_id);
            let total_items = existing_items.len() + qty;

            match self.find_contiguous_space(total_items) {
                Ok(new_contiguous_position) => {
                    let grouped_items = self.group_items_by_expiration(existing_items);

                    match self.move_items_to_contiguous_space(grouped_items) {
                        Ok(_) => {}
                        Err(e) => return Err(e),
                    }

                    let (mut r, mut c, mut z) = new_contiguous_position;
                    let first_position = new_contiguous_position;
                    while qty_added < qty {
                        if let Some(zone) = self.zone_mut(r, c, z) {
                            if zone.is_empty() {
                                let new_item = ProductItem::new(product_id, r, c, z, expiry_date);
                                match zone.add(new_item) {
                                    Ok(_) => {
                                        qty_added += 1;
                                        self.available_space -= 1;
                                    }
                                    Err(e) => return Err(e),
                                }
                            }

                            z += 1;
                            if z == self.rows[r].columns[c].zones.len() + 1 {
                                z = 1;
                                c += 1;
                                if c == self.rows[r].columns.len() + 1 {
                                    c = 1;
                                    r += 1;
                                }
                            }
                        }
                    }
                    let last_position = (r - 1, c - 1, z - 1);
                    println!(
                        "Added {} items from {:?} to {:?}",
                        qty, first_position, last_position
                    );
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
    }

    pub fn move_item(
        &mut self,
        current_zone: (usize, usize, usize),
        new_zone: (usize, usize, usize),
    ) -> Result<(), ErrorKind> {
        match self.zone(current_zone.0, current_zone.1, current_zone.2) {
            Some(zone) => match &zone.item {
                Some(item) => {
                    let new_item = item.copy_at_zone(new_zone.0, new_zone.1, new_zone.2);
                    match self.add_item(new_zone.0, new_zone.1, new_zone.2, new_item) {
                        Ok(_) => {
                            match self.remove_item(current_zone.0, current_zone.1, current_zone.2) {
                                Ok(_) => Ok(()),
                                Err(e) => Err(e),
                            }
                        }
                        Err(e) => Err(e),
                    }
                }
                _ => Err(ZoneEmpty((
                    current_zone.0,
                    current_zone.1,
                    current_zone.2,
                ))),
            },
            _ => Err(ZoneNotFound((
                current_zone.0,
                current_zone.1,
                current_zone.2,
            ))),
        }
    }

    pub fn remove_item_by_id(&mut self, product_id: u32) -> Result<(), ErrorKind> {
        match self.find_item(product_id) {
            Some(item) => match self.remove_item(item.0, item.1, item.2) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            _ => Err(NoProductFound),
        }
    }

    pub fn remove_item_by_qty(&mut self, product_id: u32, qty: usize) -> Result<(), ErrorKind> {
        let mut qty_removed = 0;
        let items = self.find_all_item_occurences(product_id);

        let mut items_with_dates: Vec<((usize, usize, usize), NaiveDate)> = items
            .iter()
            .filter_map(|&(row, col, zone)| {
                self.get_item(row, col, zone)
                    .and_then(|item| item.expiry_date.map(|date| ((row, col, zone), date)))
            })
            .collect();

        items_with_dates.sort_by(|a, b| a.1.cmp(&b.1));

        if items_with_dates.len() >= qty {
            for ((row, col, zone), _) in items_with_dates.iter().take(qty) {
                match self.remove_item(*row, *col, *zone) {
                    Ok(_) => {
                        qty_removed += 1;
                        if qty_removed == qty {
                            break;
                        }
                    }
                    Err(e) => return Err(e),
                }
            }
            Ok(())
        } else {
            println!("Insufficient quantity, removing all items");
            match self.remove_all_items(product_id) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        }
    }

    pub fn group_items_by_expiration(
        &mut self,
        items: Vec<(usize, usize, usize)>,
    ) -> HashMap<String, Vec<(usize, usize, usize)>> {
        let mut list_with_items: Vec<((usize, usize, usize), NaiveDate)> = Vec::new();

        for (row, col, zone) in items {
            if let Some(item) = self.get_item(row, col, zone) {
                if let Some(date) = item.expiry_date {
                    list_with_items.push(((row, col, zone), date));
                }
            }
        }

        list_with_items.sort_by(|a, b| a.1.cmp(&b.1));

        let mut grouping: HashMap<String, Vec<(usize, usize, usize)>> = HashMap::new();
        for ((row, col, zone), date) in list_with_items {
            let key = date.to_string();
            grouping.entry(key).or_default().push((row, col, zone));
        }

        grouping
    }

    pub fn find_contiguous_space(
        &mut self,
        required_space: usize,
    ) -> Result<(usize, usize, usize), ErrorKind> {
        if required_space > self.available_space {
            Err(InsufficientSpace)
        } else {
            let flat_map: String = self.flat_map();

            if let Some(position) = flat_map.find(&"0".repeat(required_space)) {
                match self.flat_map_position_to_zone(position) {
                    Some(zone) => Ok(zone),
                    None => Err(NoContiguousSpace),
                }
            } else {
                Err(NoContiguousSpace)
            }
        }
    }

    pub fn move_items_to_contiguous_space(
        &mut self,
        mut grouped_items: HashMap<String, Vec<(usize, usize, usize)>>,
    ) -> Result<(), ErrorKind> {
        let mut required_space = 0;
        for items in grouped_items.values() {
            required_space += items.len();
        }
        match self.find_contiguous_space(required_space) {
            Ok((mut r, mut c, mut z)) => {
                let first_position = (r, c, z);
                for (_, items) in grouped_items.iter_mut() {
                    for item in items {
                        let (row, column, zone) = *item;
                        match self.move_item((row, column, zone), (r, c, z)) {
                            Ok(_) => {}
                            Err(e) => return Err(e),
                        }
                        required_space -= 1;
                        if z + 1 == self.rows[r].columns[c].zones.len() {
                            z = 0;
                            c += 1;
                            if c == self.rows[r].columns.len() {
                                c = 0;
                                r += 1;
                            }
                        } else {
                            z += 1;
                        }
                    }
                }
                let last_position = (r - 1, c - 1, z - 1);
                println!(
                    "Moved items to zones from {:?} to {:?}",
                    first_position, last_position
                );
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn organize_items_by_id(&mut self, product_id: u32) -> Result<(), ErrorKind> {
        match self.find_all_item_occurences(product_id) {
            items if !items.is_empty() => {
                let grouped_items = self.group_items_by_expiration(items);
                match self.move_items_to_contiguous_space(grouped_items) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            }
            _ => Err(NoProductFound),
        }
    }

    pub fn remove_all_items(&mut self, product_id: u32) -> Result<(), ErrorKind> {
        match self.find_all_item_occurences(product_id) {
            items if !items.is_empty() => {
                for (row, col, zone) in items {
                    match self.remove_item(row, col, zone) {
                        Ok(_) => {}
                        Err(e) => return Err(e),
                    }
                }
                Ok(())
            }
            _ => Err(NoProductFound),
        }
    }
}

impl Default for Warehouse {
    fn default() -> Self {
        Self::new()
    }
}
