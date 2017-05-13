
extern crate byteorder;

use byteorder::LittleEndian;
use byteorder::WriteBytesExt;
use byteorder::ByteOrder;

use std::collections::BTreeMap;
use std::io::Write;
use std::io::Read;
use std::fmt;

pub struct U64ObjectBuilder {
    map: BTreeMap<u64, Value>,
    key: Option<u64>,
    parent: Option<Box<U64ObjectBuilder>>
}

impl U64ObjectBuilder {
    pub fn new() -> Box<U64ObjectBuilder> {
        let builder = U64ObjectBuilder {
            map: BTreeMap::new(),
            key: None,
            parent: None
        };

        Box::new(builder)
    }

    pub fn set(mut self:Self, key:u64, value:Value) -> U64ObjectBuilder {
        self.map.insert(key, value);
        self
    }

    pub fn start_u64_obj(self, key:u64) -> Box<U64ObjectBuilder> {
        let builder = U64ObjectBuilder {
            map: BTreeMap::new(),
            key: Some(key),
            parent: Some(Box::new(self))
        };

        Box::new(builder)
    }

    pub fn finish(mut self) -> Box<U64ObjectBuilder> {
        let mut parent =  match self.parent {
            None => panic!("error handling goes here?"),
            Some(p) => {
                p
                } };
        self.parent = None; 

        parent = Box::new(parent.set(self.key.unwrap(), self.value()));

        parent
    }

    pub fn value(self) -> Value {
        Value::U64Object(self.map)
    }
}


#[derive(PartialEq, Debug)]
pub enum Value {
//    Byte(u8),
//    I32(i32),
//    U32(u32),
//    I64(i64),
    U64(u64),
//    Char(u8),
//    F32(f32),
    F64(f64),
//    Date(u64),
//    ByteSeq(Vec<u8>),
//    I32Seq(Vec<i32>),
//    U32Seq(Vec<u32>),
//    I64Seq(Vec<i64>),
//    U64Seq(Vec<u64>),
    CharSeq(String),
//    DateSeq(Vec<u64>),

//    I64Object(BTreeMap<i64, Value>),
    U64Object(BTreeMap<u64, Value>),
//    DateObject(BTreeMap<u64, Value>),

//    ByteSeqObject(BTreeMap<Vec<u8>, Value>),
//    I64SeqObject(BTreeMap<Vec<i64>, Value>),
//    U64SeqObject(BTreeMap<Vec<u64>, Value>),
    CharSeqObject(BTreeMap<String, Value>),

}

pub enum ReadError {
    BadMagic
}



//const START_U8: u8 = 0;
//const START_I32: u8 = 1;
//const START_U32: u8 = 2;
//const START_I64: u8 = 3;
pub const START_U64: u8 = b'\x04';
//const START_CHAR: u8 = 5;
//const START_F32: u8 = 6;
const START_F64: u8 = 7;
//const START_DATE: u8 = 8;

//pub const START_BYTE_SEQ: u8 = 32;
//pub const START_I32_SEQ: u8 = 33;
//pub const START_U32_SEQ: u8 = 34;
//pub const START_I64_SEQ: u8 = 35;
//pub const START_U64_SEQ: u8 = 36;
pub const START_CHAR_SEQ: u8 = b'\x25';
//pub const START_F32_SEQ: u8 = 38;
//pub const START_F64_SEQ: u8 = 39;
//pub const START_DATE_SEQ: u8 = 40;


//pub const START_I64_OBJ: u8 = 131;
pub const START_U64_OBJ: u8 = b'\x84';
//pub const START_DATE_OBJ: u8 = 134;
pub const END_OBJ: u8 = 200;

//pub const START_BYTE_SEQ_OBJ: u8 = 135;
//pub const START_I64_SEQ_OBJ: u8 = 138;
//pub const START_U64_SEQ_OBJ: u8 = 139;
pub const START_CHAR_SEQ_OBJ: u8 = 140;
//pub const START_DATE_SEQ_OBJ: u8 = 141;
pub const MAGIC:&'static str = "HERO BIN";

pub fn write(v:&Value, w:&mut Write) {
    w.write(MAGIC.as_bytes()).unwrap();
    write_internal(v,w);
}

// TODO Implement remaining types
// TODO Improve the error messages
// TODO Add an output/streamy trait (ToHeroValue)
// TODO Add serialization

