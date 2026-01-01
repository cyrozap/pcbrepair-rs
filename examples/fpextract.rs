// SPDX-License-Identifier: GPL-3.0-or-later

/*
 *  fpextract.rs - Footprint extraction demo for ASUS FZ and ASRock CAE files.
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

use std::fs;
use std::fs::create_dir_all;
use std::path::Path;

use chrono;
use clap::Parser;

use pcbrepair::decoder::*;
use pcbrepair::interpreter::*;
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

    let interpreted = match InterpretedPcbRepairFile::from_parsed(&parsed) {
        Ok(pf) => pf,
        Err(error) => {
            eprintln!("Error interpreting file {:?}: {:?}", &args.file, error);
            return;
        }
    };

    // Create output directory based on input filename
    let base_name = Path::new(&args.file)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let input_dir = Path::new(&args.file)
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let output_dir = input_dir.join(format!("{}.pretty", base_name));
    if let Err(e) = create_dir_all(&output_dir) {
        eprintln!("Failed to create output directory: {}", e);
        return;
    }

    // Generate .kicad_mod files for each footprint
    for (name, info) in &interpreted.footprints {
        let mut content = String::new();

        // Write KiCad footprint header
        content.push_str(&format!("(footprint \"{}\"\n", name));
        let now = chrono::Utc::now();
        let current_date = now.format("%Y%m%d").to_string();
        content.push_str(&format!("  (version {})\n", current_date));
        content.push_str("  (generator pcbrepair_fpextract)\n");
        content.push_str(&format!(
            "  (descr \"Automatically generated footprint from {}\")\n",
            base_name
        ));
        content.push_str("  (tags \"generated\")\n");

        // Add reference and value text
        content.push_str("  (property \"Reference\" \"U\" (at 0 0) (effects (font (size 1 1) (thickness 0.15))))\n");
        content.push_str("  (property \"Value\" \"U1\" (at 0 1.5) (effects (font (size 1 1) (thickness 0.15))))\n");

        // Add reference and value text objects
        content.push_str("  (fp_text reference \"U\" (at 0 0) (layer \"F.SilkS\")\n");
        content.push_str("    (effects (font (size 1 1) (thickness 0.15)))\n");
        content.push_str("  )\n");
        content.push_str("  (fp_text value \"U1\" (at 0 1.5) (layer \"F.Fab\")\n");
        content.push_str("    (effects (font (size 1 1) (thickness 0.15)))\n");
        content.push_str("  )\n");

        // Add pads for each pin
        for pin in &info.pins {
            content.push_str(&format!(
                "  (pad \"{}\" smd circle (at {} {}) (size {} {}) (layers F.Cu F.Paste F.Mask)\n",
                pin.number, pin.x_mm, pin.y_mm, pin.radius_mm, pin.radius_mm
            ));
            content.push_str("  )\n");
        }

        // Close the footprint
        content.push_str(")\n");

        // Write to file
        let filename = format!("{}/{}.kicad_mod", output_dir.display(), name);
        if let Err(e) = fs::write(&filename, content) {
            eprintln!("Failed to write file {}: {}", filename, e);
        }
    }
}
