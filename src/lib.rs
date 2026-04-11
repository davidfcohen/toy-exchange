mod exchange;

use csv::{ReaderBuilder, Trim, Writer};
use exchange::{Action, Client, Exchange, Transaction};
use rust_decimal::{Decimal, prelude::*};
use serde::{Deserialize, Serialize};

use std::{error::Error, fs::File, io};

#[derive(Debug, Clone, Deserialize)]
struct Input {
    r#type: InputKind,
    client: u16,
    tx: u32,
    amount: Option<Decimal>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum InputKind {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Clone, Serialize)]
struct Output {
    client: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

pub fn run(path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(path)?;

    let mut exchange = Exchange::new();
    let mut reader = ReaderBuilder::new().trim(Trim::All).from_reader(file);

    for result in reader.deserialize() {
        let input: Input = result?;
        let (id, tx) = map_input(input);
        exchange.apply(id, tx);
    }

    let mut writer = Writer::from_writer(io::stdout());
    for (id, client) in exchange.into_clients() {
        let output = map_output(id, client);
        writer.serialize(output)?;
    }

    Ok(())
}

const SCALE_FACTOR: Decimal = dec!(1_0000);

fn map_input(input: Input) -> (u32, Transaction) {
    let map_amount = |amount: Decimal| {
        let amount = amount * SCALE_FACTOR;
        amount.to_i64().unwrap_or_default()
    };

    let Input {
        r#type: kind,
        client,
        tx: id,
        amount,
    } = input;

    let amount = amount.map(map_amount).unwrap_or_default();
    let action = map_input_kind(kind, amount);
    let tx = Transaction::new(client, action);

    (id, tx)
}

fn map_input_kind(kind: InputKind, amount: i64) -> Action {
    match kind {
        InputKind::Deposit => Action::Deposit(amount),
        InputKind::Withdrawal => Action::Withdraw(amount),
        InputKind::Dispute => Action::Dispute,
        InputKind::Resolve => Action::Resolve,
        InputKind::Chargeback => Action::Chargeback,
    }
}

fn map_output(id: u16, client: Client) -> Output {
    let map_amount = |amount| {
        Decimal::from_i64(amount)
            .map(|amount| amount / SCALE_FACTOR)
            .unwrap_or_default()
    };

    let available = client.available();
    let held = client.held();
    let total = client.total();

    Output {
        client: id,
        available: map_amount(available),
        held: map_amount(held),
        total: map_amount(total),
        locked: client.is_locked(),
    }
}
