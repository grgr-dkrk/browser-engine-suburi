use std::collections::{HashMap, HashSet};

// Node
#[derive(Debug)]
pub struct Node {
  pub children: Vec<Node>,
  pub node_type: NodeType,
}

// NodeType - テキストか要素が入るとしてのもの
#[derive(Debug)]
pub enum NodeType {
  Text(String),
  Element(ElementData),
}

// 要素のデータ、タグ名と属性名を格納する
pub type AttrMap = HashMap<String, String>;

#[derive(Debug)]
pub struct ElementData {
  pub tag_name: String,
  pub attributes: AttrMap,
}

// ノードを作成するコンストラクタ関数
pub fn text(data: String) -> Node {
  return Node { children: vec![], node_type: NodeType::Text(data) }
}

pub fn elem(name: String, attrs: AttrMap, children: Vec<Node>) -> Node {
  return Node {
    children: children,
    node_type: NodeType::Element(ElementData {
      tag_name: name,
      attributes: attrs,
    })
  }
}

impl ElementData {
  pub fn id(&self) -> Option<&String> {
    return self.attributes.get("id")
  }

  pub fn classes(&self) -> HashSet<&str> {
    return match self.attributes.get("class") {
      Some(classList) => classList.split(' ').collect(),
      None => HashSet::new()
    }
  }
}