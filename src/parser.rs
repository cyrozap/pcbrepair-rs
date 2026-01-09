// SPDX-License-Identifier: GPL-3.0-or-later

/*
 *  src/parser.rs - Parser library for ASUS FZ and ASRock CAE files.
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
 * # `parser` Module
 *
 * This module provides functionality to parse decoded ASUS FZ and ASRock CAE
 * files into structured data.
 *
 * ## Usage Example
 *
 * ```no_run
 * use std::fs::File;
 * use std::io::BufReader;
 *
 * use pcbrepair::decoder::DecodedPcbRepairFile;
 * use pcbrepair::parser::ParsedPcbRepairFile;
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
 *     // Access parsed data
 *     for pin in &parsed.content.pins {
 *         println!("Pin: {}", pin.pin_name);
 *     }
 *
 *     Ok(())
 * }
 * ```
 */

use std::str::FromStr;
use std::string::String;

use csv;
use rust_decimal::Decimal;

use crate::decoder::DecodedPcbRepairFile;

enum ParserState {
    Unknown,
    Symbol,
    Pin,
    Via,
    TestVia,
    GraphicData,
    ClassedGraphicData,
}

/// Represents the unit system used in the file (mils or millimeters).
#[derive(Debug)]
pub enum Units {
    /// Unit is in mils (1/1000 inch).
    Mils,
    /// Unit is in millimeters.
    Millimeters,
}

/// Represents a symbol in the decoded PCB file.
#[derive(Debug)]
pub struct Symbol {
    /// The reference designator (e.g., "U1") of the symbol.
    pub refdes: String,
    /// The component insertion code.
    pub comp_insertion_code: u64,
    /// The name of the symbol.
    pub sym_name: String,
    /// Whether the symbol is mirrored.
    pub sym_mirror: bool,
    /// The rotation angle of the symbol in degrees.
    pub sym_rotate: u16,
}

/// Represents a pin in the decoded PCB file.
#[derive(Debug)]
pub struct Pin {
    /// The name of the net this pin is connected to.
    pub net_name: String,
    /// The reference designator (e.g., "U1") this pin is part of.
    pub refdes: String,
    /// The number of the pin.
    pub pin_number: String,
    /// The name of the pin.
    pub pin_name: String,
    /// The X-coordinate of the pin on the PCB, in [Content::units] units.
    pub pin_x: Decimal,
    /// The Y-coordinate of the pin on the PCB, in [Content::units] units.
    pub pin_y: Decimal,
    pub test_point: String,
    /// The radius of the pin on the PCB, in [Content::units] units.
    pub radius: Decimal,
}

/// Represents a test via in the decoded PCB file.
#[derive(Debug)]
pub struct TestVia {
    /// The name of the test via.
    pub testvia: String,
    /// The name of the net this test via is connected to.
    pub net_name: String,
    pub refdes: String,
    pub pin_number: String,
    pub pin_name: String,
    /// The X-coordinate of the test via on the PCB, in [Content::units] units.
    pub via_x: Decimal,
    /// The Y-coordinate of the test via on the PCB, in [Content::units] units.
    pub via_y: Decimal,
    pub test_point: String,
    /// The radius of the test via on the PCB, in [Content::units] units.
    pub radius: Decimal,
}

/// Represents a graphic data entry in the decoded PCB file.
#[derive(Debug)]
pub struct GraphicData {
    pub graphic_data_name: String,
    pub graphic_data_number: u64,
    pub record_tag: String,
    pub graphic_data: [String; 9],
    pub subclass: String,
    pub sym_name: String,
    pub refdes: String,
}

/// Represents a classed graphic data entry in the decoded PCB file.
#[derive(Debug)]
pub struct ClassedGraphicData {
    pub class: String,
    pub subclass: String,
    pub graphic_data_name: String,
    pub graphic_data_number: u64,
    pub record_tag: String,
    pub graphic_data: [String; 9],
    pub net_name: String,
}

/// Parsed content of the decoded PCB file.
#[derive(Debug)]
pub struct Content {
    /// The unit system used in the file.
    pub units: Units,
    /// List of symbols in the file.
    pub symbols: Vec<Symbol>,
    /// List of pins in the file.
    pub pins: Vec<Pin>,
    /// List of test vias in the file.
    pub testvias: Vec<TestVia>,
    /// List of graphic data entries.
    pub graphic_data: Vec<GraphicData>,
    /// List of classed graphic data entries.
    pub classed_graphic_data: Vec<ClassedGraphicData>,
}

