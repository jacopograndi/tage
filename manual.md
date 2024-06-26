# Tage Gameplay Manual

The game is turn based.  
Every turn (also called day) you can move all of your units.  

Press `v` to open the blueprint panel. It will show information on the unit or building 
under your cursor on the map.

## Mechanics

### Research

You can pick one technology to research per day. You pay the cost this day and the effects will
be in effect at the start of the next.

### Age Up

Research is useful to get small bonuses, but it's necessary to Age Up.
Aging Up upgrades almost every unit and allows to build better buildings, train better units and 
have access to better technologies.
To Age Up, you need a Town Center to click on the button and at least:
- 3 techs if you are at the Dark Age
- 7 techs if you are at the Feudal Age
- 11 techs if you are at the Castle Age

### Resources

The resources you can get and use are food and gold.
Town Centers produces a little bit of both.

At the start of your turn, you gain food and gold based on constructed buildings.

### Building

Villagers can construct buildings:
- Mills have to be placed on food tiles, like this one `f--`, and produce 50 food
- Mines have to be placed on gold tiles, like this one `$))`, and produce 50 gold
- Farms have to be placed next to Mills
- Town Centers can be placed on most terrains
- Barracks, Stables and other town buildings can be placed near Town Centers

// wip, for more information play the Joan of Arc campaign on the nds game

## The Gameplay Interface

Once you started a game, you are presented with the map, a sidebar showing some details and a topbar.

You can undo a move with `u`.  
You can zoom in and out with `z` and `Z`.  
You can dim the map with `m`.  
You can toggle the details panel with `b`.  


## Menu Movement

You can press buttons with space.

You can move in the menu by using the wasd keys.
(rebindable by going to settings > press d to switch tab to keybinds > map your keybinds or press vim)

## Lobby Configuration

To set up a game, you have to:
- set up the players
- choose the map
- choose to spawn heroes or not
- choose the fog exploration level

### Player setup

Players control a civilization and start with a Villager and two other units.
Add players until you are satisfied, altough all maps support up to 4 players.

You can check how many players the map can spawn
by checking in the map picker how many spawn points there are.
The spawn points are the numbered tiles.

#### Team

You can choose the team that the player belongs to.

- Alone players are hostile towards everybody.
- Players in the same team are hostile towards everybody that isn't in the team.

#### Player and Symbol

The player name can be modified as well as their symbol. The symbol will be displayed on every
tile the player has a unit or building on. The symbol is just one character.

#### Controller

Pick local player if you want to control the player or someone else if you want to couch game.
- Local player (you, on the couch)
- Local player (couch friend, on the couch, will play controlling it's units)

### Fog Exploration

There are three options:
1. Visible: there is no fog of war, everything is always visible to everybody
2. Explored: the terrain is always visible, but you can see only what your units can see.
    Every unit has a sight value and all terrain tiles has a sight cost. You can see
    units if there is a path to them such that the sum of costs is less of the sight value of
    your unit.
3. Hidden: similar to explored, but the terrain is hidden until a unit sees it.
    After sight, the terrain remain revealed.
