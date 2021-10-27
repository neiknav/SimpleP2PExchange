use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::wee_alloc;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, near_bindgen,AccountId, Balance,
    collections::{ UnorderedMap, Vector,LookupMap },
    // json_types::{ U128},
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
    
    pub bank_number: String,
    pub bank_name: String,

    pub vote_up: u128,
    pub vote_down: u128,
}


#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SellInformation {
    pub balance: Balance, 
    pub available: Balance,
    pub price: Balance,
    
    pub bank_number: String,
    pub bank_name: String,

    pub vote_up: u128,
    pub vote_down: u128,
}


#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct History {
    pub buyer: AccountId,   // id of buyer
    pub seller: AccountId,  // id of seller
    pub amount: Balance,    // transaction amount 
    pub state: String,      // init, processing, cancel, success,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct SimpleP2P {
    pub accounts: UnorderedMap<AccountId, AccountInformation>,
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
        assert!(!env::state_exists(), "Already initialized");
        Self {
            accounts: UnorderedMap::new(b"a".to_vec()),
            historys: LookupMap::new(b"h".to_vec()),
        }
    }

    // create and deposit money
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
                bank_number: "".to_string(),
                bank_name: "".to_string(),
                vote_up: 0,
                vote_down: 0,
            };
        let mut account_information = self.accounts.get(&account_id).unwrap_or(default);
        account_information.balance = account_information.balance + deposit;
        self.accounts.insert(&account_id, &account_information);
    }

    // withdraw money to near testnet wallet
    pub fn withdraw(&mut self, amount: Balance){
        let account_id = env::signer_account_id();
        let account_got = self.accounts.get(&account_id);
        assert(account_got.is_some(),"Account does not exist, deposit some money to create an account");
        let mut account = account_got.unwrap();
        assert!(account.balance >= amount, "insufficient fund");

        // update balance and transfer to near account
        account.balance = account.balance - amount;
        self.accounts.insert(&account_id,&account);
        Promise::new(account_id).transfer(amount);
    }

    // set bank number and bank name as payment method
    pub fn set_bank_account(&mut self, number: String, bank_name: String){
        let account_id = env::signer_account_id();
        
        // check account exist
        let account_got = self.accounts.get(&account_id);
        assert!(account_got.is_some(), "Account does not exist, deposit some money to create an account");
        
        // update bank number and bank name
        let mut account_information = account_got.unwrap();
        account_information.bank_number = number;
        account_information.bank_name = bank_name;
        self.accounts.insert(&account_id,&account_information);
    }

    // place order sell
    pub fn order_sell(&mut self, amount: Balance, price: Balance){
        // check amount and price must > 0
        assert!(amount > 0, "Amount must > 0");
        assert!(price > 0, "Price must > 0");

        // check account exist
        let account_id = env::signer_account_id();
        let account_got = self.accounts.get(&account_id);
        assert!(account_got.is_some(), "Account does not exist, deposit some money to create an account ");
        
        // check enough balance
        let mut account_information = account_got.unwrap();
        assert!(account_information.balance > amount, "Insufficient balance to order sell");
        
        // check the account that has set the payment method 
        assert!(account_information.bank_number != "".to_string(),"Must set payment method");
        
        // update sell order
        account_information.balance = account_information.balance - amount;
        account_information.available = account_information.available +amount;
        account_information.price = price;
        self.accounts.insert(&account_id, &account_information);
    }

    // place order buy
    pub fn order_buy(&mut self, seller_id:AccountId, amount: Balance){
        // Check account of seller exist
        let account_seller_got = self.accounts.get(&seller_id);
        assert!(account_seller_got.is_some(), "Seller's account does not exist!");
        let mut account_seller = account_seller_got.unwrap();

        // check the amount is valid (> 0 and < seller's balance) 
        assert!(amount > 0, "Invalid amount, must be greater than 0 ");
        assert!(account_seller.available > amount, "Invalid amount, must be less than seller balance available");
        account_seller.available = account_seller.available - amount;
                
        // check account of buyer exist
        let buyer_id = env::signer_account_id();
        let account_buyer_got = self.accounts.get(&buyer_id);
        assert!(account_buyer_got.is_some(), "buyer account is not exist!");
        let mut account_buyer = account_buyer_got.unwrap();
        
        // create transaction
        let tx = SimpleP2P::get_hash(&buyer_id,&seller_id,&amount);
        let history = History{
            buyer: buyer_id.clone(),
            seller: seller_id.clone(),
            amount: amount,
            state: "init".to_string(),
        };

        // update balance of seller, buyer, transaction's history 
        account_seller.history_sell.push(&tx);
        account_buyer.history_buy.push(&tx);
        self.historys.insert(&tx,&history);
        self.accounts.insert(&seller_id,&account_seller);
        self.accounts.insert(&buyer_id,&account_buyer);
    }

    // Buyer confirms that money has been sent 
    pub fn confirm_sent(&mut self, tx:String){
        let mut transaction = self.get_transaction(&tx);
        assert!(transaction.state == "init".to_string(), "Previously confirmed deposit");
        assert!(env::signer_account_id() == transaction.buyer, "Only buyer can confirm sent");
        
        // update state of transaction
        transaction.state = "processing".to_string();
        self.historys.insert(&tx, &transaction);
    }

    // Seller confirms receipt of the funds and the transaction is done 
    pub fn confirm_received(&mut self, tx: String){
        let mut transaction = self.get_transaction(&tx);
        assert!(transaction.state != "init".to_string(), "Buyer has not confirmed sent");
        assert!(transaction.state == "processing".to_string(), "The transaction has ended");
        assert!(env::signer_account_id() == transaction.seller, "Only the seller can confirm received");

        // update seller's balance
        let seller_id = transaction.seller.clone();
        let mut account_seller = self.accounts.get(&seller_id).unwrap();
        account_seller.balance = account_seller.balance - transaction.amount;
        
        // update buyer's balance
        let buyer_id = transaction.buyer.clone();
        let mut account_buyer = self.accounts.get(&buyer_id).unwrap();
        account_buyer.balance = account_buyer.balance + transaction.amount;
        
        // update state of transaction
        transaction.state = "success".to_string();

        self.historys.insert(&tx, &transaction);
        self.accounts.insert(&seller_id,&account_seller);
        self.accounts.insert(&buyer_id,&account_buyer);
    }

    // Buyer cancels buy order 
    pub fn cancel_order_buy(&mut self, tx:String){
        let mut transaction = self.get_transaction(&tx);
        assert!(transaction.state != "processing".to_string(),"You have transferred the money, if you cancel the buy order, you may lose your money");
        assert!(transaction.state != "cancel".to_string(), "The transaction has been canceled");
        assert!(transaction.state != "success".to_string(), "The transaction has been completed");
        assert!(env::signer_account_id() == transaction.buyer, "Only buyer can cancel order");

        // update seller available balance
        let seller_id = transaction.seller.clone();
        let mut account_seller = self.accounts.get(&seller_id).unwrap();
        account_seller.available = account_seller.available + transaction.amount;

        // update transaction's state
        transaction.state = "cancel".to_string();

        self.historys.insert(&tx, &transaction);
        self.accounts.insert(&seller_id,&account_seller);
    }

    // Seller cancels sell order 
    pub fn cancel_order_sell(&mut self){
        // get signer account
        let account_id = env::signer_account_id();
        let account_got = self.accounts.get(&account_id);
        assert!(account_got.is_some(), "Account does not exist");
        let mut account = account_got.unwrap();

        // check if user account has sell order? 
        assert!(account.available > 0, "Account don't have order sell");

        // update state of account
        account.available = 0;
        account.price = 0;

        self.accounts.insert(&account_id, &account);
    }

    // Vote for seller 
    pub fn vote(&mut self, account_id:AccountId, value: i8){
        if value == 1 || value == -1{
            let sign_id = env::signer_account_id();
            assert!(sign_id != account_id, "Can't vote for yourself");

            // check account is valid
            let account_got = self.accounts.get(&account_id);
            assert!(account_got.is_some(),"Account does not exist");
            let mut account = account_got.unwrap();

            if value == 1{                  // vote up
                account.vote_up += 1;
            }else{                          // vote down
                account.vote_down += 1;
            }
            // update account's vote
            self.accounts.insert(&account_id, &account);
        } else {
            panic!("Invalid value");
        }
    }

    // show all accounts with sell orders 
    pub fn get_order_sell(&self)->Vec<SellInformation>{
        let accounts = self.accounts.values();
        let mut result = Vec::new();
        
        // add accounts with available balance > 0
        for x in accounts{
            if x.available > 0{ 
                let tmp = SellInformation{
                    balance: x.balance, 
                    available: x.available,
                    price: x.price,
                    bank_number: x.bank_number,
                    bank_name: x.bank_name,
                    vote_up: x.vote_up,
                    vote_down: x.vote_down,
                };
                result.push(tmp);
            }
        }
        result
    }

    // get information of user
    pub fn get_account(&self, account_id: AccountId)->SellInformation{
        let account_got = self.accounts.get(&account_id);
        assert!(account_got.is_some(), "Account does not exist");
        let account = account_got.unwrap();

        SellInformation{
            balance: account.balance, 
            available: account.available,
            price: account.price,
            bank_number: account.bank_number,
            bank_name: account.bank_name,
            vote_up: account.vote_up,
            vote_down: account.vote_down,
        }
    }

    pub fn get_transaction(&self, tx: &String)->History{
        let transaction = self.historys.get(&tx);
        assert!(transaction.is_some(),"Transaction does not exist");
        transaction.unwrap()
    }

    // Get buy history of a account
    pub fn get_history_buy(&self, account_id: AccountId)->Vec<History>{
        let account_got = self.accounts.get(&account_id);
        assert!(account_got.is_some(),"Account does not exist");
        let account = account_got.unwrap();
        let history = account.history_buy;

        let mut result = Vec::new();
        for x in history.iter() {
            result.push(self.historys.get(&x).unwrap());
        }
        result
    }

    // Get sell history of a account
    pub fn get_history_sell(&self, account_id: AccountId)->Vec<History>{
        let account_got = self.accounts.get(&account_id);
        assert!(account_got.is_some(),"Account does not exist");
        let account = account_got.unwrap();
        let history = account.history_sell;

        let mut result = Vec::new();
        for x in history.iter() {
            result.push(self.historys.get(&x).unwrap());
        }
        result
    }

    // get hash code for transaction
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

}
