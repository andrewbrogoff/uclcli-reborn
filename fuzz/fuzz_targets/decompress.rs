/*
 * decompress.rs - Fuzz target for UCL decompression
 * Copyright (C) 2020-2021  BMW Group
 * Copyright (C) 2026  Andrew Rogoff <andrew@andrewbrogoff.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

#![no_main]
use libfuzzer_sys::fuzz_target;

use uclcli::{ucl_init, decompress};

fuzz_target!(|data: &[u8]| {
    ucl_init();

    // Safely compute capacity, skipping cases where the multiplication or
    // conversion would overflow. This prevents fuzzer crashes on edge cases.
    let capacity = match data.len().checked_mul(1024) {
        Some(size) => match size.try_into() {
            Ok(c) => c,
            Err(_) => return, // Skip if capacity doesn't fit in u32
        },
        None => return, // Skip if multiplication overflows
    };

    let _result = decompress(data, capacity);
});
