extern crate serde;
extern crate serde_json;
extern crate tinyfiledialogs as tfd;
extern crate web_view;

use serde::{Serialize, Deserialize};
use tfd::MessageBoxIcon;
use web_view::{Content};

#[derive(Serialize, Deserialize)]
struct Task {
    command: String,
    data: String
}

fn print_file(file: String) {
    println!("file: {}", file);
}

fn open_file() {
    match tfd::open_file_dialog("Please choose a csv file...", "", None) {
        Some(path) => tfd::message_box_ok("File chosen", &path, MessageBoxIcon::Info),
        None => tfd::message_box_ok(
            "Warning",
            "You didn't choose a file.",
            MessageBoxIcon::Warning,
        ),
    }
}

fn main() {
    web_view::builder()
        .title("B2D2")
        .content(Content::Html(include_str!("static/index.html")))
        .size(800, 500)
        .resizable(true)
        .user_data("")
        .invoke_handler(|_webview, arg| {
            let task: Task = serde_json::from_str(arg).unwrap();

            match task.command.as_str() {
                "add_file" => print_file(task.data),
                "open" => open_file(),
                _ => (),
            }

            Ok(())
        })
        .run()
        .unwrap();
}

