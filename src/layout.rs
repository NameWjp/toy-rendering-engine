/**
 * 该模块负责将‘样式树’生成‘布局树’，结构如下：
 * {
 *    dimensions：当前节点的尺寸信息，
 *    box_type：节点类型，块状或内联，
 *    children：子节点信息
 * }
 */
use std::default::Default;

use crate::{style::{StyledNode, Display}, css::types::{Value, Unit}};

pub use self::BoxType::{AnonymousBlock, InlineNode, BlockNode};

#[derive(Debug, Default, Copy, Clone)]
pub struct Dimensions {
    // 内容区域相对于文档的位置
    pub content: Rect,
    // 边缘信息
    pub padding: EdgeSizes,
    pub border: EdgeSizes,
    pub margin: EdgeSizes,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct EdgeSizes {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

pub struct LayoutBox<'a> {
    pub dimensions: Dimensions,
    pub box_type: BoxType<'a>,
    pub children: Vec<LayoutBox<'a>>
}

// 一个或多个行内元素默认会生成一个 AnonymousBlock 匿名块容器
pub enum BoxType<'a> {
    BlockNode(&'a StyledNode<'a>),
    InlineNode(&'a StyledNode<'a>),
    AnonymousBlock,
}

impl<'a> LayoutBox<'a> {
    // 构造函数
    fn new(box_type: BoxType) -> LayoutBox {
        LayoutBox { 
            box_type, 
            dimensions: Default::default(), 
            children: Vec::new(),
        }
    }

    // 获取样式节点
    fn get_style_node(&self) -> &'a StyledNode<'a> {
        match self.box_type {
            BlockNode(node) | InlineNode(node) => node,
            AnonymousBlock => panic!("Anonymous block box has no style node")
        }
    }
}

// 转换样式树到布局树（containing_block 为外部容器的尺寸）
pub fn layout_tree<'a>(node: &'a StyledNode<'a>, mut containing_block: Dimensions) -> LayoutBox<'a> {
    // 布局高度从 0 开始计算
    containing_block.content.height = 0.0;
    let mut root_box = build_layout_tree(node);
    root_box.layout(containing_block);
    root_box
}

// 构建布局树但是不进行计算
fn build_layout_tree<'a>(style_node: &'a StyledNode<'a>) -> LayoutBox<'a> {
    // 创建根盒子
    let mut root = LayoutBox::new(match style_node.display() {
        Display::Block => BlockNode(style_node),
        Display::Inline => InlineNode(style_node),
        Display::None => panic!("Root node has display: none.")
    });

    // 递归遍历子盒子
    for child in &style_node.children {
        match child.display() {
            Display::Block => root.children.push(build_layout_tree(child)),
            Display::Inline => root.get_inline_container().children.push(build_layout_tree(child)),
            Display::None => {}
        }
    }

    root
}

impl<'a> LayoutBox<'a> {
    // 计算尺寸
    fn layout(&mut self, containing_block: Dimensions) {
        match self.box_type {
            BlockNode(_) => self.layout_block(containing_block),
            InlineNode(_) | AnonymousBlock => {}
        }
    }

    fn layout_block(&mut self, containing_block: Dimensions) {
        // 计算盒子的宽度
        self.calculate_block_width(containing_block);
        // 计算盒子定位
        self.calculate_block_position(containing_block);
        // 递归计算子框
        self.layout_block_children();
        // 计算高度
        self.calculate_block_height();
    }

