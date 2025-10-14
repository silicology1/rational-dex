use partial_idl_parser::AnchorIdlPartialData;
use partial_idl_parser::{get_idl, idl_custom_path};

use std::env;
use std::path::PathBuf;
use std::{error::Error, path::Path, rc::Rc};

use spl_token::state::Mint;
use {
    borsh::{BorshDeserialize, BorshSerialize},
    litesvm::LiteSVM,
    solana_account::Account,
    solana_instruction::{account_meta::AccountMeta, Instruction},
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_program_option::COption,
    solana_program_pack::Pack,
    solana_pubkey::{pubkey, Pubkey},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    spl_token::{
        state::{Account as TokenAccount, AccountState},
        ID as TOKEN_PROGRAM_ID,
    },
};

use solana_program::rent::Rent;

use solana_program::system_instruction::create_account;
const IDL_RAW_DATA: &str = idl_custom_path!(concat!(
    env!("CARGO_WORKSPACE_DIR"),
    "/target/idl/",
    "rational_dex.json"
));

/// Replace with your program id (you provided this earlier).
const PROGRAM_ID: Pubkey = pubkey!("EEL1Q3J9MjPxTWagTKE39jpUVBjUg7q283ztTVzbveDz");

const PROGRAM_BYTES: &[u8] = include_bytes!("../../target/deploy/rational_dex.so");

#[test]
fn test_create_proposal() {
    let program_id = PROGRAM_ID;

    let mut svm = LiteSVM::new();

    svm.add_program(program_id, PROGRAM_BYTES).unwrap();

    let author = Keypair::new();

    svm.airdrop(&author.pubkey(), 10_000_000_000).unwrap();
    // PDAs: proposal, score, weight, voter (seeded by author.pubkey())
    let (proposal_pda, _) =
        Pubkey::find_program_address(&[b"proposal", author.pubkey().as_ref()], &PROGRAM_ID);

    let score_account = Keypair::new();

    let weight_account = Keypair::new();
    let voter_account = Keypair::new();

    let score_space = 8 + 1024; // dummy sizes
    let weight_space = 8 + 1024;
    let voter_space = 8 + 1024;

    let rent = Rent::default();
    let score_rent = rent.minimum_balance(score_space);
    let weight_rent = rent.minimum_balance(weight_space);
    let voter_rent = rent.minimum_balance(voter_space);

    let total_rent = score_rent + weight_rent + voter_rent;
    svm.airdrop(&author.pubkey(), total_rent * 2).unwrap(); // give

    for (keypair, space, rent) in [
        (&score_account, score_space, score_rent),
        (&weight_account, weight_space, weight_rent),
        (&voter_account, voter_space, voter_rent),
    ] {
        let instr = create_account(
            &author.pubkey(),
            &keypair.pubkey(),
            rent,
            space as u64,
            &PROGRAM_ID,
        );

        let blockhash = svm.latest_blockhash();
        let msg = Message::new_with_blockhash(&[instr], Some(&author.pubkey()), &blockhash);

        let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&author, keypair])
            .unwrap();

        let sim_res = svm.simulate_transaction(tx.clone()).unwrap();
        println!("logs: {:?}", sim_res.meta.logs);
        let meta = svm.send_transaction(tx).unwrap();
        assert_eq!(sim_res.meta, meta);
    }

    // 1) Create proposal instruction
    // The create_proposal_handle signature you posted takes: evidence: String
    #[derive(BorshSerialize)]
    struct CreateProposalArgs {
        evidence: String,
    }
    let create_args = CreateProposalArgs {
        evidence: "Evidence: test odd median".to_string(),
    };

    let create_accounts = vec![
        AccountMeta::new(author.pubkey(), true), // author signer
        AccountMeta::new(proposal_pda, false),
        AccountMeta::new(score_account.pubkey(), false),
        AccountMeta::new(weight_account.pubkey(), false),
        AccountMeta::new(voter_account.pubkey(), false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    let parsed_idl = AnchorIdlPartialData::parse(IDL_RAW_DATA).unwrap();

    let discriminant = parsed_idl
        .get_discriminant("initialize_proposal")
        .unwrap_or_default();

    println!("Discriminant: {:?}", discriminant);

    // Build instruction data = discriminator + args
    let mut proposal_data = Vec::new();
    proposal_data.extend_from_slice(&discriminant.to_vec());
    create_args.serialize(&mut proposal_data).unwrap();

    let ix = Instruction::new_with_bytes(program_id, &proposal_data, create_accounts); // Build and send transaction
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&author.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&author]).unwrap();

    let sim_res = svm.simulate_transaction(tx.clone()).unwrap();
    println!("logs: {:?}", sim_res.meta.logs);
    let meta = svm.send_transaction(tx).unwrap();
    assert_eq!(sim_res.meta, meta);
}
