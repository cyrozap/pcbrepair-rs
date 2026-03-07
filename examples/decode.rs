// SPDX-License-Identifier: GPL-3.0-or-later

/*
 *  decode.rs - Decoder tool for ASUS FZ and ASRock CAE files.
 *  Copyright (C) 2026  Forest Crossman <cyrozap@gmail.com>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

use clap::Parser;

use pcbrepair::decoder::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The file to read.
    file: String,
}

fn main() {
    let args = Args::parse();

    let file = match File::open(&args.file) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening file {:?}: {:?}", &args.file, e);
            return;
        }
    };

    let reader = BufReader::new(file);
    let decoded = match DecodedPcbRepairFile::new(reader) {
        Ok(pf) => pf,
        Err(e) => {
            eprintln!("Error decoding file {:?}: {:?}", &args.file, e);
            return;
        }
    };

    // Get the base path of the input file
    let path = Path::new(&args.file);
    let stem = path.file_stem().unwrap().to_string_lossy();
    let extension = path.extension().unwrap().to_string_lossy();
    let parent_dir = path.parent().unwrap_or(Path::new("."));

    // Create output file paths in the same directory as the source file
    let content_path = parent_dir.join(format!("{}.{}.content.csv", stem, extension));
    let description_path = parent_dir.join(format!("{}.{}.description.csv", stem, extension));

    // Write content to file
    let mut content_file = match File::create(&content_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error creating content file {:?}: {:?}", &content_path, e);
            return;
        }
    };
    if let Err(e) = content_file.write_all(&decoded.content) {
        eprintln!("Error writing content file {:?}: {:?}", &content_path, e);
        return;
    }
    println!("Content written to: {:?}", content_path);

    // Write description to file
    let mut description_file = match File::create(&description_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!(
                "Error creating description file {:?}: {:?}",
                &description_path, e
            );
            return;
        }
    };
    if let Err(e) = description_file.write_all(&decoded.description) {
        eprintln!(
            "Error writing description file {:?}: {:?}",
            &description_path, e
        );
        return;
    }
    println!("Description written to: {:?}", description_path);
}
