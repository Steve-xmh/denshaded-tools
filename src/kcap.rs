//
// Densha De D Tools
// Copyright (C) 2021 SteveXMH
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

//! 拆解/打包 Pack 文件
// 电车D 全系列的解密密钥为 PackPass

use anyhow::{Error, Result};
use byteorder::*;
use encoding_rs::SHIFT_JIS;
use memmap::Mmap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use crate::crc32::{self, compute};

pub type KeyTable = [u8; 0x10000];

#[derive(Debug)]
pub struct KCAPEntry {
    pub name: String,
    pub offset: usize,
    pub size: usize,
    pub encrypted: bool,
}

impl KCAPEntry {
    pub fn from_read(file: &mut impl Read) -> Result<Self> {
        let mut buf = [0; 64];
        file.read_exact(&mut buf)?;
        let (file_name, _, _error) = SHIFT_JIS.decode(&buf);
        let _crc32 = file.read_u32::<LE>()? as usize;
        let _unknown = file.read_u32::<LE>()?;
        let offset = file.read_u32::<LE>()? as usize;
        let size = file.read_u32::<LE>()? as usize;
        let encrypted = file.read_u32::<LE>()? != 0;
        Ok(Self {
            name: file_name.trim_end_matches('\0').to_string(),
            offset,
            size,
            encrypted,
        })
    }
}

#[derive(Debug)]
pub struct KCAPPackReader {
    pub file: File,
    pub entries: Vec<KCAPEntry>,
    pub key_table: KeyTable,
}

