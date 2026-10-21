# pylatro

Use [pyo3](https://pyo3.rs) to add python bindings to the rust balatro engine.

### Overview

I wanted to try and experiment with applying reinforcement learning to balatro. For someone without
any experience in that area, the easiest implementation I found was [gymnasium](https://gymnasium.farama.org/index.html).
I used [pyo3](https://pyo3.rs) to provide python bindings to the rust library and then setup the roughest gymnasium
training environment and agent (I really do not know what I am doing here). So far it compiles and runs
but nothing really works correctly.

### Replay recording

`BalatroEnv()` leaves recording off during training. Pass `record_dir` to write one replay log per episode:

```python
from env import BalatroEnv  # when running from pylatro/gym

env = BalatroEnv(record_dir="data/replays")
observation, info = env.reset()
# Step through the episode, then call env.close().
```

Each log can be opened with `balatro-tui --replay FILE` or browsed with `balatro-tui --library --data-dir data`. Resetting or closing an unfinished episode records it as abandoned.
