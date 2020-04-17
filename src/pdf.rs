use lopdf::{Document, Object, ObjectId, Stream};
use lopdf::content::{Content, Operation};
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::ErrorKind;

use crate::models::Order;

fn generate_text_block(font_size: u8, pos_x: u8, pos_y: u8, text: &String,) -> Vec<Operation> {
    vec! [
        Operation::new("BT", vec![]),
        Operation::new("Tf", vec!["F1".into(), font_size.into()]),
        Operation::new("Td", vec![pos_x.into(), pos_y.into()]),
        Operation::new("Tj", vec![Object::string_literal(text.to_string())]),
        Operation::new("ET", vec![]),
    ]
}

pub fn generate_document(path: &String, orders: &HashMap<u32, Vec<Order>>) -> Result<String, String> {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type2",
        "BaseFont" => "Arial",
    });
    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! {
            "F1" => font_id,
        },
    });


    // generate each label page
    let mut page_ids: Vec<Object> = Vec::new();
    for order_id in orders.keys() {
        let order = orders.get(&order_id);
        let mut operation_list: Vec<Operation> = Vec::new();
        
        // generate the text blocks for the headings
        // generate the text blocks for the items
        let test_text: String = "Hello you".to_owned();
        let test_text_block = generate_text_block(24, 110, 210, &test_text);
        for operation in test_text_block {
            operation_list.push(operation);
        }


        let content = Content {
            operations: operation_list,
        };
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
        });

        page_ids.push(page_id.into());
    }
    let page_count: i64 = page_ids.len().try_into().expect("Number of orders is way too large! Larger than 2^63 - 1.");

    let pages = dictionary! {
        "Type" => "Pages",
        "Kids" => page_ids,
        "Count" => page_count, 
        "Resources" => resources_id,
        "MediaBox" => vec![0.into(), 0.into(), 288.into(), 432.into()],
    };
    doc.objects.insert(pages_id, Object::Dictionary(pages));
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);
    doc.compress();
    let result = doc.save(path);
    if result.is_ok() {
        return Ok(format!("The Dymo Labels for the Barn2Door orders have saved to {}", path))
    } else {
        let err = result.unwrap_err();
        if err.raw_os_error().is_some() {
            let error_code = err.raw_os_error().unwrap();
            if error_code == 32 {
                return Err(format!("The PDF file may be open, please close it before running B2D2 again.\n{}", path));
            }
        }
        return Err(format!(
            "Error saving the PDF file:
            \n{}
            \n-------------------------------------------------
            \nPlease send a picture of this error to Nathan H:
            \n-------------------------------------------------
            \n{:?}", 
            path, 
            err
        ));
    }
}