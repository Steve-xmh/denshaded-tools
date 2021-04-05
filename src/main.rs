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

mod crc32;
mod fvt;
mod kcap;

use std::fs::OpenOptions;
use std::path::Path;

use clap::clap_app;

use anyhow::{Error, Result};


use kcap::{KCAPPackReader, KCAPPackWriter};

fn unpack(file: &Path, save_dir: &Path, pass: &str) -> Result<()> {
    println!("Unpack {}", file.display());
    println!("    to {}", save_dir.display());
    let mut pack = KCAPPackReader::new(file, pass)?;
    for i in 0..pack.entries.len() {
        let name = pack.entries[i].name.clone();
        let save_file = save_dir.join(&name);
        let save_dir = save_file.parent().unwrap();
        std::fs::create_dir_all(save_dir).unwrap_or_default();
        println!("Exacting {} -> {}", &name, save_file.display());
        let mut save_file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(save_file)?;
        pack.read_to(i, &mut save_file)?;
    }
    Ok(())
}

fn pack(dir: &Path, save_file: &Path, pass: &str) -> Result<()> {
    println!("Pack {}", dir.display());
    println!("  to {}", save_file.display());
    let dir_string = dir.to_string_lossy().to_string();
    let mut pack = KCAPPackWriter::new(Some(pass.into()));
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let path = entry.path();
            let name = path.clone().to_path_buf().to_string_lossy().to_string();
            let name = name.trim_start_matches(&format!("{}\\", dir_string));
            println!("Packing {} -> {}", path.display(), name);
            pack.add_entry(path, name.into())?;
        }
    }
    println!("Writing {} -> {}", dir.display(), save_file.display());
    let mut output = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(save_file)?;
    pack.write_to(&mut output)?;
    Ok(())
}

fn fvt_decode(from: &Path, to: &Path) -> Result<()> {
    println!("Decode from {}", from.display());
    println!("         to {}", to.display());
    let mut from = OpenOptions::new().read(true).open(from)?;
    let mut to = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(to)?;
    fvt::decode(&mut from, &mut to)?;
    Ok(())
}

fn fvt_encode(from: &Path, to: &Path) -> Result<()> {
    println!("Encode from {}", from.display());
    println!("         to {}", to.display());
    let mut from = OpenOptions::new().read(true).open(from)?;
    let mut to = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(to)?;
    fvt::encode(&mut from, &mut to)?;
    Ok(())
}

fn main() -> Result<()> {
    let app = clap_app!(DenshaDeDTool =>
        (version: "1.0")
        (author: "SteveXMH <stevexmh@qq.com>")
        (about: "Game modifition tool for Densha De D series (Selene Engine)")
        (@arg verbose: -v --verbose "Print test information verbosely")
        (@subcommand unpack =>
            (about: "Unpack the game pack to a directory")
            (version: "1.0")
            (author: "SteveXMH <stevexmh@qq.com>")
            (@arg INPUT: +required "Sets the input file to use")
            (@arg OUTPUT: -o --output "Set output directory path, defaults s \"[INPUT_DIR]/unpacked/[INPUT_NAME]\"")
            (@arg PASS: -p --pass "Password for encrypted pack file, defaults is \"PackPass\" for Densha De D")
        )
        (@subcommand pack =>
            (about: "Pack everything inside a directory to a Pack file (Still work in progress)")
            (version: "1.0")
            (author: "SteveXMH <stevexmh@qq.com>")
            (@arg INPUT: +required "Sets the input directory to use")
            (@arg OUTPUT: -o --output "Set output file path, defaults s the same path and the same name of the directory")
            (@arg PASS: -p --pass "Password for encrypted pack file, defaults is \"PackPass\" for Densha De D")
        )
        (@subcommand fvt =>
            (about: "Subcommand for FVT files")
            (version: "1.0")
            (author: "SteveXMH <stevexmh@qq.com>")
            (@subcommand decode =>
                (about: "Decode FVT file into json file")
                (version: "1.0")
                (author: "SteveXMH <stevexmh@qq.com>")
                (@arg INPUT: +required "Sets the input file to use")
                (@arg OUTPUT: -o --output "Set output file path, defaults s the same name with json extension")
            )
            (@subcommand encode =>
                (about: "Encode json file into FVT file")
                (version: "1.0")
                (author: "SteveXMH <stevexmh@qq.com>")
                (@arg INPUT: +required "Sets the input file to use")
                (@arg OUTPUT: -o --output "Set output file path, defaults s the same name with json extension")
            )
        )
    );
    let matched = app.get_matches();

    if let Some(subcommand) = matched.subcommand_matches("unpack") {
        let input = subcommand.value_of("INPUT").expect("Input is not provided");
        let output = subcommand.value_of("OUTPUT");
        let pass = subcommand.value_of("PASS");

        let input = std::path::Path::new(input);
        let output = if let Some(output) = output {
            output.to_owned()
        } else {
            let output_path = std::path::Path::new(input).parent().unwrap();
            let output_path = output_path.join(input.file_stem().unwrap());
            output_path.to_str().unwrap().to_owned()
        };

        unpack(
            input,
            std::path::Path::new(&output),
            pass.unwrap_or("PackPass"),
        )
    } else if let Some(subcommand) = matched.subcommand_matches("pack") {
        let input = subcommand.value_of("INPUT").expect("Input is not provided");
        let output = subcommand.value_of("OUTPUT");
        let pass = subcommand.value_of("PASS");

        let input = std::path::Path::new(input);
        let output = if let Some(output) = output {
            output.to_owned()
        } else {
            let output_path = std::path::Path::new(input);
            let output_path = output_path.parent().unwrap();
            let name = input
                .file_name()
                .ok_or(Error::msg("Can't get name of input directory"))
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();
            let output_path = output_path.join(&format!("{}.Pack", name));
            output_path.to_str().unwrap().to_owned()
        };

        pack(
            input,
            std::path::Path::new(&output),
            pass.unwrap_or("PackPass"),
        )
    } else if let Some(subcommand) = matched.subcommand_matches("fvt") {
        if let Some(subcommand) = subcommand.subcommand_matches("encode") {
            let input = subcommand.value_of("INPUT").expect("Input is not provided");
            let output = subcommand.value_of("OUTPUT");
            let input = std::path::Path::new(input);
            let output = if let Some(output) = output {
                output.to_owned()
            } else {
                let output_path = std::path::Path::new(input);
                let output_path = output_path.parent().unwrap();
                let name = input
                    .file_stem()
                    .ok_or(Error::msg("Can't get name of input file"))
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();
                let output_path = output_path.join(&format!("{}.FVT", name));
                output_path.to_str().unwrap().to_owned()
            };
            fvt_encode(input, std::path::Path::new(&output))
        } else if let Some(subcommand) = subcommand.subcommand_matches("decode") {
            let input = subcommand.value_of("INPUT").expect("Input is not provided");
            let output = subcommand.value_of("OUTPUT");
            let input = std::path::Path::new(input);
            let output = if let Some(output) = output {
                output.to_owned()
            } else {
                let output_path = std::path::Path::new(input);
                let output_path = output_path.parent().unwrap();
                let name = input
                    .file_stem()
                    .ok_or(Error::msg("Can't get name of input file"))
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();
                let output_path = output_path.join(&format!("{}.json", name));
                output_path.to_str().unwrap().to_owned()
            };
            fvt_decode(input, std::path::Path::new(&output))
        } else {
            println!("{}", matched.usage());
            Ok(())
        }
    } else {
        println!("{}", matched.usage());
        Ok(())
    }
}
