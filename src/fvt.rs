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

//! 解编码字幕文件

use anyhow::{Error, Result};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct FVT {
    tag: String,
    u32_unknown0: u32,
    u32_unknown1: u32,
    u32_unknown2: u32,
    u32_unknown3: u32,
    u8_unknown0: u8,
    u8_unknown1: u8,
    text: String,
}

pub fn decode(input: &mut impl Read, output: &mut impl Write) -> Result<()> {
    let mut fvt = FVT::default();
    let mut tag = [0; 8];
    input.read_exact(&mut tag[0..1])?;
    input.read_exact(&mut tag[0..1])?;
    match tag[0] {
        0x45 => {
            fvt.tag = "DEND_FVT".into();
            // E: Lighting Stage
            input.read_exact(&mut tag[0..6])?;
            fvt.u32_unknown0 = input.read_u32::<LE>()?;
            fvt.u8_unknown0 = input.read_u8()?;
            let text_length = input.read_u8()?;
            fvt.u8_unknown1 = input.read_u8()?;
            let mut text = vec![0; text_length as usize];
            input.read_exact(&mut text)?;
            let (text, _, _error) = SHIFT_JIS.decode(&text);
            fvt.text = text.to_string();
            serde_json::to_writer_pretty(output, &fvt)?;
            Ok(())
        }
        0x32 => {
            fvt.tag = "D2_FVT".into();
            // 2: Burning Stage
            input.read_exact(&mut tag[0..4])?;
            fvt.u32_unknown0 = input.read_u32::<LE>()?;
            fvt.u32_unknown1 = input.read_u32::<LE>()?;
            fvt.u32_unknown2 = input.read_u32::<LE>()?;
            fvt.u8_unknown0 = input.read_u8()?;
            let text_length = input.read_u8()?;
            fvt.u8_unknown1 = input.read_u8()?;
            let mut text = vec![0; text_length as usize];
            input.read_exact(&mut text)?;
            let (text, _, _error) = SHIFT_JIS.decode(&text);
            fvt.text = text.to_string();
            serde_json::to_writer_pretty(output, &fvt)?;
            Ok(())
        }
        0x33 => {
            fvt.tag = "D3_FVT".into();
            // 3: Climax Stage & Rising Stage
            input.read_exact(&mut tag[0..4])?;
            fvt.u32_unknown0 = input.read_u32::<LE>()?;
            fvt.u32_unknown1 = input.read_u32::<LE>()?;
            fvt.u32_unknown2 = input.read_u32::<LE>()?;
            fvt.u8_unknown0 = input.read_u8()?;
            let text_length = input.read_u8()?;
            fvt.u8_unknown1 = input.read_u8()?;
            let mut text = vec![0; text_length as usize];
            input.read_exact(&mut text)?;
            let (text, _, _error) = SHIFT_JIS.decode(&text);
            fvt.text = text.to_string();
            serde_json::to_writer_pretty(output, &fvt)?;
            Ok(())
        }
        _ => Err(Error::msg("Unknown fvt type")),
    }
}

pub fn encode(input: &mut impl Read, output: &mut impl Write) -> Result<()> {
    let fvt: FVT = serde_json::from_reader(input)?;
    match fvt.tag.as_str() {
        "DEND_FVT" => {
            output.write(&fvt.tag.as_bytes())?;
            output.write_u32::<LE>(fvt.u32_unknown0)?;
            output.write_u8(fvt.u8_unknown0)?;
            let (text, _, _error) = SHIFT_JIS.encode(&fvt.text);
            if _error {
                println!("WARN: Text has occured encoding error from UTF-8 to SHIFT-JIS");
            }
            output.write_u8(text.len() as u8)?;
            output.write_u8(fvt.u8_unknown1)?;
            output.write_all(&text)?;
            Ok(())
        }
        "D2_FVT" | "D3_FVT" => {
            output.write(&fvt.tag.as_bytes())?;
            output.write_u32::<LE>(fvt.u32_unknown0)?;
            output.write_u32::<LE>(fvt.u32_unknown1)?;
            output.write_u32::<LE>(fvt.u32_unknown2)?;
            output.write_u8(fvt.u8_unknown0)?;
            let (text, _encoding, _error) = SHIFT_JIS.encode(&fvt.text);
            if _error {
                println!("WARN: Text has occured encoding error from UTF-8 to SHIFT-JIS");
            }
            output.write_u8(text.len() as u8)?;
            output.write_u8(fvt.u8_unknown1)?;
            output.write_all(&text)?;
            Ok(())
        }
        _ => Err(Error::msg("Unknown fvt type")),
    }
}
