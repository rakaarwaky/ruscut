pub mod cli_command_handler;
pub mod tui_command_handler;

pub use cli_command_handler::CliCommandHandler;
pub use tui_command_handler::TuiCommandHandler;

pub const BARREL: bool = true;