impl<'a> KCAPPackReader {
    pub fn new<P: AsRef<Path>>(path: P, pass: &str) -> Result<Self> {
        let mut file = std::fs::File::open(path)?;
        let mut buf = [0; 4];
        file.read_exact(&mut buf)?;
        if &buf[..4] != "KCAP".as_bytes() {
            return Err(Error::msg("Not a correct pack file!"));
        }
        let file_amount = file.read_i32::<LE>()?;
        let mut entries = Vec::with_capacity(file_amount as usize);
        for _ in 0..file_amount {
            let entry = KCAPEntry::from_read(&mut file)?;
            entries.push(entry);
        }
        Ok(Self {
            file,
            entries,
            key_table: create_key_table(pass),
        })
    }

    pub fn read_to(&'a mut self, index: usize, output: &mut impl Write) -> Result<()> {
        let entry = &self.entries[index];
        let map = unsafe { Mmap::map(&self.file)? };
        let transformed = (map[entry.offset..entry.offset + entry.size]
            .iter()
            .enumerate()
            .map(|(i, &x)| {
                if entry.encrypted {
                    x ^ self.key_table[i % self.key_table.len()]
                } else {
                    x
                }
            }))
        .collect::<Vec<u8>>();
        output.write(&transformed)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct KCAPEntryWrite {
    pub name: String,
    pub file: File,
    pub offset: u64,
    pub size: u64,
}

#[derive(Debug)]
pub struct KCAPPackWriter {
    pub pass: Option<String>,
    pub key_table: Option<KeyTable>,
    pub entries: Vec<KCAPEntryWrite>,
}

impl KCAPPackWriter {
    pub fn new(pass: Option<String>) -> Self {
        Self {
            key_table: if let Some(pass) = &pass {
                Some(create_key_table(pass))
            } else {
                None
            },
            pass,
            entries: Vec::with_capacity(64),
        }
    }

    pub fn calc_offset(&mut self) {
        self.entries.sort_by(|a, b| a.size.cmp(&b.size));
        let mut file_offset = 8 + self.entries.len() as u64 * (64 + 8 + 4 + 4 + 4);
        for item in &mut self.entries {
            item.offset = file_offset;
            file_offset += item.size;
        }
    }

    pub fn add_entry<P: AsRef<Path>>(&mut self, file_path: P, name: &str) -> Result<()> {
        let file_path = file_path.as_ref();
        let file_meta = file_path.metadata()?;
        self.entries.push(KCAPEntryWrite {
            name: name.into(),
            file: File::open(file_path)?,
            offset: 0,
            size: file_meta.len(),
        });
        Ok(())
    }

    pub fn write_to(&mut self, output: &mut impl Write) -> Result<()> {
        self.calc_offset();
        output.write(b"KCAP")?;
        output.write_i32::<LE>(self.entries.len() as i32)?;
        let mut buf = [0; 64];
        let encrypted = if self.key_table.is_some() { 1 } else { 0 };
        for item in &self.entries {
            let bytes = item.name.as_bytes();
            buf.fill(0);
            buf[..bytes.len()].clone_from_slice(&bytes);
            output.write(&buf)?;
            let path_crc = compute(&buf, 0, bytes.len());
            output.write_u32::<LE>(path_crc)?;
            output.write_u32::<LE>(0)?; // 空余的无用 padding？
            output.write_u32::<LE>(item.offset as u32)?;
            output.write_u32::<LE>(item.size as u32)?;
            output.write_u32::<LE>(encrypted)?;
        }
        for item in &mut self.entries {
            if let Some(key_table) = &self.key_table {
                let map = unsafe { Mmap::map(&item.file)? };
                let transformed = (map[0..item.size as usize]
                    .iter()
                    .enumerate()
                    .map(|(i, &x)| x ^ key_table[i % key_table.len()]))
                .collect::<Vec<u8>>();
                output.write(&transformed)?;
            } else {
                std::io::copy(&mut item.file, output)?;
            }
        }
        Ok(())
    }
}

#[test]
fn test_kcap_pack() {
    println!(
        "{:?}",
        KCAPPackReader::new("./test/DenD_3rd_Data.Pack", "PackPass")
            .unwrap()
            .entries
    )
}

pub fn passkey_hash(pass: &str) -> u32 {
    let (bytes, _, _err) = encoding_rs::SHIFT_JIS.encode(pass);
    crc32::compute(bytes.as_ref(), 0, bytes.len())
}

pub fn create_key_table(pass: &str) -> KeyTable {
    let pass = if pass.len() < 8 {
        "Selene.Default.Password"
    } else {
        pass
    };
    let pass_len = pass.len();
    let seed = passkey_hash(pass);
    let mut rng = KeyTableGenerator::new(seed as i32);

    let mut table = [0; 0x10000];
    for i in 0..table.len() {
        let key = rng.rand();
        let pos = (i % pass_len) as usize;
        let m = (key >> 16) as u8;
        let pass_bytes = pass.as_bytes();
        let v = pass_bytes[pos] ^ m;
        table[i] = v;
    }
    table
}

#[test]
fn test_key_table() {
    assert_eq!(
        &create_key_table("")[0..16],
        &[43, 153, 246, 46, 115, 3, 156, 205, 107, 241, 77, 219, 216, 177, 13, 71]
    );
}

#[derive(Debug, Clone)]
struct KeyTableGenerator {
    m_table: Vec<i32>,
    m_pos: usize,
}

const STATE_LENGTH: usize = 624;
const STATE_M: usize = 397;
const MATRIX_A: i32 = -1727483681;
const TEMPERING_MASK_B: i32 = -1658038656;
const TEMPERING_MASK_C: i32 = -272236544;

const MAG01: [i32; 2] = [0, MATRIX_A];

impl KeyTableGenerator {
    pub fn new(seed: i32) -> Self {
        let mut s = Self::default();
        s.s_rand(seed);
        s
    }

    pub fn s_rand(&mut self, seed: i32) {
        self.m_table[0] = seed;
        for i in 1..STATE_LENGTH as i32 {
            let last = self.m_table[i as usize - 1];
            let v = i + 0x6C078965i32.overflowing_mul(last ^ (last >> 30)).0;
            self.m_table[i as usize] = v;
        }
        self.m_pos = STATE_LENGTH;
    }

    pub fn rand(&mut self) -> i32 {
        if self.m_pos >= STATE_LENGTH {
            let mut i = 0usize;
            while i < STATE_LENGTH - STATE_M {
                let mt0 = self.m_table[i];
                let mt1 = self.m_table[i + 1];
                let x = mt0 ^ mt1;
                self.m_table[i] = self.m_table[i + STATE_M]
                    ^ MAG01[((self.m_table[i] ^ x) & 1) as usize]
                    ^ ((self.m_table[i] ^ x & 0x7FFFFFFF) >> 1);
                i += 1;
            }
            while i < STATE_LENGTH - 1 {
                let mt0 = self.m_table[i];
                let mt1 = self.m_table[i + 1];
                let x = mt0 ^ mt1;
                self.m_table[i] = self.m_table[i + STATE_M - STATE_LENGTH]
                    ^ MAG01[((self.m_table[i] ^ x) & 1) as usize]
                    ^ ((self.m_table[i] ^ x & 0x7FFFFFFF) >> 1);
                i += 1;
            }
            let z = self.m_table[STATE_LENGTH - 1]
                ^ (self.m_table[0] ^ self.m_table[STATE_LENGTH - 1]) & 0x7FFFFFFF;
            self.m_table[STATE_LENGTH - 1] =
                self.m_table[STATE_M - 1] ^ (z >> 1) ^ MAG01[(z & 1) as usize];
            self.m_pos = 0;
        }
        let y = self.m_table[self.m_pos];
        self.m_pos += 1;
        let y = y ^ (y >> 11);
        let y = y ^ (y << 7) & TEMPERING_MASK_B;
        let y = y ^ (y << 15) & TEMPERING_MASK_C;
        let y = y ^ (y >> 18);
        y
    }
}

impl Default for KeyTableGenerator {
    fn default() -> Self {
        let mut m_table = Vec::with_capacity(STATE_LENGTH);
        unsafe {
            m_table.set_len(STATE_LENGTH);
        }
        m_table.fill(0);
        Self { m_table, m_pos: 0 }
    }
}
