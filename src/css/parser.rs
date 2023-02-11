// 简单的选择器：一个标签名称、一个 ID、任意数量的类名称，或者以上的某种组合，且支持 * 选择器。
use crate::parser::Parser;

use super::types;

pub fn parse(source: String) -> types::Stylesheet {
    let mut parser = CSSParser { pos: 0, input: source };
    types::Stylesheet { rules: parser.parse_rules() }
}

fn valid_identifier_char(c: char) -> bool {
    match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => true,
        _ => false,
    }
}

struct CSSParser {
    pos: usize,
    input: String
}

impl Parser for CSSParser {
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

impl CSSParser {
    // 解析一组 css 规则
    fn parse_rules(&mut self) -> Vec<types::Rule> {
        let mut rules = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() { break; }
            rules.push(self.parse_rule());
        }
        rules
    }

    // 解析一个 css 规则，例如：`<selectors> { <declarations> }`
    fn parse_rule(&mut self) -> types::Rule {
        types::Rule { 
            selectors: self.parse_selectors(), 
            declarations: self.parse_declarations(),
        }
    }

    // 解析选择器列表 selectors
    fn parse_selectors(&mut self) -> Vec<types::Selector> {
        let mut selector  = Vec::new();
        loop {
            selector.push(types::Selector::Simple(self.parse_selector()));
            self.consume_whitespace();
            match self.next_char() {
                ',' => { self.consume_char(); self.consume_whitespace(); }
                '{' => break,
                c => panic!("Unexpected character {} in selector list", c)
            }
        }
        // 按照 css 选择器的权重排序，权重高的在前面
        selector.sort_by(|a, b| b.specificity().cmp(&a.specificity()));
        selector
    }

    // 解析单个选择器
    fn parse_selector(&mut self) -> types::SimpleSelector {
        let mut selector = types::SimpleSelector { tag_name: None, id: None, class: Vec::new() };
        while !self.eof() {
            match self.next_char() {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();
                    selector.class.push(self.parse_identifier());
                }
                '*' => {
                    // 通用选择器
                    self.consume_char();
                }
                c if valid_identifier_char(c) => {
                    selector.tag_name = Some(self.parse_identifier());
                }
                _ => break
            }
        }
        selector
    }

    // 解析一个属性名称或关键字
    fn parse_identifier(&mut self) -> String {
        self.consume_while(valid_identifier_char)
    }

    // 解析声明 declarations
    fn parse_declarations(&mut self) -> Vec<types::Declaration> {
        assert_eq!(self.consume_char(), '{');
        let mut declarations = Vec::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '}' {
                self.consume_char();
                break;
            }
            declarations.push(self.parse_declaration())
        }
        declarations
    }

    // 解析一组声明：<property>: <value>
    fn parse_declaration(&mut self) -> types::Declaration {
        let property_name = self.parse_identifier();
        self.consume_whitespace();
        assert_eq!(self.consume_char(), ':');
        self.consume_whitespace();
        let value = self.parse_value();
        self.consume_whitespace();
        assert_eq!(self.consume_char(), ';');

        types::Declaration { name: property_name, value }
    }

    fn parse_value(&mut self) -> types::Value {
        match self.next_char() {
            '0'..='9' => self.parse_length(),
            '#' => self.parse_color(),
            _ => types::Value::Keyword(self.parse_identifier())
        }
    }

    fn parse_length(&mut self) -> types::Value {
        types::Value::Length(self.parse_float(), self.parse_unit())
    }

    fn parse_float(&mut self) -> f32 {
        let s = self.consume_while(|c| match c {
            '0'..='9' | '.' => true,
            _ => false,
        });
        s.parse().unwrap()
    }

    fn parse_unit(&mut self) -> types::Unit {
        match &*self.parse_identifier().to_ascii_lowercase() {
            "px" => types::Unit::Px,
            _ => panic!("unrecognized unit")
        }
    }

    fn parse_color(&mut self) -> types::Value {
        assert_eq!(self.consume_char(), '#');
        types::Value::ColorValue(types::Color {
            r: self.parse_hex_pair(),
            g: self.parse_hex_pair(),
            b: self.parse_hex_pair(),
            a: 255
        })
    }

    fn parse_hex_pair(&mut self) -> u8 {
        let s = &self.input[self.pos .. self.pos + 2];
        self.pos += 2;
        u8::from_str_radix(s, 16).unwrap()
    }
}