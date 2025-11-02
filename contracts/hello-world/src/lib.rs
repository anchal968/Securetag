/* src/lib.rs */

// This line is mandatory.
#![no_std]

// Import the tools we need from the Soroban SDK
use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, Env, Map, Symbol,
    // We need 'contracttype' to create our own custom data struct
    contracttype, 
};

// --- This is our "Key" for storing the database of tags ---
const ITEM_LEDGER: Symbol = symbol_short!("ITEMS");

// --- This is a custom struct to hold info about one tag ---
// It's like a row in our database.
#[contracttype]
#[derive(Clone)] // We must add "Clone" so we can copy the struct
pub struct TagInfo {
    pub owner: Address,   // The wallet address (G...) of the item's owner
    pub status: Symbol,   // The status, e.g., 'ok' or 'found'
    pub message: Symbol,  // A short message from the finder, e.g., 'at_library'
}

// This line defines the name of our contract.
#[contract]
pub struct SecureTagContract;

// This block is where we write all our functions.
#[contractimpl]
impl SecureTagContract {

    /// 1. REGISTER AN ITEM (The Owner calls this ONCE)
    /// This links a Tag ID (like 'LUGGAGE_123') to your wallet.
    pub fn register_tag(env: Env, owner: Address, tag_id: Symbol) {
        // This line is for security!
        // It makes sure that the "owner" (you) is the one
        // actually running this transaction.
        owner.require_auth();

        // 1. Get the main item ledger (the Map) from storage
        let mut item_ledger: Map<Symbol, TagInfo> = 
            env.storage().instance()
               .get(&ITEM_LEDGER)
               .unwrap_or_else(|| Map::new(&env)); // If no ledger, create a new one

        // 2. Check if this tag is already registered
        if item_ledger.contains_key(tag_id.clone()) {
            // Stop someone from re-registering your tag
            panic!("This tag ID is already registered");
        }

        // 3. Create the new info for this tag
        let new_tag = TagInfo {
            owner: owner,
            status: symbol_short!("ok"), // Set initial status to 'ok'
            message: symbol_short!("none"), // No message yet
        };

        // 4. Save the new tag info in the ledger
        item_ledger.set(tag_id, new_tag);

        // 5. Save the ledger back to the blockchain
        env.storage().instance().set(&ITEM_LEDGER, &item_ledger);
    }

    /// 2. REPORT A LOST ITEM (The Finder's webpage calls this)
    /// Anonymously marks an item as "found" and leaves a message.
    /// Anyone can call this function.
    pub fn report_found(env: Env, tag_id: Symbol, message: Symbol) {
        // 1. Get the main item ledger
        let mut item_ledger: Map<Symbol, TagInfo> = 
            env.storage().instance()
               .get(&ITEM_LEDGER)
               .unwrap_or_else(|| Map::new(&env));

        // 2. Get the info for this specific tag
        // .expect() will cause an error if the tag_id doesn't exist.
        let mut tag = item_ledger
            .get(tag_id.clone())
            .expect("This tag is not registered in the system");

        // 3. Update the status and message
        tag.status = symbol_short!("found");
        tag.message = message;

        // 4. Save the updated info back to the ledger
        item_ledger.set(tag_id, tag);

        // 5. Save the ledger back to the blockchain
        env.storage().instance().set(&ITEM_LEDGER, &item_ledger);
    }

    /// 3. GET TAG INFO (The Owner's webpage calls this)
    /// Reads the logbook to see the status of an item.
    pub fn get_tag_info(env: Env, tag_id: Symbol) -> TagInfo {
        let item_ledger: Map<Symbol, TagInfo> = 
            env.storage().instance()
               .get(&ITEM_LEDGER)
               .unwrap_or_else(|| Map::new(&env));

        // This will return the TagInfo struct (or error if tag doesn't exist)
        item_ledger
            .get(tag_id)
            .expect("This tag is not registered")
    }

    /// 4. CLAIM ITEM (The Owner calls this after retrieving their item)
    /// Resets the status back to 'ok'.
    pub fn claim_item(env: Env, tag_id: Symbol) {
        // 1. Get the ledger
        let mut item_ledger: Map<Symbol, TagInfo> = 
            env.storage().instance()
               .get(&ITEM_LEDGER)
               .unwrap_or_else(|| Map::new(&env));

        // 2. Get the tag info
        let mut tag = item_ledger
            .get(tag_id.clone())
            .expect("This tag is not registered");

        // --- SECURITY! ---
        // This checks that the person calling this function
        // is the *same person* listed as the 'owner' of the tag.
        // This stops a random finder from resetting your tag.
        tag.owner.require_auth();

        // 3. Reset the status and message
        tag.status = symbol_short!("ok");
        tag.message = symbol_short!("none");

        // 4. Save the updated info
        item_ledger.set(tag_id, tag);
        env.storage().instance().set(&ITEM_LEDGER, &item_ledger);
    }
}

