# Apilar Language

The Apilar language is a stack-based assembly language. Each instruction can
read from the stack or push to the stack. Instructions can also affect
read/write heads which can be used to read and write memory, and to control
jumps. Some operations have effects on the computer they run in, or on the
world itself.

Instructions exist in memory of bytes. Not all bytes are used for instructions;
those unused are also interpreted as NOOP.

Each processor at present has a hard-coded maximum stack size of length 64; each
value on the stack is an unsigned 64 bit number.

## Basics

Apilar implements [stack machine](https://en.wikipedia.org/wiki/Stack_machine).
Let's see how that works. We start with an empty stack:

```
[]
```

If you place a number on the stack with the `N1` instruction, the stack
contains the number `1`:

```
[1]
```

Let's look at the program:

```
N1
N2
ADD
```

Let's run it, one by one:

First `N1`:

```
[1]
```

Then `N2`:

```
[1 2]
```

Then `ADD`. This pops the two top items from the stack and adds them together,
putting the result back on the stack:

```
[3]
```

You can write longer programs entirely composed of words; nothing else is
needed. Here we do `(1 + 2) * 3`:

```
1
2
ADD
3
MUL
```

## Stack maneuvers

Apilar has operations like `DUP`, `ROT` which are used to manipulate the stack
itself. It adopts these from the [Forth programming
language](<https://en.wikipedia.org/wiki/Forth_(programming_language)>), so
information about Forth is a good place to look for more details. For instance,
see the section "Stack Maneuvers" in this [Forth
manual](https://www.forth.com/starting-forth/2-stack-manipulation-operators-arithmetic/).

## Read/write heads

Besides the stack, each processor also has a number of read/write heads. These
are like pointers into memory. The `HEAD` instruction can be used to change the
current head. `READ` reads at the position of the current head. `WRITE` writes
from the stack into memory at the current head position. There are also
`BACKWARD` and `FORWARD` instructions to move the head up and down memory. The
`JMP` and `JMPIF` instructions also jump to the current head.

## Never crash

In an evolutionary simulation it's important all operations succeed; it
should never result in a crash. So, division by `0` is allowed, and results
in `0`.

When a stack overflow occurs, the stack is compacted - the bottom half of the
stack is thrown away and the top half of the stack is the new stack.

What happens with an instruction if it tries to take a value from the stack and
nothing is there depends on the instruction and is described there.

## Instructions

### NOOP - do nothing; no operation

Do absolutely nothing. The one official NOOP, besides those of unrecognized
bytes.

### Number operations

#### N0 ... N8 - number literals

The instructions N0, N2, ... N8 place the respective number on the stack.
So, N1 puts 1 on the stack, N2 puts 2 on the stack, etc.

#### RND - random byte

Places a random number (under 256) on the stack.

### Stack maneuvers

#### DUP - duplicate top

Duplicate the top of the stack. So, if you have a stack:

```
[1]
```

then executing `DUP` results in this stack:

```
[1 1]
```

Stack underflow means no op.

#### DUP2 - duplicate top 2 value

Like `DUP` but for the two top values on the stack:

```
[1 2]
```

becomes:

```
[1 2 1 2]
```

Stack underflow, meaning less than 2 values on the stack, means no op.

#### DROP - drop top of stack

Drops the top of the stack. It disappears forever:

```
[1 2]
```

becomes:

```
[1]
```

Stack underflow means no op.

#### SWAP - swaps two top values on the stack

Swaps the two top values on the stack:

```
[1 2]
```

where `2` is on top, it becomes:

```
[2 1]
```

where `1` is now on top.

Stack underflow, meaning less than 2 values on the stack, means no op.

#### OVER - duplicates second item to top

Duplicates the second value just below the top.

Given:

```
[1 2]
```

it results in:

```
[1 2 1]
```

Stack underflow, meaning less than 2 values on the stack, means no op.

#### ROT - rotates the top 3 values on the stack

Takes the third value and puts it on top instead:

```
[1 2 3]
```

becomes:

```
[2 3 1]
```

So this pulled the `1` and put it on top.

Stack underflow, meaning less than 3 values on the stack, means no op.

### Arithmetic

All the operations are wrapping so if there's an number overflow or underflow,
it wraps around. So `u64 MAX + 1` becomes `0`, and `0 - 1` becomes `u64 MAX`.

Stack underflows in these operations means the operation proceeds but u64 MAX
is popped from the stack instead.

#### ADD - add two numbers

Pops the two top numbers from the stack and places
the sum back on top.

So:

```
[1 2]
```

Becomes:

```
[3]
```

#### SUB

Substracts top of the stack from the second value on the stack, so:

```
[4 3]
```

becomes:

```
[1]
```

#### MUL

Multiplies the two top values on the stack.

#### DIV

Divides the second value of the stack by the top value on the stack, so:

```
[6 3]
```

becomes:

```
[2]
```

Division is down to the floor of the nearest integer.

It's important operations can't fail, so division by `0` results in `0`:

```
[3 ]
```

#### MOD

Divides the second value of the stack by the top value and puts the
remainder on the top of the stack, so that:

```
[5 2]
```

becomes:

```
[1]
```

### Comparison

In case of stack underflow, underflowing values popped are MAX u64 and then
compared.

#### EQ - equals

Pops the two top values on the stack. If they are equal, put `1` back
on the stack, otherwise '0'.

#### GT - greater than

Pops the two top values on the stack. If the second value is greater than the
top, put `1` back on the stack, otherwise '0'.

#### LT - lesser than

Pops the two top values on the stack. If the second value is lesser than the
top, put `1` back on the stack, otherwise '0'.

### Logic

In case of stack underflow, underflowing values popped are MAX u64.

#### NOT

Take the top of the stack. If it's greater than `0`, put `0` on the stack.
Otherwise if it's `0`, put `1` on the stack.

#### AND

Take the two values on the stack. If they are both greater than `0`, put `1` on
the stack, otherwise `0`.

#### OR

Take the two values on the stack. If either or both are greater than `0`, put
`1` on the stack, otherwise `0`.

### Memory

#### HEAD

Take a number from the top of the stack, clamped to the maximum number of
heads, and set the current head to this.

#### ADDR

Take the address of this instruction and put it in the current head, overwriting
the last value in the head.

#### COPY

Take a number from the top of the stack, clamped to the maximum number of
heads. Take its address (if any) and set the current head to this. If it has no
address, the current head remains unchanged.

#### FORWARD

Take a number from the top of the stack. Move the current head forward
in memory by that amount. If the number is bigger than the maximum amount
the head may move in one go, do not move the head.

#### BACKWARD

Take a number from the top of the stack. Move the current head backward
in memory by that amount. If the number is bigger than the maximum amount
the head may move in one go, do not move the head.

#### READ - read from memory

Read from the position in memory pointed to by the current head, putting the
result on top of the stack. If the current head is empty, no read occurs.

#### WRITE - write to memory

Take a value of the top of the stack and write it to the address indicated by
the current head. If the current empty is empty, no write occurs.

Stack values are 64 bit unsigned numbers, so can be too large for memory, which
is unsigned bytes. If the value is greater than `255`, put `255` on instead.

### Control

These instructions modify the instruction pointer of the processor - which
instruction in memory the processor is executing.

They use the current head as the jump destination.

#### JMP - unconditional jump

Jump to the address in the current head. If no address exists in the current
head, do not jump.

#### JMPIF - conditional jump

If the top of the stack is `0`, this is a no op. Otherwise, jump to the address
that's in the current head, unless it's empty, in which case do not jump.

### Processors

These operations only operate once per cycle of instructions in which they were
initiated. The length of the cycle of instructions is configured by
`instructions_per_update`.

#### START - spawn new processor

Spawns a new processor in the address by the current head. If the current head
is empty, no new processor is started.

Starting a lot of processors in a single cycle of instructions results in only
a a single processor to spawn. The last `START` instruction determines the
address.

#### END - end this processor

Destroys this processor after the cycle of instructions is finished.

### Split and merge

These operations only operate once per cycle of instructions in which they were
initiated. The length of the cycle of instructions is configured by
`instructions_per_update`.

#### SPLIT - split this computer into two parts

Take a direction off the top of the stack. A direction is either 0 (north), 1
(east), 2 (south) or 3 (west) from this computer's location. Any numbers
greater than 3 are taken as the remainder of 4, so are interpreted as a
direction as well.

Now use the address indicated by the current head as the split point - just
before this address the computer's memory is split into two. The first half
remains in place. The second half goes into the neighboring location in the
direction given, unless that is full, in which case nothing happens. If the
current head is empty, no split happens either.

Processors remain in their parts. Any addresses in processors (the instruction
pointer, heads) are automatically adjusted so they continue to point at the
same instructions. If any head ends up pointing into the wrong half, it's set
to empty.

Computer resources are divided equally between the two halves.

#### MERGE - merge this computer with neighboring computer

Take a direction off the top of the stack. A direction is either 0 (north), 1
(east), 2 (south) or 3 (west) from this computer's location. Any numbers
greater than 3 are taken as the remainder of 4, so are interpreted as a
direction as well.

This computer is then merged with the neighboring computer indicated by that
direction, if there is one. The memory of the merged computer is added to the
bottom of the computer that initiated the merging. The neighboring location is
now empty. The resources are added together and the processors all run on the
same computer.

Any addresses in processors (the instruction pointer, heads) are automatically
adjusted, so they remain pointing at the same instructions.

If the maximum amount of processors is reached after a merge, the excess
processors are eliminated.

### Resources

These operations only operate once per cycle of instructions in which they were
initiated. The length of the cycle of instructions is configured by
`instructions_per_update`.

#### EAT - take resources from location

Take the top of the stack as the amount of resources to eat, clamped by the
maximum the computer can eat in one go. Free resources in the location are
turned into bound resources in the computer.

If not enough free resources are available, all of them are eaten.

This is executed only once per update cycle. If multiple processors issue eat
instructions in a single update cycle, take the maximum.

#### GROW - use bound resources to grow memory

Take the top of the stack as the amount to grow, clamped by the
maximum the computer is allowed to grow.

Memory is grown by the amount indicated, at the end.

Each memory value grown costs 1 bound resource. If not enough bound resources
are available, use up all resources and grow the maximum possible.

This is executed only once per update cycle. If multiple processors issue grow
instructions in a single update cycle, take the maximum.

#### SHRINK - shrink memory, gain bound resources

Take the top of the stack as the amount to shrink, clamped by the
maximum the computer is allowed to shrink.

Memory is shrunk by the amount indicated, at the end. Any processors
pointing beyond the end die and are removed. Heads pointing beyond the
end are reset to empty.

Each memory value shrunk creates 1 bound resource.

This is executed only once per update cycle. If multiple processors issue
shrink instructions in a single update cycle, take the maximum.
