Near Cetificate Devoloper - Demo
================================

Simple Peer-to-Peer Exchange On NEAR

## How it works?


See how p2p exchange work [here](https://docs.google.com/presentation/d/1NUWx2-fmpYr7vTLZGqELqE8-QGx1m7TF8VfpZViYvbA/edit?usp=sharing).

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

Note before calling method from an account, log in with the command
    
    near login

Preparation
------------

I created 3 accounts: `deploy.p2pexchange.testnet` for contract deployment, `seller1.testnet` for seller account and `buyer1.testnet` for buyer account.

I created sub account `deploy.p2pexchange.testnet` for deploy contract, with command

    near create-account deploy.p2pexchange.testnet --masterAccount p2pexchange.testnet

Setting environment
--------------------

I have created 3 files `call.sh` (call from contract creator), `sellercall.sh` (call from seller), `buyercall.sh` (call from buyer), which uses the variables `DEPLOY_ID`, `SELLER_ID` and `BUYER_ID`.

Use the following commands to create the above variables

    export DEPLOY_ID=deploy.p2pexchange.testnet
    export BUYER_ID=buyer1.testnet
    export SELLER_ID=seller1.testnet

Deploy contract
----------------

    ./build.sh
    ./deploy.sh

Initialize the contract

    ./call.sh new '{}'	

Prepare seller's account
------------------

To be able to place a sell order, the seller must have a balance in the account and add a payment method

Create and deposit a amount of money into the seller's account

    ./sellercall.sh deposit '{}' --amount 10

Seller set payment method, This will be the method for the buyer to proceed with the payment, so please check it is correct

    ./sellercall.sh set_bank_account '{"number":"123456789", "bank_name":"MB Bank"}'

Create account for buyer (have to deposit some Near)

    ./buyercall.sh deposit '{}' --amount 1

Example
----------

First, the seller places a sell order 

    ./sellercall.sh order_sell '{"amount": 10, "price":2}'

Buyer check information about available sell orders

    ./buyercall.sh get_order_sell '{}'

Buyer places a buy order

    ./buyercall.sh order_buy '{"seller_id":"seller1.testnet", "amount":5}'

Buyer confirm sent 

    ./buyercall.sh confirm_sent '{"tx": "something"}'

Seller confirm received money

    ./sellercall.sh confirm_received '{"tx":"something"}'

View transaction status

    ./buyercall.sh get_transaction '{"tx":"something"}'

Rating for seller

    ./buyercall.sh vote '{"account_id":"seller1.testnet", "value":1}'

Check account information

    ./call.sh get_account '{"account_id":"seller1.testnet"}'
    ./call.sh get_account '{"account_id":"buyer1.testnet"}'

The seller cancels the sell order and withdraws the money to his account

    ./sellercall.sh cancel_order_sell '{}'
    ./sellercall.sh withdraw '{"amount":5}'

The buyer withdraw the money to his account

    ./buyercall.sh withdraw '{"amount":5}'

Troubleshooting
===============

On Windows, if you're seeing an error containing `EPERM` it may be related to spaces in your path. Please see [this issue](https://github.com/zkat/npx/issues/209) for more details.