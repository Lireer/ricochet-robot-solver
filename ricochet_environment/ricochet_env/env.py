import enum
import gym
from gym import spaces
from .ricochet_env import RustyEnvironment


class Action(enum.IntEnum):
    RED_UP = 0
    RED_RIGHT = 1
    RED_DOWN = 2
    RED_LEFT = 3
    BLUE_UP = 4
    BLUE_RIGHT = 5
    BLUE_DOWN = 6
    BLUE_LEFT = 7
    GREEN_UP = 8
    GREEN_RIGHT = 9
    GREEN_DOWN = 10
    GREEN_LEFT = 11
    YELLOW_UP = 12
    YELLOW_RIGHT = 13
    YELLOW_DOWN = 14
    YELLOW_LEFT = 15


class Target(enum.IntEnum):
    RED = 0
    BLUE = 1
    GREEN = 2
    YELLOW = 3
    ANY = 4


class RicochetEnv(gym.Env):
    def __init__(self):
        self.env = RustyEnvironment()
        self.action_space = spaces.Discrete(16)
        self.observation_space = spaces.Tuple(
            (
                spaces.MultiBinary([16, 16]),  # right walls
                spaces.MultiBinary([16, 16]),  # down walls
                spaces.Tuple(
                    (
                        spaces.MultiDiscrete([16, 16]),  # red robot
                        spaces.MultiDiscrete([16, 16]),  # blue robot
                        spaces.MultiDiscrete([16, 16]),  # green robot
                        spaces.MultiDiscrete([16, 16]),  # yellow robot
                    )
                ),  # robot positions
                spaces.MultiDiscrete([16, 16]),  # target position
                spaces.Discrete(5)
            )
        )
        self.reward_range = (0, 1)

    def step(self, action: Action):
        return self.env.step(action) + (None,)

    def reset(self):
        return self.env.reset()
