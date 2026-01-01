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

use std::string::String;

use csv;

use crate::decoder::DecodedPcbRepairFile;

enum ParserState {
    Unknown,
    Symbol,
    Net,
    Via,
    TestVia,
    GraphicData,
    ClassedGraphicData,
}

#[derive(Debug)]
pub enum Units {
    Mils,
    Millimeters,
}

#[derive(Debug)]
pub struct Symbol {
    pub refdes: String,
    pub comp_insertion_code: u64,
    pub sym_name: String,
    pub sym_mirror: bool,
    pub sym_rotate: u16,
}

#[derive(Debug)]
pub struct Net {
    pub net_name: String,
    pub refdes: String,
    pub pin_number: String,
    pub pin_name: String,
    pub pin_x: f64,
    pub pin_y: f64,
    pub test_point: String,
    pub radius: f64,
}

#[derive(Debug)]
pub struct TestVia {
    pub testvia: String,
    pub net_name: String,
    pub refdes: String,
    pub pin_number: String,
    pub pin_name: String,
    pub via_x: f64,
    pub via_y: f64,
    pub test_point: String,
    pub radius: f64,
}

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

#[derive(Debug)]
pub struct Content {
    pub units: Units,
    pub symbols: Vec<Symbol>,
    pub nets: Vec<Net>,
    pub testvias: Vec<TestVia>,
    pub graphic_data: Vec<GraphicData>,
    pub classed_graphic_data: Vec<ClassedGraphicData>,
}

impl Content {
    pub fn from_bytes(content: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut symbols = Vec::new();
        let mut nets = Vec::new();
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
                    state = ParserState::Net;
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
                    ParserState::Net => {
                        nets.push(Net {
                            net_name: String::from_utf8_lossy(&record[1]).to_string(),
                            refdes: String::from_utf8_lossy(&record[2]).to_string(),
                            pin_number: String::from_utf8_lossy(&record[3]).to_string(),
                            pin_name: String::from_utf8_lossy(&record[4]).to_string(),
                            pin_x: parse_float(&record[5])?,
                            pin_y: parse_float(&record[6])?,
                            test_point: String::from_utf8_lossy(&record[7]).to_string(),
                            radius: parse_float(&record[8])?,
                        });
                    }
                    ParserState::TestVia => {
                        testvias.push(TestVia {
                            testvia: String::from_utf8_lossy(&record[1]).to_string(),
                            net_name: String::from_utf8_lossy(&record[2]).to_string(),
                            refdes: String::from_utf8_lossy(&record[3]).to_string(),
                            pin_number: String::from_utf8_lossy(&record[4]).to_string(),
                            pin_name: String::from_utf8_lossy(&record[5]).to_string(),
                            via_x: parse_float(&record[6])?,
                            via_y: parse_float(&record[7])?,
                            test_point: String::from_utf8_lossy(&record[8]).to_string(),
                            radius: parse_float(&record[9])?,
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
            nets,
            testvias,
            graphic_data,
            classed_graphic_data,
        })
    }
}

#[derive(Debug)]
pub struct Component {
    pub part_number: String,
    pub description: String,
    pub quantity: u64,
    pub location: Vec<String>,
    pub part_number2: String,
}

#[derive(Debug)]
pub struct Description {
    pub board_model: String,
    pub revision: String,
    pub extended_board_model: String,
    pub extended_revision: String,
    pub part_number: String,
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

#[derive(Debug)]
pub struct ParsedPcbRepairFile {
    pub content: Content,
    pub description: Description,
}

impl ParsedPcbRepairFile {
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

fn parse_float(s: &[u8]) -> Result<f64, Box<dyn std::error::Error>> {
    let s = String::from_utf8_lossy(s).to_string().replace(',', ".");
    s.parse::<f64>().map_err(|e| e.into())
}
