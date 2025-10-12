use partial_idl_parser::AnchorIdlPartialData;
use partial_idl_parser::{get_idl, idl_custom_path};

use std::env;
use std::path::PathBuf;
use std::{error::Error, path::Path, rc::Rc};

use boa_engine::{
    builtins::promise::PromiseState, js_string, module::SimpleModuleLoader, Context, JsError,
    JsNativeError, JsValue, Module, NativeFunction,
};
use boa_parser::Source;
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

const IDL_RAW_DATA: &str = idl_custom_path!(concat!(
    env!("CARGO_WORKSPACE_DIR"),
    "/target/idl/",
    "rational_dex.json"
));
/// Replace with your program id (you provided this earlier).
const PROGRAM_ID: Pubkey = pubkey!("EEL1Q3J9MjPxTWagTKE39jpUVBjUg7q283ztTVzbveDz");

const PROGRAM_BYTES: &[u8] = include_bytes!("../../target/deploy/rational_dex.so");

#[test]
fn test_initialize_pool() {
    let program_id = PROGRAM_ID;

    let owner_a = Pubkey::new_unique();
    let owner_b = Pubkey::new_unique();
    let mint_a = Pubkey::new_unique();
    let mint_b = Pubkey::new_unique();
    let mint_lp = Pubkey::new_unique();
    let owner_lp = Pubkey::new_unique();

    let mut svm = LiteSVM::new();

    svm.add_program(program_id, PROGRAM_BYTES).unwrap();

    let mint_account_a = Mint {
        mint_authority: COption::Some(owner_a),
        supply: 0,
        decimals: 9,
        is_initialized: true,
        freeze_authority: COption::None,
    };

    let mut mint_bytes_a = [0u8; Mint::LEN];
    Mint::pack(mint_account_a, &mut mint_bytes_a).unwrap();

    svm.set_account(
        mint_a,
        Account {
            lamports: 1_000_000_000,
            data: mint_bytes_a.to_vec(),
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    let mint_account_b = Mint {
        mint_authority: COption::Some(owner_b),
        supply: 0,
        decimals: 9,
        is_initialized: true,
        freeze_authority: COption::None,
    };

    let mut mint_bytes_b = [0u8; Mint::LEN];
    Mint::pack(mint_account_b, &mut mint_bytes_b).unwrap();

    svm.set_account(
        mint_b,
        Account {
            lamports: 1_000_000_000,
            data: mint_bytes_b.to_vec(),
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    let mint_account_lp = Mint {
        mint_authority: COption::Some(owner_lp),
        supply: 0,
        decimals: 9,
        is_initialized: true,
        freeze_authority: COption::None,
    };

    let mut mint_bytes_lp = [0u8; Mint::LEN];
    Mint::pack(mint_account_lp, &mut mint_bytes_lp).unwrap();

    svm.set_account(
        mint_lp,
        Account {
            lamports: 1_000_000_000,
            data: mint_bytes_lp.to_vec(),
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    // Derive the pool PDA
    let (pool_pda, _bump) = Pubkey::find_program_address(&[b"pool"], &program_id);

    let (vault_a_pda, _bump) = Pubkey::find_program_address(&[b"vault_a"], &program_id);

    let (vault_b_pda, _bump) = Pubkey::find_program_address(&[b"vault_b"], &program_id);

    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap(); // 10 SOL

    let account_metas = vec![
        AccountMeta::new(payer.pubkey(), true), // signer
        AccountMeta::new(pool_pda, false),
        AccountMeta::new(vault_a_pda, false),
        AccountMeta::new(vault_b_pda, false),
        AccountMeta::new_readonly(mint_a, false),
        AccountMeta::new_readonly(mint_b, false), // mint
        AccountMeta::new(mint_lp, false),         // treasury_token_account
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false), // token_program
        AccountMeta::new_readonly(solana_program::system_program::id(), false), // system_program
    ];

    let parsed_idl = AnchorIdlPartialData::parse(IDL_RAW_DATA).unwrap();
    let discriminant = parsed_idl
        .get_discriminant("initialize_pool")
        .unwrap_or_default();

    // Serialize args (just company_name)
    #[derive(BorshSerialize)]
    pub struct PoolAccount {
        pub fee_numerator: u64,
        pub fee_denominator: u64,
    }

    let args = PoolAccount {
        fee_numerator: 1000,
        fee_denominator: 10000,
    };

    // Build instruction data = discriminator + args
    let mut data = Vec::new();
    data.extend_from_slice(&discriminant.to_vec());
    args.serialize(&mut data).unwrap();

    // Build instruction with Borsh-serialized args
    let ix = Instruction::new_with_bytes(program_id, &data, account_metas); // Build and send transaction
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();

    let sim_res = svm.simulate_transaction(tx.clone()).unwrap();
    println!("logs: {:?}", sim_res.meta.logs);
}

#[test]
fn js_test() {
    let root = env::current_dir().unwrap();
    println!("Project root: {:?}", root);

    // Build path to package.js
    let js_file_path: PathBuf = root.parent().unwrap().join("tests/modules/package.js");
    println!("JS file path: {:?}", js_file_path);

    let path: &Path = js_file_path.as_path();

    println!("{:?}", path);

    let source = Source::from_filepath(path).unwrap();

    let module_pathbuf = root.parent().unwrap().join("tests/modules");

    let module_path: &Path = module_pathbuf.as_path();

    println!("{:?}", module_path);

    // // println!("source: {:?}", source);
    let loader = Rc::new(SimpleModuleLoader::new(module_path).unwrap());
    // // Instantiate the execution context
    let context = &mut Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap(); // // Add the runtime intrisics
    let module = Module::parse(source, None, context).unwrap();

    loader.insert(
        Path::new(module_path)
            .canonicalize()
            .unwrap()
            .join("main.mjs"),
        module.clone(),
    );
    let promise_result = module
        // Initial load that recursively loads the module's dependencies.
        // This returns a `JsPromise` that will be resolved when loading finishes,
        // which allows async loads and async fetches.
        .load(context)
        .then(
            Some(
                NativeFunction::from_copy_closure_with_captures(
                    |_, _, module, context| {
                        // After loading, link all modules by resolving the imports
                        // and exports on the full module graph, initializing module
                        // environments. This returns a plain `Err` since all modules
                        // must link at the same time.
                        module.link(context)?;
                        Ok(JsValue::undefined())
                    },
                    module.clone(),
                )
                .to_js_function(context.realm()),
            ),
            None,
            context,
        )
        .then(
            Some(
                NativeFunction::from_copy_closure_with_captures(
                    // Finally, evaluate the root module.
                    // This returns a `JsPromise` since a module could have
                    // top-level await statements, which defers module execution to the
                    // job queue.
                    |_, _, module, context| Ok(module.evaluate(context).into()),
                    module.clone(),
                )
                .to_js_function(context.realm()),
            ),
            None,
            context,
        );

    // Very important to push forward the job queue after queueing promises.
    context.run_jobs();

    match promise_result.state() {
        PromiseState::Pending => {
            println!("Module can't load");
        }
        PromiseState::Fulfilled(v) => {
            assert_eq!(v, JsValue::undefined());
        }
        PromiseState::Rejected(err) => {
            println!(
                "{:?}",
                JsError::from_opaque(err).try_native(context).unwrap()
            );
        }
    }

    // We can access the full namespace of the module with all its exports.
    let namespace = module.namespace(context);

    let hello_world_namespace = namespace.get(js_string!("helloWorld"), context).unwrap();
    let hello_world = hello_world_namespace
        .as_callable()
        .ok_or_else(|| JsNativeError::typ().with_message("mix export wasn't a function!"))
        .unwrap();
    let result = hello_world
        .call(&JsValue::undefined(), &[], context)
        .unwrap();

    println!("result = {}", result.display());
}
