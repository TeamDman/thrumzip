#[allow(clippy::module_inception)]
mod command;
pub mod profile_list_command;
pub mod profile_show_command;
pub mod profile_use_command;
pub mod validate_command;
pub use command::*;
pub mod profile_add_command;
pub mod profile_command;
pub mod sync_command;
