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

    // --- 1. Compute PDAs ---
    let (author_state_pda, _) =
        Pubkey::find_program_address(&[b"author_state", author.pubkey().as_ref()], &PROGRAM_ID);

    // initial proposal_count = 0 => first proposal PDA
    let (proposal_pda, _) = Pubkey::find_program_address(
        &[
            b"proposal",
            author.pubkey().as_ref(),
            &0u64.to_string().as_bytes(),
        ],
        &PROGRAM_ID,
    );

    // --- 3. Prepare instruction args ---
    #[derive(BorshSerialize)]
    struct CreateProposalArgs {
        evidence: String,
    }

    let args = CreateProposalArgs {
        evidence: "Evidence: proposal median check".to_string(),
    };

    // --- 4. Build account metas ---
    let create_accounts = vec![
        AccountMeta::new(author.pubkey(), true),   // signer
        AccountMeta::new(author_state_pda, false), // author state
        AccountMeta::new(proposal_pda, false),     // proposal
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    // --- 5. Get discriminant ---
    let parsed_idl = AnchorIdlPartialData::parse(IDL_RAW_DATA).unwrap();
    let discriminant = parsed_idl
        .get_discriminant("initialize_proposal")
        .unwrap_or_default();
    println!("Discriminant: {:?}", discriminant);

    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(&discriminant.to_vec());
    args.serialize(&mut instruction_data).unwrap();

    let ix = Instruction::new_with_bytes(program_id, &instruction_data, create_accounts);

    // --- 6. Send transaction ---
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&author.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&author]).unwrap();

    let sim_res = svm.simulate_transaction(tx.clone()).unwrap();
    println!("Program logs: {:?}", sim_res.meta.logs);
    let meta = svm.send_transaction(tx).unwrap();
    assert_eq!(sim_res.meta, meta);

    println!("✅ Proposal created successfully for {}", author.pubkey());
}

#[test]
fn test_vote_proposal() {
    let program_id = PROGRAM_ID;
    let mut svm = LiteSVM::new();
    svm.add_program(program_id, PROGRAM_BYTES).unwrap();

    // --- 1. Create author and voter ---
    let author = Keypair::new();
    let voter = Keypair::new();
    svm.airdrop(&author.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&voter.pubkey(), 10_000_000_000).unwrap();

    // --- 2. Compute PDAs ---
    let (author_state_pda, _) =
        Pubkey::find_program_address(&[b"author_state", author.pubkey().as_ref()], &program_id);

    let proposal_count = 0u64;
    let (proposal_pda, _) = Pubkey::find_program_address(
        &[
            b"proposal",
            author.pubkey().as_ref(),
            &proposal_count.to_string().as_bytes(),
        ],
        &program_id,
    );

    let (scores_pda, _) = Pubkey::find_program_address(
        &[b"scores", &proposal_count.to_string().as_bytes()],
        &program_id,
    );

    let (voter_account_pda, _) = Pubkey::find_program_address(
        &[
            b"voter",
            &proposal_count.to_string().as_bytes(),
            voter.pubkey().as_ref(),
        ],
        &program_id,
    );

    // --- 3. Create proposal first ---
    #[derive(BorshSerialize)]
    struct CreateProposalArgs {
        evidence: String,
    }
    let create_args = CreateProposalArgs {
        evidence: "Test proposal".to_string(),
    };

    let create_accounts = vec![
        AccountMeta::new(author.pubkey(), true),
        AccountMeta::new(author_state_pda, false),
        AccountMeta::new(proposal_pda, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    let parsed_idl = AnchorIdlPartialData::parse(IDL_RAW_DATA).unwrap();
    let discriminant = parsed_idl
        .get_discriminant("initialize_proposal")
        .unwrap_or_default();

    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(&discriminant.to_vec());
    create_args.serialize(&mut instruction_data).unwrap();

    let create_ix = Instruction::new_with_bytes(program_id, &instruction_data, create_accounts);
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[create_ix], Some(&author.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&author]).unwrap();
    svm.send_transaction(tx).unwrap();

    // --- 4. Vote on proposal ---
    #[derive(BorshSerialize)]
    struct VoteProposalArgs {
        proposal_count: u64,
        score: u8,
        conviction: u8,
    }

    let vote_args = VoteProposalArgs {
        proposal_count,
        score: 7,      // choose score between 0–10
        conviction: 3, // choose conviction 0–6
    };

    let vote_accounts = vec![
        AccountMeta::new(voter.pubkey(), true),
        AccountMeta::new(scores_pda, false),
        AccountMeta::new(voter_account_pda, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    let discriminant = parsed_idl
        .get_discriminant("conviction_vote")
        .unwrap_or_default();

    let mut vote_data = Vec::new();
    vote_data.extend_from_slice(&discriminant.to_vec());
    vote_args.serialize(&mut vote_data).unwrap();

    let vote_ix = Instruction::new_with_bytes(program_id, &vote_data, vote_accounts);
    let msg = Message::new_with_blockhash(&[vote_ix], Some(&voter.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&voter]).unwrap();

    let sim_res = svm.simulate_transaction(tx.clone()).unwrap();
    println!("Program logs: {:?}", sim_res.meta.logs);

    let meta = svm.send_transaction(tx).unwrap();
    assert_eq!(sim_res.meta, meta);

    println!("✅ Voted successfully for proposal {}", proposal_count);
}
