#[derive(Debug, Clone)]
pub struct Transaction {
    client: u16,
    action: Action,
}

impl Transaction {
    pub fn new(client: u16, action: Action) -> Self {
        Self { client, action }
    }

    pub fn client(&self) -> u16 {
        self.client
    }

    pub fn action(&self) -> Action {
        self.action
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Deposit(i64),
    Withdraw(i64),
    Dispute,
    Resolve,
    Chargeback,
}
