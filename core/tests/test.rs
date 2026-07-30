use balatro_rs::{
    action::Action,
    config::{Config, RngMode},
    game::Game,
    stage::Stage,
    voucher::Voucher,
};
use rand::Rng;

fn run_game_gen_actions() {
    run_game_gen_actions_with(Game::default());
}

fn run_game_gen_actions_with(mut g: Game) {
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
        assert!(action_res.is_ok());
    }
    let result = g.result();
    // Ensure game is over at end
    assert!(result.is_some());
    // Check game state at end
    assert!(matches!(g.stage, Stage::End(_)));
}

fn run_game_action_space() {
    let mut g = Game::default();

    g.start();
    while !g.is_over() {
        // Get action space and vector
        let space = g.gen_action_space();
        let space_vec = space.to_vec();
        assert!(!space.is_empty());

        // Pick a random move and ensure its unmasked
        let mut i: usize;
        loop {
            i = rand::thread_rng().gen_range(0..space_vec.len());
            if space_vec[i] == 1 {
                break;
            }
        }
        let action = space.to_action(i, &g).expect("valid index to action");
        // dbg!("game state:\n{}", g.clone());
        // dbg!("play action: {}", action.clone());
        let action_res = g.handle_action(action.clone());
        // dbg!(action);
        assert!(action_res.is_ok());
    }
    let result = g.result();
    // Ensure game is over at end
    assert!(result.is_some());
    // Check game state at end
    assert!(matches!(g.stage, Stage::End(_)));
    // dbg!("game action history: {:?}", g.action_history);
}

#[test]
fn test_game() {
    run_game_gen_actions();
    run_game_action_space();
}

// `RngMode::Real` routes shop and pack generation through `balatro-seed`,
// including the shop-playing-card branch that only the Magic Trick voucher
// can reach — it used to be an outright `panic!`, since nothing could
// activate it before vouchers existed.
#[test]
fn test_real_rng_mode_shop_playing_cards() {
    for seed in ["TEST", "ABCD1234", "BALATRO"] {
        let mut g = Game::new(Config {
            rng_mode: RngMode::Real,
            seed_str: Some(seed.to_string()),
            ..Config::default()
        });
        g.start();
        // Straight to a stocked shop without having to win a blind.
        g.stage = Stage::PostBlind();
        g.handle_action(Action::CashOut(0)).expect("cash out");

        g.shop.voucher = Some(Voucher::MagicTrick);
        g.money = 1000;
        g.handle_action(Action::BuyVoucher(Voucher::MagicTrick))
            .expect("buy Magic Trick");

        let mut bought = 0;
        for _ in 0..200 {
            while let Some(card) = g.shop.cards.first().copied() {
                let deck_before = g.deck.cards().len();
                g.money = 1000;
                g.handle_action(Action::BuyPlayingCard(card))
                    .expect("buy shop playing card");
                assert_eq!(g.deck.cards().len(), deck_before + 1);
                bought += 1;
            }
            g.money = 1000;
            g.handle_action(Action::Reroll()).expect("reroll");
        }
        assert!(
            bought > 0,
            "seed {seed}: Magic Trick never produced a shop playing card"
        );
    }
}

#[test]
#[ignore]
fn test_games_gen_actions() {
    for _ in 0..1000 {
        run_game_gen_actions();
    }
}

#[test]
#[ignore]
fn test_games_action_space() {
    for _ in 0..1000 {
        run_game_action_space();
    }
}
