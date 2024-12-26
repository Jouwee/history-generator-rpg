# Doing

## feat - Review worldgen scene

- ✓ Show year, enter to interrupt
- ✓ New biome - Grassland
- ✓ Render as Tilemap
- ✓ Pixel art Scale
- ✓ Settlements rendering
- ✓ Size vs window

# TODO - 0.1.0 - Final polish

## refactor - Load definitions from yaml/toml files

- stuff like materials and biomes need a "string-key" to be referenced by other configs

## refactor - Re-implement noise

Package-size, learning

## bug - Names like Anea00000000000

## feat - Review world scene

- Save current position
- Better cursor
- Scale up pixel art
- Current tile info

## feat - Review game scene

## feat - Death

- Save current position
- Better cursor
- Scale up pixel art
- Current tile info

## bug - History generation is not deterministic

## feat - More interesting worldgen

## feat - Main menu Scene

- Option to start new game
- Option to quit game
- Start new game dialog (options)
  - Seed
  - Great beast frequency

# TODO - 0.?.? - Main menu, Saving and loading

## feat - Saving and loading

# Backlog

## refactor - Actor composition system

## bug - Markov Chain doesn't work with order different than 3

## feat - Switch between turn-based and realtime

a turn = 2 seconds. Realtime is just doing turns with delays based on action points, and the player moves out of turn.

## bug - Cities go 100's of years without someone rising to power

## bug - A spouse of a king has the same chance of founding a new city than anyone else

## refactor - You need to pass too many parameters to read an asset

Like the size of the spritesheet, not part of the key

## feat - Scale up pixel art

Not trivial to sync input, layout & graphics

## bug - Weird battle results

"In 61, Axial Peregrins attacked Green Axis at Borchpør, and emerged vitorious.
In the end, the attackers suffered no casualties. While the defenders suffered no casualties."

## refactor - Better spatial searches

The majority of the current time spent in history generation is searching who is in a town to simulate battles

## feat - Interactions - Ask about

- Option to ask people about something
  - Dialog option "Ask about..." - Select something
  - Check rumors, and chance to know about events of "relatives" (same city, family, etc)
- Rumors and knowledge must be deterministic, hopefully not needing to store it.

# Release - 0.0.9 - More interesting world generation

## feat - Object layer in local map

- ✓ Different layer above ground tiles
  - ✓ Walls will generate in this layer
  - ✓ Tile rendering
  - ✓ Sprites can exceed the 16x16 grid
  - ✓ Z-Ordering with actors
  - ✓ Blocks movement
  - ✓ Blocks spawning

## feat - Better wilderness generation

- ✓ Vegetation tile metric
- ✓ Trees based on vegetation

## feat - Better settlement generation

- ✓ More buildings, based on settlement size
- ✓ Open space for doors
- ✓ Some furniture
  - ✓ Beds
  - ✓ Tables
  - ✓ Chairs
- ✓ Trees and tall grass

## feat - Layered dual-grid terrain

- ✓ Layered dual grid system
- ✓ New sprites
  - ✓ Grass
  - ✓ Dirt
  - ✓ Stone
  - ✓ Sand

## feat - Beds allow you to rest

- Beds allow you to rest

# Release 0.0.8 - All about attributes

## feat - New attributes

- ✓ STR
  - ✓ Affect damage of mellee weapons
- ✓ AGI
  - ✓ Increases AP
- ✓ CON
  - ✓ Increases HP

## feat - Level up

- ✓ On level up, you can choose to increase 1 attribute

# Release 0.0.7 - Inventory and weapons

## feat - Basic Inventory

- ✓ Player will have a inventory structure to hold items
- ✓ Inventory dialog
  - ✓ Grid of items
  - ✓ Switch equiped item
- ✓ Attack with equipped
  - ✓ Compute item stats
  - ✓ Adapt attack action to use equipped item
  - ✓ Unequipped must do a unarmed attack

## feat - Obtaining loot

- ✓ Items on the ground
  - ✓ Data structure in chunk
  - ✓ Render
    - ✓ Sprite based on material
  - ✓ Pick-up action to add to inventory
- ✓ When the player enters an abandoned settlement tile, spawn giant spiders
  - ✓ Also spawns an item on the ground to be picked up

