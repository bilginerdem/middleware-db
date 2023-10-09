use async_trait::async_trait;

use crate::models::command::{CommandRequest, CommandResponse};

#[async_trait]
pub trait Db {
    async fn execute(req: CommandRequest) -> Result<CommandResponse, std::io::Error>;
}
