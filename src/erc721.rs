use alloc::{string::String, vec::Vec};
use core::marker::PhantomData;
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    alloy_sol_types::sol,
    evm, msg,
    prelude::*,
};

sol! {
    event Transfer(address indexed from, address indexed to, uint256 indexed id);
    event Approval(address indexed owner, address indexed spender, uint256 indexed id);
    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);

    error NonexistentToken(uint256 id);
    error NotOwner();
    error NotAuthorized();
    error InvalidRecipient();
    error NotApprovedForAll();

}

pub trait ERC721Params {
    const NAME: &'static str;
    const SYMBOL: &'static str;
}

sol_storage! {
    pub struct ERC721<T> {
        string name;
        string symbol;
        mapping (uint256 => address) _ownerOf;
        mapping (address => uint256) _balanceOf;
        mapping (uint256 => address) getApproved;
        mapping (address => mapping (address => bool)) isApprovedForAll;
        PhantomData<T> phantom;

    }
}

pub enum ERC721Error {
    NotOwner,
    NotAuthorized,
    InvalidRecipient,
    NotApprovedForAll,
    NonexistentToken(U256),
    NotMinted,
}

impl From<ERC721Error> for Vec<u8> {
    fn from(err: ERC721Error) -> Vec<u8> {
        match err {
            ERC721Error::NonexistentToken(e) => format!("ERC721: nonexistent token {}", e)
                .to_string()
                .into_bytes(),
            ERC721Error::NotOwner => b"ERC721: caller is not owner".to_vec(),
            ERC721Error::NotAuthorized => b"ERC721: Not Authorized".to_vec(),
            ERC721Error::InvalidRecipient => b"ERC721: Invalid Recipient".to_vec(),
            ERC721Error::NotApprovedForAll => {
                b"ERC721: transfer caller is not owner nor approved for all".to_vec()
            }
            ERC721Error::NotMinted => b"ERC721: not minted".to_vec(),
        }
    }
}
/// Define an implementation of the generated Counter struct, defining a set_number
/// and increment method using the features of the Stylus SDK.
#[external]
impl<T: ERC721Params> ERC721<T> {
    /// Gets the number from storage.
    pub fn name(&self) -> Result<String, ERC721Error> {
        Ok(T::NAME.into())
    }

    pub fn symbol(&self) -> Result<String, ERC721Error> {
        Ok(T::NAME.into())
    }

    pub fn owner_of(&self, id: U256) -> Result<Address, ERC721Error> {
        if self._ownerOf.get(id) == Address::ZERO {
            return Err(ERC721Error::NonexistentToken(id));
        }
        Ok(self._ownerOf.get(id))
    }

    pub fn balance_of(&self, owner: Address) -> Result<U256, ERC721Error> {
        Ok(self._balanceOf.get(owner))
    }

    pub fn get_approved(&self, id: U256) -> Result<Address, ERC721Error> {
        Ok(self.getApproved.get(id))
    }

    pub fn is_approved_for_all(
        &self,
        owner: Address,
        operator: Address,
    ) -> Result<bool, ERC721Error> {
        Ok(self.isApprovedForAll.get(owner).get(operator))
    }

    pub fn approve(&mut self, spender: Address, id: U256) -> Result<(), ERC721Error> {
        let owner = self._ownerOf.get(id);
        let caller = msg::sender();
        if caller != owner || self.is_approved_for_all(owner, caller)? {
            return Err(ERC721Error::NotOwner);
        }
        let mut approvee = self.getApproved.setter(id);
        approvee.set(spender);

        evm::log(Approval { owner, spender, id });
        Ok(())
    }

    pub fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), ERC721Error> {
        let caller = msg::sender();

        self.isApprovedForAll
            .setter(caller)
            .setter(operator)
            .set(approved);

        evm::log(ApprovalForAll {
            owner: msg::sender(),
            operator,
            approved,
        });

        Ok(())
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
    ) -> Result<(), ERC721Error> {
        if self._ownerOf.get(id) != from {
            return Err(ERC721Error::NotOwner.into());
        }

        if to == Address::ZERO {
            return Err(ERC721Error::NotAuthorized.into());
        }
        let caller = msg::sender();

        if caller != from
            && !self.is_approved_for_all(from, caller)?
            && self.get_approved(id)? != caller
        {
            return Err(ERC721Error::NotAuthorized.into());
        }

        let old_balance_from = self._balanceOf.get(from);
        let mut from_balance_setter = self._balanceOf.setter(from);
        from_balance_setter.set(old_balance_from - U256::from(1));

        let old_balance_to = self._balanceOf.get(to);
        let mut to_balance_setter = self._balanceOf.setter(to);
        to_balance_setter.set(old_balance_to + U256::from(1));

        let mut owner_of_setter = self._ownerOf.setter(id);
        owner_of_setter.set(to);

        let mut approved_setter = self.getApproved.setter(id);
        approved_setter.set(Address::ZERO);
        evm::log(Transfer { from, to, id });
        Ok(())
    }

    pub fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
    ) -> Result<(), ERC721Error> {
        self.transfer_from(from, to, id)?;

        // Solidity equivalent
        // require(
        //    to.code.length == 0 ||
        //        ERC721TokenReceiver(to).onERC721Received(msg.sender, from, id, "") ==
        //        ERC721TokenReceiver.onERC721Received.selector,
        //    "UNSAFE_RECIPIENT"
        // );

        Ok(())
    }

    pub fn safe_transfer_from_with_data(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        data: Vec<u8>,
    ) -> Result<(), ERC721Error> {
        self.transfer_from(from, to, id)?;

        // Solidity equivalent
        // require(
        //     to.code.length == 0 ||
        //         ERC721TokenReceiver(to).onERC721Received(msg.sender, from, id, data) ==
        //         ERC721TokenReceiver.onERC721Received.selector,
        //     "UNSAFE_RECIPIENT"
        // );

        Ok(())
    }

    // I have no idea how this is supposed to work
    pub fn supports_interface(&self, interface_id: Vec<u8>) -> Result<bool, ERC721Error> {
        Ok(interface_id == vec![0x01, 0xff, 0xc9, 0xa7]
            || interface_id == vec![0x80, 0xac, 0x58, 0xcd]
            || interface_id == vec![0x5b, 0x5e, 0x13, 0x9f])
    }
}

