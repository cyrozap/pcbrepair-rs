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

use std::convert::TryInto;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

use flate2::read::ZlibDecoder;

const FZ_EXPANDED_KEY: [u32; 44] = [
    0x25d8d248, 0xe1502405, 0x56b5d486, 0x69213fe0, 0xa22490ec, 0x01fdd9fa, 0x0681955f, 0x0fac202d,
    0xdac9eeb4, 0xf6024aba, 0xcd8b4cc6, 0x9f307c8e, 0x4ab8fad7, 0x232f967d, 0x5e8666a3, 0xde966d4b,
    0xc64bfb1c, 0xea7fb092, 0x1a751a7e, 0x37e8f0bc, 0x3359c8f3, 0x969ac22b, 0x610f5804, 0xd99d10e6,
    0xc58d54d6, 0x1f9aea8b, 0x8e388c1a, 0xe4f7d2ed, 0x3e5da1f6, 0xedfe818a, 0x7252b016, 0xb503a170,
    0xc4128fb6, 0x2c93ceeb, 0x53539a6e, 0xdacf7668, 0x3ab78e52, 0x8ee9d815, 0x7043f799, 0xc6a05dcf,
    0x727f1da2, 0x0dfd983b, 0x78c53872, 0x00945692,
];

const CAE_EXPANDED_KEY: [u32; 44] = [
    0x477fa6a2, 0xfb9b5e2b, 0x77bcac57, 0x2d7cef8c, 0x69825182, 0xfa231194, 0x96ee6d48, 0x520a9b74,
    0x0619cb60, 0x95918dfb, 0x1c829771, 0x03f6655c, 0xbba3b302, 0xf3cbcc66, 0xb42e9ac7, 0x417b37dd,
    0x34854b8c, 0xf95a9547, 0x7950401e, 0xc3271f83, 0x0e7c9a6e, 0xcfa7f799, 0x616d9d05, 0x200ac08f,
    0x7cdb242f, 0x30d3bc5e, 0x2983cc29, 0x9da249c9, 0x7509f015, 0x6632580e, 0x83247f04, 0x6525ed71,
    0x02fa242a, 0x47b12928, 0x7ed51b5d, 0xf69cd51b, 0x66f24c77, 0x042856b9, 0x00e37970, 0x88b6624d,
    0x6826cd76, 0xd2a4c9fe, 0x2eff487a, 0x09648fae,
];

const LOGW: u32 = 5;
const ROUNDS: usize = 20;

fn rc6_encrypt_block(block: &[u8; 16], expanded_key: &[u32; 44]) -> (u32, u32, u32, u32) {
    let mut a = u32::from_le_bytes(block[0..4].try_into().unwrap());
    let mut b = u32::from_le_bytes(block[4..8].try_into().unwrap());
    let mut c = u32::from_le_bytes(block[8..12].try_into().unwrap());
    let mut d = u32::from_le_bytes(block[12..16].try_into().unwrap());

    b = b.wrapping_add(expanded_key[0]);
    d = d.wrapping_add(expanded_key[1]);

    for i in 1..=ROUNDS {
        let t = (b.wrapping_mul(2u32.wrapping_mul(b) + 1)).rotate_left(LOGW);
        let u = (d.wrapping_mul(2u32.wrapping_mul(d) + 1)).rotate_left(LOGW);
        a = (a ^ t).rotate_left(u).wrapping_add(expanded_key[2 * i]);
        c = (c ^ u).rotate_left(t).wrapping_add(expanded_key[2 * i + 1]);

        let temp = a;
        a = b;
        b = c;
        c = d;
        d = temp;
    }

    a = a.wrapping_add(expanded_key[2 * ROUNDS + 2]);
    c = c.wrapping_add(expanded_key[2 * ROUNDS + 3]);

    (a, b, c, d)
}

fn decrypt(data: &[u8], expanded_key: &[u32; 44]) -> Vec<u8> {
    let mut result = data.to_vec();
    let mut keystream = [0u8; 16];

    for i in 0..result.len() {
        let (a, _b, _c, _d): (u32, u32, u32, u32) = rc6_encrypt_block(&keystream, expanded_key);

        keystream.copy_within(1..16, 0);
        keystream[15] = result[i];

        result[i] ^= <u32 as TryInto<u8>>::try_into(a & 0xFF).unwrap();
    }

    result
}

fn decompress(capacity: usize, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut buffer = Vec::with_capacity(capacity);
    let s = decoder.read_to_end(&mut buffer)?;
    if s != capacity {
        return Err("Decompressed size mismatch".into());
    }
    Ok(buffer)
}

#[derive(Debug)]
pub struct DecodedPcbRepairFile {
    pub content: Vec<u8>,
    pub description: Vec<u8>,
}

impl DecodedPcbRepairFile {
    pub fn from_filename(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(filename)?;
        let mut reader = BufReader::new(file);

        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        fn try_process(
            data: &[u8],
            key: Option<&[u32; 44]>,
        ) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
            let decrypted = match key {
                Some(k) => {
                    let d = decrypt(data, k);
                    d
                }
                None => data.to_vec(),
            };

            if decrypted[4] != 0x78 {
                return Err("Invalid zlib header".into());
            }

            let pointer_offset_maybe: usize =
                u32::from_le_bytes(decrypted[decrypted.len() - 4..].try_into().unwrap())
                    .try_into()
                    .unwrap();

            let content_len: usize = u32::from_le_bytes(decrypted[..4].try_into().unwrap())
                .try_into()
                .unwrap();
            let content = decompress(content_len, &decrypted[4..])?;

            let pointer_maybe: usize = u32::from_le_bytes(
                decrypted[decrypted.len() - pointer_offset_maybe - 4
                    ..decrypted.len() - pointer_offset_maybe]
                    .try_into()
                    .unwrap(),
            )
            .try_into()
            .unwrap();

            let description_len: usize = u32::from_le_bytes(
                decrypted[pointer_maybe..pointer_maybe + 4]
                    .try_into()
                    .unwrap(),
            )
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
