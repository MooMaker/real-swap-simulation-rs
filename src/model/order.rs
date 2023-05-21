use crate::MOO_CONTRACT_ADDRESS;
use ethers::abi::{encode, Token};
use ethers::contract::abigen;
use ethers::prelude::transaction::eip712::{EIP712Domain, Eip712};
use ethers::prelude::{Address, U256};
use ethers::utils::keccak256;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

abigen!(MooMaker, "contracts/settlement_contract.json");

pub struct EipErr {}

impl Debug for EipErr {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Display for EipErr {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for EipErr {}

impl Eip712 for Order {
    type Error = EipErr;

    fn domain(&self) -> Result<EIP712Domain, Self::Error> {
        Ok(EIP712Domain {
            name: Some("MooMaker".into()),
            version: Some("1".into()),
            chain_id: Some(5.into()),
            verifying_contract: Some(Address::from_str(MOO_CONTRACT_ADDRESS).unwrap()),
            salt: None,
        })
    }

    fn type_hash() -> Result<[u8; 32], Self::Error> {
        Ok(keccak256(b"Order(address tokenIn,uint256 amountIn,address tokenOut,uint256 amountOut,uint256 validTo,address maker,bytes uid)"))
    }

    fn struct_hash(&self) -> Result<[u8; 32], Self::Error> {
        let tokens: [Token; 8] = [
            Token::Uint(U256::from(Self::type_hash().unwrap())),
            Token::Address(self.token_in),
            Token::Uint(self.amount_in),
            Token::Address(self.token_out),
            Token::Uint(self.amount_out),
            Token::Uint(self.valid_to),
            Token::Address(self.maker),
            Token::Bytes(self.uid.to_vec()),
        ];
        Ok(keccak256(encode(&tokens)))
    }
}
