# Doing

# TODO: 0.0.1 - Basic history generation

## feat - Basic settlement and faction simulation

- Factions
  - Name
  - Relations
    - Opinion
    - Towards
    - How does it change?
  - Leader: Person
  - Add one field per type in people
- Natural resource
  - Gold/year
- Region
  - Chance for resource
  - Soil fertility range
- Tile
  - Natural resources
  - Soil fertility
- Settlement Projects
  - Name
  - Cost in gold
  - Resource Buff
  - E.g:
    - Aqueduct (+fertility)
    - Customs House (+gold)
    - Wall (+defence)
- Settlement
  - Political faction
  - ✓ Population (growth based on soil fertility and population)
  - Gold (based on resources)
- Conflicts
  - Between factions
  - Chance for an attack between enemies every hit
    - Uses gold and population

## feat - Legends simulation

- Some random events that can happen, such as:
  - Survived a great battle
  - Slain a great beast
  - Slept with the whole town
  - Add titles based on feats
  - Generate artifacts based on feats

## feat - Trait simulation

- People can have personality traits that affect chances, such as:
  - Violent - Increase chance of starting conflict

# TODO: 0.0.2 - Basic world navigation

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

## debt - Remove colorize

Too simple to be a dep

## idea - History simulation of Technology

Start with no tech, chance of someone/some faction learning a tech in a tech tree, that gives buffs. Chance to exchange techs on trades.

# Release 0.0.1

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
