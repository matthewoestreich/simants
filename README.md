# simants

Ant simulation in Rust with Raylib

# Behavior

At a high level:

- Ants have 3 sensors, which act like their antennae, and sample the environment at some distance in front of them
- An ant that has food will sample it's environment for the strongest scent of the "to colony" pheromone, and follow it
- An ant that is searching for food will sample it's environment for the strongest scent of the "to food" pheromone, and follow it
- If an ant senses food, and is searching for food, it will steer towards it
- If an ant senses the colony, and is carrying food, it will steer towards it
- If an ant has not sensed anything, it wanders randomly
- Ants have a limited amount of pheromones, which get topped up after finding food and after delivering food
- An ant will only drop it's pheromone if the amount it plans to drop is greater than the existing amount
- Pheromones will evaporate in the environment exponentially

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

| Key | Action                        |
| --- | ----------------------------- |
| `a` | Toggles rendering ants        |
| `s` | Toggles rendering ant sensors |
| `p` | Toggles rendering pheromones  |
| `b` | Toggles rendering border      |
| `g` | Toggles rendering grid        |
