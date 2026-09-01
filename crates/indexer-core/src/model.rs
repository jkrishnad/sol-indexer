use anyhow::{Error, Ok};
use bs58::encode;
use serde::{Deserialize, Serialize};
use yellowstone_grpc_proto::prelude::{self as yp};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockUpdate {
    pub slot: u64,
    pub blockhash: String,
    pub parent_slot: u64,
    pub parent_blockhash: String,
    pub executed_transaction_count: u64,
    pub transactions: Vec<TransactionUpdateInfo>,
    pub updated_account_count: u64,
    pub accounts: Vec<AccountInfo>,
    pub entries_count: u64,
    pub entries: Vec<EntryUpdate>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EntryUpdate {
    pub slot: u64,
    pub index: u64,
    pub num_hashes: u64,
    pub hash: Vec<u8>,
    pub executed_transaction_count: u64,
    pub starting_transaction_index: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TransactionUpdate {
    pub slot: u64,
    pub transaction: Option<TransactionUpdateInfo>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TransactionUpdateInfo {
    pub signature: Vec<u8>,
    pub is_vote: bool,
    pub index: u64,
    pub meta: Option<TransactionMeta>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TransactionMeta {
    pub fee: u64,
    pub compute_units_consumed: Option<u64>,
    pub pre_balances: Vec<u64>,
    pub post_balances: Vec<u64>,
    pub log_messages: Vec<String>,
    pub pre_token_balance: Vec<TokenBalance>,
    pub post_token_balance: Vec<TokenBalance>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TransactionStatusUpdate {
    pub slot: u64,
    pub signature: Vec<u8>,
    pub is_vote: bool,
    pub index: u64,
    pub err: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TokenBalance {
    pub account_index: u32,
    pub mint: String,
    pub owner: String,
    pub program_id: String,
}

#[derive(Debug, Clone)]
pub struct AccountUpdate {
    pub slot: u64,
    pub is_startup: bool,
    pub info: AccountInfo,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountInfo {
    pub pubkey: Vec<u8>,
    pub lamports: u64,
    pub owner: Vec<u8>,
    pub executable: bool,
    pub rent_epoch: u64,
    pub data: Vec<u8>,
    pub write_version: u64,
    pub txn_signature: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlotUpdate {
    pub slot: u64,
    pub parent: Option<u64>,
    pub status: i32,
    pub dead_error: Option<String>,
}

impl TryFrom<yp::SubscribeUpdateAccount> for AccountUpdate {
    type Error = Error;

    fn try_from(value: yp::SubscribeUpdateAccount) -> Result<Self, Self::Error> {
        let acc = value
            .account
            .ok_or_else(|| anyhow::anyhow!("SubscribeUpdateAccount had no account info"))?;

        Ok(AccountUpdate {
            slot: value.slot,
            is_startup: value.is_startup,
            info: AccountInfo {
                pubkey: acc.pubkey,
                data: acc.data,
                executable: acc.executable,
                lamports: acc.lamports,
                owner: acc.owner,
                rent_epoch: acc.rent_epoch,
                write_version: acc.write_version,
                txn_signature: acc.txn_signature,
            },
        })
    }
}

impl AccountInfo {
    pub fn pubkey_string(&self) -> String {
        encode(&self.pubkey).into_string()
    }

    pub fn owner_string(&self) -> String {
        encode(&self.owner).into_string()
    }

    pub fn txn_signature_string(&self) -> Option<String> {
        self.txn_signature
            .as_ref()
            .map(|sig| encode(sig).into_string())
    }
}

impl TryFrom<yp::SubscribeUpdateTransaction> for TransactionUpdate {
    type Error = Error;
    fn try_from(value: yp::SubscribeUpdateTransaction) -> Result<Self, Self::Error> {
        Ok(TransactionUpdate {
            slot: value.slot,
            transaction: value.transaction.map(|tx| TransactionUpdateInfo {
                index: tx.index,
                is_vote: tx.is_vote,
                signature: tx.signature,
                meta: tx.meta.map(|meta| TransactionMeta {
                    fee: meta.fee,
                    log_messages: meta.log_messages,
                    pre_balances: meta.pre_balances,
                    compute_units_consumed: meta.compute_units_consumed,
                    post_balances: meta.post_balances,
                    post_token_balance: meta
                        .post_token_balances
                        .into_iter()
                        .map(|tb| TokenBalance {
                            account_index: tb.account_index,
                            mint: tb.mint,
                            owner: tb.owner,
                            program_id: tb.program_id,
                        })
                        .collect(),
                    pre_token_balance: meta
                        .pre_token_balances
                        .into_iter()
                        .map(|tb| TokenBalance {
                            account_index: tb.account_index,
                            mint: tb.mint,
                            owner: tb.owner,
                            program_id: tb.program_id,
                        })
                        .collect(),
                }),
            }),
        })
    }
}

impl TryFrom<yp::SubscribeUpdateBlock> for BlockUpdate {
    type Error = Error;

    fn try_from(value: yp::SubscribeUpdateBlock) -> Result<Self, Self::Error> {
        Ok(BlockUpdate {
            slot: value.slot,
            blockhash: value.blockhash,
            parent_slot: value.parent_slot,
            parent_blockhash: value.parent_blockhash,
            executed_transaction_count: value.executed_transaction_count,
            transactions: value
                .transactions
                .into_iter()
                .map(|tx| TransactionUpdateInfo {
                    index: tx.index,
                    is_vote: tx.is_vote,
                    signature: tx.signature,
                    meta: tx.meta.map(|meta| TransactionMeta {
                        fee: meta.fee,
                        log_messages: meta.log_messages,
                        pre_balances: meta.pre_balances,
                        compute_units_consumed: meta.compute_units_consumed,
                        post_balances: meta.post_balances,
                        post_token_balance: meta
                            .post_token_balances
                            .into_iter()
                            .map(|tb| TokenBalance {
                                account_index: tb.account_index,
                                mint: tb.mint,
                                owner: tb.owner,
                                program_id: tb.program_id,
                            })
                            .collect(),
                        pre_token_balance: meta
                            .pre_token_balances
                            .into_iter()
                            .map(|tb| TokenBalance {
                                account_index: tb.account_index,
                                mint: tb.mint,
                                owner: tb.owner,
                                program_id: tb.program_id,
                            })
                            .collect(),
                    }),
                })
                .collect(),
            updated_account_count: value.updated_account_count,
            accounts: value
                .accounts
                .into_iter()
                .map(|acc| AccountInfo {
                    pubkey: acc.pubkey,
                    data: acc.data,
                    executable: acc.executable,
                    lamports: acc.lamports,
                    owner: acc.owner,
                    rent_epoch: acc.rent_epoch,
                    write_version: acc.write_version,
                    txn_signature: acc.txn_signature,
                })
                .collect(),
            entries_count: value.entries_count,
            entries: value
                .entries
                .into_iter()
                .map(|entry| EntryUpdate {
                    slot: entry.slot,
                    index: entry.index,
                    num_hashes: entry.num_hashes,
                    hash: entry.hash,
                    executed_transaction_count: entry.executed_transaction_count,
                    starting_transaction_index: entry.starting_transaction_index,
                })
                .collect(),
        })
    }
}

impl TryFrom<yp::SubscribeUpdateEntry> for EntryUpdate {
    type Error = Error;

    fn try_from(value: yp::SubscribeUpdateEntry) -> Result<Self, Self::Error> {
        Ok(EntryUpdate {
            slot: value.slot,
            hash: value.hash,
            index: value.index,
            num_hashes: value.num_hashes,
            executed_transaction_count: value.executed_transaction_count,
            starting_transaction_index: value.starting_transaction_index,
        })
    }
}

impl TryFrom<yp::SubscribeUpdateSlot> for SlotUpdate {
    type Error = Error;

    fn try_from(value: yp::SubscribeUpdateSlot) -> Result<Self, Self::Error> {
        Ok(SlotUpdate {
            slot: value.slot,
            parent: value.parent,
            status: value.status,
            dead_error: value.dead_error,
        })
    }
}

impl TryFrom<yp::SubscribeUpdateTransactionStatus> for TransactionStatusUpdate {
    type Error = Error;

    fn try_from(value: yp::SubscribeUpdateTransactionStatus) -> Result<Self, Self::Error> {
        Ok(TransactionStatusUpdate {
            slot: value.slot,
            is_vote: value.is_vote,
            signature: value.signature,
            index: value.index,
            err: None,
        })
    }
}
