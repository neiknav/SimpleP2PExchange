Near Cetificate Devoloper - Demo
================================

Simple Peer-to-Peer Exchange On NEAR

## How it works?


cvbncbncmvb

Exploring The Code
==================

The contract code lives in the `/src/lib.rs`


About Contract
===================

(It's need to be mentioned that it is a pure dapp project, which means there is no centralized backend nor data server, all persistent information is stored and managed on NEAR chain by a contract.)

## Contract Structure

Contract named: `SimpleP2P`. The structure of the contract include: `accounts` is an UnorderedMap that maps each account_id to that person's account, `historys` is a LookupMap that maps from tx (hashcode) to the transaction history
```rust
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct SimpleP2P {
    pub accounts: UnorderedMap<AccountId, AccountInformation>,
    pub historys: LookupMap<String, History>,
}
```

Struct named 'AccountInformation': Information about a user's account, including: Balance, Available, Selling Price, Trade history, Payment method, rating
```rust
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
```

Struct name `SellInformation`: Used to display information, not to store. information of sell orders, including: Seller id, Balance, Available, Selling price, Payment method, Rating
```rust
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
```

Struct named `History`: History of transactions, including: buyer/seller id, price(in dollars), amount, value(in dollars), state of the transaction
```rust
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct History {
    pub buyer: AccountId, 
    pub seller: AccountId,  
    pub amount: Balance,    
    pub price: Balance,
    pub value: Balance,
    pub state: String,      // init, processing, cancel, success,
}
```

## Implement contract

```rust
#[near_bindgen]
impl SimpleP2P {
    #[init]
    pub fn new()->Self{}

    // create and deposit money
    #[payable]
    pub fn deposit(&mut self){}

    // withdraw money to near testnet wallet
    pub fn withdraw(&mut self, amount: u128){}

    // set bank number and bank name as payment method
    pub fn set_bank_account(&mut self, number: String, bank_name: String){}

    // place order sell
    pub fn order_sell(&mut self, amount: u128, price: u128){}

    // place order buy
    pub fn order_buy(&mut self, seller_id:AccountId, amount: u128){}

    // Buyer confirms that money has been sent 
    pub fn confirm_sent(&mut self, tx:String){}

    // Seller confirms receipt of the funds and the transaction is done 
    pub fn confirm_received(&mut self, tx: String){}

    // Buyer cancels buy order 
    pub fn cancel_order_buy(&mut self, tx:String){}

    // Seller cancels sell order 
    pub fn cancel_order_sell(&mut self){}

    // Vote for seller 
    pub fn vote(&mut self, account_id:AccountId, value: i8){}

    // show all accounts with sell orders 
    pub fn get_order_sell(&self)->Vec<SellInformation>{}

    // get information of user
    pub fn get_account(&self, account_id: AccountId)->SellInformation{}

    pub fn get_transaction(&self, tx: &String)->History{}

    // Get buy history of a account
    pub fn get_history_buy(&self, account_id: AccountId)->Vec<History>{}

    // Get sell history of a account
    pub fn get_history_sell(&self, account_id: AccountId)->Vec<History>{}
    
    // get hash code for transaction
    pub fn get_hash(buyer:&String, seller:&String, amount:&Balance)->String{}
}

```

Get Started
===========

Preparation
------------

1. Near testnet account

    Every smart contract in NEAR has its [own associated account][NEAR accounts]. When you run `yarn dev`, your smart contract gets deployed to the live NEAR TestNet with a throwaway account.

    Here, i create account `deploy.p2pexchange.testnet` for contract deployment, `seller1.testnet` for seller account and `buyer1.testnet` for buyer account.
2. Install near-cli

    [near-cli] is a command line interface (CLI) for interacting with the NEAR blockchain. It was installed to the local `node_modules` folder when you ran `yarn install`, but for best ergonomics you may want to install it globally:

    yarn install --global near-cli

    Or, if you'd rather use the locally-installed version, you can prefix all `near` commands with `npx`

    Ensure that it's installed with `near --version` (or `npx near --version`)

3. Environment

    jfhsjdfjsfd



Deploy contract
----------------

    ./build.sh
    ./deploy.sh

Example
----------




Troubleshooting
===============

On Windows, if you're seeing an error containing `EPERM` it may be related to spaces in your path. Please see [this issue](https://github.com/zkat/npx/issues/209) for more details.


  [Vue]: https://vuejs.org/
  [create-near-app]: https://github.com/near/create-near-app
  [Node.js]: https://nodejs.org/en/download/package-manager/
  [jest]: https://jestjs.io/
  [NEAR accounts]: https://docs.near.org/docs/concepts/account
  [NEAR Wallet]: https://wallet.testnet.near.org/
  [near-cli]: https://github.com/near/near-cli
  [gh-pages]: https://github.com/tschaub/gh-pages
