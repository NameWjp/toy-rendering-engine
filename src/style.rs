use std::collections::HashMap;

use crate::{css::types::{Value, SimpleSelector, Selector, Rule, Specificity, Stylesheet}, html::types::{Node, ElementData, NodeType}};

// 一个元素应用的样式
type PropertyMap = HashMap<String, Value>;

// 一个元素可以有多个 MatchedRule，Specificity 用来判断 css 的优先级
type MatchedRule<'a> = (Specificity, &'a Rule);

pub enum Display {
    Inline,
    Block,
    None,
}

#[derive(Debug)]
pub struct StyledNode<'a> {
    pub node: &'a Node,
    pub specified_values: PropertyMap,
    pub children: Vec<StyledNode<'a>>,
}

impl <'a> StyledNode<'a> {
    // 如果属性存在则返回这个值，否则返回 None
    pub fn value(&self, name: &str) -> Option<Value> {
        self.specified_values.get(name).map(|v| v.clone())
    }

    // 显示 display 属性的值
    pub fn display(&self) -> Display {
        match self.value("display") {
            Some(Value::Keyword(s)) => match &*s {
                "block" => Display::Block,
                "none" => Display::None,
                _ => Display::Inline
            },
            _ => Display::Inline
        }
    }

    // 返回 name 或 fallback_name 属性对应的值，如果都不存在则返回 default
    pub fn lookup(&self, name: &str, fallback_name: &str, default: &Value) -> Value {
        self.value(name).unwrap_or_else(|| self.value(fallback_name).unwrap_or_else(|| default.clone()))
    }
}

pub fn style_tree<'a>(root: &'a Node, stylesheet: &'a Stylesheet) -> StyledNode<'a> {
    StyledNode { 
        node: root, 
        specified_values: match root.node_type {
            NodeType::Element(ref elem) => specified_values(elem, stylesheet),
            NodeType::Text(_) => HashMap::new()
        }, 
        children: root.children.iter().map(|child| style_tree(child, stylesheet)).collect(),
    }
}

// 获取元素的样式列表
fn specified_values(elem: &ElementData, stylesheet: &Stylesheet) -> PropertyMap {
    let mut values = HashMap::new();
    let mut rules = matching_rules(elem, stylesheet);

    // 按照 css 选择器的权重渲染，权重低的先渲染
    rules.sort_by(|&(a, _), &(b, _)| a.cmp(&b));
    for (_, rule) in rules {
        for declaration in &rule.declarations {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }
    
    values
}

fn matching_rules<'a>(elem: &ElementData, stylesheet: &'a Stylesheet) -> Vec<MatchedRule<'a>> {
    stylesheet.rules.iter().filter_map(|rule| match_rule(elem, rule)).collect()
}

fn match_rule<'a>(elem: &ElementData, rule: &'a Rule) -> Option<MatchedRule<'a>> {
    // 找到第一个匹配的选择器
    rule.selectors.iter()
        .find(|selector| matches(elem, *selector))
        .map(|selector| (selector.specificity(), rule))
}

fn matches(elem: &ElementData, selector: &Selector) -> bool {
    match *selector {
        Selector::Simple(ref simple_selector) => matches_simple_selector(elem, simple_selector)
    }
}

fn matches_simple_selector(elem: &ElementData, selector: &SimpleSelector) -> bool {
    // 如果有标签，并且和当前匹配元素的标签对不上，则返回
    if selector.tag_name.iter().any(|name| elem.tag_name != *name) {
        return false;
    }

    // 如果有 ID，并且和当前匹配元素的 ID 对不上，则返回
    if selector.id.iter().any(|id| elem.id() != Some(id)) {
        return false;
    }

    // 如果某个类选择器不在当前匹配的元素中，则返回
    let elem_classes = elem.classes();
    if selector.class.iter().any(|class| !elem_classes.contains(&**class)) {
        return false;
    }

    return true;
}