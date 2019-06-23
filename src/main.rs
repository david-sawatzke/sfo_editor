// Based on descriptions from
//
// - https://www.psdevwiki.com/ps3/PARAM.SFO
// - https://playstationdev.wiki/psvitadevwiki/index.php?title=PARAM.SFO
// - https://www.psdevwiki.com/ps3/Eboot.PBP#PARAM.SFO
// - https://www.psdevwiki.com/ps4/Param.sfo

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::mem::size_of;
use std::num::ParseIntError;
use std::path::PathBuf;

use byteorder::{ByteOrder, LittleEndian};
use scroll::{Pread, LE};
use scroll_derive::Pread;
use structopt::StructOpt;

#[derive(StructOpt, Debug, PartialEq, Eq)]
#[structopt(name = "sfo_editor")]
struct Opt {
    /// Activate debug mode
    #[structopt(short = "d", long = "debug")]
    debug: bool,
    /// Sfo file
    #[structopt(parse(from_os_str))]
    file: PathBuf,
    #[structopt(subcommand)]
    cmd: Command,
}

fn parse_hex(src: &str) -> Result<u32, ParseIntError> {
    if src.starts_with("0x") == false {
        // This is evil & bad, but I'm lazy
        u32::from_str_radix("INVALID NUMBER", 16)?;
    }
    u32::from_str_radix(&src[2..], 16)
}

#[derive(StructOpt, Debug, PartialEq, Eq)]
enum Command {
    /// Simply output the fields
    #[structopt(name = "read")]
    Read,
    /// Set a Integer parameter
    #[structopt(name = "write")]
    Write {
        /// parameter name
        name: String,
        /// parameter value
        #[structopt(parse(try_from_str = "parse_hex"))]
        value: u32,
    },
}
#[derive(Pread, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Header {
    pub magic: u32,
    pub version: u32,
    pub key_table_start: u32,
    pub data_table_start: u32,
    pub table_entries: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EntryData {
    Utf8(String),
    Integer(u32),
}

impl fmt::Display for EntryData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EntryData::Utf8(string) => write!(f, "\"{}\"", string),
            EntryData::Integer(i) => write!(f, "{:#010x}", i),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub index_table_entry: SfoIndexTableEntry,
    pub num: u32,
    pub data: EntryData,
}
impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.data)
    }
}

#[derive(Pread, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct SfoIndexTableEntry {
    pub key_offset: u16,
    pub data_fmt: u16,
    pub data_len: u32,
    pub data_max_len: u32,
    pub data_offset: u32,
}

fn main() {
    let opt = Opt::from_args();
    let mut bytes = fs::read(&opt.file).unwrap();
    let data = bytes.pread_with::<Header>(0, LE).unwrap();
    let index: BTreeMap<String, Entry> = (0..data.table_entries)
        .map(|num| {
            (
                num,
                bytes
                    .pread_with::<SfoIndexTableEntry>(
                        0x14 + size_of::<SfoIndexTableEntry>() * num as usize,
                        LE,
                    )
                    .unwrap(),
            )
        })
        .map(|(num, entry)| {
            let name = String::from_utf8(
                bytes[data.key_table_start as usize + entry.key_offset as usize..]
                    .iter()
                    .take_while(|e| **e != 0)
                    .map(|e| *e)
                    .collect(),
            )
            .unwrap();
            assert!(entry.data_len <= entry.data_max_len);
            let offset = (data.data_table_start + entry.data_offset) as usize;
            let data = match entry.data_fmt {
                0x0404 => EntryData::Integer(LittleEndian::read_u32(&bytes[offset..offset + 4])),
                0x0004 | 0x0204 => EntryData::Utf8(
                    // Includes the NULL byte
                    String::from_utf8(bytes[offset..offset + entry.data_len as usize - 1].to_vec())
                        .unwrap(),
                ),
                _ => panic!("Undefined data pattern"),
            };

            (
                name.clone(),
                Entry {
                    name,
                    index_table_entry: entry,
                    data,
                    num,
                },
            )
        })
        .collect();

    let data = bytes.pread_with::<Header>(0, LE).unwrap();
    if opt.debug == true {
        println!("{:#x?}", data);
        println!("{:#x?}", index);
    }
    match opt.cmd {
        Command::Read => {
            for entry in index.values() {
                println!("{}", entry);
            }
        }
        Command::Write { name, value } => {
            let entry = index.get(&name).expect("Couldn't find entry with key");
            let offset = (data.data_table_start + (*entry).index_table_entry.data_offset) as usize;
            LittleEndian::write_u32(&mut bytes[offset..offset + 4], value);
        }
    }
    fs::write(&opt.file, bytes).unwrap();
}
