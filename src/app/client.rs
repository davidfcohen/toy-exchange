#[derive(Debug, Default, Clone)]
pub struct Client {
    total: u64,
    held: u64,
    is_locked: bool,
}

impl Client {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn total(&self) -> u64 {
        self.total
    }

    pub fn held(&self) -> u64 {
        self.held
    }

    pub fn available(&self) -> u64 {
        self.total - self.held
    }

    pub fn deposit(&mut self, amount: u64) {
        if !self.is_locked {
            self.total += amount;
        }
    }

    pub fn withdraw(&mut self, amount: u64) {
        if !self.is_locked && self.total >= amount {
            self.total -= amount;
        }
    }

    pub fn dispute(&mut self, amount: u64) {
        self.held += amount;
    }

    pub fn resolve(&mut self, amount: u64) {
        self.held -= amount;
    }

    pub fn chargeback(&mut self, amount: u64) {
        self.held -= amount;
        self.total -= amount;
        self.is_locked = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deposit() {
        let mut client = Client::new();

        client.deposit(100);
        assert_eq!(client.total(), 100);
        assert_eq!(client.available(), 100);

        client.deposit(200);
        assert_eq!(client.total(), 300);
        assert_eq!(client.available(), 300);
    }

    #[test]
    fn test_withdraw() {
        let mut client = Client::new();
        client.deposit(300);

        client.withdraw(200);
        assert_eq!(client.total(), 100);
        assert_eq!(client.available(), 100);

        client.withdraw(100);
        assert_eq!(client.total(), 0);
        assert_eq!(client.available(), 0);
    }

    #[test]
    fn test_withdraw_insufficient() {
        let mut client = Client::new();
        client.deposit(300);

        client.withdraw(999);
        assert_eq!(client.total(), 300);
        assert_eq!(client.available(), 300);
    }

    #[test]
    fn test_dispute() {
        let mut client = Client::new();
        client.deposit(300);

        client.dispute(100);
        assert_eq!(client.total(), 300);
        assert_eq!(client.available(), 200);
        assert_eq!(client.held(), 100);

        client.dispute(200);
        assert_eq!(client.total(), 300);
        assert_eq!(client.available(), 0);
        assert_eq!(client.held(), 300);
    }

    #[test]
    fn test_resolve() {
        let mut client = Client::new();
        client.deposit(300);

        client.dispute(100);
        client.dispute(100);

        client.resolve(100);
        assert_eq!(client.total(), 300);
        assert_eq!(client.available(), 200);
        assert_eq!(client.held(), 100);
    }

    #[test]
    fn test_chargeback() {
        let mut client = Client::new();
        client.deposit(300);

        client.dispute(100);
        client.dispute(100);

        client.chargeback(100);
        assert_eq!(client.total(), 200);
        assert_eq!(client.available(), 100);
        assert_eq!(client.held(), 100);
    }
}
