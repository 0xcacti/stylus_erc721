use core::marker::PhantomData;

use stylus_sdk::{
    alloy_primitives::{Address, U256},
    alloy_sol_types::{sol, SolError},
    evm, msg,
    prelude::*,
};

sol! {
    event Transfer(address indexed from, address indexed to, uint256 indexed id);
    event Approval(address indexed owner, address indexed spender, uint256 indexed id);
    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);

    error NonexistentToken(uint256 id);
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
}

impl From<ERC721Error> for Vec<u8> {
    fn from(err: ERC721Error) -> Vec<u8> {
        match err {
            ERC721Error::NonexistentToken(e) => format!("ERC721: nonexistent token {}", e)
                .to_string()
                .into_bytes(),
            NotOwner => b"ERC721: caller is not owner".to_vec(),
            NotAuthorized => b"ERC721: Not Authorized".to_vec(),
            InvalidRecipient => b"ERC721: Invalid Recipient".to_vec(),
            NotApprovedForAll => {
                b"ERC721: transfer caller is not owner nor approved for all".to_vec()
            }
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

    pub fn balance_of(&self, owner: Address) -> Result<U256, ERC721Error> {
        Ok(self._balanceOf.get(owner))
    }

    pub fn owner_of(&self, id: U256) -> Result<Address, ERC721Error> {
        if self._ownerOf.get(id) == Address::ZERO {
            return Err(ERC721Error::NonexistentToken(id));
        }
        Ok(self._ownerOf.get(id))
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

    pub fn get_approved(&self, id: U256) -> Result<Address, ERC721Error> {
        Ok(self.getApproved.get(id))
    }

    pub fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), ERC721Error> {
        let caller = msg::sender();
        self.isApprovedForAll
            .setter(msg::sender())
            .setter(operator)
            .set(approved);

        evm::log(ApprovalForAll {
            owner: msg::sender(),
            operator,
            approved,
        });

        Ok(())
    }

    pub fn is_approved_for_all(
        &self,
        owner: Address,
        operator: Address,
    ) -> Result<bool, ERC721Error> {
        Ok(self.isApprovedForAll.get(owner).get(operator))
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
        let mut balance_setter = self._balanceOf.setter(from);
        let mut balance_getter = self._balanceOf.getter(from);
        balance_setter.set(from) - U256::from(1));

        self._balanceOf
            .setter(to)
            .set(self._balanceOf.get(to) + U256::from(1));
        self._ownerOf.setter(id).set(to);
        self.getApproved.setter(id).set(Address::ZERO);
        evm::log(Transfer { from, to, id });
        Ok(())
    }
}
