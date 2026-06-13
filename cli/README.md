# balatro-cli

Interactive terminal interface for the balatro-rs game engine.

## Running

```bash
# from workspace root
cargo run -p balatro-cli

# or from cli/
cargo run
```

## Options

| Flag            | Description                              |
| --------------- | ---------------------------------------- |
| `--load <FILE>` | Load a saved game state from a JSON file |

```bash
cargo run -p balatro-cli -- --load game_1781391549.json
```

## Controls

At each prompt, enter the number of the action you want to take.

| Input   | Effect                                       |
| ------- | -------------------------------------------- |
| `0`     | Print full game state                        |
| `1`–`N` | Execute that action                          |
| `s`     | Save current game to `game_<timestamp>.json` |

## Saving and loading

Press `s` at any action prompt to save. The file is written to the current working directory:

```
Saved to game_1781391549.json
```

Reload it later with `--load`:

```bash
cargo run -p balatro-cli -- --load game_1781391549.json
```

This is useful for setting up and replaying specific scenarios.

## Example session

```
Starting game...
Select action:
[0] Show game state
[1] SelectCard: K♤
[2] SelectCard: Q♤
[3] SelectCard: 4♧
[4] SelectCard: T♤
[5] SelectCard: 4♢
[6] SelectCard: T♧
[7] SelectCard: 2♤
[8] SelectCard: T♡
[9] MoveCard: Q♤ - left
...
[22] MoveCard: 2♤ - right
0

hand: K♤ Q♤ 4♧ T♤ 4♢ T♧ 2♤ T♡
discard pile: 0
deck: 44
jokers: (none)
consumables: (none)
planetarium: HC:L1 | 1P:L1 | 2P:L1 | 3K:L1 | ST:L1 | FL:L1 | FH:L1 | 4K:L1 | SF:L1 | RF:L1 | 5K:L1 | FLH:L1 | FF:L1
stage: Blind(Small)
ante: One
blind: Some(Small)
round: 0
hands remaining: 4
discards remaining: 4
money: 0
score: 0  target: 300
```