fn write_internal(v:&Value, w:&mut Write) {
    match v {
        &Value::U64(v) => {   
            w.write_u8(START_U64).unwrap();
            w.write_u64::<LittleEndian>(v).unwrap();
        },
        &Value::F64(v) => {
            w.write_u8(START_F64).unwrap();
            w.write_f64::<LittleEndian>(v).unwrap();
        }
        &Value::CharSeq(ref string) => {
            w.write_u8(START_CHAR_SEQ).unwrap();
            w.write_u64::<LittleEndian>(string.len() as u64).unwrap();
            w.write_all(string.as_bytes()).unwrap();
        },
        &Value::CharSeqObject(ref map) => {
            w.write_u8(START_CHAR_SEQ_OBJ).unwrap();

            for (key, value) in map.iter() {
                w.write_u8(START_CHAR_SEQ).unwrap();
                w.write_u64::<LittleEndian>(key.len() as u64).unwrap();
                w.write_all(key.as_bytes()).unwrap();
                write_internal(&value, w);
            }

            w.write_u8(END_OBJ).unwrap();
        },
        &Value::U64Object(ref map) => {
            w.write_u8(START_U64_OBJ).unwrap();

            for (key, value) in map.iter() {
                w.write_u8(START_U64).unwrap();
                w.write_u64::<LittleEndian>(*key).unwrap();
                write_internal(&value, w);
            }

            w.write_u8(END_OBJ).unwrap();
        }
    }
}

fn fmt_internal(val:&Value, f: &mut fmt::Formatter, depth: usize) -> fmt::Result {
    match val {
        &Value::U64(ref v) => {
            write!(f, "{}", v)
        },
        &Value::F64(ref v) => {
            write!(f, "{}", v)
        },
        &Value::CharSeq(ref v) => {
            write!(f, "{}", v)
        },
        &Value::CharSeqObject(ref v) => {
            try!(pad(f, depth*4));
            try!(writeln!(f, "{{"));
            
            for (key, value) in v {
                try!(pad(f, (depth+1) * 4));
                try!(write!(f, "\"{}\":", key));
                try!(fmt_internal(value, f, depth + 1));
                try!(writeln!(f, ""));
            }
            
            try!(pad(f, depth*4));
            try!(writeln!(f, "}}"));

            Ok(())
        },
        &Value::U64Object(ref v) => {
            try!(pad(f, depth*4));
            try!(writeln!(f, "{{"));
            
            for (key, value) in v {
                try!(pad(f, (depth+1) * 4));
                try!(write!(f, "{}:", key));
                try!(fmt_internal(value, f, depth + 1));
                try!(writeln!(f, ""));
            }
            
            try!(pad(f, depth*4));
            try!(writeln!(f, "}}"));

            Ok(())
        }
    }
}

fn pad(f: &mut fmt::Formatter, spaces: usize) -> fmt::Result {
    for _ in 0..spaces {
        try!(write!(f, " "));
    }

    Ok(())
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_internal(self, f, 0)
    }
}

pub fn from_bytes <R: Read>(read: &mut R) -> Result<Value, ()> {
    let magic = MAGIC.as_bytes();
    let mut buf:[u8; 8] = [0; 8];
    
    read.read_exact(&mut buf).unwrap(); // TODO better error
    for pos in 0..MAGIC.len() {
        if magic[pos] != buf[pos] {
            return Err(())
        }
    }
    from_bytes_internal_r(read)
}

