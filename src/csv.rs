use std::{error::Error, io};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Transaction {
    pub r#type: Action,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug)]
pub struct Reader<R> {
    reader: csv::Reader<R>,
}

impl<R: io::Read> Reader<R> {
    pub fn new(reader: R) -> Self {
        Reader {
            reader: csv::Reader::from_reader(reader),
        }
    }

    pub fn read(&mut self, count: usize) -> Result<Vec<Transaction>, Box<dyn Error>> {
        let mut records = Vec::new();

        for result in self.reader.deserialize().take(count) {
            let record: Transaction = result?;
            records.push(record);
        }

        Ok(records)
    }

    pub fn is_done(&self) -> bool {
        self.reader.is_done()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Client {
    pub client: u16,
    pub available: String,
    pub held: String,
    pub total: String,
    pub locked: bool,
}

#[derive(Debug)]
pub struct Writer<W: io::Write> {
    writer: csv::Writer<W>,
}

impl<W: io::Write> Writer<W> {
    pub fn new(writer: W) -> Self {
        Writer {
            writer: csv::Writer::from_writer(writer),
        }
    }

    pub fn write(&mut self, records: &[Client]) -> Result<(), Box<dyn Error>> {
        for record in records {
            self.writer.serialize(record)?;
        }

        self.writer.flush()?;
        Ok(())
    }
}
