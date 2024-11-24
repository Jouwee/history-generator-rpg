# Doing

## refactor - Better data sctructures for the universe

- ✓ Everything has IDs in a map
- Solve unnecessary iteration of dead people
- Solve mutability of people during simulation

# TODO: 0.0.1 - Basic history generation

# TODO: 0.0.2 - Basic world navigation

# Backlog

## feat - Basic settlement simulation

- Population
- Conflicts

## refactor - Load definitions from yaml/toml files

## refactor - Re-implement noise

Package-size, learning

## bug - Markov Chain doesn't work with order different than 3

## debt - Remove colorize

Too simple to be a dep

# Release 0.0.1

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
