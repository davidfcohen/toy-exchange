use std::collections::HashMap;

use super::{
    client::Client,
    transaction::{Action, Transaction},
};

#[derive(Debug, Default, Clone)]
pub struct Exchange {
    clients: HashMap<u16, Client>,
    records: HashMap<u32, Record>,
}

impl Exchange {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply(&mut self, block: impl Iterator<Item = (u32, Transaction)>) {
        for (id, tx) in block {
            self.apply_one(id, tx);
        }
    }

    pub fn apply_one(&mut self, id: u32, tx: Transaction) {
        let client = self.clients.entry(tx.client()).or_default();

        match tx.action() {
            Action::Deposit(amount) => {
                client.deposit(amount);
            }
            Action::Withdraw(amount) => {
                client.withdraw(amount);
            }
            Action::Dispute => {
                if let Some(amount) = dispute_deposit_record(&mut self.records, id) {
                    client.dispute(amount);
                }
            }
            Action::Resolve => {
                if let Some(amount) = resolve_deposit_record(&mut self.records, id) {
                    client.resolve(amount);
                }
            }
            Action::Chargeback => {
                if let Some(amount) = chargeback_deposit_record(&mut self.records, id) {
                    client.chargeback(amount);
                }
            }
        }

        let record = Record::new(&tx);
        self.records.insert(id, record);
    }

    pub fn into_clients(self) -> impl Iterator<Item = (u16, Client)> {
        self.clients.into_iter()
    }
}

fn dispute_deposit_record(records: &mut HashMap<u32, Record>, id: u32) -> Option<u64> {
    records.get_mut(&id).and_then(|record| match record.action {
        Action::Deposit(amount) if !record.is_disputed => {
            record.is_disputed = true;
            Some(amount)
        }
        _ => None,
    })
}

fn resolve_deposit_record(records: &mut HashMap<u32, Record>, id: u32) -> Option<u64> {
    records.get_mut(&id).and_then(|record| match record.action {
        Action::Deposit(amount) if record.is_disputed => {
            record.is_disputed = false;
            Some(amount)
        }
        _ => None,
    })
}

fn chargeback_deposit_record(records: &mut HashMap<u32, Record>, id: u32) -> Option<u64> {
    records.remove(&id).and_then(|record| match record.action {
        Action::Deposit(amount) if record.is_disputed => Some(amount),
        _ => None,
    })
}

#[derive(Debug, Clone)]
struct Record {
    client: u16,
    action: Action,
    is_disputed: bool,
}

impl Record {
    fn new(tx: &Transaction) -> Self {
        Self {
            client: tx.client(),
            action: tx.action(),
            is_disputed: false,
        }
    }
}
