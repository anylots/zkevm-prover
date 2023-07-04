extern crate types as prove_types;
use prove_types::eth::BlockTrace;
use ethers::prelude::*;
use ethers::signers::Wallet;
use ethers::types::{Address};
use std::{error::Error, str::FromStr, sync::Arc};
use ethers::providers::{Http, Provider};

const CONTRACT_ADDRESS: &str = "0xf646fb8e8f78cf032663d1879ccaa967903741da";
const PRIVATE_KEY: &str = "0fd69a11726700cae9c24e2861d81f0fbfc93ef815d6a6c66ec52d8999048851";

async fn test() -> Result<(), Box<dyn Error>> {

    let result = call().await;
    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            println!("call error:");
            Err(e)
        }
    }
    
}

async fn call() -> Result<(), Box<dyn Error>> {
    let provider: Provider<Http> = Provider::<Http>::try_from("http://8.217.116.59:9933")?;
    let wallet: LocalWallet = Wallet::from_str(PRIVATE_KEY)?;

    let signer = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.with_chain_id(9000 as u64),
    ));

    abigen!(TestZkEVM, "./resource/abi/TestZkEVM.json");

    let contract: TestZkEVM<SignerMiddleware<Provider<Http>, _>> =
        TestZkEVM::new(Address::from_str(CONTRACT_ADDRESS)?, signer);

    let tx = contract.transfer(
        Address::from_str("0xa210b31C70737AA2E09A0fFC151CF21e18365954").unwrap(),
        10.into(),
    ).legacy();
    let receipt = tx.send().await;
    match receipt {
        Ok(sent_tx) => println!("====transaction ID: {:?}", sent_tx),
        Err(e) => println!("call exception: {:?}", e),
    }
    Ok(())
}

async fn deploy() -> Result<(), Box<dyn Error>> {
    let provider: Provider<Http> = Provider::<Http>::try_from("http://8.217.116.59:9933")?;
    let wallet: LocalWallet = Wallet::from_str(PRIVATE_KEY)?;

    let signer = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.with_chain_id(9000 as u64),
    ));

    abigen!(TestZkEVM, "./resource/abi/TestZkEVM.json");
    let a: u64 = 10;

    let tx = TestZkEVM::deploy(signer, a.pow(18))?.legacy();
    let contract = tx.send().await;

    match contract {
        Ok(sent_tx) => println!("====testZkEVM: {:?}", sent_tx),
        Err(e) => println!("call exception: {:?}", e),
    }

    Ok(())
}
