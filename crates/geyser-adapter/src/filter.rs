use std::{collections::HashMap, fs};

use anyhow::Result;
use serde::Deserialize;
use yellowstone_grpc_proto::geyser::{
    CommitmentLevel, SubscribeRequest, SubscribeRequestFilterAccounts,
    SubscribeRequestFilterAccountsFilter, SubscribeRequestFilterAccountsFilterMemcmp,
    SubscribeRequestFilterBlocks, SubscribeRequestFilterSlots, SubscribeRequestFilterTransactions,
    subscribe_request_filter_accounts_filter::Filter as AccountsFilterOneof,
    subscribe_request_filter_accounts_filter_memcmp::Data as MemcmpData,
};

#[derive(Debug, Clone, Deserialize)]
pub struct TxFilter {
    pub vote: Option<bool>,
    pub failed: Option<bool>,
    pub account_include: Vec<String>,
    pub account_exclude: Vec<String>,
    pub account_required: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountMemcmp {
    pub offset: u64,
    pub base58: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Accounts {
    pub accounts: Vec<String>,
    pub owners: Vec<String>,
    pub filters: Vec<AccountFilter>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountFilter {
    pub memcmp: Vec<AccountMemcmp>,
    pub datasize: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Filters {
    /// accounts to watch
    pub accounts: Vec<Accounts>,
    /// whether to subscribe blocks and which inclusions
    #[serde(default)]
    pub include_blocks: bool,

    /// slots to watch
    #[serde(default)]
    pub include_slots: bool,

    #[serde(default)]
    pub blocks_include_transactions: Option<bool>,
    #[serde(default)]
    pub blocks_include_accounts: Option<bool>,
    #[serde(default)]
    pub blocks_include_entries: Option<bool>,

    /// accounts to block from blocks
    #[serde(default)]
    pub block_accounts: Option<Vec<String>>,

    /// Whether to subscribe to transactions and how to filter them
    #[serde(default)]
    pub transactions: Option<TxFilter>,
}

impl Filters {
    // this function loads filters from the json file
    pub fn from_file(path: &str) -> Result<Self> {
        let s: String = fs::read_to_string(path)?;
        serde_json::from_str::<Self>(&s)
            .map_err(|e| anyhow::anyhow!("Failed to parse filters from file {}: {}", path, e))
    }

    pub fn to_subscribe_request(&self) -> SubscribeRequest {
        // Accounts
        let mut accounts: HashMap<String, SubscribeRequestFilterAccounts> = HashMap::new();
        for acc in &self.accounts {
            let mut filters = vec![];

            // add datasize if present
            if let Some(size) = acc.filters.iter().filter_map(|f| f.datasize).next() {
                filters.push(SubscribeRequestFilterAccountsFilter {
                    filter: Some(AccountsFilterOneof::Datasize(size)),
                });
            }

            // add memcmp filters
            for f in &acc.filters {
                for m in &f.memcmp {
                    filters.push(SubscribeRequestFilterAccountsFilter {
                        filter: Some(AccountsFilterOneof::Memcmp(
                            SubscribeRequestFilterAccountsFilterMemcmp {
                                offset: m.offset,
                                data: Some(MemcmpData::Base58(m.base58.clone())),
                            },
                        )),
                    });
                }
            }

            accounts.insert(
                "client".to_owned(),
                SubscribeRequestFilterAccounts {
                    account: acc.accounts.clone(),
                    owner: acc.owners.clone(),
                    filters,
                    nonempty_txn_signature: None,
                    cuckoo_accounts_filter: None,
                },
            );
        }

        // Transactions
        let mut transactions: HashMap<String, SubscribeRequestFilterTransactions> = HashMap::new();
        if let Some(tx) = &self.transactions {
            transactions.insert(
                "client".to_owned(),
                SubscribeRequestFilterTransactions {
                    vote: tx.vote,
                    failed: tx.failed,
                    signature: None,
                    account_include: tx.account_include.clone(),
                    account_exclude: tx.account_exclude.clone(),
                    account_required: tx.account_required.clone(),
                    cuckoo_account_include: None,
                    token_accounts: None,
                },
            );
        }

        // blocks
        let mut blocks: HashMap<String, SubscribeRequestFilterBlocks> = HashMap::new();
        if self.include_blocks {
            blocks.insert(
                "client".to_owned(),
                SubscribeRequestFilterBlocks {
                    account_include: self.block_accounts.clone().unwrap_or_default(),
                    include_accounts: self.blocks_include_accounts,
                    include_entries: self.blocks_include_entries,
                    include_transactions: self.blocks_include_transactions,
                    cuckoo_account_include: None,
                },
            );
        }

        let mut slots: HashMap<String, SubscribeRequestFilterSlots> = HashMap::new();
        if self.include_slots {
            slots.insert(
                "client".to_owned(),
                SubscribeRequestFilterSlots {
                    filter_by_commitment: None,
                    interslot_updates: None,
                },
            );
        }

        // Construct and return the SubscribeRequest
        SubscribeRequest {
            slots,
            accounts,
            blocks,
            blocks_meta: HashMap::new(),
            transactions,
            transactions_status: HashMap::new(),
            entry: HashMap::new(),
            accounts_data_slice: vec![],
            // Processed can be rolled back by a fork and we have no reorg
            // handling yet, so index confirmed instead.
            commitment: Some(CommitmentLevel::Confirmed as i32),
            from_slot: None,
            ping: None,
        }
    }
}
