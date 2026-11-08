# Manual playtest checklist

These are the final human checks. Automated tests cover rules and entity wiring;
they cannot prove that the screen looks right or that pointer input reaches the
buttons on this machine.

## Launch

```sh
cargo run
```

- [ ] The window opens at 1280 × 800 with title "Dungeon escape".
- [ ] The terminal shows no missing-asset errors or panics.
- [ ] Closing the window exits normally.

## Dungeon and controls

- [ ] The dungeon, floor/wall tiles, player, two enemy types, boss, and door
      render correctly with crisp pixel art.
- [ ] The player starts in the lower-left room.
- [ ] `WASD` and arrow keys move exactly one tile per fresh press.
- [ ] Holding a movement key does not continuously slide the player.
- [ ] Space waits without moving.
- [ ] Walking into a wall or the map border is blocked and does not advance the
      turn counter.
- [ ] The HUD shows numeric HP, a health bar, level, XP progress, attack, and
      heal-on-kill.
- [ ] Damage, blocked movement, waits, kills, XP gains, and level-ups produce
      readable feedback text.

## Combat and enemies

- [ ] Walking into a skeleton or brute attacks it instead of overlapping it.
- [ ] Enemies take one action after each valid player turn, not several.
- [ ] The skeleton pursues and attacks when adjacent.
- [ ] The brute visibly acts every other player turn.
- [ ] Defeated enemies disappear, stop blocking tiles, and award XP only once.
- [ ] The player can lose all HP and reaches the loss screen.
- [ ] After death, dungeon actions stop.

## Level-up

- [ ] Crossing an XP threshold opens a panel with exactly three distinct cards.
- [ ] The dungeon is frozen while the panel is open.
- [ ] Each card states the actual current-to-next numerical effect.
- [ ] Keys `1`, `2`, and `3` select the corresponding card.
- [ ] Mouse clicks on all three cards work, including hover feedback.
- [ ] Choosing applies one upgrade, does not advance the turn, and returns to
      play (or shows the next queued selection if two levels were earned).
- [ ] Vitality raises max HP and heals; Strength raises damage; Recovery raises
      the heal after a kill.

## Boss and exit

- [ ] The boss is visually distinct, has a visible health bar, and blocks the
      exit alcove.
- [ ] The boss warning is visible (red text and red marked tile) before the
      player commits the next action.
- [ ] Standing on the marked tile when the heavy attack resolves deals damage.
- [ ] Moving off the marked tile avoids the hit.
- [ ] The boss recovers for one turn after a heavy attack; the HUD says so.
- [ ] The exit stays visually closed and refuses entry until the boss dies.
- [ ] After the boss dies, the door opens and entering it shows VICTORY.
- [ ] The victory screen shows the result and a restart action.

## Restart

- [ ] The restart button works after both victory and defeat.
- [ ] R also restarts after both results.
- [ ] Restart resets HP, XP, level, upgrades, enemies, boss, exit lock, turn
      count, feedback, and any pending level-up choices.
- [ ] Restart does not duplicate the camera, HUD, map, actors, or panels.
- [ ] A second complete run can be played immediately.

## Visual balance observations

- [ ] Text is readable and has enough contrast against the panels.
- [ ] The map, HUD, upgrade panel, and end screen fit in the window without
      clipped text or overlapping controls.
- [ ] The run feels short enough to finish in one sitting.
- [ ] The boss warning is understandable without reading the source.

If a check fails, note the key/tile/action that triggered it and keep the
terminal log. Do not record a visual pass without an actual playtest.
