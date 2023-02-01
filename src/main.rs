pub mod dom;
pub mod html;

fn main() {
    let html = String::from("<html name='a'><body>Hello, world!</body></html>");
    let root_node = html::parse(html);
    println!("{:?}", root_node);
}
