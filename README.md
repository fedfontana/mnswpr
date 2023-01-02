# Minesweeper

My personal implementation of a terminal minesweeper game.

## Controls

You can move the cursor (`[]`) both by using arrows or `wasd`. 

Uncover the cell under the cursor by pressing the spacebar, or flag (or un-flag it) by pressing `f`.

Press `q` at any moment to quit.

## CLI options

The field can be customized via CLI flags:
- `-w` or `--width` controls the width of the field
- `--height` controls the height of the field
- `-m` or `--mine-percentage` controls the % of mines in the field
  
If you don't want to specify the dimensions, you can use the `-p` or `--preset` flags and provide one of the provided presets:
- `tiny`: 20x13 field
- `small`: 30x20 field
- `medium`: 40x25 field
- `large`: 50x30 field
- `huge`: 60x40 field

Note that the sizes the field will always be constrained by the size of the terminal. As such, width and height will be clamped between 1 and you terminal's width/height minus some padding

## Screenshots

![example game](imgs/Screenshot%202023-01-02%20at%2017.42.44.png)