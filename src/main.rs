mod model;

use crate::model::erc20::ERC20;
use crate::model::order::{EipErr, MooMaker, Order};
use crate::model::weth::WETH;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::Wallet;
use ethers::types::{Signature, H160, H256};
use ethers::{
    core::types::{Address, U256},
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::transaction::eip712::Eip712,
};
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

static CHAIN_ID: Mutex<Option<u64>> = Mutex::const_new(None);
const MOO_CONTRACT_ADDRESS: &str = "0xcEe38fB7D7c6ed6BABc18898BDEF67ED572Cc9D0";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let provider = create_provider().await?;

    let router = MOO_CONTRACT_ADDRESS.parse::<Address>()?;
    let router = MooMaker::new(router, provider.clone());

    let order = create_order().await;
    let signature = sign_order(&order).await?;

    println!("{signature}");

    // allows WETH.json
    // allow_settlement_contract_weth(
    //     order.token_out,
    //     order.amount_out,
    // ).await?;

    // allows MooCoin
    // allow_settlement_contract(
    //     order.token_in,
    //     order.amount_in,
    // ).await?;

    router
        .swap(order, signature.to_vec().into())
        .send()
        .await?
        .await?
        .unwrap();

    Ok(())
}

#[allow(dead_code)]
async fn allow_settlement_contract(token: Address, amount: U256) -> Result<(), Box<dyn Error>> {
    let provider = create_provider().await?;
    let router = ERC20::new(token, provider.clone());
    let settlement_contract_address = MOO_CONTRACT_ADDRESS.parse()?;

    let tx = router.approve(settlement_contract_address, amount);
    let mined = tx.send().await?;
    let result = mined.await?.unwrap();
    println!("{:#?}", result);
    Ok(())
}

#[allow(dead_code)]
async fn allow_settlement_contract_weth(
    address: Address,
    amount: U256,
) -> Result<(), Box<dyn Error>> {
    let provider = create_provider().await?;
    let router = WETH::new(address, provider.clone());
    let settlement_contract_address = MOO_CONTRACT_ADDRESS.parse()?;

    let tx = router.approve(settlement_contract_address, amount);
    let mined = tx.send().await?;
    let result = mined.await?.unwrap();
    println!("{:#?}", result);
    Ok(())
}

async fn create_provider(
) -> Result<Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>, Box<dyn Error>> {
    Ok(Arc::new({
        // connect to the network
        let provider = Provider::<Http>::try_from(
            "https://goerli.infura.io/v3/cbed669ca3324c7a80325674c0edf7a5",
        )?;
        *CHAIN_ID.lock().await = Some(provider.get_chainid().await?.as_u64());
        let wallet = create_maker_wallet().await;

        SignerMiddleware::new(provider, wallet)
    }))
}

async fn sign_order(order: &Order) -> Result<Signature, EipErr> {
    let encoded = order.encode_eip712()?;
    let signature = sign_hash(encoded.into()).await;
    Ok(signature)
}

async fn sign_hash(hash: H256) -> Signature {
    let wallet = create_maker_wallet().await;
    wallet.sign_hash(hash).unwrap()
}

async fn create_maker_wallet() -> Wallet<SigningKey> {
    // private key
    let maker_private_key =
        std::env::var("MAKER_PRIVATE_KEY").expect("Set MAKER_PRIVATE_KEY env variable");
    maker_private_key
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(CHAIN_ID.lock().await.unwrap())
}

async fn create_order() -> Order {
    let amount_in = U256::from_dec_str("100000000000000000").unwrap();
    let token_in = H160::from_str("0x6778aC35E1C9aca22a8D7d820577212A89544Df9").unwrap();
    let amount_out = U256::from_dec_str("1000000000000000000").unwrap();
    let token_out = H160::from_str("0xB4FBF271143F4FBf7B91A5ded31805e42b2208d6").unwrap();
    let valid_to = U256::from_dec_str("1747179215").unwrap();
    let maker = create_maker_wallet().await.address();

    Order {
        token_in,
        amount_in,
        token_out,
        amount_out,
        valid_to,
        maker,
        uid: vec![1].into(),
    }
}
