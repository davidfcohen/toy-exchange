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

    pub fn apply(&mut self, block: impl IntoIterator<Item = (u32, Transaction)>) {
        for (id, tx) in block {
            self.apply_one(id, tx);
        }
    }

    pub fn apply_one(&mut self, id: u32, tx: Transaction) {
        let client = self.clients.entry(tx.client()).or_default();

        match tx.action() {
            Action::Deposit(amount) => {
                client.deposit(amount);
                let record = Record::new(&tx);
                self.records.insert(id, record);
            }
            Action::Withdraw(amount) => {
                client.withdraw(amount);
                let record = Record::new(&tx);
                self.records.insert(id, record);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_given_example() {
        let mut exchange = Exchange::new();
        exchange.apply([
            (1, Transaction::new(1, Action::Deposit(1_0000))),
            (2, Transaction::new(2, Action::Deposit(2_0000))),
            (3, Transaction::new(1, Action::Deposit(2_0000))),
            (4, Transaction::new(1, Action::Withdraw(1_5000))),
            (5, Transaction::new(2, Action::Withdraw(3_0000))),
        ]);

        let clients: HashMap<_, _> = exchange.into_clients().collect();

        let c1 = clients.get(&1).unwrap();
        assert_eq!(c1.total(), 1_5000);
        assert_eq!(c1.available(), 1_5000);
        assert_eq!(c1.held(), 0);
        assert!(!c1.is_locked());

        let c2 = clients.get(&2).unwrap();
        assert_eq!(c2.total(), 2_0000);
        assert_eq!(c2.available(), 2_0000);
        assert_eq!(c2.held(), 0);
        assert!(!c2.is_locked());
    }

    #[test]
    fn test_dispute() {
        let mut exchange = Exchange::new();
        exchange.apply([
            (1, Transaction::new(1, Action::Deposit(1_0000))),
            (2, Transaction::new(1, Action::Deposit(2_0000))),
            (1, Transaction::new(1, Action::Dispute)),
        ]);

        let clients: HashMap<_, _> = exchange.into_clients().collect();
        let c1 = clients.get(&1).unwrap();
        assert_eq!(c1.total(), 3_0000);
        assert_eq!(c1.available(), 2_0000);
        assert_eq!(c1.held(), 1_0000);
        assert!(!c1.is_locked());
    }

    #[test]
    fn test_dispute_not_found() {
        let mut exchange = Exchange::new();
        exchange.apply([
            (1, Transaction::new(1, Action::Deposit(1_0000))),
            (2, Transaction::new(1, Action::Deposit(2_0000))),
            (9, Transaction::new(1, Action::Dispute)),
        ]);

        let clients: HashMap<_, _> = exchange.into_clients().collect();
        let c1 = clients.get(&1).unwrap();
        assert_eq!(c1.total(), 3_0000);
        assert_eq!(c1.available(), 3_0000);
        assert_eq!(c1.held(), 0);
        assert!(!c1.is_locked());
    }

    #[test]
    fn test_resolve() {
        let mut exchange = Exchange::new();
        exchange.apply([
            (1, Transaction::new(1, Action::Deposit(1_0000))),
            (2, Transaction::new(1, Action::Deposit(2_0000))),
            (1, Transaction::new(1, Action::Dispute)),
            (1, Transaction::new(1, Action::Resolve)),
        ]);

        let clients: HashMap<_, _> = exchange.into_clients().collect();
        let c1 = clients.get(&1).unwrap();
        assert_eq!(c1.total(), 3_0000);
        assert_eq!(c1.available(), 3_0000);
        assert_eq!(c1.held(), 0);
        assert!(!c1.is_locked());
    }

    #[test]
    fn test_resolve_not_found() {
        let mut exchange = Exchange::new();
        exchange.apply([
            (1, Transaction::new(1, Action::Deposit(1_0000))),
            (2, Transaction::new(1, Action::Deposit(2_0000))),
            (1, Transaction::new(1, Action::Dispute)),
            (9, Transaction::new(1, Action::Resolve)),
        ]);

        let clients: HashMap<_, _> = exchange.into_clients().collect();
        let c1 = clients.get(&1).unwrap();
        assert_eq!(c1.total(), 3_0000);
        assert_eq!(c1.available(), 2_0000);
        assert_eq!(c1.held(), 1_0000);
        assert!(!c1.is_locked());
    }

    #[test]
    fn test_resolve_undisputed() {
        let mut exchange = Exchange::new();
        exchange.apply([
            (1, Transaction::new(1, Action::Deposit(1_0000))),
            (2, Transaction::new(1, Action::Deposit(2_0000))),
            (1, Transaction::new(1, Action::Resolve)),
        ]);

        let clients: HashMap<_, _> = exchange.into_clients().collect();
        let c1 = clients.get(&1).unwrap();
        assert_eq!(c1.total(), 3_0000);
        assert_eq!(c1.available(), 3_0000);
        assert_eq!(c1.held(), 0);
        assert!(!c1.is_locked());
    }

    #[test]
    fn test_chargeback() {
        let mut exchange = Exchange::new();
        exchange.apply([
            (1, Transaction::new(1, Action::Deposit(1_0000))),
            (2, Transaction::new(1, Action::Deposit(2_0000))),
            (1, Transaction::new(1, Action::Dispute)),
            (1, Transaction::new(1, Action::Chargeback)),
        ]);

        let clients: HashMap<_, _> = exchange.into_clients().collect();
        let c1 = clients.get(&1).unwrap();
        assert_eq!(c1.total(), 2_0000);
        assert_eq!(c1.available(), 2_0000);
        assert_eq!(c1.held(), 0);
        assert!(c1.is_locked());
    }

    #[test]
    fn test_chargeback_undisputed() {
        let mut exchange = Exchange::new();
        exchange.apply([
            (1, Transaction::new(1, Action::Deposit(1_0000))),
            (2, Transaction::new(1, Action::Deposit(2_0000))),
            (1, Transaction::new(1, Action::Chargeback)),
        ]);

        let clients: HashMap<_, _> = exchange.into_clients().collect();
        let c1 = clients.get(&1).unwrap();
        assert_eq!(c1.total(), 3_0000);
        assert_eq!(c1.available(), 3_0000);
        assert_eq!(c1.held(), 0);
        assert!(!c1.is_locked());
    }

    #[test]
    fn test_chargeback_not_found() {
        let mut exchange = Exchange::new();
        exchange.apply([
            (1, Transaction::new(1, Action::Deposit(1_0000))),
            (2, Transaction::new(1, Action::Deposit(2_0000))),
            (1, Transaction::new(1, Action::Dispute)),
            (9, Transaction::new(1, Action::Chargeback)),
        ]);

        let clients: HashMap<_, _> = exchange.into_clients().collect();
        let c1 = clients.get(&1).unwrap();
        assert_eq!(c1.total(), 3_0000);
        assert_eq!(c1.available(), 2_0000);
        assert_eq!(c1.held(), 1_0000);
        assert!(!c1.is_locked())
    }
}
