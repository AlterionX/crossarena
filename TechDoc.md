What core technology are you implementing?
How does this core technology relate to your game play?
What other systems will you need to implement to support this?
What game engine will you be working in?

# Overall design

A simple top down 2d fighting game against waves of AI enemies.

## Core technology

Focus is on the `Advanced Software Systems` section. There are two systems in play:
- Health/Combat
- Materials/Crafting/Consumables (if time allows)

These two interact with each other in more or less obvious ways as detailed below.

## Gameplay

Combat consists of three components:
- a triple dash (Direction can be changed between two dashes.) (i-frames if time allows)
- a simple melee attack combo
- a simple projectile w/ aiming time
- a charged projectile (ricochets) (charges after simple projectile finishes aiming)

The focus is on avoiding attacks and managing the health bar. Health is a discrete value.

Waves get progressively harder and enemies will drop materials that are picked up automatically.
Lower tiered materials can be refined to higher tiered versions. This data is to be loaded in
dynamically. The player is also healed on wave conclusion.

However, the main use is for crafting into equipment to upgrade the character. For now, this just
bumps up raw stats (health, damage, etc).If time allows, allow this to also change the core combat,
i.e. add a dash, create a longer melee attack combo, etc.

If time allows, implement crafting and quick usage of consumables.

Game will continue until "death" (when hp reaches 0). Potentially autosave between waves or save
replays should time allow.

## Game engine

Godot will be used. Godot's GDNative interface will be used via godot-rust.

# Software architecture

## Implementation

### Combat

Combat will be implemented with the use of packed scenes representing specific attacks.

For melee attacks, these will be loaded, added to the scene, and have collision run for the objects
once using collision masks. This will trigger damage for enemies and player alike.

Projectiles will also be loaded, and treated as instantaneous attacks unless they hit nothing. If
they don't impact on initialization, they are now treated as just any other entity within the scene
and will have termination logic embedded within themselves. (An example might be to delete self if
impacts with enemy or something similar.)

### Crafting

All enemies have an associated drop rate for a specific kind of material. This will be loaded in at
start time as a singleton and can be looked up by various scripts.

A node that allows for player interaction (via a mouse click if the player is close enough) will be
created and placed within the arena. This is enabled on wave end, and disabled on wave start.
Interacting with this will bring up a crafting menu. This will be generated from some sort of csv.

An equipment management screen will be created as well to allow for equipping different items. This
will be accessed through a pause menu of some sort.

## Todos

As this is an entirely new game, everything other than the physics and rendering needs to be built
out.

- [x] Simple arena
- [x] Simple player
- [x] WASD movement
- [x] Dashing
- [ ] Enemy data
- [x] General health management
- [x] Enemy spawn
- [x] Health UI
- [x] Attack scenes (aka hitbox frames) + naming scheme
- [x] Small melee combo
- [x] Projectiles
- [x] Indicators for when charged shot is ready
- [ ] Charged projectiles (ricochet)
- [x] Damage

Along with the more "stretch" goals.

- [ ] Item data
- [ ] Inventory UI
- [ ] Drops
- [ ] Crafting data
- [ ] Crafting UI
- [ ] Crafting
- [ ] Equipment data
- [ ] Equipment UI
- [ ] Equipment

# Division of Labor

None, as I am working alone.

