// SPDX-License-Identifier: GPL-3.0-or-later

/*
 *  src/interpreter.rs - Interpreter library for ASUS FZ and ASRock CAE files.
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
 * # `interpreter` Module
 *
 * This module provides functionality to interpret parsed ASUS FZ and ASRock CAE
 * files into structured footprint data.
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

use std::collections::HashMap;

use rust_decimal::Decimal;

use crate::parser::ParsedPcbRepairFile;
use crate::parser::Units;

/// Represents a pin in a footprint.
#[derive(Debug)]
pub struct Pin {
    /// The name of the pin.
    pub name: String,
    /// The number of the pin.
    pub number: String,
    /// The X-coordinate in millimeters.
    pub x_mm: Decimal,
    /// The Y-coordinate in millimeters.
    pub y_mm: Decimal,
    /// The radius of the pin in millimeters.
    pub radius_mm: Decimal,
}

/// Information about a footprint, including its pins.
#[derive(Debug)]
pub struct FootprintInfo {
    /// List of pins in the footprint.
    pub pins: Vec<Pin>,
}

/// A fully interpreted PCB repair file, containing footprint data.
#[derive(Debug)]
pub struct InterpretedPcbRepairFile {
    /// A map of footprint names to their associated pin information.
    pub footprints: HashMap<String, FootprintInfo>,
}

impl InterpretedPcbRepairFile {
    /// Converts a parsed PCB file into an interpreted format.
    ///
    /// This includes unit conversion and centering of footprint pins.
    ///
    /// # Arguments
    ///
    /// * `parsed` - The parsed PCB file data.
    ///
    /// # Returns
    ///
    /// A `Result` containing the interpreted file or an error.
    pub fn from_parsed(parsed: &ParsedPcbRepairFile) -> Result<Self, Box<dyn std::error::Error>> {
        let mm_per_mil: Decimal = Decimal::new(254, 4);

        let content = &parsed.content;

        let mut footprint_pins = HashMap::new();

        for board_pin in &content.pins {
            let fp_name = board_pin.refdes.clone();

            // Fixup invalid pin numbers
            let pin_number = match board_pin.pin_number.as_str() {
                "" => board_pin.pin_name.clone(),
                "0" => board_pin.pin_name.clone(),
                _ => board_pin.pin_number.clone(),
            };

            // Use a more descriptive name
            let pin_name = if pin_number != board_pin.pin_name {
                board_pin.pin_name.clone()
            } else {
                board_pin.net_name.clone()
            };

            // Convert coordinates to millimeters
            let x = match content.units {
                Units::Mils => board_pin.pin_x * mm_per_mil,
                Units::Millimeters => board_pin.pin_x,
            };
            let y = match content.units {
                Units::Mils => board_pin.pin_y * mm_per_mil,
                Units::Millimeters => board_pin.pin_y,
            };

            let radius = match content.units {
                Units::Mils => board_pin.radius * mm_per_mil,
                Units::Millimeters => board_pin.radius,
            };

            let pin = Pin {
                name: pin_name,
                number: pin_number,
                x_mm: x,
                y_mm: y,
                radius_mm: radius,
            };

            footprint_pins
                .entry(fp_name)
                .or_insert_with(Vec::new)
                .push(pin);
        }

        let mut footprints = HashMap::new();

        // Center each footprint's pins around (0, 0)
        for (fp_name, pins) in footprint_pins {
            if pins.is_empty() {
                continue;
            }

            let total_x: Decimal = pins.iter().map(|p| p.x_mm).sum();
            let total_y: Decimal = pins.iter().map(|p| p.y_mm).sum();
            let pin_count = Decimal::new(pins.len().try_into()?, 0);
            let avg_x = total_x / pin_count;
            let avg_y = total_y / pin_count;

            let centered_pins: Vec<Pin> = pins
                .into_iter()
                .map(|mut p| {
                    p.x_mm -= avg_x;
                    p.y_mm -= avg_y;
                    p
                })
                .collect();

            footprints.insert(
                fp_name,
                FootprintInfo {
                    pins: centered_pins,
                },
            );
        }

        Ok(Self { footprints })
    }
}
