// Based on descriptions from
//
// - https://www.psdevwiki.com/ps3/PARAM.SFO
// - https://playstationdev.wiki/psvitadevwiki/index.php?title=PARAM.SFO
// - https://www.psdevwiki.com/ps3/Eboot.PBP#PARAM.SFO
// - https://www.psdevwiki.com/ps4/Param.sfo

//use std::str::{from_utf8, Utf8Error};
use byteorder::{ByteOrder, LittleEndian};
use scroll::{Pread, LE};
use scroll_derive::Pread;
use std::fs;
use std::mem::size_of;

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

#[derive(Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub index_table_entry: SfoIndexTableEntry,
    pub num: u32,
    pub data: EntryData,
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
    let bytes = fs::read("./PARAM.SFO").unwrap();
    let data = bytes.pread_with::<Header>(0, LE).unwrap();
    let index: Vec<Entry> = (0..data.table_entries)
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

            Entry {
                name,
                index_table_entry: entry,
                data,
                num,
            }
        })
        .collect();

    let data = bytes.pread_with::<Header>(0, LE).unwrap();
    println!("{:#x?}", data);
    println!("{:#x?}", index);
}
