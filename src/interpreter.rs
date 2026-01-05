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

use std::collections::HashMap;

use rust_decimal::Decimal;

use crate::parser::ParsedPcbRepairFile;
use crate::parser::Units;

#[derive(Debug)]
pub struct Pin {
    pub name: String,
    pub number: String,
    pub x_mm: Decimal,
    pub y_mm: Decimal,
    pub radius_mm: Decimal,
}

#[derive(Debug)]
pub struct FootprintInfo {
    pub pins: Vec<Pin>,
}

#[derive(Debug)]
pub struct InterpretedPcbRepairFile {
    pub footprints: HashMap<String, FootprintInfo>,
}

impl InterpretedPcbRepairFile {
    pub fn from_parsed(parsed: &ParsedPcbRepairFile) -> Result<Self, Box<dyn std::error::Error>> {
        let mm_per_mil: Decimal = Decimal::new(254, 4);

        let content = &parsed.content;

        let mut footprint_pins = HashMap::new();

        for net in &content.nets {
            let fp_name = net.refdes.clone();

            // Fixup invalid pin numbers
            let pin_number = match net.pin_number.as_str() {
                "" => net.pin_name.clone(),
                "0" => net.pin_name.clone(),
                _ => net.pin_number.clone(),
            };

            // Use a more descriptive name
            let pin_name = if pin_number != net.pin_name {
                net.pin_name.clone()
            } else {
                net.net_name.clone()
            };

            // Convert coordinates to millimeters
            let x = match content.units {
                Units::Mils => net.pin_x * mm_per_mil,
                Units::Millimeters => net.pin_x,
            };
            let y = match content.units {
                Units::Mils => net.pin_y * mm_per_mil,
                Units::Millimeters => net.pin_y,
            };

            let radius = match content.units {
                Units::Mils => net.radius * mm_per_mil,
                Units::Millimeters => net.radius,
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