// Implement internal methods
impl<T: ERC721Params> ERC721<T> {
    pub fn _mint(&mut self, to: Address, id: U256) -> Result<(), ERC721Error> {
        if self._ownerOf.get(id) != Address::ZERO {
            return Err(ERC721Error::InvalidRecipient);
        }

        let old_balance = self._balanceOf.get(to);
        let mut balance_setter = self._balanceOf.setter(to);
        balance_setter.set(old_balance + U256::from(1));

        let mut owner_of_setter = self._ownerOf.setter(id);
        owner_of_setter.set(to);

        evm::log(Transfer {
            from: Address::ZERO,
            to,
            id,
        });
        Ok(())
    }

    pub fn _burn(&mut self, owner: Address, id: U256) -> Result<(), ERC721Error> {
        if self._ownerOf.get(id) != owner {
            return Err(ERC721Error::NotMinted);
        }

        let old_balance = self._balanceOf.get(owner);
        let mut balance_setter = self._balanceOf.setter(owner);
        balance_setter.set(old_balance - U256::from(1));

        let mut owner_of_setter = self._ownerOf.setter(id);
        owner_of_setter.set(Address::ZERO);

        let mut approved_setter = self.getApproved.setter(id);
        approved_setter.set(Address::ZERO);

        evm::log(Transfer {
            from: owner,
            to: Address::ZERO,
            id,
        });
        Ok(())
    }

    // pub fn _safe_mint(&mut self, to: Address, id: U256) -> Result<(), ERC721Error> {
    //     self._mint(to, id)?;
    //
    //     // Solidity equivalent
    //     // require(
    //     //    to.code.length == 0 ||
    //     //        ERC721TokenReceiver(to).onERC721Received(msg.sender, address(0), id, "") ==
    //     //        ERC721TokenReceiver.onERC721Received.selector,
    //     //    "UNSAFE_RECIPIENT"
    //     // );
    //     Ok(())
    // }

    // pub fn _same_mint_with_data(
    //     &mut self,
    //     to: Address,
    //     id: U256,
    //     data: Vec<u8>,
    // ) -> Result<(), ERC721Error> {
    //    self._mint(to, id)?;
    //
    //      // Solidity equivalent
    //      // require(
    //      //    to.code.length == 0 ||
    //      //        ERC721TokenReceiver(to).onERC721Received(msg.sender, address(0), id, "") ==
    //      //        ERC721TokenReceiver.onERC721Received.selector,
    //      //    "UNSAFE_RECIPIENT"
    //      // );
    //      //    Ok(())
    // }
}

// trait SafeTransferFromWithData {
//     fn safe_transfer_from(
//         &mut self,
//         from: Address,
//         to: Address,
//         id: U256,
//         data: Vec<u8>,
//     ) -> Result<(), ERC721Error>;
// }
//
// trait SafeTransferFrom {
//     fn safe_transfer_from(
//         &mut self,
//         from: Address,
//         to: Address,
//         id: U256,
//     ) -> Result<(), ERC721Error>;
// }
//
// #[external]
// impl<T: ERC721Params> SafeTransferFrom for ERC721<T> {
//     fn safe_transfer_from(
//         &mut self,
//         from: Address,
//         to: Address,
//         id: U256,
//     ) -> Result<(), ERC721Error> {
//         self.safe_transfer_from_with_data(from, to, id, Vec::new())
//     }
// }
//
// #[external]
// impl<T: ERC721Params> SafeTransferFromWithData for ERC721<T> {
//     fn safe_transfer_from(
//         &mut self,
//         from: Address,
//         to: Address,
//         id: U256,
//         data: Vec<u8>,
//     ) -> Result<(), ERC721Error> {
//         self.transfer_from(from, to, id)?;
//         if !self.check_on_erc721_received(from, to, id, data)? {
//             return Err(ERC721Error::InvalidRecipient);
//         }
//         Ok(())
//     }
// }
