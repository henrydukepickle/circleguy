# Overview

general puzzle simulator for circle puzzles. currently supports custom puzzle definitions and keybinds.

# Keybinds

keybinds are configured in the Configs/keybinds.kdl file.

currently, the `Z` key is reserved for undo. using the `Z` key in your own keybind set is not recommended.

in keybinds.kdl there are 2 relevant kinds of blocks, `binds` and `override`. there is only one `binds` block but there can be any number of `override` blocks.

## Binds

the `binds` block sets default binds that work across all puzzles. the `binds` block consists of a series of three-argument commands. the first argument is the key you want to bind, the second one is the identity of the turn you want to bind it to, and the final argument is the multiple of the turn you want to bind. for instance,

`j L 2`

will do the turn `L2` when you press the `j` key.

## Override

`override` blocks function almost identically to the `binds` block, except that they define puzzle-specific keybinds. as such they take an extra argument at the top of the block, i.e., a block might look like:

```override Stars { ... }```

the body of the block is the same format as the body of the `binds` block.

# Puzzle Definition Format

puzzle definitions are written in the hyperpuzzlescript (hps) language. for broad documentation about hps, see [the hps docs](https://github.com/HactarCE/Hyperspeedcube/tree/main/crates/hyperpuzzlescript#learn-hyperpuzzlescript-in-y-minutes). note that the sections in those docs regarding euclidian geometry are not relevant and will not parse in circleguy `hps` files. this section will document the types and functions unique to circleguy.

## Types

`Point`: represents a point in 2d space. has `x` and `y` fields.

`Vector`: represents a 2d vector. has `x` and `y` fields.

`Circle`: represents a (nondegenerate) circle in 2d space. circles are stored with orientation (in or out). has `c` and `r` fields.

`Turn`: represents a turn, specified by a turn region (a circle) and an angle, which must be a rational multiple of `pi`. has `circ` and `order` fields.

`Color`: a color. as of right now there are a number of builtin colors, with no way to construct your own.

## Functions

### Constructors

`point(Num, Num) -> Point`: constructs a point from x and y coordinates.

`vector(Num, Num) -> Vector`: construcs a vector from x and y coordinates.

`circle(Num, Num, Num) -> Circle`: constructs a circle from its center's x and y coordinates, and its radius, respectively.

`circle(Point, Num) -> Circle`: constructs a circle from its center and radius.

note: the circle constructors always construct a circle with a positive orientation, i.e., a circle that contains its center.

`turn(Circle, Num) -> Turn`: construcs a turn from a circle and an order. the second argument is not the turn's angle, but its order; the angle of the turn will be `2pi/order`. the second argument should be an integer.

### Utilities

`rotate(Point, Point, Num) -> Point`: rotates the first point around the second point, according to the angle.

`rotate(Circle, Point, Num) -> Circle`: rotates the circle around the point, according to the angle.

`mag(Vector) -> Num`: the magnitude of a vector. 

`normalize(Vector) -> Vector`: normalize a vector. throws an error if passed a zero vector.

`mult(Turn, Num) -> Turn`: multiplies a turn by an integer. negative integers are accepted.

`inverse(Turn, Num) -> Turn`: inverts a turn.

`mult([Turn], Num) -> [Turn]`: multiplies a list of turns, corresponding to repeated concatenation. negative integers may be passed, and will first invert the sequence before multiplying.

`inverse([Turn], Num) -> [Turn]`: inverts a list of turns, reversing its order and inverting every turn.

`powers(Turn) -> [Turn]`: gives a list of all the powers of a turn, up to its order, and including 0. for instance, if `t.order = 3`, then `powers(t) = [0, t, t2]`. used for cutting symmetrically.

#### Arithmetic Operations

`+`: you can add a vector to a point, or a vector to a vector.

`-`: you can subtract a point from a point (yielding a vector), or subtract a vector from a vector.

`~`: you can negate a vector, or negate a circle, flipping its orientation.

`*`: you can multiply a vector by a scalar (a `Num`).

### Puzzle Construction Functions

the main command behind the puzzle definitions is `add_puzzle`, which takes `3` mandatory keyword arguments and `1` optional one. these arguments are:

`name: String`: the name of the puzzle. two puzzles cannot have the same name.

`authors: [String]`: the authors of the puzzle.

`scramble: Num` (optional): the scramble depth of the puzzle. if not specified, defaults to `500`.

`build: Fn () -> ()`: the function for building the puzzle. can be specified anonymously like `build = fn () { ... }`.

the `build` function does not take or return any arguments. instead, the puzzle is modified using puzzle construction functions inside the `build` function, which are specified below.

#### Functions

`add_circles([Circle])`: adds circles to the base of the puzzle. circles are added sequentially, and each circle added is cut by all previous circles added to ensure that the pieces do not overlap. this is exactly like the `base` command in the old definition format, except that multiple `add_circles` commands can be present.

`add_turns([Turn], [String])`: adds turns to the puzzle, using the names in the second argument. see the `Turn Naming Conventions` section below.

`add_circle(Circle)`: single circle version of the above function.

`add_turn(Turn, String)`: single turn version of the above function.

`cut([Turn])`: executes the turn sequence, cutting as it turns. undoes the turns afterwards.

`cut([Circle], [Turn])`: applies the `cut([Turn])` command, but only to the pieces in the specified region. all pieces not initially in the region will be excluded from the cuts.

`turn([Turn])`: turns the puzzle by the turn sequence, adding the turns to the stack. does not undo afterwards. still cuts as it executes the sequence.

`turn(Turn)`: single turn version of the above function.

`undo()`: undoes one turn from the `turn` commands.

`undo(Num)`: undoes a number of turns from the `turn` commands.

`undo_all()`: undoes all the turns from `turn` commands.

`color([Circle], Color)`: colors the region.

### A Note on Colors

the rgb values the builtin colors correspond to are fixed right now, but will be customizable in the future. the color constants in the program are exactly the constants in `egui::Color32`. their names are exactly the same, except lowercase.

the more obscure colors, specifically the `light` and `dark` versions of colors, should not be used except when the normal version is already used, i.e., don't use `light_red` unless `red` is already taken and you need a distinct color.

### Naming Conventions

this section is currently incomplete. good conventions are suggested, both to make log files more consistent, and to make keybinds work across as many similar puzzles as possible.

some basic, sensible conventions are:

- turn names should be 1-3 capital latin letters. fewer is better.

- two-circle puzzles should have two turns, `L` and `R`.

- when possible, try to build turn names from `U, D, L, R, M` for `up, down, left, right, middle`. for instance, a puzzle with 4 circles in a square would have turns `UL, UR, DL, DR`.

- when combining the above letters, `M` comes before `U, D`, which come before `L, R`. opposite letters should not be combined (`LR` is invalid).

# Final Notes

definitions should have the `hps` extension and should be put in `Puzzles/Definitions/` and log files belong in `Puzzles/Logs/`.

puzzle files (not the names specified in those files) should be lowercase with underscores, and should correspond to the puzzle names as closely as possible.

puzzles should have sensible names. they can be named, for instance, after shapes or patterns in the puzzle, or some puzzle-theoretic property they have, or something else reasonable. for instance, many puzzles are named after flowers they resemble.

if you make a good puzzle, make a pull request on GitHub and i'll add your puzzle at my earliest convenience if i like it.

if you're having any issues with the puzzle definitions, or you have suggestions on how to improve it, please contact me at:

discord: henryduke

email: henrydukepickle@gmail.com

i'm more likely to respond to you more quickly on discord. dm-ing me without asking is totally fine.
