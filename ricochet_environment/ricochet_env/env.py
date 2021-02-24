import enum
import gym
import numpy as np
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
    """An OpenAI Gym compatible environment for the board game Ricochet Robots."""

    def __init__(
        self,
        board_size=16,
        walls="variants",
        targets="variants",
        robots="random",
        seed=None,
    ):
        """Create an environment for the ricochet robots game.

        Parameters
        ----------
        board_size : int
            The side length of the square board. (*Default* `16`)
        walls : str or int
            Decides how the walls should be set, possible values are
            - "fixed": One board that will always be the same.
            - "variants": A board is randomly chosen from a finite set of
                          boards. The cardinality of the set is 486. (*Default*)
            - int: Same as using `"variants"` but gives control over the
                   cardinality of the set.
            - "random": A board is randomly chosen from a practically infinite
                        set.
        targets : str or Tuple(int,int) or List[Tuple(int,int)]
            Decides how the targets will be chosen, possible values are
            - "variants": Chooses the target depending on the board variant.
                          Not usable with walls set to "random". (*Default*)
            - [(Target, (int,int))]: The target will be chosen randomly from the
                                     given list.
        robots: str or List[Tuple(int, int)]
            Decides where the robots are located before making the first move.
            - [(int,int)]: There have to be four elements in the list, each of
                           which decides the positions of the robots in the
                           order red, blue, green, yellow.
            - "random": The starting positions are chosen randomly. (*Default*)
        seed: int
            Can be set to make the environment reproducible. (*Default* `None`)
        """

        if seed is None:
            self.env = RustyEnvironment(board_size, walls, targets, robots)
        else:
            self.env = RustyEnvironment.new_seeded(
                board_size, walls, targets, robots, seed
            )

        self.action_space = spaces.Discrete(16)
        self.observation_space = spaces.Tuple(
            (
                spaces.MultiBinary([board_size, board_size]),  # right walls
                spaces.MultiBinary([board_size, board_size]),  # down walls
                spaces.Tuple(
                    (
                        spaces.MultiDiscrete([board_size, board_size]),  # red robot
                        spaces.MultiDiscrete([board_size, board_size]),  # blue robot
                        spaces.MultiDiscrete([board_size, board_size]),  # green robot
                        spaces.MultiDiscrete([board_size, board_size]),  # yellow robot
                    )
                ),
                spaces.MultiDiscrete([board_size, board_size]),  # target position
                spaces.Discrete(5),
            )
        )
        self.reward_range = (0, 1)

    def step(self, action: Action):
        return self.env.step(action) + (None,)

    def reset(self):
        return self.env.reset()

    def render(self):
        return self.env.render().replace("\\n", "\n")

    def get_state(self):
        return self.env.get_state()

    def board_size(self):
        return self.env.board_size
