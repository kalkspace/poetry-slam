use anyhow::{anyhow, Context, Result};
use std::{fs::File, io::Write};

use image::GenericImageView;

const MAX_WIDTH: u32 = 384;
const MAX_HEIGHT: u32 = 192;

const FOOTER_ROWS: usize = 4;

const PRINTER_PATH: &'static str = "/tmp/DEVTERM_PRINTER_IN";
const UNICODE_MODE: &'static str = "\x1b\x21\x01";

pub struct PoetryPrinter {
    printer: File,
    header: Vec<u8>,
    footer: Vec<u8>,
}

fn generate_footer() -> Vec<u8> {
    let footer = [
        [0xff, 0x0].repeat((MAX_WIDTH as usize) / 8 / 2 * 8),
        [0x0, 0xff].repeat((MAX_WIDTH as usize) / 8 / 2 * 8),
    ];

    let mut buf = vec![
        0x1d,
        0x76,
        0x30,
        0x0,
        (MAX_WIDTH / 8) as u8,
        0,
        (FOOTER_ROWS * 8) as u8,
        0,
    ];
    for i in 0..FOOTER_ROWS {
        buf.extend_from_slice(&footer[i % 2]);
    }
    buf
}

impl PoetryPrinter {
    pub fn new() -> Result<PoetryPrinter> {
        let printer = File::options()
            .append(true)
            .open(PRINTER_PATH)
            .with_context(|| format!("Failed to open printer path {}", PRINTER_PATH))?;

        let img = include_bytes!("poetryslam.png");
        let img = image::load_from_memory(img)?;

        let dimensions = img.dimensions();
        if dimensions.0 % 8 > 0 {
            return Err(anyhow!(
                "Image must have a width that is divisible by 8. Width: {}",
                dimensions.0,
            ));
        }
        if dimensions.0 > MAX_WIDTH {
            return Err(anyhow!(
                "Image too wide. Max: {}. Width: {}",
                MAX_WIDTH,
                dimensions.0,
            ));
        }

        if dimensions.1 > MAX_HEIGHT {
            return Err(anyhow!(
                "Image too high. Max: {}. Height: {}",
                MAX_HEIGHT,
                dimensions.1,
            ));
        }

        // https://github.com/clockworkpi/DevTerm/blob/main/Code/thermal_printer/devterm_thermal_printer.c#L669
        let mut converted = vec![
            0x1d,
            0x76,
            0x30,
            0x0,
            (img.dimensions().0 / 8) as u8,
            0,
            (img.dimensions().1) as u8,
            0,
        ];
        // convert each pixel to a bit
        let mut shift = 7u8;
        let mut b = 0u8;
        for pixel in img.pixels() {
            if pixel.2[0] == 0 {
                b += 1 << shift;
            }
            if shift == 0 {
                converted.push(b);
                b = 0;
                shift = 7;
            } else {
                shift = shift - 1;
            }
        }
        Ok(PoetryPrinter {
            header: converted,
            footer: generate_footer(),
            printer,
        })
    }

    pub fn print_poem(&mut self, name: &str, poem: &str, cheat_mode: bool) -> Result<()> {
        let poem = [
            b"\n\n\n",
            self.header.as_slice(),
            b"\n\n\n",
            UNICODE_MODE.as_bytes(),
            format!("Gedicht von {}", name).as_bytes(),
            b"\n\n\n",
            if cheat_mode {
                "#### CHEAT MODE ####\n\n"
            } else {
                ""
            }
            .as_bytes(),
            poem.as_bytes(),
            b"\n\n\n",
            self.footer.as_slice(),
            b"\n\n\n",
            b"\n\n\n",
            b"\n\n\n",
            b"\n\n\n",
            b"\n\n\n",
            b"\n\n\n",
            b"\n\n\n",
            b"\n\n\n",
            b"\n\n\n",
        ]
        .into_iter()
        .fold(vec![], |mut v, s| {
            v.extend_from_slice(s);
            v
        });
        self.printer
            .write(&poem)
            .with_context(|| "Failed to write poem")?;
        Ok(())
    }
}
