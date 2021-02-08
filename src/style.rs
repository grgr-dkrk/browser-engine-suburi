use std::collections::HashMap;
use dom::{Node, NodeType, ElementData};
use css::{StyleSheet, Rule, Selector, SimpleSelector, Value, Specificity};

/**
 * HTML Parser + CSS Parser から生成した DOM ツリー, Rules ツリーから Style ツリーを生成するところ
 */

type PropertyMap = HashMap<String, Value>;
type MatchedRule<'a> = (Specificity, &'a Rule);

#[derive(Debug)]
pub struct StyledNode<'a> {
  pub node: &'a Node,
  pub specified_values: PropertyMap,
  pub children: Vec<StyledNode<'a>>,
}

// セレクターマッチング（要素を見て simple_selector を探すだけ）
fn matches(elem: &ElementData, selector: &Selector) -> bool {
  return match *selector {
    Selector::Simple(ref simple_selector) => matches_simple_selector(elem, simple_selector)
  }
}

// 要素に対して一致するスタイルを探す(TODO: ハッシュ探索で高速化できる)
fn matching_rules<'a>(elem: &ElementData, stylesheet: &'a StyleSheet) -> Vec<MatchedRule<'a>> {
  return stylesheet.rules.iter().filter_map(|rule| match_rule(elem, rule)).collect();
}
fn match_rule<'a>(elem:&ElementData, rule: &'a Rule) -> Option<MatchedRule<'a>> {
  return rule.selectors.iter()
    .find(|selector| matches(elem, *selector))
    .map(|selector| (selector.specificity(), rule))
}

// セレクターが要素と一致するかどうか調べる
fn matches_simple_selector(elem: &ElementData, selector: &SimpleSelector) -> bool {

  // タグ名
  if selector.tag_name.iter().any(|name| elem.tag_name != *name) {
    return false;
  }

  // ID
  if selector.id.iter().any(|id| elem.id() != Some(id)) {
    return false;
  }

  // Class
  let elem_classes = elem.classes();
  if selector.class.iter().any(|class| !elem_classes.contains(&**class)) {
    return false;
  }

  return true;
}

// 要素にスタイルを適用して、指定されたスタイルを返す
fn specified_values(elem: &ElementData, stylesheet: &StyleSheet) -> PropertyMap {
  let mut values = HashMap::new();
  let mut rules = matching_rules(elem, stylesheet);

  rules.sort_by(|&(a, _), &(b, _)| a.cmp(&b)); // 詳細度の高いルールが後ろに行く（上書きされる）
  for (_, rule) in rules {
    for declaration in &rule.declarations {
      values.insert(declaration.name.clone(), declaration.value.clone());
    }
  }
  return values;
}

// ルートとなる Node から StyleSheet を適用して、 Style ツリーを生成する。
pub fn style_tree<'a>(root: &'a Node, stylesheet: &'a StyleSheet) -> StyledNode<'a> {
  return StyledNode {
    node: root,
    specified_values: match root.node_type {
      NodeType::Element(ref elem) => specified_values(elem, stylesheet),
      NodeType::Text(_) => HashMap::new(),
    },
    children: root.children.iter().map(|child| style_tree(child, stylesheet)).collect(),
  }
}

