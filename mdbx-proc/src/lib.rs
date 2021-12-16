#![feature(proc_macro_span)]

extern crate proc_macro;
use proc_macro::{token_stream::IntoIter, TokenStream};
use std::collections::HashMap;
use std::fmt::Write;
use std::iter::IntoIterator;

#[derive(Debug)]
struct Cls {
  cls: String,
  map: HashMap<String, String>,
}

impl Cls {
  fn new(t: &impl ToString) -> Self {
    Cls {
      cls: t.to_string(),
      map: HashMap::new(),
    }
  }
}

fn parse(mut iter: IntoIter) -> Vec<Cls> {
  let mut vec = vec![];
  if let Some(t) = iter.next() {
    let mut cls = Cls::new(&t);
    let span = t.span().start();
    let mut line = span.line;
    let mut indent_base = span.column;
    let mut key = String::new();
    let mut is_key = true;
    for t in iter {
      let span = t.span().start();
      if span.line != line {
        line = span.line;
        let column = span.column;
        if column <= indent_base {
          vec.push(cls);
          cls = Cls::new(&t);
          indent_base = column;
          is_key = false;
        } else {
          key = t.to_string();
          cls.map.insert(key.clone(), "".into());
          is_key = true;
        }
      } else if is_key {
        cls.map.insert(
          key.clone(),
          cls.map.get(&key).unwrap().to_owned() + &t.to_string(),
        );
      }
    }
    vec.push(cls);
  }
  vec
}

const ONE: &str = "One";
const DUP: &str = "Dup";

const LITTLE_ENDIAN: bool = cfg!(target_endian = "little");

macro_rules! _static {
  ($x:ident) => {
    if ["Bin", "Str"].contains(&$x.as_str()) {
      $x = format!("{}<'static>", $x);
    }
  };
}

#[proc_macro]
pub fn mdbx(input: TokenStream) -> TokenStream {
  let env;
  let mut iter = input.into_iter();
  let mut code = String::new();

  if let Some(t) = iter.next() {
    env = t.to_string();
    let vec = parse(iter);
    for i in vec {
      let map = i.map;

      macro_rules! get {
        ($key: ident) => {
          get!($key, "mdbx::r#type::Bin<'static>")
        };
        ($key: ident, $default: expr) => {
          map
            .get(stringify!($key))
            .map_or($default, String::as_str)
            .to_string()
        };
      }

      let mut key = get!(key);
      let mut val = get!(val);

      let flag = &get!(flag, "DB_DEFAULTS");
      let mut flag = flag.split('|').collect::<Vec<&str>>();

      let kind = if flag.contains(&"DUPSORT") { DUP } else { ONE };

      if key == "u32" || key == "u64" || key == "usize" {
        flag.push("INTEGERKEY");
      } else if ["u16", "u128", "i16", "i32", "i64", "i128", "isize"].contains(&key.as_str()) {
        if LITTLE_ENDIAN {
          flag.push("REVERSEKEY");
        }
      } else {
        _static!(key)
      };

      _static!(val);

      if kind == DUP
        && [
          "usize", "u128", "u64", "u32", "u16", "u8", "isize", "i128", "i64", "i32", "i16", "i8",
        ]
        .contains(&val.as_str())
      {
        flag.push("DUPFIXED");
        if val == "u32" || val == "u64" || val == "usize" {
          flag.push("INTEGERDUP")
        } else if LITTLE_ENDIAN {
          flag.push("REVERSEDUP");
        }
      }

      flag.sort_unstable();
      flag.dedup();

      let flag = flag
        .into_iter()
        .map(|x| format!("mdbx::flag::DB::MDBX_{x}"))
        .collect::<Vec<String>>()
        .join("|");

      //println!("{} flag {}", i.cls,flag.replace("mdbx::flag::DB::",""));

      writeln!(
        &mut code,
        "mdbx::Db!({env},{},mdbx::db::kind::{kind},{key},{val},{flag});\n",
        i.cls
      )
      .unwrap();
    }
  }

  //println!("{code}");
  let code: TokenStream = code.parse().unwrap();
  code
}
