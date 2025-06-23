Puzzle Definition Format

puzzles are defined using the KDL document format. each definition consists of a sequence of commands in an order.

there are 2 types of command. the first type is written in one line with no braces, e.g.:

```name "puzzle"```

the other type is written over multiple lines all enclosed in one brace. these commands are used to define several variables at once, for instance:

```
colors {  
	RED 255 0 0  
	GREEN 0 255 0  
}
```

in KDL, floats must be typed as '10.0' instead of '10'. the only part of definitions that currently uses floats is the position and radius of the circles in the 'circles' command.

strings can either be quoted (e.g. ' "9ABCD 789" ') or unquoted (e.g. ' puzzle '). unquoted strings are more limited, most notably not allowing whitespace or leading numbers. refer to kdl.dev for more information.

below is a list of the supported commands.

name: name the puzzle

example: 

```
name "666 Sphenic Triaxe"
```

author: add to the list of authors.

example: 

```
author "Henry Pickle"
```

note: when adding multiple authors, the author command can be used more than once.

circles { } : define the circles for further use in the definition. each circle is given a name, and then defined by the x and y coordinates of its center, and its radius. 
all three values are floats (don't forget to add a decimal, even if your floats happen to be integers!)

example:

```
circles {  
	A x=-0.5 y=0.0 r=1.0  
	B x=0.0 y=0.0 r=1.0  
	C x=0.5 y=0.0 r=1.0  
	SYM x=0.0 y=0.0 r=10.0  
}
```

note: this creates three small circles called A, B, and C, and a large circle called SYM.

a note on floats: currently, there is no way to specify numbers like sqrt(3) precisely. for now, specifying the float to 7-8 decimal points should work for most use cases. 
a more robust system for specifying these numbers is in development.

a note on scale: a circle of radius 1.0 is a reasonable size using the default rendering settings.

base : create the base of the puzzle from some circles. this will necessarily cut by every circle passed into it.

example:

```
base A B C
```

note: you will often want to define circles not used in the 'base' command for other uses in the definition, for instance, coloring.

twists { } : define the twists of the puzzle. each twist is given a name, and defined by a circle and a positive integer. the integer determines the angle of the turn, which is always (2PI / n) clockwise.
often, it is most convenient to give your turns the same names as the circles they use.

note: every twist defined will be automatically added to the puzzle (i.e., the user will be able to use the turn when solving the puzzle). 
if you want to exclude a twist from being usable, add a "!" as an extra argument after the integer.

example: 

```
twists {  
	A A 4  
	B B 4  
	C C 4  
	SYM SYM 2 !  
}
```

note: this makes 4 turns, one around each circle. the first three turn by PI/2 radians clockwise and the last by PI radians clockwise. the first three turns will be usable in the final puzzle, while SYM will not.

note: when using twists, the notation A2 refers to the twist done by performing the turn A twice, and the notation A' refers to the inverse twist of A (counterclockwise). A2' does the inverse of A twice. 
for this reason, avoid using numbers or "'" in your twist names.

compounds { } : define compound twists in terms of basic twists (defined using twists { }) and also other compound twists.

note: for a compound twist COMPOUND, the notation COMPOUND2 performs COMPOUND twice (it does not simply double the angle of each turn in COMPOUND) and similarly COMPOUND' performs the inverse of compound. 
as with twists { }, avoid using "'" or numbers in your names.

example:

```
compounds {  
	COMM_AB A B A' B'  
	COMM_BC B C B' C'  
	SUPER_COMM COMM_AB COMM_BC COMM_AB' COMM_BC'  
}
```

note: this defines 3 compound twists. the final compound twist is equivalent to A B A' B' B C B' C' B A B' A'.

cut : cuts the puzzle along a series of (compound) twists. 
essentially, you pass a sequence of (compound, or normal) twists into the cut command in sequence, and it performs these turns on the puzzle, cutting the puzzle wherever necessary to make these turns possible.

note: after performing the given turns, the cut command undoes the turns it was given. thus writing something like "cut R R' " is redundant, as "cut R" would achieve exactly the same effect.

example:

```
cut A B2 C' SUPER_COMM2'
```

* notation: you may add a * immediately after a NORMAL twist in a cutting sequence (compound twists with * are not supported and will not parse!) will essentially replace the twist with all possible multiples of itself, creating multiple cut commands.
* for example, if R is a 4-fold turn (PI/2 radians), then the command:

```
cut R* L
```

is equivalent to the series of commands:

```
cut L  
cut R L  
cut R2 L  
cut R3 L
```

twists with * don't have to be initial, and there can be multiple in one cut command (i would, however, advise against using too many, as the complexity of the command, of course, grows multiplicatively).

note: notation such as R2* and R'* is not supported. the latter would be redundant. 
in cases where the former would be useful, just define a new turn and use that in the cut (remember to exclude it from the actual end puzzle using "!", see above)

note: often, using additional turns to reduce the number of cut commands needed can make the definition more clean. 
for instance, the turn SYM defined above along with the circles defined above that allows each cut command to be applied with 2-fold rotational symmetry to the puzzle by beginning each cut command with SYM*. 
the convention for these whole-puzzle turns is to be of radius 10.0.

twist : turns the puzzle according to a (compound or normal) twist. this twist will not cut, and may be stopped if the puzzle is bandaged along the twist. 
unlike the twists from the cut command, these twists are not automatically undone. this is often useful as a setup to a series of cuts, or as a setup to a coloring.

example:

```
twist SUPER_COMM3'
```

note: * notation is not supported in twist commands.

undo : undoes some number of previous twist commands. undo is typically passed a positive integer, which is the number of twists to undo. "undo" alone is equivalent to "undo 1" and "undo *" undoes all twists.

note: each undo undoes the most recent twist COMMAND, not just one twist.

example: 
```
	twist SUPER_COMM3'  
	twist A B  
	cut COMM_AB  
	undo  
	cut COMM_BC  
	twist A B  
	undo 2  
	twist B A  
	undo *
```

colors { } : define your own colors using integer RGB values.

example:

```
colors {  
	RED 255 0 0  
	GREEN 0 255 0  
	BLUE 0 0 255  
}
```

note: there are many default colors that you can use without the colors { } command. the full list is:
RED, BLUE, GREEN, YELLOW, PURPLE, MAGENTA, CYAN, ORANGE, LIGHT_RED, LIGHT_BLUE, LIGHT_GREEN, LIGHT_GRAY, LIGHT_YELLOW, DARK_RED, DARK_GRAY, DARK_BLUE, DARK_GREEN, GOLD, WHITE, BLACK, KHAKI, and BROWN. 
defining colors with these names using colors { } will overwrite their value for this puzzle definition. the value of these colors is exactly the same as the color constants from the egui::Color32 module with the same names.

note: BLACK is the same color as the outlines of the pieces and GRAY is the same color as pieces that have not been colored.

color : colors a region a given color. the color is passed first, followed by a region. the region is specified by passing a series of circles, which can each also be negated using !. 
essentially, the region starts as all of 2d space, and each circle "A" added to the list intersects the current region with the circle A, while adding the circle "!A" to the list intersects the complement of A with the current region. 
for instance, "A !B C" refers to the pieces that fall within the circles A and C and outside of the circle B.

note: pieces that cross the border of any of the passed circles will not be colored. 
every piece is only one color -- multicolored pieces are not supported. as a patchwork solution, you can cut the piece into smaller pieces using a special turn to achieve a similar effect (this can be annoying). 
for an example, see the "666 Oriented Sphenic Triaxe" puzzle definition. multicolored pieces support are in development.

example:

```
color RED A !B  
twist SYM  
color BLUE A !B  
undo *
```

for examples of full puzzle definitions, see the puzzles included in the program on GitHub.

a few final notes on current convention:

when naming your puzzle, please start the name with the order of each turn, in descending order. for instance, the puzzle with the twists defined at the start would have a name that started with "444" (SYM is not an actual turn on the puzzle and so is excluded). 
the puzzle "Fluorine" is called "93 Fluorine".

the actual files for the puzzle definitions should be .kdl files, and the names should be of the format "666oriented_sphenic_triaxe.kdl" or such.

try to make your cuts efficient, in terms of runtime. cuts with 5 or more * twists in their definition will often take upwards of a minute to generate and should be avoided. there is almost always a more efficient solution.

commands should be run in roughly the order: name, author, circles { }, base, twists { }, compounds { }, cut, colors { }, color. twist and undo commands can be used wherever needed, after twists { } is used. 
if you use a different order, your definition may not parse, as many commands depend on other commands (i.e., twists { } needs circles { } to have already been parsed)

the commands that have { } in their format can be used multiple times, but should only be used once. there is no reason to use them more than once.

not all commands will always be used, but almost all puzzles will use name, author, circles { }, base, twists { }, cut, and color. the other commands may or may not be useful. technically, however, no commands are strictly necessary.

after making and generating your puzzle in the program, if reasonably possible, use the piece counter on the left to ensure that you didn't accidentally create any tiny pieces due to floats not being specified precisely enough.

if for some reason you want to specify a log file, just put "--LOG FILE" in a new line below the definition, followed by a new line, followed by a comma-separated list of twist names. 
only twists accessible in the puzzle (basic twists defined without ! in the twists { } command) are allowed, and numbers are also not allowed. for instance, it could look like:

```
[definition]  
--LOG FILE  
R,L,L,R,R',L,L,L',L,R,R',R,R,R
```

note the lack of whitespace and terminal comma. only twists actually performed in the program itself are added to the log file--twists done via the 'twist' command are not stored there.

definitions belong in Puzzles/Definitions/ and log files belong in Puzzles/Logs/

please make sure to use the name and author commands in every puzzle definition for organization reasons.

if you make a good puzzle, make a pull request on GitHub and i'll add your puzzle at my earliest convenience if i like it.

if you're having any issues with the puzzle definition format, or you have suggestions on how to improve it, please contact me at:

discord: henryduke

email: henrydukepickle@gmail.com

i'm more likely to respond to you more quickly on discord. dm-ing me without asking is totally fine.
