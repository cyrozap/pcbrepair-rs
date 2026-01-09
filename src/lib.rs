// SPDX-License-Identifier: GPL-3.0-or-later

/*
 *  src/lib.rs - Decoder and parser library for ASUS FZ and ASRock CAE files.
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

/*!
 * # `pcbrepair` Crate
 *
 * A library for decoding, parsing, and interpreting ASUS FZ and ASRock CAE
 * files.
 *
 * This crate provides a full pipeline for working with proprietary ASUS/ASRock
 * PCB repair files:
 *
 * 1. [decoder]: Handles decryption and decompression of the file.
 * 2. [parser]: Converts the decoded bytes into structured data.
 * 3. [interpreter]: Transforms parsed data into usable footprint information.
 *
 * ## Usage Example
 *
 * ```no_run
 * use std::fs::File;
 * use std::io::BufReader;
 *
 * use pcbrepair::decoder::DecodedPcbRepairFile;
 * use pcbrepair::parser::ParsedPcbRepairFile;
 * use pcbrepair::interpreter::InterpretedPcbRepairFile;
 *
 * fn main() -> Result<(), Box<dyn std::error::Error>> {
 *     // Open the file
 *     let file = File::open("example.fz")?;
 *     let reader = BufReader::new(file);
 *
 *     // Decode the file
 *     let decoded = DecodedPcbRepairFile::new(reader)?;
 *
 *     // Parse the decoded file
 *     let parsed = ParsedPcbRepairFile::from_decoded(&decoded)?;
 *
 *     // Interpret the parsed file
 *     let interpreted = InterpretedPcbRepairFile::from_parsed(&parsed)?;
 *
 *     // Access interpreted footprints
 *     for (name, info) in &interpreted.footprints {
 *         println!("Footprint: {}", name);
 *         for pin in &info.pins {
 *             println!("  Pin: {} at ({}, {})", pin.number, pin.x_mm, pin.y_mm);
 *         }
 *     }
 *
 *     Ok(())
 * }
 * ```
 */

pub mod decoder;
pub mod interpreter;
pub mod parser;
