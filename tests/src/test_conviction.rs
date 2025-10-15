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
        &[b"proposal", author.pubkey().as_ref(), &0u64.to_le_bytes()],
        &PROGRAM_ID,
    );

    let (proposal_accounts_pda, _) =
        Pubkey::find_program_address(&[b"proposal_accounts", proposal_pda.as_ref()], &PROGRAM_ID);

    // --- 2. Create zero-copy accounts manually ---
    let score_account = Keypair::new();
    let weight_account = Keypair::new();
    let voter_account = Keypair::new();

    let score_space = 8 + 1024;
    let weight_space = 8 + 1024;
    let voter_space = 8 + 1024;

    let rent = Rent::default();
    let score_rent = rent.minimum_balance(score_space);
    let weight_rent = rent.minimum_balance(weight_space);
    let voter_rent = rent.minimum_balance(voter_space);

    let total_rent = score_rent + weight_rent + voter_rent;
    svm.airdrop(&author.pubkey(), total_rent * 2).unwrap();

    for (kp, space, rent) in [
        (&score_account, score_space, score_rent),
        (&weight_account, weight_space, weight_rent),
        (&voter_account, voter_space, voter_rent),
    ] {
        let instr = create_account(
            &author.pubkey(),
            &kp.pubkey(),
            rent,
            space as u64,
            &PROGRAM_ID,
        );

        let blockhash = svm.latest_blockhash();
        let msg = Message::new_with_blockhash(&[instr], Some(&author.pubkey()), &blockhash);

        let tx =
            VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&author, kp]).unwrap();

        let sim_res = svm.simulate_transaction(tx.clone()).unwrap();
        println!("create_account logs: {:?}", sim_res.meta.logs);
        svm.send_transaction(tx).unwrap();
    }

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
        AccountMeta::new(author.pubkey(), true),        // signer
        AccountMeta::new(author_state_pda, false),      // author state
        AccountMeta::new(proposal_pda, false),          // proposal
        AccountMeta::new(proposal_accounts_pda, false), // proposal_accounts
        AccountMeta::new(score_account.pubkey(), false),
        AccountMeta::new(weight_account.pubkey(), false),
        AccountMeta::new(voter_account.pubkey(), false),
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
fn test_create_proposal_fail_score_account() {
    let program_id = PROGRAM_ID;

    let mut svm = LiteSVM::new();
    svm.add_program(program_id, PROGRAM_BYTES).unwrap();

    let author = Keypair::new();
    svm.airdrop(&author.pubkey(), 10_000_000_000).unwrap();

    // --- 1. Compute PDAs ---
    let (author_state_pda, _) =
        Pubkey::find_program_address(&[b"author_state", author.pubkey().as_ref()], &PROGRAM_ID);
    let (proposal_pda, _) = Pubkey::find_program_address(
        &[b"proposal", author.pubkey().as_ref(), &0u64.to_le_bytes()],
        &PROGRAM_ID,
    );
    let (proposal_accounts_pda, _) =
        Pubkey::find_program_address(&[b"proposal_accounts", proposal_pda.as_ref()], &PROGRAM_ID);

    // --- 2. Zero-copy accounts ---
    let score_account = Keypair::new();
    let weight_account = Keypair::new();
    let voter_account = Keypair::new();

    let score_space = 8 + 1024;
    let weight_space = 8 + 1024;
    let voter_space = 8 + 1024;

    let rent = Rent::default();
    let score_rent = rent.minimum_balance(score_space);
    let weight_rent = rent.minimum_balance(weight_space);
    let voter_rent = rent.minimum_balance(voter_space);

    let total_rent = score_rent + weight_rent + voter_rent;
    svm.airdrop(&author.pubkey(), total_rent * 2).unwrap();

    // --- 3. Create accounts: make SCORE fail (wrong owner) ---
    let accounts_to_create = vec![
        (
            &score_account,
            score_space,
            score_rent,
            solana_program::system_program::id(),
        ), // ❌ wrong owner
        (&weight_account, weight_space, weight_rent, PROGRAM_ID),
        (&voter_account, voter_space, voter_rent, PROGRAM_ID),
    ];

    for (kp, space, rent, owner) in accounts_to_create {
        let instr = create_account(&author.pubkey(), &kp.pubkey(), rent, space as u64, &owner);
        let blockhash = svm.latest_blockhash();
        let msg = Message::new_with_blockhash(&[instr], Some(&author.pubkey()), &blockhash);

        let tx =
            VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&author, kp]).unwrap();

        let sim_res = svm.simulate_transaction(tx.clone()).unwrap();
        println!("create_account logs: {:?}", sim_res.meta.logs);

        // Send transaction; SCORE creation will succeed at system level, but program will fail later
        svm.send_transaction(tx).unwrap();
    }

    // --- 4. Prepare instruction args ---
    #[derive(BorshSerialize)]
    struct CreateProposalArgs {
        evidence: String,
    }

    let args = CreateProposalArgs {
        evidence: "Evidence: fail test".to_string(),
    };

    let create_accounts = vec![
        AccountMeta::new(author.pubkey(), true),
        AccountMeta::new(author_state_pda, false),
        AccountMeta::new(proposal_pda, false),
        AccountMeta::new(proposal_accounts_pda, false),
        AccountMeta::new(score_account.pubkey(), false), // ❌ wrong owner triggers require!
        AccountMeta::new(weight_account.pubkey(), false),
        AccountMeta::new(voter_account.pubkey(), false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    // --- 5. Instruction discriminant ---
    let parsed_idl = AnchorIdlPartialData::parse(IDL_RAW_DATA).unwrap();
    let discriminant = parsed_idl
        .get_discriminant("initialize_proposal")
        .unwrap_or_default();

    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(&discriminant.to_vec());
    args.serialize(&mut instruction_data).unwrap();

    let ix = Instruction::new_with_bytes(program_id, &instruction_data, create_accounts);

    // --- 6. Send transaction ---
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&author.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&author]).unwrap();

    // Simulate transaction
    let sim_result = svm.simulate_transaction(tx.clone());

    match sim_result {
        Ok(sim_res) => {
            println!("Program logs (expected fail): {:?}", sim_res.meta.logs);
        }
        Err(failed_tx) => {
            // ✅ Simulation returned an error as expected
            println!("Program logs (expected fail): {:?}", failed_tx.meta.logs);

            // Check that the program error is exactly Anchor AccountOwnedByWrongProgram
            if let solana_sdk::transaction::TransactionError::InstructionError(
                _,
                solana_sdk::instruction::InstructionError::Custom(code),
            ) = failed_tx.err
            {
                assert_eq!(
                    code, 3007,
                    "Expected AccountOwnedByWrongProgram error (3007)"
                );
                println!("Transaction error: {:?}", failed_tx.err);
            } else {
                panic!("Unexpected transaction error: {:?}", failed_tx.err);
            }
        }
    }
}
