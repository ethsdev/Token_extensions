mod utils;

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signature::Keypair, signer::Signer,
    system_instruction, transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address_with_program_id, instruction::create_associated_token_account,
};
use spl_token_2022::{
    extension::ExtensionType,
    instruction::{
        burn_checked, initialize_mint, initialize_permanent_delegate, mint_to, transfer_checked,
    },
    state::Mint,
};
use utils::check_request_airdrop;

fn main() {
    let client = RpcClient::new("http://localhost:8899".to_string());

    let mint_authority = Keypair::new();
    let mint_account = Keypair::new();
    let party1 = Keypair::new();

    let decimals = 0;
    let mint_pubkey = mint_account.pubkey();

    println!("AUTHORITY: {}", mint_authority.pubkey());
    println!("MINT: {}", mint_pubkey);
    println!("TOKEN DECIMALS: {}", decimals);

    // Ensure accounts are funded
    check_request_airdrop(&client, &mint_authority.pubkey(), 2);
    check_request_airdrop(&client, &party1.pubkey(), 2);

    initialize_token_mint(&client, &mint_authority, &mint_account, decimals);

    let mint_authority_ata = create_ata(&client, &mint_authority, &mint_authority.pubkey(), &mint_pubkey);
    let party1_ata = create_ata(&client, &party1, &party1.pubkey(), &mint_pubkey);

    println!("PARTY1: {}", party1.pubkey());
    println!("PARTY1_ATA: {}", party1_ata);
    println!("MINT_AUTHORITY_ATA: {}", mint_authority_ata);

    mint_tokens(&client, &mint_authority, &mint_account, &party1_ata, 10);
    transfer_tokens(&client, &mint_authority, &mint_pubkey, &party1_ata, &mint_authority_ata, 10);
    burn_tokens(&client, &mint_authority, &mint_pubkey, &mint_authority_ata, 3);
}

fn initialize_token_mint(client: &RpcClient, authority: &Keypair, mint: &Keypair, decimals: u8) {
    let extensions = [ExtensionType::PermanentDelegate];
    let mint_len = ExtensionType::try_calculate_account_len::<Mint>(&extensions).unwrap();
    let rent = client.get_minimum_balance_for_rent_exemption(mint_len).unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &authority.pubkey(),
                &mint.pubkey(),
                rent,
                mint_len as u64,
                &spl_token_2022::id(),
            ),
            initialize_permanent_delegate(&spl_token_2022::id(), &mint.pubkey(), &authority.pubkey()).unwrap(),
            initialize_mint(
                &spl_token_2022::id(),
                &mint.pubkey(),
                &authority.pubkey(),
                Some(&authority.pubkey()),
                decimals,
            ).unwrap(),
        ],
        Some(&authority.pubkey()),
        &[authority, mint],
        client.get_latest_blockhash().unwrap(),
    );
    client.send_and_confirm_transaction_with_spinner(&tx).unwrap();
}

fn create_ata(client: &RpcClient, payer: &Keypair, owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    let ata = get_associated_token_address_with_program_id(owner, mint, &spl_token_2022::id());
    let tx = Transaction::new_signed_with_payer(
        &[create_associated_token_account(
            &payer.pubkey(),
            owner,
            mint,
            &spl_token_2022::id(),
        )],
        Some(&payer.pubkey()),
        &[payer],
        client.get_latest_blockhash().unwrap(),
    );
    client.send_and_confirm_transaction_with_spinner(&tx).unwrap();
    ata
}

fn mint_tokens(client: &RpcClient, authority: &Keypair, mint: &Keypair, destination: &Pubkey, amount: u64) {
    let tx = Transaction::new_signed_with_payer(
        &[mint_to(
            &spl_token_2022::id(),
            &mint.pubkey(),
            destination,
            &authority.pubkey(),
            &[&authority.pubkey(), &mint.pubkey()],
            amount,
        ).unwrap()],
        Some(&authority.pubkey()),
        &[authority, mint],
        client.get_latest_blockhash().unwrap(),
    );
    client.send_and_confirm_transaction_with_spinner(&tx).unwrap();
}

fn transfer_tokens(client: &RpcClient, authority: &Keypair, mint: &Pubkey, source: &Pubkey, destination: &Pubkey, amount: u64) {
    let tx = Transaction::new_signed_with_payer(
        &[transfer_checked(
            &spl_token_2022::id(),
            source,
            mint,
            destination,
            &authority.pubkey(),
            &[&authority.pubkey()],
            amount,
            0,
        ).unwrap()],
        Some(&authority.pubkey()),
        &[authority],
        client.get_latest_blockhash().unwrap(),
    );
    client.send_and_confirm_transaction_with_spinner(&tx).unwrap();
}

fn burn_tokens(client: &RpcClient, authority: &Keypair, mint: &Pubkey, from: &Pubkey, amount: u64) {
    let tx = Transaction::new_signed_with_payer(
        &[burn_checked(
            &spl_token_2022::id(),
            from,
            mint,
            &authority.pubkey(),
            &[&authority.pubkey()],
            amount,
            0,
        ).unwrap()],
        Some(&authority.pubkey()),
        &[authority],
        client.get_latest_blockhash().unwrap(),
    );
    client.send_and_confirm_transaction_with_spinner(&tx).unwrap();
}
