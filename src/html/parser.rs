use std::collections::HashMap;

use crate::parser::Parser;

use super::types;

pub fn parse(source: String) -> types::Node {
    let mut nodes = HTMLParser { pos: 0, input: source }.parse_nodes();

    // 如果这个文档包含一个根节点，那么直接返回，否则创建一个
    if nodes.len() == 1 {
        nodes.swap_remove(0)
    } else {
        types::elem("html".to_string(), HashMap::new(), nodes)
    }
}

struct HTMLParser {
    pos: usize,
    input: String
}

impl Parser for HTMLParser {
    // 获取当前输入的值
    fn get_input(&self) -> &String {
        &self.input
    }

    // 获取当前位置
    fn get_pos(&self) -> usize {
        self.pos
    }

    // 设置当前位置
    fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }
}

impl HTMLParser {
    // 解析一组节点
    fn parse_nodes(&mut self) -> Vec<types::Node> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }
        return nodes;
    }

    // 解析一个节点
    fn parse_node(&mut self) -> types::Node {
        match self.next_char() {
            '<' => self.parse_element(),
            _ => self.parse_text()
        }
    }

    // 解析一个元素，包括开始标签，内容，关闭标签
    fn parse_element(&mut self) -> types::Node {
        // 开始标签
        assert!(self.consume_char() == '<');
        let tag_name = self.parse_tag_name();
        let attrs = self.parse_attributes();
        assert!(self.consume_char() == '>');

        // 内容
        let children = self.parse_nodes();

        // 关闭标签
        assert!(self.consume_char() == '<');
        assert!(self.consume_char() == '/');
        assert!(self.parse_tag_name() == tag_name);
        assert!(self.consume_char() == '>');

        return types::elem(tag_name, attrs, children)
    }

    // 解析标签或属性名称
    fn parse_tag_name(&mut self) -> String {
        self.consume_while(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => true,
            _ => false
        })
    }

    // 解析一个文本
    fn parse_text(&mut self) -> types::Node {
        types::text(self.consume_while(|c| c != '<'))
    }

    // 解析一组属性对，例如：name="value"
    fn parse_attributes(&mut self) -> types::AttrMap {
        let mut attributes = HashMap::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            }
            let (name, value) = self.parse_attr();
            attributes.insert(name, value);
        }
        return attributes;
    }

    // 解析单个属性，例如：name="value"
    fn parse_attr(&mut self) -> (String, String) {
        let name = self.parse_tag_name();
        assert!(self.consume_char() == '=');
        let value = self.parse_attr_value();
        return (name, value);
    }

    // 解析单个值，例如："value"
    fn parse_attr_value(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote == '"' || open_quote == '\'');
        let value = self.consume_while(|c| c != open_quote);
        assert!(self.consume_char() == open_quote);
        return value;
    }
}