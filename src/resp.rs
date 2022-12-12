use std::str;
const CRLF: &[u8] = "\r\n".as_bytes();

#[derive(Debug)]
pub enum RESPType<'a> {
    SimpleString(&'a str),
    Error(&'a str),
    Integer(usize),
    BulkString(&'a str),
    Array(Vec<RESPType<'a>>),
}

fn take_until_crlf(bytes: &[u8]) -> usize {
    let mut n = 0;
    while &(bytes[n..n + 2]) != CRLF {
        n += 1;
    }
    return n;
}

impl<'a> RESPType<'a> {
    pub fn pack(self: &'a Self) -> Vec<u8> {
        let mut result: Vec<u8> = vec![];
        match self {
            Self::BulkString(s) => {
                result.push(b'$');
                let mut str_bytes = s.as_bytes().to_owned();
                let mut length_bytes = str_bytes.len().to_string().as_bytes().to_owned();
                result.append(&mut length_bytes);
                result.append(&mut CRLF.clone().to_owned());
                result.append(&mut str_bytes);
                result.append(&mut CRLF.clone().to_owned());
            }
            Self::SimpleString(s) => {
                result.push(b'+');
                let mut str_bytes = s.as_bytes().to_owned();
                result.append(&mut str_bytes);
                result.append(&mut CRLF.clone().to_owned());
            }
            _ => {
                panic!("Not implemented")
            }
        }
        return result;
    }
    pub fn unpack(bytes: &'a [u8]) -> (Self, usize) {
        if bytes[0] == b'+' {
            let n = take_until_crlf(&bytes[1..]);
            return (
                RESPType::SimpleString(str::from_utf8(&bytes[1..n + 1]).unwrap()),
                n + 3,
            );
        } else if bytes[0] == b'-' {
            let n = take_until_crlf(&bytes[1..]);
            return (
                RESPType::Error(str::from_utf8(&bytes[1..n + 1]).unwrap()),
                n + 3,
            );
        } else if bytes[0] == b':' {
            let n = take_until_crlf(&bytes[1..]);
            return (
                RESPType::Integer(str::from_utf8(&bytes[1..n + 1]).unwrap().parse().unwrap()),
                n + 3,
            );
        } else if bytes[0] == b'$' {
            let len_len = take_until_crlf(&bytes[1..]);
            let len: usize = str::from_utf8(&bytes[1..len_len + 1])
                .unwrap()
                .parse()
                .unwrap();
            return (
                RESPType::BulkString(
                    str::from_utf8(&bytes[1 + len_len + 2..1 + len_len + 2 + len]).unwrap(),
                ),
                1 + len_len + 2 + len + 2,
            );
        } else if bytes[0] == b'*' {
            let len_len = take_until_crlf(&bytes[1..]);
            println!("{}", len_len);
            let num_elements: usize = str::from_utf8(&bytes[1..len_len + 1])
                .unwrap()
                .parse()
                .unwrap();
            let mut result: Vec<RESPType<'a>> = vec![];
            let mut used_length_in_elements = 0;
            let header_size = 1 + len_len + 2;
            for _ in 0..num_elements {
                let (element, used_size) =
                    RESPType::unpack(&bytes[header_size + used_length_in_elements..]);
                result.push(element);
                used_length_in_elements += used_size
            }
            return (
                RESPType::Array(result),
                header_size + used_length_in_elements,
            );
        } else {
            return (RESPType::SimpleString(&"123"), 0);
        }
    }
}
