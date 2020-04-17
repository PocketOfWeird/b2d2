extern crate csv;
#[macro_use]
extern crate lopdf;
extern crate serde;
extern crate tinyfiledialogs as tfd;

use tfd::{MessageBoxIcon, YesNo};
use std::collections::HashMap;

mod models;
mod pdf;

use models::Order;

fn handle_csv_conversion_error(path: &String, error: csv::Error) -> bool {
    if path.ends_with(".csv") {
        let message = format!(
            "Error converting the CSV file:
            \n{}
            \n----------------------------------------------------------------
            \nPlease send the file and a picture of this error to Nathan H:
            \n----------------------------------------------------------------
            \n{:?}", 
            path, 
            error
        );
        return file_error(&message, false);
    } else {
        let message = format!("The file is not a CSV file:\n{}\nWould you like to choose another file?", path);
        return file_error(&message, true);
    }
}

fn process_csv(path: String) -> (Option<HashMap<u32, Vec<Order>>>, bool) {
    let rdr = csv::Reader::from_path(&path);
    if rdr.is_ok() {
        let mut orders: HashMap<u32, Vec<Order>> = HashMap::new();
        for result in rdr.unwrap().deserialize() {
            if result.is_ok() {
                let order: Order = result.unwrap();
                if orders.contains_key(&order.id) {
                    let items = orders.get_mut(&order.id).unwrap();
                    items.push(order);
                } else {
                    orders.insert(order.id, vec![order]);
                }
            } else {
                let trying_again = handle_csv_conversion_error(&path, result.unwrap_err());
                if trying_again {
                    return (None, true);
                } else {
                    return (None, false);
                }
            }
        }
        return (Some(orders), false);
    } else {
        let question = format!("There was an error opening the file:\n{}\nWould you like to try again?", path);
        let trying_again = file_confirm(&question, Some(true));
        return (None, trying_again);
    }
}

fn create_pdf(path: String, orders: HashMap<u32, Vec<Order>>) {
    match pdf::generate_document(&path, &orders) {
        Ok(message) => tfd::message_box_ok("Complete", &message, MessageBoxIcon::Info),
        Err(error) => tfd::message_box_ok("Error", &error, MessageBoxIcon::Error),
    }
}

fn file_error(message: &String, confirm: bool) -> bool {
    if confirm {
        return file_confirm(message, Some(true));
    } else {
        tfd::message_box_ok("Error", &message, MessageBoxIcon::Error);
        return false;
    } 
}

fn file_warning() {
    tfd::message_box_ok("Warning", "You didn't choose a file.", MessageBoxIcon::Warning);
}

fn file_confirm(question: &String, error: Option<bool>) -> bool {
    let title = match error.is_some() {
        true => "File Error",
        false => "Confirm File",
    };
    let icon = match error.is_some() {
        true => MessageBoxIcon::Error,
        false => MessageBoxIcon::Question,
    };

    match tfd::message_box_yes_no(title, question, icon, YesNo::Yes) {
        YesNo::Yes => true,
        YesNo::No => false,
    }
}

fn save_file() -> Option<String> {
    let path = tfd::save_file_dialog("Save the labels PDF file...", "order-labels.pdf");
    if path.is_some() {
        return path;
    } else {
        file_warning();
        return None;
    }
}


fn open_file() -> Option<String> {
    let path = tfd::open_file_dialog("Please choose a Barn2Door csv file...", "", None);
    if path.is_some() {
        let file = path.unwrap();
        let question = format!("You would like to make labels for\n{}", file);
        if file_confirm(&question, None) {
            return Some(file);
        } else {
            return None;
        }
    } else {
        file_warning();
        return None;
    }
}

fn main() {
    let file_path = open_file();
    if file_path.is_some() {
        let (orders, trying_again) = process_csv(file_path.unwrap());
        if orders.is_some() {
            let save_path = save_file();
            if save_path.is_some() {
                create_pdf(save_path.unwrap(), orders.unwrap());
            }
        } else if trying_again {
            // recursive call to main, until the user chooses not to try again after an error, 
            // or the pdf creation completes successfully
            main();
        }
    }
}

