use lopdf::{Document, Object, Stream};
use lopdf::content::{Content, Operation};
use std::collections::HashMap;
use std::convert::TryInto;

use crate::models::Order;

fn generate_text_block(font_type: &'static str, font_size: u8, pos_x: u16, pos_y: u16, text: &String, operations: &mut Vec<Operation>) {
    operations.push(Operation::new("BT", vec![]));
    operations.push(Operation::new("Tf", vec![font_type.into(), font_size.into()]));
    operations.push(Operation::new("Td", vec![pos_x.into(), pos_y.into()]));
    operations.push(Operation::new("Tj", vec![Object::string_literal(text.to_string())]));
    operations.push(Operation::new("ET", vec![]));
}

fn draw_line(line_height: u16, operations: &mut Vec<Operation>) {
    operations.push(Operation::new("w", vec![0.25.into()]));
    operations.push(Operation::new("m", vec![18.into(), line_height.into()]));
    operations.push(Operation::new("l", vec![270.into(), line_height.into()]));
    operations.push(Operation::new("S", vec![]));
}

fn generate_page(order: &Vec<Order>, page_ids: &mut Vec<Object>, pages_id: (u32, u16), doc: &mut Document, customer_part: Option<String>) {
    let mut operations: Vec<Operation> = Vec::new();

    // generate the text blocks for the headings
    // Customer on top
    let first_order_line = order.first().unwrap();
    // if this is a split label, the customer_part will be present
    let customer = match customer_part.is_some() {
        true => customer_part.unwrap(),
        false => first_order_line.customer.to_owned(),
    }; 
    generate_text_block("F2", 22, 18, 390, &customer, &mut operations);
    // Fullfillment Type next, making sure it isn't blank
    if first_order_line.fulfillment.is_some() {
        let fulfillment = first_order_line.fulfillment.as_ref().unwrap();
        generate_text_block("F2", 18, 18, 369, &fulfillment, &mut operations)
    } else {
            let fulfillment = order.last().unwrap().fulfillment.as_ref().unwrap();
            generate_text_block("F2", 18, 18, 369, &fulfillment, &mut operations)
    }
    // Order Id is next
    let id_text = format!("Order # {}", first_order_line.id);
    generate_text_block("F1", 12, 18, 354, &id_text, &mut operations);
    // Column Headings are next
    generate_text_block("F1", 10, 18, 340, &"Item".to_owned(), &mut operations);
    generate_text_block("F1", 10, 180, 340, &"QTY".to_owned(), &mut operations);
    generate_text_block("F1", 10, 212, 340, &"Unit".to_owned(), &mut operations);

    
    // generate the text blocks for the items
    let row_pos: u16 = 323;
    let mut i: u16 = 0;
    
    for order_line in order {
        // skip blank Fullfullment Type items in the order
        if order_line.fulfillment.is_none() {
            continue;
        } 
        // skip Delivery Charge items in the order
        if order_line.item.contains("Delivery") {
            continue;
        }
        let row_height = row_pos - (i * 15);

        // draw line above
        let line_height = row_height + 14;
        draw_line(line_height, &mut operations);
        
        // add item
        let item = match order_line.item.len() > 23 {
            true => format!("{}..", order_line.item[0..21].to_owned()),
            false => order_line.item.to_owned(),
        }; 
        generate_text_block("F1", 13, 18, row_height, &item, &mut operations);
        
        // add qyt
        generate_text_block("F1", 11, 180, row_height, &order_line.quantity.unwrap().to_string(), &mut operations);

        // add unit 
        generate_text_block("F1", 10, 212, row_height, &order_line.unit_size.as_ref().unwrap(), &mut operations);

        // if last row, draw line below
        let length: u16 = order.len().try_into().expect("Number of items in order is way too large! Larger than 2^63 - 1.");
        if length == i + 1 {
            let line_height = row_height - 1;
            draw_line(line_height, &mut operations);
        }
        i = i + 1; 
    }
    
    
    // Add the operations to the content
    let content = Content {
        operations: operations,
    };
    // Add the content and new page to the document
    let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "Contents" => content_id,
    });

    page_ids.push(page_id.into());
}

pub fn generate_document(path: &String, orders: &HashMap<u32, Vec<Order>>) -> Result<String, String> {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font_id_reg = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Arial",
    });
    let font_id_bold = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type2",
        "BaseFont" => "Arial",
    });
    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! {
            "F1" => font_id_reg,
            "F2" => font_id_bold,
        },
    });


    // generate each label page
    let mut page_ids: Vec<Object> = Vec::new();
    for order_id in orders.keys() {
        let order = orders.get(&order_id).unwrap();
        // check to make sure the number of items will fit on the label
        if order.len() * 16 > 323 - 18 {
           // if it won't fit, split order into two labels
           let split_point = order.len() / 2;
           let order_p1 = &order[0..split_point];
           let mut order_p2: Vec<Order> = Vec::new();
           for order_line in &order[split_point..] {
               order_p2.push(order_line.clone());
           }
           let customer_p1 = format!("{} 1of2", order_p1.first().unwrap().customer);
           let customer_p2 = format!("{} 2of2", order_p2.first().unwrap().customer);
           generate_page(&Vec::from(order_p1), &mut page_ids, pages_id, &mut doc, Some(customer_p1));
           generate_page(&Vec::from(order_p2), &mut page_ids, pages_id, &mut doc, Some(customer_p2));
        } else {
            // Just make one label for the order
            generate_page(order, &mut page_ids, pages_id, &mut doc, None);
        }
    }

    let page_count: i64 = page_ids.len().try_into().expect("Number of orders is way too large! Larger than 2^63 - 1.");

    let pages = dictionary! {
        "Type" => "Pages",
        "Kids" => page_ids,
        "Count" => page_count, 
        "Resources" => resources_id,
        "MediaBox" => vec![0.into(), 0.into(), 288.into(), 432.into()], // 288 x 432 = 4 inches x 6 inches
    };
    doc.objects.insert(pages_id, Object::Dictionary(pages));
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);

    // Compress and try to Save the file
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