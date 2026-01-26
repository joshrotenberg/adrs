//! CLI command implementations.

mod config;
mod doctor;
mod edit;
mod export;
mod generate;
mod import;
mod init;
mod link;
mod list;
mod new;
mod status;
mod template;

pub use config::config_with_discovery;
pub use doctor::doctor;
pub use edit::edit;
pub use export::export_json;
pub use generate::{generate_book, generate_graph, generate_toc};
pub use import::import_json;
pub use init::init;
pub use link::link;
pub use list::list;
pub use new::new;
pub use status::status;
pub use template::{list as template_list, show as template_show};
