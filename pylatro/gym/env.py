import gymnasium as gym
from gymnasium import spaces
import pylatro
from typing import Optional
import numpy as np
from pathlib import Path
import time


class BalatroEnv(gym.Env):
    def __init__(self, record_dir: Optional[str] = None):
        super(BalatroEnv, self).__init__()

        config = pylatro.Config()
        config.ante_end = 1

        self._config = config
        self._game = pylatro.GameEngine(self._config)
        self._record_dir = Path(record_dir) if record_dir is not None else None
        self._record_count = 0
        self._score = 0
        self._last_score = 0
        self._target_score = 0
        self._high_score = 0

        self.action_space = spaces.Discrete(len(self._game.gen_action_space()))
        self.observation_space = gym.spaces.Dict(
            {
                "score": gym.spaces.Discrete(100_000),
                "target": gym.spaces.Discrete(100_000),
                "stage": gym.spaces.Discrete(config.stage_max + 1),
                "round": gym.spaces.Discrete(config.ante_end + 1),
                "plays": gym.spaces.Discrete(config.plays + 1),
                "discards": gym.spaces.Discrete(config.discards + 1),
                "money": gym.spaces.Discrete(config.money_max + 1),
                "deck_len": gym.spaces.Discrete(config.deck_max + 1),
                "selected_len": gym.spaces.Discrete(config.selected_max + 1),
                "available_len": gym.spaces.Discrete(config.available_max + 1),
                "discarded_len": gym.spaces.Discrete(config.discarded_max + 1),
                "jokers_len": gym.spaces.Discrete(config.joker_slots_max + 1),
            }
        )
        self.score_queue = []
        self.actions_queue = []

    def _get_obs(self):
        state = self._game.state
        obs = {
            "score": self._score,
            "target": self._target_score,
            "stage": state.stage.int(),
            "round": state.round,
            "plays": state.plays,
            "discards": state.discards,
            "money": state.money,
            "deck_len": len(state.deck),
            "selected_len": len(state.selected),
            "available_len": len(state.available),
            "discarded_len": len(state.discarded),
            "jokers_len": len(state.jokers),
        }
        return obs

    def _get_info(self):
        return {"difference": self._target_score - self._score}

    def step(self, index):
        legal = False
        space = self._game.gen_action_space()
        # Action must be legal
        if space[index] == 1:
            legal = True
            self._game.handle_action_index(index)

        terminated = self._game.is_over
        truncated = terminated

        self._last_score = self._score
        self._score = self._game.state.score
        self._target_score = self._game.state.required_score

        if self._score > self._high_score:
            self._high_score = self._score
            # print(f"new high score: {score}")
            # print(f"win: {self._game.is_win}")

        reward = 0
        score_diff = self._score - self._last_score
        if score_diff > 0:
            reward = score_diff / 100
        if terminated:
            self._game.finish_recording()
            reward = 20 if self._game.is_win else 0
            # if self._game.is_win:
            # print(f"game win: {self._game.state}")
            # print(f"score: {self._game.state.score}")

            self.actions_queue.append(len(self._game.state.action_history))
            self.score_queue.append(self._score)
        if not legal:
            reward = -10
            print("illegal")
            print(index)
            print(self.action_mask())

        observation = self._get_obs()
        info = self._get_info()
        return observation, reward, terminated, truncated, info

    def reset(self, seed: Optional[int] = None, options: Optional[dict] = None):
        super().reset(seed=seed)
        self._game.finish_recording()
        self._config.seed = seed
        if seed is not None:
            self._config.seed_str = None
        self._game = pylatro.GameEngine(self._config)
        if self._record_dir is not None:
            self._record_dir.mkdir(parents=True, exist_ok=True)
            self._record_count += 1
            path = self._record_dir / f"{self._game.state.seed}_{time.time_ns()}_{self._record_count}.log"
            self._game.start_recording(str(path))
        self._score = 0
        self._last_score = 0
        self._target_score = 0
        observation = self._get_obs()
        info = self._get_info()

        return observation, info

    def close(self):
        self._game.finish_recording()
        super().close()

    def render(self, mode="human"):
        return

    def action_mask(self):
        mask = np.asarray(self._game.gen_action_space(), dtype=np.int8)
        return mask


def register():
    gym.register(
        id="gymnasium_env/Balatro-v0",
        entry_point=BalatroEnv,
    )
