# Doing

# TODO: 0.0.2 - Basic world navigation

## feat - Settlement sprites

- Different sprites for different sizes

## feat - Render world while generating

## feat - Cursor for viewing details of a tile

## feat - Key for opening the Gospel

## debt - Remove colorize

Too simple to be a dep

# TODO: 0.0.3 - Basic chunk generation

# TODO: 0.0.4 - Basic gameplay

# Backlog

## refactor - Load definitions from yaml/toml files

## refactor - Re-implement noise

Package-size, learning

## refactor - Person builder

Generate person has too many parameters. Create a builder.

## bug - Markov Chain doesn't work with order different than 3

## bug - Settlements can be placed too close to one another

## idea - History simulation of Technology

Start with no tech, chance of someone/some faction learning a tech in a tech tree, that gives buffs. Chance to exchange techs on trades.

## idea - Legends simulation

- Some random events that can happen, such as:
  - Survived a great battle
  - Slain a great beast
  - Slept with the whole town
  - Add titles based on feats
  - Generate artifacts based on feats

## idea - Trait simulation

- People can have personality traits that affect chances, such as:
  - Violent - Increase chance of starting conflict

# Release 0.0.2 - Basic world rendering and navigation

## feat - Render world using Piston

- ✓ Render terrain
- ✓ Render settlements
- ✓ Render settlement factions

# Release 0.0.1 - Basic history generation

## feat - Basic settlement and faction simulation

- ✓ Factions
  - ✓ Name
  - ✓ Relations
    - ✓ Opinion
    - ✓ Towards
    - How does it change?
  - ✓ Leader: Person
- ✓ Region
  - Chance for resource
  - ✓ Soil fertility range
- ✓ Tile
  - ✓ Gold/year
  - ✓ Soil fertility
- ✓ Settlement
  - ✓ Political faction
  - ✓ Population (growth based on soil fertility and population)
  - ✓ Gold (based on resources)
- ✓ Conflicts
  - ✓ Between factions
  - ✓ Chance for an attack between enemies every year
    - ✓ Uses gold and population

## refactor - Better seed derivation

For now I take the seed and sum something. Create a derive method that takes any hashable and creates a new Rand

## refactor - Better data sctructures for the universe

- ✓ Everything has IDs in a map
- ✓ Solve unnecessary iteration of dead people
- ✓ Solve mutability of people during simulation

## feat - Better family trees

- ✓ Record who is next-of-kin of who
- ✓ Better settlement inheritance
- ✓ Family surnames
- ✓ Fertility curve
- ✓ Simulation importance level

## feat - Better name generation

- ✓ Implement actual Markov Chains
- ✓ Train model only once
- ✓ First vs Last names
- ✓ Min/max length
- ✓ Markov Chain order
- ✓ Male vs Female names
- ✓ Capitalization
- ✓ Increase training-set size

## feat - Simple world map generation

- ✓ Simple noise elevation
- ✓ Simple noise temperature
- ✓ Map Regions based on those two criteria
- ✓ Rename Village -> Settlement
- ✓ On settlement creation, place it on suitable random area and link region accordingly
- ✓ Simple ASCII map rendering every year
