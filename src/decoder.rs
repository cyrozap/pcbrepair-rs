// SPDX-License-Identifier: GPL-3.0-or-later

/*
 *  src/decoder.rs - Decoder library for ASUS FZ and ASRock CAE files.
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
 * # `decoder` Module
 *
 * This module provides functionality to decode ASUS FZ and ASRock CAE files.
 * It handles both decryption and decompression of the file content.
 *
 * ## Usage Example
 *
 * ```no_run
 * use std::fs::File;
 * use std::io::BufReader;
 *
 * use pcbrepair::decoder::DecodedPcbRepairFile;
 *
 * fn main() -> Result<(), Box<dyn std::error::Error>> {
 *     // Open the file
 *     let file = File::open("example.fz")?;
 *     let reader = BufReader::new(file);
 *
 *     // Decode the file
 *     let decoded = DecodedPcbRepairFile::new(reader)?;
 *
 *     // Use decoded.content and decoded.description here
 *
 *     Ok(())
 * }
 * ```
 */

use std::io::prelude::*;

use flate2::read::ZlibDecoder;

use crate::crypto::*;

fn decompress(capacity: usize, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut buffer = Vec::with_capacity(capacity);
    let s = decoder.read_to_end(&mut buffer)?;
    if s != capacity {
        return Err("Decompressed size mismatch".into());
    }
    Ok(buffer)
}

/// A decoded PCB repair file, containing raw content and description data.
#[derive(Debug)]
pub struct DecodedPcbRepairFile {
    /// The decoded content of the file.
    pub content: Vec<u8>,
    /// The decoded description of the file.
    pub description: Vec<u8>,
}

impl DecodedPcbRepairFile {
    /// Reads and decodes a PCB repair file from a reader.
    ///
    /// This function handles decryption and decompression of the file.
    ///
    /// # Arguments
    ///
    /// * `reader` - A reader over the file data.
    ///
    /// # Returns
    ///
    /// A `Result` containing the decoded file or an error.
    pub fn new<R: std::io::Read>(mut reader: R) -> Result<Self, Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        fn try_process(
            data: &[u8],
            key: Option<&[u32; 44]>,
        ) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
            let decrypted = match key {
                Some(k) => decrypt(data, k),
                None => data.to_vec(),
            };

            if decrypted.get(4) != Some(&0x78) {
                return Err("Invalid zlib header".into());
            }

            let content_len_bytes = decrypted
                .get(..4)
                .ok_or("Not enough data to read content length")?;
            let content_len: usize = u32::from_le_bytes(content_len_bytes.try_into().unwrap())
                .try_into()
                .unwrap();

            let content = decompress(content_len, &decrypted[4..])?;

            let pointer_offset_maybe_bytes = decrypted
                .get((decrypted.len() - 4)..)
                .ok_or("Not enough data to read pointer offset")?;
            let pointer_offset_maybe: usize =
                u32::from_le_bytes(pointer_offset_maybe_bytes.try_into().unwrap())
                    .try_into()
                    .unwrap();

            let pointer_maybe_start = decrypted.len() - pointer_offset_maybe - 4;
            let pointer_maybe_bytes = decrypted
                .get(pointer_maybe_start..pointer_maybe_start + 4)
                .ok_or("Not enough data to read pointer value")?;
            let pointer_maybe: usize = u32::from_le_bytes(pointer_maybe_bytes.try_into().unwrap())
                .try_into()
                .unwrap();

            let description_len_bytes = decrypted
                .get(pointer_maybe..pointer_maybe + 4)
                .ok_or("Not enough data to read description length")?;
            let description_len: usize =
                u32::from_le_bytes(description_len_bytes.try_into().unwrap())
                    .try_into()
                    .unwrap();

            let description = decompress(
                description_len,
                &decrypted[pointer_maybe + 4..decrypted.len() - 4],
            )?;

            Ok((content, description))
        }

        let (content, description) = try_process(&buffer, None)
            .or_else(|_| try_process(&buffer, Some(&FZ_EXPANDED_KEY)))
            .or_else(|_| try_process(&buffer, Some(&CAE_EXPANDED_KEY)))?;

        Ok(Self {
            content,
            description,
        })
    }
}