    fn calculate_block_width(&mut self, containing_block: Dimensions) {
        let style = self.get_style_node();

        // width 的默认值是 auto
        let auto = Value::Keyword("auto".to_string());
        let mut width = style.value("width").unwrap_or(auto.clone());

        // margin, border, padding 初始值是 0
        let zero = Value::Length(0.0, Unit::Px);

        let mut margin_left = style.lookup("margin-left", "margin", &zero);
        let mut margin_right = style.lookup("margin-right", "margin", &zero);

        let border_left = style.lookup("border-left-width", "border-width", &zero);
        let border_right = style.lookup("border-right-width", "border-width", &zero);

        let padding_left = style.lookup("padding-left", "padding", &zero);
        let padding_right = style.lookup("padding-right", "padding", &zero);

        let total = sum([
            &margin_left, &margin_right, &border_left, &border_right, &padding_left, &padding_right, &width
        ].iter().map(|v| v.to_px()));

        // 如果宽度不是 auto，并且总长大于盒子宽度，则视 merge 的 auto 为 0
        if width != auto && total > containing_block.content.width {
            if margin_left == auto {
                margin_left = Value::Length(0.0, Unit::Px);
            }
            if margin_right == auto {
                margin_right = Value::Length(0.0, Unit::Px);
            }
        }

        // 溢出或剩余的空间
        let underflow = containing_block.content.width - total;

        // 调整尺寸使 width 和 total 相等
        match (width == auto, margin_left == auto, margin_right == auto) {
            // 如果都为 false，则代表过度约束，计算 margin_right 的值
            (false, false, false) => {
                margin_right = Value::Length(margin_right.to_px() + underflow, Unit::Px);
            }
            // 如果 margin 恰好有一个尺寸使 auto，则计算它使其自等
            (false, false, true) => {
                margin_right = Value::Length(underflow, Unit::Px);
            }
            (false, true, false) => {
                margin_left = Value::Length(underflow, Unit::Px);
            }
            // 如果 width 是 auto，则其它的 auto 将成为 0
            (true, _, _) => {
                if margin_left == auto {
                    margin_left = Value::Length(0.0, Unit::Px);
                }
                if margin_right == auto {
                    margin_right = Value::Length(0.0, Unit::Px);
                }
                if underflow >= 0.0 {
                    // 展开宽度填满容器
                    width = Value::Length(underflow, Unit::Px);
                } else {
                    // 宽度不能是负的，调整右边距
                    width = Value::Length(0.0, Unit::Px);
                    margin_right = Value::Length(margin_right.to_px() + underflow, Unit::Px);
                }
            }
            // 如果 margin-left 和 margin-right 都是 auto，则每个一半的值
            (false, true, true) => {
                margin_left = Value::Length(underflow / 2.0, Unit::Px);
                margin_right = Value::Length(underflow / 2.0, Unit::Px);
            }
        }

        let d = &mut self.dimensions;
        d.content.width = width.to_px();

        d.padding.left = padding_left.to_px();
        d.padding.right = padding_right.to_px();

        d.border.left = border_left.to_px();
        d.border.right = border_right.to_px();

        d.margin.left = margin_left.to_px();
        d.margin.right = margin_right.to_px();
    }

    fn calculate_block_position(&mut self, containing_block: Dimensions) {
        let style = self.get_style_node();
        let d = &mut self.dimensions;

        let zero = Value::Length(0.0, Unit::Px);

        // 如果 margin-top、margin-bottom 是 auto，则使用 0
        d.margin.top = style.lookup("margin-top", "margin", &zero).to_px();
        d.margin.bottom = style.lookup("margin-bottom", "margin", &zero).to_px();
        
        d.border.top = style.lookup("border-top-width", "border-width", &zero).to_px();
        d.border.bottom = style.lookup("border-bottom-width", "border-width", &zero).to_px();

        d.padding.top = style.lookup("padding-top", "padding", &zero).to_px();
        d.padding.bottom = style.lookup("padding-bottom", "padding", &zero).to_px();

        d.content.x = containing_block.content.x + d.margin.left + d.border.left + d.padding.left;
        d.content.y = containing_block.content.y + d.margin.top + d.border.top + d.padding.top;
    }

    fn layout_block_children(&mut self) {
        let d = &mut self.dimensions;
        for child in &mut self.children {
            child.layout(*d);
            // 计算高度
            d.content.height = d.content.height + child.dimensions.margin_box().height;
        }
    }

    fn calculate_block_height(&mut self) {
        // 如果高度显示的设置，则使用该值
        if let Some(Value::Length(h, Unit::Px)) = self.get_style_node().value("height") {
            self.dimensions.content.height = h;
        }
    }

    // 创建匿名块容器
    fn get_inline_container(&mut self) -> &mut LayoutBox<'a> {
        match self.box_type {
            // 如果自己本身是内联元素则不用生成
            InlineNode(_) | AnonymousBlock => self,
            // 如果自己是块状元素则需要生成匿名块元素
            BlockNode(_) => {
                // 如果前一个内联元素已经生成过匿名块元素，则直接复用
                match self.children.last() {
                    Some(&LayoutBox { box_type: AnonymousBlock, .. }) => {},
                    _ => self.children.push(LayoutBox::new(AnonymousBlock))
                }
                self.children.last_mut().unwrap()
            }
        }
    }
}

impl Rect {
    pub fn expanded_by(self, edge: EdgeSizes) -> Rect {
        Rect { 
            x: self.x - edge.left,
            y: self.y - edge.top,
            width: self.width + edge.left + edge.right,
            height: self.height + edge.top + edge.bottom,
        }
    }
}

// 非常优美的写法，从内容区依次向外扩张尺寸
impl Dimensions {
    pub fn padding_box(self) -> Rect {
        self.content.expanded_by(self.padding)
    }

    pub fn border_box(self) -> Rect {
        self.padding_box().expanded_by(self.border)
    }

    pub fn margin_box(self) -> Rect {
        self.border_box().expanded_by(self.margin)
    }
}

fn sum<I>(iter: I) -> f32 where I: Iterator<Item=f32> {
    iter.fold(0., |a, b| a + b)
}