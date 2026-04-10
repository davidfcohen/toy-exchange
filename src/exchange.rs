mod client;
mod transaction;

use std::collections::HashMap;

pub use client::Client;
pub use transaction::{Action, Transaction};

#[derive(Debug, Default, Clone)]
pub struct Exchange {
    clients: HashMap<u16, Client>,
    records: HashMap<u32, Deposit>,
}

impl Exchange {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply(&mut self, id: u32, tx: Transaction) {
        let client = self.clients.entry(tx.client()).or_default();
        let records = &mut self.records;

        match tx.action() {
            Action::Deposit(amount) => {
                client.deposit(amount);
                records.insert(id, Deposit::new(amount));
            }
            Action::Withdraw(amount) => {
                client.withdraw(amount);
            }
            Action::Dispute => {
                if let Some(amount) = dispute_deposit(records, id) {
                    client.dispute(amount);
                }
            }
            Action::Resolve => {
                if let Some(amount) = resolve_deposit(records, id) {
                    client.resolve(amount);
                }
            }
            Action::Chargeback => {
                if let Some(amount) = chargeback_deposit(records, id) {
                    client.chargeback(amount);
                }
            }
        }
    }

    pub fn into_clients(self) -> impl Iterator<Item = (u16, Client)> {
        self.clients.into_iter()
    }
}

fn dispute_deposit(records: &mut HashMap<u32, Deposit>, id: u32) -> Option<u64> {
    records
        .get_mut(&id)
        .filter(|deposit| !deposit.is_disputed)
        .map(|deposit| {
            deposit.is_disputed = true;
            deposit.amount
        })
}

fn resolve_deposit(records: &mut HashMap<u32, Deposit>, id: u32) -> Option<u64> {
    records
        .get_mut(&id)
        .filter(|deposit| deposit.is_disputed)
        .map(|deposit| {
            deposit.is_disputed = false;
            deposit.amount
        })
}

fn chargeback_deposit(records: &mut HashMap<u32, Deposit>, id: u32) -> Option<u64> {
    records
        .remove(&id)
        .filter(|deposit| deposit.is_disputed)
        .map(|deposit| deposit.amount)
}

#[derive(Debug, Clone)]
struct Deposit {
    amount: u64,
    is_disputed: bool,
}

impl Deposit {
    fn new(amount: u64) -> Self {
        Self {
            amount,
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