impl Content {
    /// Parses the decoded content into structured data.
    ///
    /// # Arguments
    ///
    /// * `content` - The raw decoded bytes from the file.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `Content` or an error.
    pub fn from_bytes(content: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut symbols = Vec::new();
        let mut pins = Vec::new();
        let mut testvias = Vec::new();
        let mut graphic_data = Vec::new();
        let mut classed_graphic_data = Vec::new();

        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b'!')
            .flexible(true)
            .has_headers(false)
            .from_reader(content);

        let mut state = ParserState::Unknown;
        let mut units = Units::Mils;

        for result in reader.byte_records() {
            let record = result?;
            if record.is_empty() {
                continue;
            }

            let first = &record[0];
            if first == b"A" {
                if &record[1] == b"UNIT" {
                    if &record[2] == b"mils" {
                        units = Units::Mils;
                    } else {
                        units = Units::Millimeters;
                    }
                } else if &record[1] == b"REFDES" {
                    state = ParserState::Symbol;
                } else if &record[1] == b"NET_NAME" {
                    state = ParserState::Pin;
                } else if &record[1] == b"VIAID" {
                    state = ParserState::Via;
                } else if &record[1] == b"TESTVIA" {
                    state = ParserState::TestVia;
                } else if &record[1] == b"GRAPHIC_DATA_NAME" {
                    state = ParserState::GraphicData;
                } else if &record[1] == b"CLASS" {
                    state = ParserState::ClassedGraphicData;
                } else if &record[1] == b"LOGOInfo" {
                } else if &record[1] == b"UnDrawSym" {
                } else {
                    state = ParserState::Unknown;
                }
            } else if first == b"S" {
                match state {
                    ParserState::Symbol => {
                        symbols.push(Symbol {
                            refdes: String::from_utf8_lossy(&record[1]).to_string(),
                            comp_insertion_code: String::from_utf8_lossy(&record[2])
                                .to_string()
                                .parse::<u64>()?,
                            sym_name: String::from_utf8_lossy(&record[3]).to_string(),
                            sym_mirror: &record[4] == b"YES",
                            sym_rotate: String::from_utf8_lossy(&record[5])
                                .to_string()
                                .parse::<u16>()?,
                        });
                    }
                    ParserState::Pin => {
                        pins.push(Pin {
                            net_name: String::from_utf8_lossy(&record[1]).to_string(),
                            refdes: String::from_utf8_lossy(&record[2]).to_string(),
                            pin_number: String::from_utf8_lossy(&record[3]).to_string(),
                            pin_name: String::from_utf8_lossy(&record[4]).to_string(),
                            pin_x: parse_decimal(&record[5])?,
                            pin_y: parse_decimal(&record[6])?,
                            test_point: String::from_utf8_lossy(&record[7]).to_string(),
                            radius: parse_decimal(&record[8])?,
                        });
                    }
                    ParserState::TestVia => {
                        testvias.push(TestVia {
                            testvia: String::from_utf8_lossy(&record[1]).to_string(),
                            net_name: String::from_utf8_lossy(&record[2]).to_string(),
                            refdes: String::from_utf8_lossy(&record[3]).to_string(),
                            pin_number: String::from_utf8_lossy(&record[4]).to_string(),
                            pin_name: String::from_utf8_lossy(&record[5]).to_string(),
                            via_x: parse_decimal(&record[6])?,
                            via_y: parse_decimal(&record[7])?,
                            test_point: String::from_utf8_lossy(&record[8]).to_string(),
                            radius: parse_decimal(&record[9])?,
                        });
                    }
                    ParserState::GraphicData => {
                        graphic_data.push(GraphicData {
                            graphic_data_name: String::from_utf8_lossy(&record[1]).to_string(),
                            graphic_data_number: String::from_utf8_lossy(&record[2])
                                .to_string()
                                .parse::<u64>()?,
                            record_tag: String::from_utf8_lossy(&record[3]).to_string(),
                            graphic_data: [
                                String::from_utf8_lossy(&record[4]).to_string(),
                                String::from_utf8_lossy(&record[5]).to_string(),
                                String::from_utf8_lossy(&record[6]).to_string(),
                                String::from_utf8_lossy(&record[7]).to_string(),
                                String::from_utf8_lossy(&record[8]).to_string(),
                                String::from_utf8_lossy(&record[9]).to_string(),
                                String::from_utf8_lossy(&record[10]).to_string(),
                                String::from_utf8_lossy(&record[11]).to_string(),
                                String::from_utf8_lossy(&record[12]).to_string(),
                            ],
                            subclass: String::from_utf8_lossy(&record[13]).to_string(),
                            sym_name: String::from_utf8_lossy(&record[14]).to_string(),
                            refdes: String::from_utf8_lossy(&record[15]).to_string(),
                        });
                    }
                    ParserState::ClassedGraphicData => {
                        classed_graphic_data.push(ClassedGraphicData {
                            class: String::from_utf8_lossy(&record[1]).to_string(),
                            subclass: String::from_utf8_lossy(&record[2]).to_string(),
                            graphic_data_name: String::from_utf8_lossy(&record[3]).to_string(),
                            graphic_data_number: String::from_utf8_lossy(&record[4])
                                .to_string()
                                .parse::<u64>()?,
                            record_tag: String::from_utf8_lossy(&record[5]).to_string(),
                            graphic_data: [
                                String::from_utf8_lossy(&record[6]).to_string(),
                                String::from_utf8_lossy(&record[7]).to_string(),
                                String::from_utf8_lossy(&record[8]).to_string(),
                                String::from_utf8_lossy(&record[9]).to_string(),
                                String::from_utf8_lossy(&record[10]).to_string(),
                                String::from_utf8_lossy(&record[11]).to_string(),
                                String::from_utf8_lossy(&record[12]).to_string(),
                                String::from_utf8_lossy(&record[13]).to_string(),
                                String::from_utf8_lossy(&record[14]).to_string(),
                            ],
                            net_name: String::from_utf8_lossy(&record[15]).to_string(),
                        });
                    }
                    _ => (),
                }
            }
        }

