use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::wee_alloc;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, near_bindgen,AccountId, Balance,
    collections::{ UnorderedMap, Vector,LookupMap },
    json_types::{ U128},
};
use chrono::Utc;
use blake2::{Blake2s, Digest};
use std::str;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountInformation {
    pub balance: Balance, 
    pub available: Balance,
    pub price: Balance,

    pub history_buy: Vector<String>,
    pub history_sell: Vector<String>,
    
    pub bank_account: UnorderedMap<String, String>,

    pub vote_up: U128,
    pub vote_down: U128,
}

// #[derive(BorshDeserialize, BorshSerialize)]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct History {
    pub buyer: AccountId,
    pub seller: AccountId,
    pub amount: Balance,
    pub state: String, // init, processing, cancel, success,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct SimpleP2P {
    pub accounts: UnorderedMap<AccountId, AccountInformation>,
    // pub processing: LookupMap<String, History>,
    pub historys: LookupMap<String, History>,
}

impl Default for SimpleP2P {
    fn default() -> Self {
        panic!("Should be initialized before usage")
    }
}

#[near_bindgen]
impl SimpleP2P {
    #[init]
    pub fn new()->Self{
        // assert!(env::is_valid_account_id(owner_id.as_bytes()), "Invalid owner account");
        assert!(!env::state_exists(), "Already initialized");
        Self {
            accounts: UnorderedMap::new(b"a".to_vec()),
            // processing: LookupMap::new(b"p".to_vec()),
            historys: LookupMap::new(b"h".to_vec()),
        }
    }

    #[payable]
    pub fn deposit(&mut self){
        let account_id = env::signer_account_id();
        let deposit = env::attached_deposit();

        let default = AccountInformation{
                balance: 0, 
                available: 0,
                price: 0,
                history_buy: Vector::new(Vec::new()),
                history_sell: Vector::new(Vec::new()),
                bank_account: UnorderedMap::new(b"n".to_vec()),
                vote_up: 0.into(),
                vote_down: 0.into(),
            };
        let mut account_information = self.accounts.get(&account_id).unwrap_or(default);
        account_information.balance = account_information.balance + deposit;
        self.accounts.insert(&account_id, &account_information);
    }

    pub fn set_bank_account(&mut self, number: String, type_account: String){
        let account_id = env::signer_account_id();

        let account_got = self.accounts.get(&account_id);
        assert!(account_got.is_some(), "don't exist this account, please deposit some for create account");
        let mut account_information = account_got.unwrap();
        account_information.bank_account.insert(&number, &type_account);
        self.accounts.insert(&account_id,&account_information);
    }

    #[payable]
    pub fn withdraw(&mut self){

    }

    pub fn order_sell(&mut self, amount: Balance, price: Balance){
        assert!(amount > 0, "Amount must > 0");
        assert!(price > 0, "Price must > 0");


        let account_id = env::signer_account_id();

        let account_got = self.accounts.get(&account_id);
        assert!(account_got.is_some(), "don't exist this account, please deposit some for create account");
        
        let mut account_information = account_got.unwrap();
        assert!(account_information.balance > amount, "Insufficient balance to order");

        account_information.balance = account_information.balance - amount;
        account_information.available = account_information.available +amount;
        account_information.price = price;
        self.accounts.insert(&account_id, &account_information);
    }

    pub fn get_hash(buyer:&String, seller:&String, amount:&Balance)->String{
        
        let dt = Utc::now();
        let timestamp: i64 = dt.timestamp();

        let hash = Blake2s::new()
            .chain(buyer)
            .chain(seller)
            .chain(amount.to_string())
            .chain(timestamp.to_string())
            .finalize();
        match str::from_utf8(hash.as_slice()) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        }.to_string()
    }

    pub fn order_buy(&mut self, seller_id:AccountId, amount: Balance){
        // Check account of seller
        let account_seller_got = self.accounts.get(&seller_id);
        assert!(account_seller_got.is_some(), "seller account is not exist!");
        let mut account_seller = account_seller_got.unwrap();
        assert!(account_seller.available > amount, "nguoi ban khong du so du");
        account_seller.available = account_seller.available - amount;
        
        
        // check account of buyer
        let buyer_id = env::signer_account_id();
        let account_buyer_got = self.accounts.get(&buyer_id);
        assert!(account_buyer_got.is_some(), "buyer account is not exist!");

        // create transaction
        let tx = SimpleP2P::get_hash(&buyer_id,&seller_id,&amount);
        let history = History{
            buyer: buyer_id,
            seller: seller_id.clone(),
            amount: amount,
            state: "init".to_string(),
        };

        // update into contract
        self.historys.insert(&tx,&history);
        self.accounts.insert(&seller_id,&account_seller);
    }

    pub fn confirm_sent(&mut self, tx:String){
        let mut transaction = self.get_transaction(&tx);
        assert!(transaction.state == "init".to_string(), "buyer confirmed đã gửi tiền rồi");
        assert!(env::signer_account_id() == transaction.buyer, "chỉ người mua mới được kí cái này");
        
        transaction.state = "processing".to_string();
        self.historys.insert(&tx, &transaction);
    }

    pub fn confirm_received(&mut self, tx: String){
        let mut transaction = self.get_transaction(&tx);
        assert!(transaction.state != "init".to_string(), "người mua chưa gửi tiền");
        assert!(transaction.state == "processing".to_string(), "giao dịch đã hoan thanh");
        assert!(env::signer_account_id() == transaction.seller, "chỉ người bán mới đực kí cái này");

        // update seller's 
        let seller_id = transaction.seller;
        let mut account_seller = self.accounts.get(&seller_id).unwrap();
        account_seller.balance = account_seller.balance - transaction.amount;
        
        // update buyer's balance
        let buyer_id = transaction.buyer;
        let mut account_buyer = self.accounts.get(&buyer_id).unwrap();
        account_buyer.balance = account_buyer.balance + transaction.amount;
        
        // update state of transaction
        transaction.state = "success".to_string();

        // update contract
        self.historys.insert(&tx, &transaction);
        self.accounts.insert(&seller_id,&account_seller);
        self.accounts.insert(&buyer_id,&account_buyer);
    }

    pub fn cancel_order_buy(&mut self){

    }

    pub fn cancel_order_sell(&mut self, amount:Balance){

    }

    pub fn vote(&mut self, account_id:AccountId, value: u8){
        
    }

    pub fn get_order_sell(&self){

    }

    // pub fn get_account(&self, account_id: AccountId)->AccountInformation{
    //     let account_got = self.accounts.get(&account_id);
    //     assert!(account_got.is_some(), "Account is not exist!");
    //     account_got.unwrap()
    // }

    pub fn get_transaction(&self, tx: &String)->History{
        let transaction = self.historys.get(&tx);
        assert!(transaction.is_some(),"khong ton tai transaction");
        transaction.unwrap()
    }

}
