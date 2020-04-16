extern crate csv;
extern crate serde;
extern crate serde_json;
extern crate tinyfiledialogs as tfd;
extern crate web_view;

use serde::{Serialize, Deserialize};
use tfd::MessageBoxIcon;
use web_view::{Content};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    command: String,
    data: String
}
#[derive(Debug, Deserialize)]
struct Order {
    #[serde(rename = "Order #")]
    id: u32,
    #[serde(rename = "Fulfillment Type")]
    fulfillment: Option<String>,
    #[serde(rename = "Paid")]
    paid: String,
    #[serde(rename = "Item")]
    item: String,
    #[serde(rename = "Unit Size")]
    unit_size: Option<String>,
    #[serde(rename = "Quantity")]
    quantity: Option<u32>,
    #[serde(rename = "Price")]
    price: Option<String>,
    #[serde(rename = "Customer")]
    customer: String,
    #[serde(rename = "Pickup Address")]
    addr_pickup: Option<String>,
    #[serde(rename = "Delivery Address")]
    addr_delivery: Option<String>,
}

fn process_csv(path: String) -> Option<HashMap<u32, Vec<Order>>> {
    let rdr = csv::Reader::from_path(&path);
    if rdr.is_ok() {
        let mut orders: HashMap<u32, Vec<Order>> = HashMap::new();
        for result in rdr.unwrap().deserialize() {
            let order: Order = result.unwrap();
            if orders.contains_key(&order.id) {
                let items = orders.get_mut(&order.id).unwrap();
                items.push(order);
            } else {
                orders.insert(order.id, vec![order]);
            }
        }
        return Some(orders);
    } else {
        tfd::message_box_ok("Error", &format!("Error opening the file: {}", path), MessageBoxIcon::Error);
        return None;
    }
}

fn create_pdf(path: String, orders: HashMap<u32, Vec<Order>>) {
    
    tfd::message_box_ok("Complete", "The Dymo Labels for the B2D Orders have been created.", MessageBoxIcon::Info);
}

fn file_warning() {
    tfd::message_box_ok("Warning", "You didn't choose a file.", MessageBoxIcon::Warning);
}

fn save_file() -> Option<String> {
    return tfd::save_file_dialog("Save the labels PDF file...", "");
}


fn open_file() -> Option<String> {
    return tfd::open_file_dialog("Please choose a Barn2Door csv file...", "", None);
}

fn main() {
    let file_path = open_file();
    if file_path.is_some() {
        let orders = process_csv(file_path.unwrap());
        if orders.is_some() {
            let save_path = save_file();
            if save_path.is_some() {
                create_pdf(save_path.unwrap(), orders.unwrap());
            } else {
                file_warning();
            }
        } else {
            tfd::message_box_ok("Error", "Error processing the csv file", MessageBoxIcon::Error);    
        }
    } else {
        file_warning();
    }
    /*
    web_view::builder()
        .title("B2D2")
        .content(Content::Html(include_str!("static/index.html")))
        .size(800, 500)
        .resizable(true)
        .user_data("")
        .invoke_handler(|_webview, arg| {
            let task: Task = serde_json::from_str(arg).unwrap();

            match task.command.as_str() {
                "open" => open_file(),
                _ => (),
            }

            Ok(())
        })
        .run()
        .unwrap();
    */
}

