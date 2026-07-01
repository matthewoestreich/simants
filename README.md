# simants

Ant simulation in Rust with Raylib (and no AI)

<p align="center">
  <img width="32%" height="10%" alt="ants" src="https://github.com/user-attachments/assets/e81afa52-79bf-4088-b365-deffa2af544e" />
  <img width="32%" height="10%" alt="ants_pheromones" src="https://github.com/user-attachments/assets/ce8050ca-6129-40c6-8781-13c51d43f5e2" />
  <img width="32%" height="10%" alt="pheromones" src="https://github.com/user-attachments/assets/3bb47e96-7496-4f0d-8b23-489bc0d21813" />
</p>
<p align="center">
  <img width="32%" height="10%" alt="ants_pheromones_sensors" src="https://github.com/user-attachments/assets/3ec2abd9-b752-4d55-8e5c-39db90b29fcc" />
  <img width="32%" height="10%" alt="everything_rendered" src="https://github.com/user-attachments/assets/e8c53256-eeff-40ff-a15d-6c8525195400" />
  <img width="32%" height="10%" alt="over_time" src="https://github.com/user-attachments/assets/d13e7f15-ff0f-47e9-8fef-aa82aee48b76" />
</p>
<small>(all screenshots taken within 15 minutes of eachother)</small>

# Behavior

Ants do **not** follow a fixed or pre-programmed path. Instead, each ant makes decisions using a simple state machine.

At a high level:

- Each ant has three sensors (similar to antennae) that sample the environment a short distance in front of it.
- An ant searching for food follows the strongest **"to food"** pheromone it detects.
- An ant carrying food follows the strongest **"to colony"** pheromone it detects.
- If a searching ant detects food, it steers toward the sensor that detected it.
- If an ant carrying food detects the colony, it steers toward the sensor that detected it.
- If none of its sensors detect anything of interest, the ant wanders randomly.
- Ants have a limited pheromone supply, which is replenished after collecting food and after delivering food to the colony.
- An ant deposits pheromone only if the amount it intends to leave is greater than the pheromone already present at that location.
- An ants speed fluctuates randomly within a given range.
- Ants carrying food move slower than ants searching for food.
- An ant will "pause" randomly, per some probability.
- Pheromones evaporate over time using exponential decay.

Most of these environmental variables can be set in `src/settings.rs`.

# Color Scheme

### Ants

**Red ant:**

- An ant colored red is currently searching for food
- It's objective is to find food
- It seeks the "to food" pheromone
- It leaves the "to colony" pheromone on its trail

**Green ant:**

- An ant colored green has found food
- It's object is to return harvested food to the colony
- It seeks the "to colony" pheromone
- It leaves the "to food" pheromone on it's trail

### Pheromones

**Red pheromone:**

- Left behind by ants searching for food
- It's purpose is to communicate to ants that **have** found food how to find their way back to the colony
- It says "if you follow me, you'll find the colony"

**Green pheromone:**

- Left behind by ants that have found food
- It's meant to communicate to ants that **have not** found food how they can find food
- It says "if you follow me, you'll find food"

### Misc

- **Food** is colored yellow
- **Colony** is colored blue

# Hotkeys

- **Left mouse click:** If clicked on an ant, prints ant info to terminal.
- **Right mouse click:** If right click on map (even if an ant is right clicked) it prints info for the underlying cell to terminal.

| Key     | Action                                           |
| ------- | ------------------------------------------------ |
| `a`     | Toggles rendering ants                           |
| `s`     | Toggles rendering ant sensors                    |
| `p`     | Enters "pheromone mode" (denoted by cyan border) |
| `b`     | Toggles rendering border                         |
| `g`     | Toggles rendering grid                           |
| `SPACE` | Toggles paused state                             |

**Pheromone Mode**

While in "pheromone mode" there will be a dotted cyan border.

| Key | Action                                   |
| --- | ---------------------------------------- |
| `p` | Exit "pheromone mode"                    |
| `a` | Toggles rendering all pheromones         |
| `f` | Toggles rendering "to food" pheromones   |
| `h` | Toggles rendering "to colony" pheromones |
