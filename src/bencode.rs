//! bencode decode/encode
//! 
//! Most messges transmitted by peers are encoded in bencode format. 
//! The mod provide the bencode data strucutres and functions
//! to convert between bencode data strucutres and byte stream.
//! 
//! # Design
//! 
//! # Example

use anyhow::{anyhow, bail, Error};
use pretty::{Doc, RcDoc};
use std::{
    collections::HashMap,
    fmt::{self, Display},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BDict(HashMap<Vec<u8>, BItem>);

impl From<HashMap<Vec<u8>, BItem>> for BDict {
    fn from(value: HashMap<Vec<u8>, BItem>) -> Self {
        BDict(value)
    }
}

impl From<BDict> for HashMap<Vec<u8>, BItem> {
    fn from(value: BDict) -> Self {
        value.0
    }
}

// Trait to adopt TryFrom::Error to anyhow::Error
pub trait ExactAnyhowError {
    fn id(self) -> anyhow::Error;
}

impl ExactAnyhowError for anyhow::Error {
    fn id(self) -> anyhow::Error {
        self
    }
}

impl BDict {
    pub fn remove<T>(&mut self, key: &str) -> anyhow::Result<T>
    where
        T: TryFrom<BItem>,
        <T as TryFrom<BItem>>::Error: ExactAnyhowError,
    {
        let item: BItem = self
            .0
            .remove(key.as_bytes())
            .ok_or_else(|| anyhow!(format!("no entry for {}", key)))?;
        T::try_from(item).map_err(|e| e.id())
    }
}

impl TryFrom<BItem> for Vec<BItem> {
    type Error = Error;

    fn try_from(value: BItem) -> anyhow::Result<Vec<BItem>> {
        match value {
            BItem::List(l) => Ok(l),
            _ => Err(anyhow!("Should be List!, but got {}", &value)),
        }
    }
}

impl TryFrom<BItem> for i64 {
    type Error = Error;

    fn try_from(value: BItem) -> anyhow::Result<i64> {
        match value {
            BItem::Int(i) => Ok(i),
            _ => Err(anyhow!("Should be Integer!, but got {}", &value)),
        }
    }
}

impl TryFrom<BItem> for BDict {
    type Error = Error;

    fn try_from(value: BItem) -> anyhow::Result<BDict> {
        match value {
            BItem::Dict(d) => Ok(d),
            _ => Err(anyhow!("Should be Dict!, but got {}", &value)),
        }
    }
}

impl TryFrom<BItem> for String {
    type Error = Error;

    fn try_from(value: BItem) -> anyhow::Result<String> {
        match value {
            BItem::String(s) => String::from_utf8(s).map_err(|e| Error::new(e)),
            _ => Err(anyhow!("Should be String!, but got {}", &value)),
        }
    }
}

impl TryFrom<BItem> for Vec<u8> {
    type Error = Error;

    fn try_from(value: BItem) -> anyhow::Result<Vec<u8>> {
        match value {
            BItem::String(s) => Ok(s),
            _ => Err(anyhow!("Should be Vec<u8>!, but got {}", &value)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BItem {
    String(Vec<u8>),
    Int(i64),
    List(Vec<BItem>),
    Dict(BDict),
}

impl BItem {
    /// Return a pretty printed format of self.
    pub fn to_doc(&self) -> RcDoc<()> {
        match self {
            BItem::String(s) => {
                let s = String::from_utf8(s.clone()).unwrap_or("Not UTF-8 encoded".to_owned());
                RcDoc::text(s)
            }
            BItem::Int(i) => RcDoc::text(i.to_string()),
            BItem::List(items) => RcDoc::text("[")
                .append(
                    RcDoc::intersperse(items.into_iter().map(|x| x.to_doc()), Doc::line()).nest(1),
                )
                .append(RcDoc::text("]")),
            BItem::Dict(dict) => {
                let item_docs = dict.0.iter().map(|(key, val)| {
                    let key =
                        String::from_utf8(key.clone()).unwrap_or("Not UTF-8 encoded".to_owned());
                    let val_doc = val.to_doc();
                    RcDoc::text(format!("{} : ", key)).append(val_doc)
                });
                let dict_doc = RcDoc::intersperse(item_docs, Doc::line());
                RcDoc::text("{")
                    .append(RcDoc::line())
                    .append(dict_doc)
                    .append(RcDoc::line())
                    .append(RcDoc::text("}"))
                    .nest(2)
            }
        }
    }
}

impl Display for &BItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_doc().pretty(80))
    }
}

struct Decoder<'a> {
    raw: &'a [u8],
    ptr: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(raw: &'a [u8]) -> Decoder<'a> {
        Decoder { raw, ptr: 0 }
    }

    pub fn decode(&mut self) -> anyhow::Result<BItem> {
        match self.validate_peek()? {
            b'i' => self.deseri_int(),
            b'l' => self.deseri_list(),
            b'd' => self.deseri_dict(),
            _ => self.deseri_string(),
        }
    }

    #[inline]
    fn peek(&self) -> Option<u8> {
        self.raw.get(self.ptr as usize).copied()
    }

    #[inline]
    fn validate_peek(&self) -> anyhow::Result<u8> {
        self.peek().ok_or(anyhow!("Invalid peek"))
    }

    #[inline]
    fn pop(&mut self) -> Option<u8> {
        let head = self.peek();
        match head {
            None => None,
            Some(b) => {
                self.ptr += 1;
                Some(b)
            }
        }
    }

    #[inline]
    fn validate_pop(&mut self) -> anyhow::Result<u8> {
        self.pop().ok_or(anyhow!("Invalid pop"))
    }

    #[inline]
    fn move_ptr(&mut self) {
        self.ptr += 1;
    }

    fn deseri_string(&mut self) -> anyhow::Result<BItem> {
        let len = match self.consume_int(b':')? {
            BItem::Int(i) => i,
            _ => panic!(),
        };
        let mut res: Vec<u8> = Vec::with_capacity(len as usize);
        for _ in 0..len {
            res.push(self.validate_pop()?);
        }
        Ok(BItem::String(res))
    }

    #[inline]
    fn deseri_int(&mut self) -> anyhow::Result<BItem> {
        self.move_ptr();
        let res = self.consume_int(b'e')?;
        Ok(res)
    }

    #[inline]
    fn consume_int(&mut self, end: u8) -> anyhow::Result<BItem> {
        let mut res: i64 = 0;
        while self.peek().ok_or(anyhow!(".."))? != end {
            let digit = (self.validate_pop()? - b'0') as i64;
            if digit >= 10 {
                bail!(format!(
                    "Err while decodeing {:?}, position {:?} should be a 10-based digit",
                    self.raw, self.ptr
                ));
            }
            res = res * 10 + digit;
        }
        self.move_ptr();
        Ok(BItem::Int(res))
    }

    fn deseri_list(&mut self) -> anyhow::Result<BItem> {
        self.move_ptr();
        let mut list: Vec<BItem> = Vec::new();
        while self.validate_peek()? != b'e' {
            list.push(self.decode()?);
        }
        self.move_ptr();
        Ok(BItem::List(list))
    }

    fn deseri_dict(&mut self) -> anyhow::Result<BItem> {
        self.move_ptr();
        // Keys must be strings and appear in sorted order (sorted as raw strings, not alphanumerics)
        // Not check here
        let mut dict: HashMap<Vec<u8>, BItem> = HashMap::new();
        while self.validate_peek()? != b'e' {
            let key = match self.deseri_string()? {
                BItem::String(s) => s,
                _ => bail!("key must be a string"),
            };
            let val = self.decode()?;
            dict.insert(key, val);
        }
        self.move_ptr();
        Ok(BItem::Dict(BDict(dict)))
    }
}

impl BItem {
    pub fn seri_decons(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::new();
        match &self {
            &BItem::String(s) => {
                res.extend_from_slice(s.len().to_string().as_bytes());
                res.push(b':');
                res.extend_from_slice(s);
                res
            }
            &BItem::Int(i) => {
                res.push(b'i');
                res.extend_from_slice(i.to_string().as_bytes());
                res.push(b'e');
                res
            }
            &BItem::List(l) => {
                res.push(b'l');
                for item in l {
                    res.extend(item.seri_decons());
                }
                res.push(b'e');
                res
            }
            &BItem::Dict(d) => {
                res.push(b'd');
                let mut keys: Vec<&Vec<u8>> = d.0.keys().collect();
                keys.sort();
                for key in keys {
                    res.extend_from_slice(key.len().to_string().as_bytes());
                    res.push(b':');
                    res.extend(key);
                    res.extend(d.0[key].seri_decons());
                }
                res.push(b'e');
                res
            }
        }
    }

    pub fn deseri_cons(raw: &[u8]) -> anyhow::Result<Self> {
        let mut decoder = Decoder::new(raw);
        decoder.decode()
    }
}

#[cfg(test)]
mod test {

    macro_rules! bvec {
        ($s:expr) => {
            $s.as_bytes().to_owned()
        };
    }

    use std::collections::HashMap;

    use super::BDict;
    use super::BItem;

    #[test]
    fn seri_decons_test() {
        let bitem = BItem::Dict(BDict(HashMap::from([
            (bvec!("a"), BItem::Int(9)),
            (bvec!("b"), BItem::String(bvec!("hello"))),
            (bvec!("c"), BItem::List(vec![BItem::Int(9), BItem::Int(8)])),
        ])));
        let expected = "d1:ai9e1:b5:hello1:cli9ei8eee".as_bytes();
        assert_eq!(bitem.seri_decons(), expected);
    }

    #[test]
    fn wtr() {
        let raw = "d8:completei4e10:downloadedi0e10:incompletei3e8:intervali1800e12:min intervald8:completei4e10:downloadedi0e10:incompletei3e8:intervali1800e12:min intervali1800eee".as_bytes();
        let result = BItem::deseri_cons(raw).unwrap();
        println!("{}", &result);
    }

    #[test]
    fn deseri_cons_test() {
        let raw = "d1:ai9e1:b5:hello1:cli9ei8eee".as_bytes();
        let expected = BItem::Dict(BDict(HashMap::from([
            (bvec!("a"), BItem::Int(9)),
            (bvec!("b"), BItem::String(bvec!("hello"))),
            (bvec!("c"), BItem::List(vec![BItem::Int(9), BItem::Int(8)])),
        ])));
        assert_eq!(BItem::deseri_cons(raw).unwrap(), expected);
    }
}
