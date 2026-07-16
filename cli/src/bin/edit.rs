use anyhow::{Context, Result};
use balatro_rs::ante::Ante;
use balatro_rs::card::{Card, Edition, Enhancement, Seal, Suit, Value};
use balatro_rs::consumable::Consumable;
use balatro_rs::game::Game;
use balatro_rs::joker::{Jokers, Stickers};
use balatro_rs::planet::Planets;
use balatro_rs::tarot::Tarot;
use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Read, Write};

#[derive(Parser)]
struct Args {
    #[arg(long, value_name = "FILE")]
    load: Option<String>,
    #[arg(long, value_name = "FILE")]
    out: Option<String>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Show,
    SetMoney {
        amount: usize,
    },
    AddMoney {
        amount: usize,
    },
    SetRerollCost {
        amount: usize,
    },
    SetAnte {
        ante: usize,
    },
    SetRound {
        round: usize,
    },
    SetPlays {
        plays: usize,
    },
    SetDiscards {
        discards: usize,
    },
    SetScore {
        score: usize,
    },
    SetChips {
        chips: usize,
    },
    SetMult {
        mult: usize,
    },
    SetJokerSlots {
        slots: usize,
    },
    AddJoker {
        name: Jokers,
        #[arg(long)]
        edition: Option<Edition>,
        #[arg(long)]
        eternal: bool,
        #[arg(long)]
        perishable: bool,
        #[arg(long)]
        rental: bool,
    },
    AddTarot {
        name: Tarot,
    },
    AddPlanet {
        name: Planets,
    },
    ClearConsumables,
    AddCard {
        value: Value,
        suit: Suit,
        #[arg(long)]
        enhancement: Option<Enhancement>,
        #[arg(long)]
        edition: Option<Edition>,
        #[arg(long)]
        seal: Option<Seal>,
    },
}

fn load_game(load: &Option<String>) -> Result<Game> {
    let contents = match load {
        Some(path) => {
            fs::read_to_string(path).with_context(|| format!("failed to read {path}"))?
        }
        None => {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .context("failed to read stdin")?;
            buf
        }
    };
    Game::from_json(&contents).context("failed to parse game state")
}

fn write_game(game: &Game, load: &Option<String>, out: &Option<String>) -> Result<()> {
    let json = game.to_json().context("failed to serialize game state")?;
    match out.as_ref().or(load.as_ref()) {
        Some(path) => fs::write(path, json).with_context(|| format!("failed to write {path}")),
        None => io::stdout()
            .write_all(json.as_bytes())
            .context("failed to write stdout"),
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut game = load_game(&args.load)?;

    match args.command {
        Command::Show => {
            println!("{game}");
            return Ok(());
        }
        Command::SetMoney { amount } => game.money = amount,
        Command::AddMoney { amount } => game.money = game.money.saturating_add(amount),
        Command::SetRerollCost { amount } => game.reroll_cost = amount,
        Command::SetAnte { ante } => {
            game.ante_current = Ante::try_from(ante)
                .map_err(|_| anyhow::anyhow!("invalid ante: {ante} (must be 0-8)"))?;
        }
        Command::SetRound { round } => game.round = round,
        Command::SetPlays { plays } => game.plays = plays,
        Command::SetDiscards { discards } => game.discards = discards,
        Command::SetScore { score } => game.score = score,
        Command::SetChips { chips } => game.chips = chips,
        Command::SetMult { mult } => game.mult = mult,
        Command::SetJokerSlots { slots } => game.config.joker_slots = slots,
        Command::AddJoker {
            name,
            edition,
            eternal,
            perishable,
            rental,
        } => {
            let mut joker = name;
            if let Some(edition) = edition {
                joker.set_edition(edition);
            }
            if eternal || perishable || rental {
                joker.set_stickers(Stickers {
                    eternal,
                    perishable,
                    rental,
                });
            }
            game.jokers.push(joker);
        }
        Command::AddTarot { name } => game.consumables.push(Consumable::Tarot(name)),
        Command::AddPlanet { name } => game.consumables.push(Consumable::Planet(name)),
        Command::ClearConsumables => game.consumables.clear(),
        Command::AddCard {
            value,
            suit,
            enhancement,
            edition,
            seal,
        } => {
            let mut card = Card::new(value, suit);
            card.enhancement = enhancement;
            if let Some(edition) = edition {
                card.edition = edition;
            }
            card.seal = seal;
            game.deck.push(card);
        }
    }

    write_game(&game, &args.load, &args.out)
}
