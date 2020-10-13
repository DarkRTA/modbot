#[macro_use]
extern crate log;
mod config;
mod modules;

use crate::modules::Module;
use std::error::Error;
use twitchchat::{connector, messages::Commands, AsyncRunner, Status, UserConfig, Writer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut modules = config::modules();

    let config = UserConfig::builder()
        .name(config::USERNAME)
        .token(config::TOKEN)
        .enable_all_capabilities()
        .build()?;

    info!("connecting...");
    let connector = connector::tokio::Connector::twitch()?;
    let mut runner = AsyncRunner::connect(connector, &config).await?;

    info!("joining channel...");
    runner.join(config::CHANNEL).await?;

    main_loop(runner, &mut modules).await?;
    Ok(())
}

async fn main_loop(
    mut runner: AsyncRunner,
    mut modules: &mut Vec<Box<dyn Module>>,
) -> Result<(), Box<dyn Error>> {
    info!("connected...");
    loop {
        match runner.next_message().await? {
            Status::Message(msg) => handle_message(msg, runner.writer(), &mut modules)?,
            Status::Quit => {
                break;
            }
            Status::Eof => {
                break;
            }
        }
    }

    Ok(())
}

fn handle_message(
    msg: Commands,
    writer: Writer,
    modules: &mut Vec<Box<dyn Module>>,
) -> Result<(), Box<dyn Error>> {
    for module in modules {
        module.tick(writer.clone());
        match msg {
            Commands::Privmsg(ref msg) => module.privmsg(&msg, writer.clone()),
            _ => (),
        }
    }
    Ok(())
}