## feat - Artifact loot

- ✓ If you kill someone wielding an artifact, they drop it
- ✓ Other actors can use their posessions

# Release 0.0.6 - Great beasts and artifacts

## feat - Great beasts

- ✓ People must be of several species, and allow for "wild" (uncivilised) people
- ✓ Every year, a unnocupied tile can spawn a great beast
- ✓ The great beast can be of a few types: Fiend, Leshen and have a level

## feat - New event - Fought a great beast

- ✓ Great beasts will hunt in a territory, maybe hitting towns and lowering their reputation
  - ✓ Random hunt chance
  - ✓ Battle resolution
  - ✓ Battle effects

## refactor - Improve way of handling simulation

- ✓ People
  - ✓ Create an enum of things that can happen
  - ✓ Later, apply the action
  - ✓ Apply to every action

## refactor - Apply new battle resolution to conflicts in wars

## feat - FPS counter

- ✓ F3 to open debugging overlay
- ✓ Show FPS and TPS at debugger overlay
- ✓ Limit FPS to 30
- ✓ Limit TPS to 30

## feat - Artifact making

- ✓ Materials and items
- ✓ Creature drops
- ✓ Upon defeating a great beast, an artifact could be created
- ✓ Track who killed who
- ✓ The artifact is now in possession of whoever killed the beast
- ✓ The artifact can be inherited

## bug - Immortal non-remarkable people

## feat - Spawn great beasts on local map

- ✓ Show them in the map as covens
- ✓ Sprite - Fiend
- ✓ Sprite - Leshen
- ✓ Hostility and sprite based on species
- ✓ Damage and health based on attributes

## feat - Interactions - Rumors

- ✓ Important events might become rumors
  - ✓ Important events include important people
  - ✓ Do I need a struct? From the event I can compute rumor radius and strenght
- ✓ When you talk to people, you can ask what rumor they've heard recently
  - ✓ Interacting with someone will open a dialog
  - ✓ Option for asking if they heard any rumors
  - ✓ Search backwards for a rumor

## feat - Knowledge codex

Creation of a "knowledge codex" - stuff you know and can ask about

- ✓ Data structure
  - ✓ People & Creatures
  - ✓ Places
  - ✓ Artifacts
- ✓ Add information to codex
  - ✓ When talking to someone
  - ✓ When hearing rumors
  - ✓ When visiting a place
- ✓ Dialog
  - ✓ People & Creatures
  - ✓ Places
  - ✓ Artifacts
  - ✓ V-List
  - ✓ Clear V-List

## Fix button UI

- ✓ Background
- ✓ Size

## Fix dialog UI

- ✓ Background

# Release 0.0.5 - Aimless improvements

## refactor - Structure defining actions

## refactor - Turn world map into a Scene

## feat - Persist character through scenes

# Release 0.0.4 - Connect global and local world

## feat - Place important people on map

- ✓ Check who's on the tile
- ✓ Add NPCs linked to said person
- ✓ When using the talk action, present theirselves
- ✓ If you kill the NPC, the world person must die

## feat - Generate local chunk following biome

# Release 0.0.3 - Basic gameplay

## feat - Basic Gameplay

- Swap between world map and local chunk
- Player character
- Turns
- Attack action

## feat - Basic enemy AI

## feat - Ability to talk to people

## feat - Character sprites

## refactor - Asset manager

# Release 0.0.2 - Basic world rendering and navigation

## feat - Render world using Piston

- ✓ Render terrain
- ✓ Render settlements
- ✓ Render settlement factions

## debt - Remove colorize

## feat - Cursor for viewing details of a tile

- ✓ Cursor (x,y)
- ✓ Panel on the right for inspecting
- ✓ Text rendering
- ✓ Description generator

## feat - Settlement sprites

- Different sprites for different sizes

## feat - Render world while generating

## refactor - Better ID map structure

- Use vectors
- Id type that is already validated - remove need for unwraps
- Each struct should not have an Id
- "Make invalid states unrepresentable"

## refactor - Better event structure

- Create struct for every event
- Event enum has only 2 parameters - Date and event
- Indexing by person, settlement and faction (faster searches)

## bug - Settlements can be placed too close to one another (even on top)

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
