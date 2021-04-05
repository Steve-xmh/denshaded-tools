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

//! 简易的 crc32 摘要算法

use lazy_static::lazy_static;

lazy_static! {
    static ref CRC_TABLE: [u32; 256] = {
        let mut table = [0u32; 256];
        for n in 0..256 {
            let mut c = n;
            for _ in 0..8 {
                if 0 != (c & 1) {
                    c = 0xedb88320 ^ (c >> 1);
                } else {
                    c = c >> 1;
                }
            }
            table[n] = c as u32;
        }
        table
    };
}

pub fn update_crc(crc: u32, buf: &[u8], pos: usize, len: usize) -> u32 {
    let mut c = crc;
    for n in 0..len {
        c = CRC_TABLE[(((c ^ (buf[pos + n] as u32)) as usize) & 0xFF) as usize] ^ (c >> 8);
    }
    c
}

pub fn compute(buf: &[u8], pos: usize, len: usize) -> u32 {
    update_crc(0xFFFFFFFF, buf, pos, len) ^ 0xFFFFFFFF
}
