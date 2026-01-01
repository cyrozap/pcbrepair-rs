// SPDX-License-Identifier: GPL-3.0-or-later

/*
 *  parse.rs - Parser demo for ASUS FZ and ASRock CAE files.
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

use clap::Parser;

use pcbrepair::decoder::*;
use pcbrepair::parser::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The file to read.
    file: String,
}

fn main() {
    let args = Args::parse();

    let decoded = match DecodedPcbRepairFile::from_filename(&args.file) {
        Ok(pf) => pf,
        Err(error) => {
            eprintln!("Error opening file {:?}: {:?}", &args.file, error);
            return;
        }
    };

    let parsed = match ParsedPcbRepairFile::from_decoded(&decoded) {
        Ok(pf) => pf,
        Err(error) => {
            eprintln!("Error parsing file {:?}: {:?}", &args.file, error);
            return;
        }
    };

    println!("{:?}", parsed);
}
