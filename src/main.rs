use std::fs;

pub mod parser;
pub mod html;
pub mod css;

fn main() {
    // html 解析
    let html = fs::read_to_string("src/examples/test.html").unwrap();
    let root_node = html::parser::parse(html);
    println!("{:?}", root_node);
    
    // css 解析
    let css = fs::read_to_string("src/examples/test.css").unwrap();
    let rules = css::parser::parse(css);
    println!("{:?}", rules);
}
