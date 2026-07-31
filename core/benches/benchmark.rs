use balatro_rs::{
    action::Action,
    config::Config,
    game::Game,
    hand::SelectHand,
    stage::{Blind, Stage},
};
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rand::Rng;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("run game gen actions", |b| b.iter(run_game_gen_actions));

    // Per-action cost. `handle_action` is dominated by whichever action you
    // happen to take — a Play scores a whole hand, a SelectCard flips a
    // flag — so a blended average is close to meaningless. Each case clones
    // a prepared state in setup (excluded from the measurement) so the
    // action runs against identical input every iteration.
    let mut group = c.benchmark_group("handle_action");

    group.bench_function("SelectCard", |b| {
        let game = game_in_blind();
        b.iter_batched(
            || {
                let g = game.clone();
                let card = g.available.cards()[0];
                (g, card)
            },
            |(mut g, card)| g.handle_action(Action::SelectCard(card)),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("Play", |b| {
        let game = game_with_selection(5);
        b.iter_batched(
            || game.clone(),
            |mut g| g.handle_action(Action::Play()),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("Discard", |b| {
        let game = game_with_selection(5);
        b.iter_batched(
            || game.clone(),
            |mut g| g.handle_action(Action::Discard()),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("SelectBlind", |b| {
        let game = fresh_game();
        b.iter_batched(
            || game.clone(),
            |mut g| g.handle_action(Action::SelectBlind(Blind::Small)),
            BatchSize::SmallInput,
        )
    });

    group.finish();

    // The other half of an RL step: deciding what is legal.
    let mut group = c.benchmark_group("move generation");
    let game = game_in_blind();
    group.bench_function("gen_actions", |b| b.iter(|| game.gen_actions().count()));
    group.bench_function("gen_action_space", |b| b.iter(|| game.gen_action_space()));
    group.bench_function("gen_action_space to_vec", |b| {
        b.iter(|| game.gen_action_space().to_vec())
    });
    group.finish();

    // A whole env step as a training loop would issue it.
    c.bench_function("rl step (gen_action_space + handle_action)", |b| {
        let game = game_in_blind();
        b.iter_batched(
            || {
                let g = game.clone();
                let card = g.available.cards()[0];
                (g, card)
            },
            |(mut g, card)| {
                let _ = g.gen_action_space().to_vec();
                g.handle_action(Action::SelectCard(card))
            },
            BatchSize::SmallInput,
        )
    });

    // `Play` dominates every other action, so break it into its parts.
    let mut group = c.benchmark_group("play internals");
    let selected = game_with_selection(5).available.selected();
    group.bench_function("SelectHand::new", |b| {
        b.iter(|| SelectHand::new(selected.clone()))
    });
    group.bench_function("best_hand", |b| {
        let hand = SelectHand::new(selected.clone());
        b.iter(|| hand.best_hand())
    });
    group.bench_function("calc_score", |b| {
        let game = game_with_selection(5);
        let made = SelectHand::new(selected.clone()).best_hand().unwrap();
        b.iter_batched(
            || (game.clone(), made.clone()),
            |(mut g, made)| g.calc_score(made),
            BatchSize::SmallInput,
        )
    });
    group.finish();

    // Tree search copies states constantly, so this is its own budget line.
    c.bench_function("game clone", |b| {
        let game = game_in_blind();
        b.iter(|| game.clone())
    });
}

/// Fixed seed so every run benches the same deck and shop.
fn fresh_game() -> Game {
    let mut g = Game::new(Config {
        seed: Some(0xBA1A7809),
        ..Config::default()
    });
    g.start();
    g
}

fn game_in_blind() -> Game {
    let mut g = fresh_game();
    g.handle_action(Action::SelectBlind(Blind::Small))
        .expect("select blind");
    debug_assert!(matches!(g.stage, Stage::Blind(_)));
    g
}

fn game_with_selection(n: usize) -> Game {
    let mut g = game_in_blind();
    let cards: Vec<_> = g.available.cards().iter().take(n).copied().collect();
    for card in cards {
        g.handle_action(Action::SelectCard(card))
            .expect("select card");
    }
    g
}

fn run_game_gen_actions() {
    let mut g = Game::default();

    g.start();
    while !g.is_over() {
        // Get all available moves
        let actions: Vec<Action> = g.gen_actions().collect();
        if actions.is_empty() {
            break;
        }

        // Pick a random move and execute it
        let i = rand::thread_rng().gen_range(0..actions.len());
        let action = actions[i].clone();
        let action_res = g.handle_action(action.clone());
        debug_assert!(action_res.is_ok());
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