        Ok(Self {
            units,
            symbols,
            pins,
            testvias,
            graphic_data,
            classed_graphic_data,
        })
    }
}

/// Represents a component in the decoded PCB file's description.
#[derive(Debug)]
pub struct Component {
    /// The part number of the component.
    pub part_number: String,
    /// The description/name of the component.
    pub description: String,
    /// The number of times this component is used on the PCB.
    pub quantity: u64,
    /// List of reference designators on the PCB where this component is used.
    pub location: Vec<String>,
    /// An alternate part number.
    pub part_number2: String,
}

/// The PCB file's description information.
#[derive(Debug)]
pub struct Description {
    /// PCB model number.
    pub board_model: String,
    /// PCB revision.
    pub revision: String,
    /// Longer PCB model number.
    pub extended_board_model: String,
    /// Longer PCB revision.
    pub extended_revision: String,
    /// Part number of the PCB.
    pub part_number: String,
    /// List of components on the PCB.
    pub components: Vec<Component>,
}

impl Description {
    pub fn from_bytes(description: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let description_str = String::from_utf8_lossy(description);
        let lines = description_str.split("\r\n").collect::<Vec<_>>();
        if lines.is_empty() {
            return Err("Description is empty".into());
        }

        let header = lines[0].split('|').collect::<Vec<_>>();
        if header.len() < 5 {
            return Err("Invalid header format".into());
        }

        let board_model = header[0].to_string();
        let revision = header[1].to_string();
        let extended_board_model = header[2].to_string();
        let extended_revision = header[3].to_string();
        let part_number = header[4].to_string();

        let mut component_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .flexible(true)
            .has_headers(false)
            .from_reader(description);

        let mut components = Vec::new();
        for result in component_reader.byte_records().skip(2) {
            let record = result?;
            if record.len() < 5 {
                continue; // Skip malformed lines
            }

            components.push(Component {
                part_number: String::from_utf8_lossy(&record[0]).to_string(),
                description: String::from_utf8_lossy(&record[1]).to_string(),
                quantity: String::from_utf8_lossy(&record[2])
                    .to_string()
                    .parse::<u64>()?,
                location: String::from_utf8_lossy(&record[3])
                    .split_whitespace()
                    .map(String::from)
                    .collect(),
                part_number2: String::from_utf8_lossy(&record[4]).to_string(),
            });
        }

        Ok(Self {
            board_model,
            revision,
            extended_board_model,
            extended_revision,
            part_number,
            components,
        })
    }
}

/// A fully parsed PCB repair file, containing both content and description.
#[derive(Debug)]
pub struct ParsedPcbRepairFile {
    /// The parsed content of the file.
    pub content: Content,
    /// The parsed description of the file.
    pub description: Description,
}

impl ParsedPcbRepairFile {
    /// Parses a decoded PCB repair file into a structured format.
    ///
    /// # Arguments
    ///
    /// * `decoded` - The decoded file data.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `ParsedPcbRepairFile` or an error.
    pub fn from_decoded(
        decoded: &DecodedPcbRepairFile,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let content = Content::from_bytes(decoded.content.as_slice())?;
        let description = Description::from_bytes(decoded.description.as_slice())?;

        Ok(Self {
            content,
            description,
        })
    }
}

fn parse_decimal(s: &[u8]) -> Result<Decimal, Box<dyn std::error::Error>> {
    let s = String::from_utf8_lossy(s).to_string().replace(',', ".");
    Decimal::from_str(s.as_str()).map_err(|e| e.into())
}
