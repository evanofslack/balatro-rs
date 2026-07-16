# balatro-cli

Command-line tools for the balatro-rs game engine. This package ships two binaries:

- [`balatro-cli`](#balatro-cli-1) — interactive terminal play
- [`balatro-edit`](#balatro-edit) — non-interactive save-state editor, for quickly setting up test scenarios

## balatro-cli

Interactive terminal interface for the balatro-rs game engine.

### Running

```bash
# from workspace root
cargo run -p balatro-cli

# or from cli/
cargo run
```

### Options

| Flag            | Description                              |
| --------------- | ----------------------------------------- |
| `--load <FILE>` | Load a saved game state from a JSON file |

```bash
cargo run -p balatro-cli -- --load game_1781391549.json
```

### Controls

At each prompt, enter the number of the action you want to take.

| Input   | Effect                                       |
| ------- | --------------------------------------------- |
| `0`     | Print full game state                        |
| `1`–`N` | Execute that action                          |
| `s`     | Save current game to `game_<timestamp>.json` |

### Saving and loading

Press `s` at any action prompt to save. The file is written to the current working directory:

```
Saved to game_1781391549.json
```

Reload it later with `--load`:

```bash
cargo run -p balatro-cli -- --load game_1781391549.json
```

This is useful for setting up and replaying specific scenarios.

### Example session

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

## balatro-edit

Non-interactive editor for save-state JSON files. Each invocation loads a game, applies **one**
mutation, and writes the result back out — chain multiple invocations (in bash, or via piping) to
build up a scenario without playing through it by hand.

### Running

```bash
# from workspace root
cargo run -p balatro-cli --bin balatro-edit -- <OPTIONS> <COMMAND>

# or from cli/
cargo run --bin balatro-edit -- <OPTIONS> <COMMAND>

# via justfile
just edit <OPTIONS> <COMMAND>
```

### Options

| Flag            | Description                                                                                                              |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `--load <FILE>` | Read game state from a file. Omit to read JSON from stdin.                                                              |
| `--out <FILE>`  | Write the result to a file. Omit to overwrite `--load` in place, or print to stdout if reading from stdin. |

`show` never writes — it only reads and prints.

### Commands

Names for jokers/tarots/planets/cards are matched case-insensitively against the same identifiers
already used in the save JSON itself (e.g. a joker saved as `"HalfJoker"` is added with
`add-joker halfjoker`, `add-joker HalfJoker`, etc.) — so you can read a save file to find the exact
name to type.

| Command                                 | Effect                                    |
| ---------------------------------------- | ------------------------------------------ |
| `show`                                  | Print full game state, unchanged           |
| `set-money <amount>`                    | Set money to an exact amount               |
| `add-money <amount>`                    | Add to current money                       |
| `set-reroll-cost <amount>`              | Set the shop reroll cost                   |
| `set-ante <n>`                          | Set current ante (`0`–`8`)                |
| `set-round <n>`                         | Set the round number                       |
| `set-plays <n>`                         | Set hands remaining                        |
| `set-discards <n>`                      | Set discards remaining                     |
| `set-score <n>`                         | Set current score                          |
| `set-chips <n>`                         | Set current chips                          |
| `set-mult <n>`                          | Set current mult                           |
| `set-joker-slots <n>`                   | Set max joker slots                        |
| `add-joker <name>`                      | Add a joker (see flags below)              |
| `add-tarot <name>`                      | Add a tarot consumable, e.g. `Fool`        |
| `add-planet <name>`                     | Add a planet consumable, e.g. `Mercury`    |
| `clear-consumables`                     | Remove all held consumables                |
| `add-card <value> <suit>`               | Add a playing card to the deck (see flags below) |

`add-joker` flags:

| Flag              | Description                                          |
| ------------------ | ------------------------------------------------------ |
| `--edition <e>`   | `base`, `foil`, `holographic`, `polychrome`, `negative` |
| `--eternal`       | Mark the joker eternal                               |
| `--perishable`    | Mark the joker perishable                            |
| `--rental`        | Mark the joker rental                                |

`add-card` flags:

| Flag                 | Description                                        |
| --------------------- | ----------------------------------------------------- |
| `--enhancement <e>`  | `bonus`, `mult`, `wild`, `glass`, `steel`, `stone`, `gold`, `lucky` |
| `--edition <e>`      | `base`, `foil`, `holographic`, `polychrome`, `negative` |
| `--seal <s>`         | `gold`, `red`, `blue`, `purple`                     |

`<value>` is a card rank (`two`…`ten`, `jack`, `queen`, `king`, `ace`); `<suit>` is `spade`,
`club`, `heart`, or `diamond`.

There is currently no way to remove a specific joker or card — only add, set, or clear.

### Examples

Bump money and force the ante up on a save, in place:

```bash
just edit --load game_1781391549.json add-money 100
just edit --load game_1781391549.json set-ante 6
```

Add a specific joker to test its interaction, with an edition and a sticker:

```bash
just edit --load game_1781391549.json add-joker Blueprint --edition negative --eternal
```

Add a card to the deck with an enhancement:

```bash
just edit --load game_1781391549.json add-card ace spade --enhancement glass
```

Give yourself a tarot to test, then inspect the result:

```bash
just edit --load game_1781391549.json add-tarot Fool
just edit --load game_1781391549.json show
```

Chain several edits into a fresh scenario file by piping through stdin/stdout instead of editing
in place:

```bash
cat game_1781391549.json \
  | just edit add-money 200 \
  | just edit set-ante 6 \
  | just edit add-joker Blueprint \
  > scenario.json

cargo run -p balatro-tui -- --load scenario.json
```

Invalid names fail fast with a clean error instead of a panic:

```bash
$ just edit --load game_1781391549.json add-joker NotAJoker
error: invalid value 'NotAJoker' for '<NAME>': Matching variant not found
```
