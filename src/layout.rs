pub use self::BoxType::{AnonymousBlock, BlockNode, InlineNode};
use css::Unit::Px;
use css::Value;
use css::Value::{Keyword, Length};
use std::default::Default;
use style::{StyledNode, Display};

#[derive(Clone, Copy, Default, Debug)]
pub struct Dimensions {
  pub content: Rect,
  pub padding: EdgeSizes,
  pub border: EdgeSizes,
  pub margin: EdgeSizes,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Rect {
  pub x: f32,
  pub y: f32,
  pub width: f32,
  pub height: f32,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct EdgeSizes {
  pub left: f32,
  pub right: f32,
  pub top: f32,
  pub bottom: f32,
}

// レイアウト内容
#[derive(Debug)]
pub struct LayoutBox<'a> {
  pub dimensions: Dimensions,
  pub box_type: BoxType<'a>,
  pub children: Vec<LayoutBox<'a>>,
}

// block か、inline か
// anonymous は DOM ツリーの Node に関連づけられていないブロック
// (inline 要素がふたつ以上集まり暗黙的にできるブロックなど)
#[derive(Debug)]
pub enum BoxType<'a> {
  BlockNode(&'a StyledNode<'a>),
  InlineNode(&'a StyledNode<'a>),
  AnonymousBlock,
}

pub fn layout_tree<'a>(node: &'a StyledNode<'a>, mut containing_block: Dimensions) -> LayoutBox<'a> {
  containing_block.content.height = 0.0;
  let mut root_box = build_layout_tree(node);
  root_box.layout(containing_block);
  return root_box;
}

// レイアウトツリーの作成
fn build_layout_tree<'a>(style_node: &'a StyledNode<'a>) -> LayoutBox<'a> {
  // ルートのレイアウトを格納
  let mut root = LayoutBox::new(match style_node.display() {
    Block => BlockNode(style_node),
    Inline => InlineNode(style_node),
    DisplayNone => panic!("Root node has display: none."),
  });

  // 子のレイアウトを格納
  for child in &style_node.children {
    match child.display() {
      Block => root.children.push(build_layout_tree(child)),
      Inline => root
        .get_inline_container()
        .children
        .push(build_layout_tree(child)),
      DisplayNone => {} // 何もしない
    }
  }

  return root;
}

impl Rect {
  // rect を出す
  pub fn expanded_by(self, edge: EdgeSizes) -> Rect {
    Rect {
        x: self.x - edge.left,
        y: self.y - edge.top,
        width: self.width + edge.left + edge.right,
        height: self.height + edge.top + edge.bottom,
    }
  }
}


impl Dimensions {
  // padding 部分の rect を出す
  pub fn padding_box(self) -> Rect {
    self.content.expanded_by(self.padding)
  }

  // border 部分の rect を出す
  pub fn border_box(self) -> Rect {
    self.padding_box().expanded_by(self.border)
  }

  // margin 部分の rect を出す
  pub fn margin_box(self) -> Rect {
    self.border_box().expanded_by(self.margin)
  }
}

impl<'a> LayoutBox<'a> {
  fn new(box_type: BoxType) -> LayoutBox {
    LayoutBox {
      box_type: box_type,
      dimensions: Default::default(),
      children: Vec::new(),
    }
  }

  fn get_style_node(&self) -> &'a StyledNode<'a> {
    match self.box_type {
      BlockNode(node) | InlineNode(node) => node,
      AnonymousBlock => panic!("Anonymous block box has no style node"),
    }
  }
}

impl<'a> LayoutBox<'a> {
  fn layout(&mut self, containing_block: Dimensions) {
    match self.box_type {
      BlockNode(_) => self.layout_block(containing_block),
      InlineNode(_) | AnonymousBlock => {} // TODO
    }
  }

  fn layout_block(&mut self, containing_block: Dimensions) {
    self.calculate_block_width(containing_block);
    self.calculate_block_position(containing_block);
    self.layout_block_children();
    self.calculate_block_height();
  }

