extern crate image;

use std::fs::File;
use std::io::{BufWriter, Read};

pub mod css;
pub mod dom;
pub mod html;
pub mod layout;
pub mod paint;
pub mod style;

fn main() {
  let html = read_source("test.html".to_string());
  let css = read_source("test.css".to_string());

  let root_node = html::parse(html);
  println!("DOMTree: {:?}", root_node);
  let stylesheet = css::parse(css);
  let style_root = style::style_tree(&root_node, &stylesheet);
  println!("StyleTree: {:?}", style_root);

  let mut viewport: layout::Dimensions = Default::default();
  viewport.content.width = 800.0;
  viewport.content.height = 600.0;
  let layout_root = layout::layout_tree(&style_root, viewport);
  println!("Layout: {:?}", layout_root);

  let filename = "capture.png";
  let mut file = BufWriter::new(File::create(&filename).unwrap());
  let canvas = paint::paint(&layout_root, viewport.content);
  let (w, h) = (canvas.width as u32, canvas.height as u32);
  let img = image::ImageBuffer::from_fn(w, h, move |x, y| {
    let color = canvas.pixels[(y * w + x) as usize];
    image::Pixel::from_channels(color.r, color.g, color.b, color.a)
  });
  let ok = image::ImageRgba8(img).save(&mut file, image::PNG).is_ok();
  if ok {
    println!("Saved output as {}", filename)
  } else {
    println!("Error saving output as {}", filename)
  }
}

fn read_source(filename: String) -> String {
  let mut str = String::new();
  File::open(filename).unwrap().read_to_string(&mut str).unwrap();
  return str
}
