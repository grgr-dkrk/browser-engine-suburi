use std::io::{Read, BufWriter};
use std::fs::File;

pub mod dom;
pub mod html;
pub mod css;
pub mod style;

fn main() {
    let html = read_source("test.html".to_string());
    let css  = read_source("test.css".to_string());

    let root_node = html::parse(html);
    println!("DOMTree: {:?}", root_node);
    let stylesheet = css::parse(css);
    let style_root = style::style_tree(&root_node, &stylesheet);
    println!("StyleTree: {:?}", style_root)
}

fn read_source(filename: String) -> String {
    let mut str = String::new();
    File::open(filename).unwrap().read_to_string(&mut str).unwrap();
    str
}