  fn calculate_block_width(&mut self, containing_block: Dimensions) {
    let style = self.get_style_node();

    // width(default: auto)
    let auto = Keyword("auto".to_string());
    let mut width = style.value("width").unwrap_or(auto.clone());

    // margin, border, padding(default: 0)
    let zero = Length(0.0, Px);
    let mut margin_left = style.lookup("margin-left", "margin", &zero);
    let mut margin_right = style.lookup("margin-right", "margin", &zero);

    let mut border_left = style.lookup("border-left-width", "border-width", &zero);
    let mut border_right = style.lookup("border-right-width", "border-width", &zero);

    let padding_left = style.lookup("padding-left", "padding", &zero);
    let padding_right = style.lookup("padding-right", "padding", &zero);

    let total = sum(
      [
        &margin_left,
        &margin_right,
        &border_left,
        &border_right,
        &padding_left,
        &padding_right,
        &width,
      ]
      .iter()
      .map(|v| v.to_px()),
    );

    if width != auto && total > containing_block.content.width {
      if margin_left == auto {
        margin_left = Length(0.0, Px);
      }
      if margin_right == auto {
        margin_right = Length(0.0, Px);
      }
    }
    let underflow = containing_block.content.width - total;

    match (width == auto, margin_left == auto, margin_right == auto) {
      (false, false, false) => margin_right = Length(margin_right.to_px() + underflow, Px),
      (false, false, true) => {
        margin_right = Length(underflow, Px);
      }
      (false, true, false) => {
        margin_left = Length(underflow, Px);
      }
      (true, _, _) => {
        if margin_left == auto {
          margin_left = Length(0.0, Px);
        }
        if margin_right == auto {
          margin_right = Length(0.0, Px);
        }
        if underflow >= 0.0 {
          width = Length(underflow, Px);
        } else {
          width = Length(0.0, Px);
          margin_right = Length(margin_right.to_px() + underflow, Px);
        }
      }
      (false, true, true) => {
        margin_left = Length(underflow / 2.0, Px);
        margin_right = Length(underflow / 2.0, Px);
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

    let zero = Length(0.0, Px);

    d.margin.top = style.lookup("margin-top", "margin", &zero).to_px();
    d.margin.bottom = style.lookup("margin-bottom", "margin", &zero).to_px();

    d.border.top = style
      .lookup("border-top-width", "border-width", &zero)
      .to_px();
    d.border.bottom = style
      .lookup("border-bottom-width", "border-width", &zero)
      .to_px();

    d.padding.top = style.lookup("padding-top", "padding", &zero).to_px();
    d.padding.bottom = style.lookup("padding-bottom", "padding", &zero).to_px();

    d.content.x = containing_block.content.x + d.margin.left + d.border.left + d.padding.left;
    d.content.y = containing_block.content.height
      + containing_block.content.y
      + d.margin.top
      + d.border.top
      + d.padding.top;
  }

  fn layout_block_children(&mut self) {
    let d = &mut self.dimensions;
    for child in &mut self.children {
      child.layout(*d);
      d.content.height = d.content.height + child.dimensions.margin_box().height;
    }
  }

  fn calculate_block_height(&mut self) {
    if let Some(Length(h, Px)) = self.get_style_node().value("height") {
        self.dimensions.content.height = h;
    }
  }

  fn get_inline_container(&mut self) -> &mut LayoutBox<'a> {
    match self.box_type {
      // inline の子が含まれる Node はそれを含む anonymous ブロックを作成
      InlineNode(_) | AnonymousBlock => self,
      BlockNode(_) => {
        match self.children.last() {
          Some(&LayoutBox {
            box_type: AnonymousBlock,
            ..
          }) => {}
          // BlockNode に複数の inline が並ぶ場合、新しい anonymousBlock を作成
          _ => self.children.push(LayoutBox::new(AnonymousBlock)),
        }
        self.children.last_mut().unwrap()
      }
    }
  }
}

fn sum<I>(iter: I) -> f32
where
  I: Iterator<Item = f32>,
{
  iter.fold(0., |a, b| a + b)
}
