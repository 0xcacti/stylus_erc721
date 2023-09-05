// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use crate::erc721::{ERC721Params, ERC721};
use alloc::{string::String, vec::Vec};
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    msg,
    prelude::*,
};

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
        self.erc721._mint(msg::sender(), self.token_id.into())?;
        Ok(())
    }

    pub fn token_uri(&self, token_id: U256) -> Result<String, Vec<u8>> {
        if self.erc721.owner_of(token_id)? == Address::ZERO {
            return Err("Token does not exist".into());
        }
    }
}
