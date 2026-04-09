mod app;
mod csv;

use app::{Action, Client, Exchange, Transaction};
use std::{error::Error, fs::File, io};

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
    let amount = tx.amount.as_deref().and_then(parse_amount).unwrap_or(0);

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

fn parse_amount(s: &str) -> Option<u64> {
    let (whole, frac) = match s.split_once('.') {
        Some((w, f)) => (w, f),
        None => (s, ""),
    };

    if frac.len() > 4 {
        return None;
    }

    let whole: u64 = whole.parse().ok()?;

    let frac_padded = format!("{:0<4}", frac);
    let frac: u64 = frac_padded.parse().ok()?;

    let amount = whole * 1_0000 + frac;
    Some(amount)
}

fn map_client(id: u16, client: Client) -> csv::Client {
    let format_amount = |amount: u64| {
        let num = amount / 1_0000;
        let dec = amount % 1_0000;
        format!("{num}.{dec}")
    };

    csv::Client {
        client: id,
        available: format_amount(client.available()),
        held: format_amount(client.held()),
        total: format_amount(client.total()),
        locked: client.is_locked(),
    }
}
