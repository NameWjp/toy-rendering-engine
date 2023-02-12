use std::fs;

use image::{Rgba, ImageBuffer};

pub mod parser;
pub mod html;
pub mod css;
pub mod style;
pub mod layout;
pub mod painting;

fn main() {
    // 获取文件字符串
    let html = fs::read_to_string("src/examples/test.html").unwrap();
    let css = fs::read_to_string("src/examples/test.css").unwrap();
    
    // 创建一个可视区域
    let mut viewport: layout::Dimensions = Default::default();
    viewport.content.width = 800.0;
    viewport.content.height = 600.0;

    // 解析结构
    let root_node = html::parser::parse(html);
    let stylesheet = css::parser::parse(css);
    let style_root = style::style_tree(&root_node, &stylesheet);
    let layout_root = layout::layout_tree(&style_root, viewport);

    // 绘制图形
    let canvas = painting::paint(&layout_root, viewport.content);
    let (w, h) = (canvas.width as u32, canvas.height as u32);
    let img = ImageBuffer::from_fn(w, h, move |x, y| {
        let color = canvas.pixels[(y * w + x) as usize];
        Rgba([color.r, color.g, color.b, color.a])
    });
    img.save("output.png").unwrap();
}
