/*
 * unucl - command line decompressor using libucl
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

use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use memmap2::MmapMut;

use uclcli::{decompress, decompress_into_buffer, ucl_init};

/// Maximum allowed input size (2GB) to prevent memory exhaustion attacks (DoS).
const MAX_INPUT_SIZE: u64 = 2 * 1024 * 1024 * 1024;

/// Maximum allowed buffer size (1GB) to prevent resource exhaustion.
/// This matches the MAX_DST_CAPACITY in the library.
const MAX_BUFFER_SIZE: u32 = 1_073_741_824;

/// Default decompression buffer size (512MB).
/// This is a reasonable default for most use cases. Users can override this
/// with the --buffersize flag if they know the expected decompressed size.
const DEFAULT_BUFFER_SIZE: u32 = 512 * 1024 * 1024;

/// libucl (NRV) decompressor
#[derive(Parser)]
#[command(name = "unucl", version = "0.2", author = "Kjell Braden <kjell.braden@bmw.de>")]
struct Args {
    /// Sets the input file to use [defaults to stdin]
    #[arg(short = 'i', long = "input")]
    input: Option<PathBuf>,

    /// Sets the output file to use [defaults to stdout]
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,

    /// Sets the decompression buffer size - set this if you know how much data
    /// to expect after decompression [defaults to 512MB]
    #[arg(short = 'b', long = "buffersize")]
    buffersize: Option<u32>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    ucl_init();

    let buffer_size = args.buffersize.unwrap_or(DEFAULT_BUFFER_SIZE);

    if buffer_size > MAX_BUFFER_SIZE {
        anyhow::bail!(
            "buffer size {} exceeds maximum allowed {} bytes",
            buffer_size,
            MAX_BUFFER_SIZE
        );
    }

    let input: Box<dyn Read> = match &args.input {
        Some(path) => Box::new(
            OpenOptions::new()
                .read(true)
                .open(path)
                .context("could not open input file")?,
        ),
        None => Box::new(io::stdin()),
    };

    let mut inbuffer = Vec::new();
    let bytes_read = input
        .take(MAX_INPUT_SIZE + 1)
        .read_to_end(&mut inbuffer)
        .context("failed to read input")?;

    if bytes_read as u64 > MAX_INPUT_SIZE {
        anyhow::bail!(
            "input size exceeds maximum supported {} bytes",
            MAX_INPUT_SIZE
        );
    }

    match &args.output {
        Some(path) => {
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)
                .context("could not create output file")?;
            file.set_len(buffer_size.into())
                .context("could not resize output file")?;

            let numbytes = {
                let mut mmap =
                    unsafe { MmapMut::map_mut(&file).context("failed to map output file")? };
                let nb =
                    decompress_into_buffer(&inbuffer, &mut mmap).context("decompression failed")?;
                mmap.flush().context("failed to write output")?;
                nb
            };
            file.set_len(numbytes.into())
                .context("failed to truncate output file")?;
        }
        None => {
            let dst = decompress(&inbuffer, buffer_size).context("decompression failed")?;
            io::stdout().write_all(&dst)?;
        }
    }

    Ok(())
}