fn from_bytes_internal_r<R: Read>(read: &mut R) -> Result<Value, ()> {
    let mut type_buf:[u8; 1] = [0; 1];
    read.read_exact(&mut type_buf).unwrap(); // TODO better error

    match type_buf[0] {
        START_U64 => {
            let mut u64_buf:[u8; 8] = [0; 8];
            read.read_exact(&mut u64_buf).unwrap(); // TODO better error
            let value = LittleEndian::read_u64(&u64_buf);

            return Ok(Value::U64(value))
        },
        START_CHAR_SEQ => {
            let mut len_buf:[u8; 8] = [0; 8];
            read.read_exact(&mut len_buf).unwrap();
            let len = LittleEndian::read_u64(&len_buf);

            let mut str_buf: Vec<u8> = Vec::new();
            let mut small_buf:[u8; 1] = [0; 1];
            for _ in 0..len {
                read.read_exact(&mut small_buf).unwrap();
                str_buf.push(small_buf[0]);
            }
            let s = String::from_utf8(str_buf).unwrap();

            Ok(Value::CharSeq(s))
        },
        START_U64_OBJ => {
            let mut map = BTreeMap::new();

            loop {
                let mut key_type:[u8; 1] = [0; 1];
                read.read_exact(&mut key_type).unwrap();
                if key_type[0] == START_U64 {
                    let mut key_buf:[u8; 8] = [0; 8];
                    read.read_exact(&mut key_buf).unwrap();
                    let key = LittleEndian::read_u64(&key_buf);
                    let value = from_bytes_internal_r(read);
                    map.insert(key, value.unwrap());
                } else if key_type[0] == END_OBJ {
                    break;
                } else {
                    panic!();
                }        
            }

            Ok(Value::U64Object(map))
        },
        START_CHAR_SEQ_OBJ => {
            let mut map = BTreeMap::new();

            loop {
                let mut key_type:[u8; 1] = [0; 1];
                read.read_exact(&mut key_type).unwrap();
                if key_type[0] == START_CHAR_SEQ_OBJ {
                    let key_value = from_bytes_internal_r(read).unwrap();
                    let key = match key_value {
                        Value::CharSeq(s) => s,
                        _ => {panic!();}
                    };

                    let value = from_bytes_internal_r(read).unwrap();

                    map.insert(key, value);
                } else if key_type[0] == END_OBJ {
                    break;
                } else {
                    panic!();
                }
            }

            Ok(Value::CharSeqObject(map))
        },
        _ => {
            panic!();
        }        
    } 
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use byteorder::LittleEndian;
    use byteorder::WriteBytesExt;

    #[test]
    fn test_magic_len() {
        assert_eq!(MAGIC.as_bytes().len(), 8);
    }

    #[test]
    fn test_u64_roundtrip() {
        let v = Value::U64(99);
        
        let mut buf = Vec::new();
        write(&v, &mut buf);
        println!("{:?}", buf);
        let v2 = from_bytes(&mut Cursor::new(&buf)).unwrap();
        if let Value::U64(new_value) = v2 {
            assert_eq!(99, new_value);
        } else {
            assert!(false);
        }
        
    }

    #[test]
    fn test_charseq_roundtrip() {
        let v = Value::CharSeq(String::from("hello world"));
        
        let mut buf = Vec::new();
        write(&v, &mut buf);
        let v2 = from_bytes(&mut Cursor::new(&buf)).unwrap();
        if let Value::CharSeq(new_value) = v2 {
            assert_eq!("hello world", new_value);
        } else {
            assert!(false);
        }
        
    }

    #[test]
    fn test_u64obj_roundtrip() {
        let v = U64ObjectBuilder::new()
            .set(4, Value::U64(12))
            .start_u64_obj(5)
                .set(1, Value::U64(2))
                .start_u64_obj(2)
                    .set(2, Value::U64(3))
                    .finish()
                .finish()
            .value();

        let mut buf = Vec::new();
        write(&v, &mut buf);
        let v2 = from_bytes(&mut Cursor::new(&buf)).unwrap();

        assert_eq!(v, v2);
    }

    #[test]
    fn test_u64() {
        let v = Value::U64(12);

        let mut buf = Vec::new();
        write(&v, &mut buf);
        
        let mut expected:Vec<u8> = Vec::new();
        expected.extend_from_slice(MAGIC.as_bytes());
        expected.push(START_U64);
        expected.write_u64::<LittleEndian>(12).unwrap();

        assert_eq!(buf, expected);
    }

   #[test]
    fn test_u64obj() {
        let v = U64ObjectBuilder::new()
            .set(4, Value::U64(12))
            .set(19, Value::CharSeq(String::from("helloworld")))
            .value();

        let mut buf = Vec::new();
        write(&v, &mut buf);

        let mut expected:Vec<u8> = Vec::new();
        expected.extend_from_slice(MAGIC.as_bytes());
        expected.push(START_U64_OBJ);
        expected.push(START_U64);
        expected.write_u64::<LittleEndian>(4).unwrap();
        expected.push(START_U64);
        expected.write_u64::<LittleEndian>(12).unwrap();
        expected.push(START_U64);
        expected.write_u64::<LittleEndian>(19).unwrap();
        expected.push(START_CHAR_SEQ);
        expected.write_u64::<LittleEndian>(10).unwrap();
        expected.extend_from_slice(b"helloworld");
        expected.push(END_OBJ);
        assert_eq!(buf, expected);

    }

  

    #[test]
    fn test_charseq() {
        let v = Value::CharSeq(String::from("helloworld"));

        let mut buf = Vec::new();
        write(&v, &mut buf);

        let mut expected:Vec<u8> = Vec::new();
        expected.extend_from_slice(MAGIC.as_bytes());
        expected.push(START_CHAR_SEQ);
        expected.write_u64::<LittleEndian>(10).unwrap();
        expected.extend_from_slice(b"helloworld");
        assert_eq!(buf, expected);
    }

    
 
    #[test]
    fn test_u64obj_nested() {
        let v = U64ObjectBuilder::new()
            .set(4, Value::U64(12))
            .start_u64_obj(5)
                .set(1, Value::U64(2))
                .start_u64_obj(2)
                    .set(2, Value::U64(3))
                    .finish()
                .finish()
            .value();

        let mut buf = Vec::new();
        write(&v, &mut buf);

        
    }
}
