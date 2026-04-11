# Toy Exchange

Toy Exchange is a simple toy payments engine that reads a series of
transactions from a CSV file, updates client accounts, handles disputes and
chargebacks, and then outputs the state of clients accounts as a CSV.

## Completeness

Toy Exchange handles deposits, withdrawals, disputes, resolutions, and
chargebacks. It blocks deposits and withdrawals on frozen clients.

*I'm not sure whether disputes, resolutions, or chargebacks should be allowed
for frozen clients. I believe that most banks would execute chargebacks on
frozen and misbehaving accounts. That being said, I chose to allow it.*

## Safety and Robustness

Toy Exchange is written in 100% safe Rust, prevents integer underflow, and
doesn't panic. An amount is represented as `u64` after a scaling factor is
applied. `rust_decimal` is used for parsing and display purposes.

## Efficiency

The `csv::Reader` streams files into an 8 KB buffer by default. Toy Exchange
takes advantage of this buffer by applying transactions at read time.

## Maintainability

Toy Exchange is maintanable because the business rules are encapsulated in
`exchange.rs`. If we decide to use gRPC instead of CSV files and `stdout`, this
module could be used again. Error handling is centralized in `main`. `Input`
and  `Output` are intentionally seperate from business concepts like
`Transaction` and `Client`.

