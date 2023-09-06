// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use std::f64::consts::PI;

use crate::erc721::{ERC721Params, ERC721};
use alloc::{string::String, vec::Vec};
use base64::{engine::general_purpose, Engine as _};
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    msg,
    prelude::*,
};

use image::codecs::png::PngEncoder;
use image::ImageEncoder;
use image::RgbImage;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

/// Initializes a custom, global allocator for Rust programs compiled to WASM.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Import the Stylus SDK along with alloy primitive types for use in our program.
mod erc721;

struct JuliaParams;

impl ERC721Params for JuliaParams {
    const NAME: &'static str = "Julia";
    const SYMBOL: &'static str = "JUL";
}

sol_storage! {
    #[entrypoint]
    pub struct Julia {
        #[borrow]
        ERC721<JuliaParams> erc721;
        uint256 token_id;
    }
}

/// Define an implementation of the generated Counter struct, defining a set_number
/// and increment method using the features of the Stylus SDK.
#[external]
#[inherit(ERC721<JuliaParams>)]
impl Julia {
    #[payable]
    pub fn mint(&mut self) -> Result<(), Vec<u8>> {
        self.erc721
            ._mint(msg::sender(), self.token_id.clone().into())?;
        Ok(())
    }

    pub fn token_uri(&self, token_id: U256) -> Result<String, Vec<u8>> {
        if self.erc721.owner_of(token_id)? == Address::ZERO {
            return Err("Token does not exist".into());
        }

        let img = self.generate_julia(token_id)?;
        let mut buffer: Vec<u8> = Vec::new();
        let encoder = PngEncoder::new(&mut buffer);

        encoder
            .write_image(
                img.as_ref(),
                img.width(),
                img.height(),
                image::ColorType::Rgb8,
            )
            .expect("Failed to encode image to PNG");

        // let base64_img = Engine::encode(&buffer, &config);
        let base64_img = general_purpose::STANDARD.encode(&buffer);

        let html = format!(
            "<img src=\"data:image/png;base64,{}\" alt=\"Julia Set\">",
            base64_img
        );
        Ok(html)
    }
}

impl Julia {
    fn generate_julia(&self, token_id: U256) -> Result<RgbImage, Vec<u8>> {
        let mut rng = SmallRng::seed_from_u64(token_id.wrapping_to::<u64>());

        let width: u32 = 800;
        let height: u32 = 800;
        let max_iter: u32 = 1000;

        let theta: f64 = rng.gen_range(0.0..2.0 * PI);

        let cx = 0.7885 * theta.cos();
        let cy = 0.7885 * theta.sin();
        let scalex = 3.0 / width as f64;
        let scaley = 3.0 / height as f64;
        let mut img = RgbImage::new(width, height);

        for x in 0..width {
            for y in 0..height {
                let zx = x as f64 * scalex - 1.5;
                let zy = y as f64 * scaley - 1.5;
                let mut xi = zx;
                let mut yi = zy;
                let mut iter = 0;

                while xi * xi + yi * yi < 4.0 && iter < max_iter {
                    let temp = xi * xi - yi * yi + cx;
                    yi = 2.0 * xi * yi + cy;
                    xi = temp;
                    iter += 1;
                }

                let pixel = if iter == max_iter {
                    [0, 0, 0]
                } else {
                    let zn = (xi * xi + yi * yi).sqrt();
                    let nu = (zn.log2()).log2();
                    let smooth = (iter as f64 + 1.0 - nu) / max_iter as f64;
                    let color = self.gradient_color(smooth);
                    img.put_pixel(x, y, image::Rgb([color.0, color.1, color.2]));

                    [color.0, color.1, color.2]
                };

                img.put_pixel(x, y, image::Rgb(pixel));
            }
        }

        Ok(img)
    }

    fn gradient_color(&self, t: f64) -> (u8, u8, u8) {
        let r = (9.0 * (1.0 - t) * t * t * t * 255.0) as u8;
        let g = (15.0 * (1.0 - t) * (1.0 - t) * t * t * 255.0) as u8;
        let b = (8.5 * (1.0 - t) * (1.0 - t) * (1.0 - t) * t * 255.0) as u8;

        (r, g, b)
    }
}
