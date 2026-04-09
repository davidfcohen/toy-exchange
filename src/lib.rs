mod app;
mod csv;

use std::{error::Error, fs::File, io};

use app::{Action, Client, Exchange, Transaction};

pub fn run(path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(path)?;

    let mut exchange = Exchange::new();
    let mut reader = csv::Reader::new(file);

    const BATCH_SIZE: usize = 1000;
    while !reader.is_done() {
        let block = reader.read(BATCH_SIZE)?;
        let block = block.into_iter().map(map_transaction);
        exchange.apply(block);
    }

    let clients: Vec<csv::Client> = exchange
        .into_clients()
        .map(|(id, c)| map_client(id, c))
        .collect();

    let mut writer = csv::Writer::new(io::stdout());
    writer.write(&clients)?;
    Ok(())
}

fn map_transaction(tx: csv::Transaction) -> (u32, Transaction) {
    let id = tx.tx;
    let action = tx.r#type;
    let amount = tx.amount.unwrap_or(0.0);
    let amount = (amount * 1_0000.0) as u64;

    let action = match action {
        csv::Action::Deposit => Action::Deposit(amount),
        csv::Action::Withdrawal => Action::Withdraw(amount),
        csv::Action::Dispute => Action::Dispute,
        csv::Action::Resolve => Action::Resolve,
        csv::Action::Chargeback => Action::Chargeback,
    };

    let transaction = Transaction::new(tx.client, action);
    (id, transaction)
}

fn map_client(id: u16, client: Client) -> csv::Client {
    let map_amount = |amount: u64| amount as f64 / 1_0000.0;
    csv::Client {
        client: id,
        available: map_amount(client.available()),
        held: map_amount(client.held()),
        total: map_amount(client.total()),
        locked: client.is_locked(),
    }
}
