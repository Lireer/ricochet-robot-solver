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
        observation="tensor",
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
        observation: str
            Decides how an observation is shaped.
            - "vector": The observation is an array. The first
                        `board_size**2` values mark fields with walls on their
                        right side. The next `board_size**2` values mark fields
                        with walls below them. Followed by the robot positions
                        in order red, blue, green, yellow as `(column, row)`
                        tuples. The next two values are the position of the
                        target and the final five values are the one hot encoded
                        target type/color.
            - "tensor": The observation is a tensor of the shape (board_size,
                        board_size, 11). These layers are right walls, down
                        walls, red robot, blue robot, green robot, yellow robot,
                        red target, blue target, green target, and yellow
                        target. (*Default*)
        """

        if seed is None:
            self.env = RustyEnvironment(board_size, walls, targets, robots)
        else:
            self.env = RustyEnvironment.new_seeded(
                board_size, walls, targets, robots, seed
            )

        self.action_space = spaces.Discrete(16)
        if observation == "vector":
            # right walls, down walls, 4 robot positions, 1 target position,
            # and 5 one hot encoded target types
            values = 2 * (board_size ** 2) + 8 + 2 + 5
            low_bounds = np.zeros(values)
            high_bounds = np.concatenate(
                [
                    np.ones(2 * (board_size ** 2)),
                    np.full(8 + 2, board_size - 1),
                    np.ones(5),
                ]
            )
            self.observation_space = spaces.Box(
                low_bounds, high_bounds, (values,), np.int16
            )
        elif observation == "tensor":
            self.observation_space = spaces.Box(
                0, 1, (board_size, board_size, 11), np.int16
            )
        else:
            raise ValueError(
                'observation style {} is not supported, use "vector" or "tensor"'.format(
                    self.observation
                )
            )
        self.reward_range = (0, 1)
        self.observation = observation

    def step(self, action: Action):
        obs, reward, done = self.env.step(action)
        return (self._fit_observation(obs), reward, done, {})

    def reset(self):
        return self._fit_observation(self.env.reset())

    def render(self):
        return self.env.render().replace("\\n", "\n")

    def get_state(self):
        return self._fit_observation(self.env.get_state())

    def board_size(self):
        return self.env.board_size

    def _fit_observation(self, rust_obs):
        right_walls, down_walls, robots, target_pos, target = rust_obs
        right_walls = np.array(right_walls, dtype=int)
        down_walls = np.array(down_walls, dtype=int)
        if self.observation == "vector":
            # One hot encode the target type
            target_one_hot = np.zeros(5)
            target_one_hot[target] = 1
            return np.concatenate(
                [
                    right_walls.flatten(),
                    down_walls.flatten(),
                    np.array(robots).flatten(),
                    target_pos,
                    target_one_hot,
                ]
            )
        elif self.observation == "tensor":
            robot_boards = np.zeros((4, *right_walls.shape))
            for (i, (col, row)) in enumerate(robots):
                robot_boards[i, row, col] = 1

            target_boards = np.zeros((5, *right_walls.shape))
            target_boards[target, target_pos[1], target_pos[0]] = 1

            return np.dstack([right_walls, down_walls, *robot_boards, *target_boards])
        else:
            raise ValueError(
                'observation style {} is not supported, use "vector" or "tensor"'.format(
                    self.observation
                )
            )
