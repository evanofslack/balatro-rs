use balatro_rs::{action::Action, game::Game, stage::Stage};
use rand::Rng;

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
