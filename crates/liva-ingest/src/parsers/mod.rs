//! Statement parsers module.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

pub mod bidv;
pub mod camt053;
pub mod mt940;
pub mod multi_bank;
pub mod profile_parser;
pub mod tcb;
pub mod vcb;

pub use bidv::BidvParser;
pub use camt053::Camt053Parser;
pub use mt940::Mt940Parser;
pub use multi_bank::MultiBankTableParser;
pub use profile_parser::ProfileStatementParser;
pub use tcb::TcbParser;
pub use vcb::VcbParser;

use crate::models::StatementParser;

/// Returns the complete list of standard bank and format parsers.
pub fn default_parsers() -> Vec<Box<dyn StatementParser>> {
    vec![
        Box::new(Camt053Parser),
        Box::new(Mt940Parser),
        Box::new(VcbParser),
        Box::new(TcbParser),
        Box::new(BidvParser),
        Box::new(MultiBankTableParser),
    ]
}
