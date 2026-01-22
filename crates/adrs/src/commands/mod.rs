//! CLI command implementations.

mod config;
mod edit;
mod generate;
mod init;
mod link;
mod list;
mod new;

pub use config::config;
pub use edit::edit;
pub use generate::{generate_book, generate_graph, generate_toc};
pub use init::init;
pub use link::link;
pub use list::list;
pub use new::new;
