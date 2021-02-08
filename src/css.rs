#[derive(Debug)]
pub struct StyleSheet {
  pub rules: Vec<Rule>,
}

// { prop: val } の 1 つか複数のセレクター
#[derive(Debug)]
pub struct Rule {
  pub selectors: Vec<Selector>,
  pub declarations: Vec<Declaration>,
}

#[derive(Debug)]
pub enum Selector {
  Simple(SimpleSelector),
}

// とりあえずシンプルなセレクターを定義（タグ名、id, class）
#[derive(Debug)]
pub struct SimpleSelector {
  pub tag_name: Option<String>,
  pub id: Option<String>,
  pub class: Vec<String>,
}

// 宣言（propName: value のセミコロンで終わるペア）
#[derive(Debug)]
pub struct Declaration {
  pub name: String,
  pub value: Value,
}

// 値
#[derive(Debug, Clone)]
pub enum Value {
  Keyword(String),   // 文字列
  Length(f32, Unit), // 数値
  ColorValue(Color), // カラー値
}

// 単位
#[derive(Debug, Clone)]
pub enum Unit {
  Px,
}

// RGB
#[derive(Debug, Clone)]
pub struct Color {
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub a: u8,
}

pub struct Parser {
  pub pos: usize,
  pub input: String,
}

pub type Specificity = (usize, usize, usize);

impl Selector {
  // 詳細度の計算
  pub fn specificity(&self) -> Specificity {
    let Selector::Simple(ref simple) = *self;
    let a = simple.id.iter().count();
    let b = simple.class.len();
    let c = simple.tag_name.iter().count();
    return (a, b, c);
  }
}

// id が valid か返す
fn valid_identifier_char(c: char) -> bool {
  return match c {
    'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => true,
    _ => false,
  };
}

impl Parser {
  /**
   * html のメソッドおさらい
   */
  fn next_char(&self) -> char {
    return self.input[self.pos..].chars().next().unwrap();
  }
  fn eof(&self) -> bool {
    return self.pos >= self.input.len();
  }
  fn consume_whitespace(&mut self) {
    self.consume_while(char::is_whitespace);
  }
  fn consume_while<F>(&mut self, test: F) -> String
  where
    F: Fn(char) -> bool,
  {
    let mut result = String::new();
    while !self.eof() && test(self.next_char()) {
      result.push(self.consume_char());
    }
    return result;
  }
  fn consume_char(&mut self) -> char {
    let mut iter = self.input[self.pos..].char_indices();
    let (_, cur_char) = iter.next().unwrap();
    let (next_pos, _) = iter.next().unwrap_or((1, ' '));
    self.pos += next_pos;
    return cur_char;
  }

  /**
   * ここから
   */

  fn parse_identifier(&mut self) -> String {
    return self.consume_while(valid_identifier_char)
  }

  fn parse_simple_selector(&mut self) -> SimpleSelector {
    let mut selector = SimpleSelector {
      tag_name: None,
      id: None,          // id は一意なので 1 つ
      class: Vec::new(), // class は複数あるので配列
    };
    while !self.eof() {
      match self.next_char() {
        // ID セレクタ
        '#' => {
          println!("css: found ID Selector");
          self.consume_char();
          selector.id = Some(self.parse_identifier());
        }
        // Class セレクタ
        '.' => {
          println!("css: found class Selector");
          self.consume_char();
          selector.class.push(self.parse_identifier());
        }
        // * セレクタ
        '*' => {
          println!("css: found universal Selector");
          self.consume_char();
        }
        // タグ名
        c if valid_identifier_char(c) => {
          println!("css: found tagName Selector");
          selector.tag_name = Some(self.parse_identifier());
        }
        _ => break,
      }
    }
    return selector;
  }

  // ルール
  fn parse_rule(&mut self) -> Rule {
    return Rule {
      selectors: self.parse_selectors(),
      declarations: self.parse_declarations(),
    };
  }

  // セレクタ
  fn parse_selectors(&mut self) -> Vec<Selector> {
    let mut selectors = Vec::new();
    loop {
      selectors.push(Selector::Simple(self.parse_simple_selector()));
      self.consume_whitespace();
      match self.next_char() {
        // 複数
        ',' => {
          self.consume_char();
          self.consume_whitespace();
        },
        // declaration
        '{' => break, 
        c => panic!("Unexpected character {} in selector list", c),
      }
    }
    selectors.sort_by(|a, b| b.specificity().cmp(&a.specificity()));
    return selectors;
  }

  // 値が float のパーサー
  fn parse_float(&mut self) -> f32 {
    let s = self.consume_while(|c| match c {
      '0'..='9' | '.' => true,  // 数値か小数点のみ
      _ => false
    });
    return s.parse().unwrap();
  }

  // 値が px などのパーサー
  fn parse_unit(&mut self) -> Unit {
    return match &*self.parse_identifier().to_ascii_lowercase() {
      "px" => Unit::Px,
      _ => panic!("unrecognized unit") // 対応していない単位には panic 置いとく
    }
  }

  // color
  fn parse_color(&mut self) -> Value {
    assert_eq!(self.consume_char(), '#');
    Value::ColorValue(Color {
      r: self.parse_hex_pair(),
      g: self.parse_hex_pair(),
      b: self.parse_hex_pair(),
      a: 255,
    })
  }

  // HEX 値
  fn parse_hex_pair(&mut self) -> u8 {
    let s = &self.input[self.pos .. self.pos + 2]; // 2 ずつ rga に取る
    self.pos += 2;
    return u8::from_str_radix(s, 16).unwrap();
  }

  // 値が数値の時のパーサー
  fn parse_length(&mut self) -> Value {
    return Value::Length(self.parse_float(), self.parse_unit());
  }

  // 値
  fn parse_value(&mut self) -> Value {
    match self.next_char() {
      '0'..='9' => self.parse_length(), // 数値
      '#' => self.parse_color(), // カラー値
      _ => Value::Keyword(self.parse_identifier()), // キーワード
    }
  }

  // 宣言
  fn parse_declaration(&mut self) -> Declaration {
    let property_name = self.parse_identifier(); // プロパティ名
    self.consume_whitespace();
    assert_eq!(self.consume_char(), ':'); // :
    self.consume_whitespace();
    let value = self.parse_value(); // 値
    self.consume_whitespace();
    assert_eq!(self.consume_char(), ';'); // ;

    println!("css: found {}: {:?}", property_name, value);

    return Declaration {
      name: property_name,
      value: value,
    };
  }

  // 全宣言
  fn parse_declarations(&mut self) -> Vec<Declaration> {
    assert_eq!(self.consume_char(), '{');
    let mut declarations = Vec::new();
    loop {
      self.consume_whitespace();
      if self.next_char() == '}' {
        // } ならスコープの閉じなので終わり
        self.consume_char();
        break;
      }
      declarations.push(self.parse_declaration())
    }
    return declarations;
  }

  // 全ルール
  fn parse_rules(&mut self) -> Vec<Rule> {
    let mut rules = Vec::new();
    loop {
      self.consume_whitespace();
      if self.eof() {
        break;
      }
      rules.push(self.parse_rule());
    }
    return rules;
  }
}

pub fn parse(source: String) -> StyleSheet {
  let mut parser = Parser { pos: 0, input: source };
  return StyleSheet { rules: parser.parse_rules() }
}