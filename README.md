# Toy Exchange

Toy Exchange is a simple toy payments engine that reads a series of
transactions from a CSV file, updates client accounts, handles disputes and
chargebacks, and then outputs the state of clients accounts as a CSV.

## Completeness

Toy Exchange handles deposits, withdrawals, disputes, resolutions, and
chargebacks. It blocks deposits and withdrawals on frozen clients. It blocks
deposits and withdrawals with negative amounts.

*It's not explicitly stated whether negative balances are allowed. While it's
clearly stated that withdrawals exceeding the account balance should be
blocked, the behavior is unclear for disputes, resolutions, and chargebacks. For example,
a new client could first deposit `1.0` then withdraw `1.0`. If their first
transaction is disputed, the clients available balance is negative. I believe that's
how a bank account would behave, so, I used `i64` to represent the amount.*

*I'm not sure whether disputes, resolutions, or chargebacks should be allowed
for frozen clients. I believe that most banks would execute chargebacks on
frozen and misbehaving accounts. That being said, I chose to allow it.*

## Safety and Robustness

Toy Exchange is written in 100% safe Rust. Release builds will not panic. An
amount is represented as `i64` after a scaling factor is applied.
`rust_decimal` is used for parsing and display purposes.

## Efficiency

The `csv::Reader` streams files into an 8 KB buffer by default. Toy Exchange
takes advantage of this buffer by applying transactions at read time.

## Maintainability

Toy Exchange is maintainable because the business rules are encapsulated in
`exchange.rs`. If we decide to use gRPC instead of CSV files and `stdout`, this
module could be used again. Error handling is centralized in `main`. `Input`
and  `Output` are intentionally seperate from business concepts like
`Transaction` and `Client`.

