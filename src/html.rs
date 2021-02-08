use dom;
use std::collections::HashMap;

struct Parser {
  pos: usize, // 文字列内の現在の位置。usize は C++ の `size_t`
  input: String, // 入力された文字列
}

impl Parser {
  // char の読み取り
  fn next_char(&self) -> char {
    return self.input[self.pos..].chars().next().unwrap()
  }

  // 次の文字が、引数 s で始まるか
  fn starts_with(&self, s: &str) -> bool {
    return self.input[self.pos..].starts_with(s)
  }

  // EOF
  fn eof(&self) -> bool {
    return self.pos >= self.input.len()
  }

  // マルチバイト文字に対応するためのメソッド
  fn consume_char(&mut self) -> char {
    // `char_indices()`で文字列の開始位置を入れる
    let mut iter = self.input[self.pos..].char_indices();
    // 次の char をとる
    let (_, cur_char) = iter.next().unwrap();
    let (next_pos, _) = iter.next().unwrap_or((1, ' '));

    // advance
    self.pos += next_pos;

    println!("html: cur_char:  {}", cur_char);

    // 現在の文字を返す
    return cur_char;
  }

  // 連続する文字列を返すためのメソッド
  fn consume_while<F>(&mut self, test: F) -> String
    // test には bool が入る関数
    where F: Fn(char) -> bool {
      println!("html: consume_while_start");
      let mut result = String::new();

      // EOF でなく、次の char が test の条件を満たす間、`consume_char()` の返り値を追加
      while !self.eof() && test(self.next_char()) {
        result.push(self.consume_char());
      }

      println!("html: consume_while_end");
      return result;
    }

  // スペース文字
  fn consume_whitespace(&mut self) {
    self.consume_while(char::is_whitespace);
  }

  // タグ
  fn parse_tag_name(&mut self) -> String {
    return self.consume_while(|c| match c {
      'a'..='z' | 'A'..='Z' | '0'..='9' => true,
      _ => false
    })
  }

  // テキスト
  fn parse_text(&mut self) -> dom::Node {
    return dom::text(self.consume_while(|c| c != '<'))
  }

  // 属性の値
  fn parse_attr_value(&mut self) -> String {
    let open_quote = self.consume_char();
    assert!(open_quote == '"' || open_quote == '\''); // " か ' が含まれるため
    let value = self.consume_while(|c| c != open_quote);
    assert_eq!(self.consume_char(), open_quote);
    return value;
  }

  // 属性
  fn parse_attr(&mut self) -> (String, String) { // (属性名、値)を返す
    let name = self.parse_tag_name();
    assert_eq!(self.consume_char(), '=');
    let value = self.parse_attr_value();
    return (name, value);
  }

  // 全属性
  fn parse_attributes(&mut self) -> dom::AttrMap {
    let mut attributes = HashMap::new();
    loop {
      self.consume_whitespace(); // スペースは除外
      if self.next_char() == '>' {
        break;
      }
      let (name, value) = self.parse_attr();
      attributes.insert(name, value);
    }
    return attributes;
  }

  // 要素
  fn parse_element(&mut self) -> dom::Node {

    // 開始の開始〜終了
    assert_eq!(self.consume_char(), '<'); // 開始
    let tag_name = self.parse_tag_name(); // タグ名
    let attrs = self.parse_attributes(); // 属性
    assert_eq!(self.consume_char(), '>'); //　終了

    // 子
    let children = self.parse_nodes(); // children

    // 閉じの開始〜終了
    assert_eq!(self.consume_char(), '<'); // 開始
    assert_eq!(self.consume_char(), '/'); // slash
    assert_eq!(self.parse_tag_name(), tag_name); // 開始時とタグ名が一致しているか
    assert_eq!(self.consume_char(), '>'); // 終了

    return dom::elem(tag_name, attrs, children);
  }

  // Node
  fn parse_node(&mut self) -> dom::Node {
    return match self.next_char() {
      '<' => self.parse_element(),
      _ => self.parse_text()
    }
  }

  // 全 Node
  fn parse_nodes(&mut self) -> Vec<dom::Node> {
    let mut nodes = Vec::new();
    loop {
      println!("html: nodes_start");
      self.consume_whitespace(); // スペースは除外
      if self.eof() || self.starts_with("</") {
        println!("html: nodes_end");
        break;
      }
      nodes.push(self.parse_node());
    }
    return nodes;
  }
}

// Parse
pub fn parse(source: String) -> dom::Node {
  println!("html: start");
  let mut nodes = Parser { pos: 0, input: source }.parse_nodes();
  println!("html: end");

  if nodes.len() == 1 {
    return nodes.swap_remove(0) 
  } else {
    return dom::elem("html".to_string(), HashMap::new(), nodes)
  }
}