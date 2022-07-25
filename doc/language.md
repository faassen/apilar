# Apilar Language

The Apilar language is a stack-based assembly language. Each instruction can read
from the stack or push to the stack. Some operations have effects on the
computer they run in, or on the world itself.

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

#### N1 ... N8 - number literals

The instructions N1, N2, ... N8 place the respective number on the stack.
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

### Control

These instructions modify the instruction pointer of the processor - which
instruction in memory the processor is executing.

They all pop values from the stack interpreting them as an address. An address
out of range results in a no op instead. An address is out of range if it
points outside of memory, or if it's further away in either direction than the
address distance from the current address of the instruction pointer --- this
distance is currently hardcoded to `1024`.

#### JMP - unconditional jump

Take the top of the stack and jump to its address.

#### JMPIF - conditional jump

If the top of the stack is `0`, this is a no op. Otherwise, jump to the address
that's the second value of the stack.

#### CALL - unconditional call

Take the top of the stack and jump to its address. Before the jump, put
the return address on the top of the stack. This is the address that
initiated the call and the code continues from there.

#### CALLIF - conditional call

If the top of the stack is `0`, this is a no op. Otherwise, jump to the address
that's the second value of the stack.

Before the jump, put the return address on the top of the stack. This is the
address that initiated the call and the code continues from there.

### Memory

#### ADDR

Take the address of this instruction and put it on the top of the stack. This
can then be used as a jump destination.

#### READ - read from memory

Interpret the top of the stack as an address and read from it, putting the
result on top of the stack. If the address is out of range, this operation puts
`255` on top of the stack. A stack underflow puts `256` on the stack too.

#### WRITE - write to memory

Take a value of the top of the stack and the second value is interpreted as an
address. If the address is in range, write the value there. Stack values are 64
bit unsigned numbers, so can be too large for memory, which is unsigned bytes.
If the value is greater than `255`, put `255` on instead. If the address is
out of bounds, no write occurs.

### Processors

These operations only operate once per cycle of instructions in which they were
initiated. The length of the cycle of instructions is configured by
`instructions_per_update`.

#### START - spawn new processor

Spawns a new processor in the address given by the top of the stack. If the
top of the stack is out of range, this is a no op.

Starting a lot of processors in a single cycle of instructions results in only
a a single processor to spawn. The last `START` instruction determines the
address.

#### END - end this processor

Destroys this processor after the cycle of instructions is finished.

### Resources

These operations only operate once per cycle of instructions in which they were
initiated. The length of the cycle of instructions is configured by
`instructions_per_update`.

#### EAT - take resources from location

Take the amount of resources configured by `eat_amount` at the end of this
cycle of instructions from the location into the computer's resources.

#### GROW - use resource to grow memory

Take 1 resource from the computer's resource pool and grow memory 1 position at
the end. Executed only once per cycle of instructions.

### Split and merge

These operations only operate once per cycle of instructions in which they were
initiated. The length of the cycle of instructions is configured by
`instructions_per_update`.

#### SPLIT - split this computer into two parts

Take a direction off the top of the stack. A direction is either 0 (north), 1
(east), 2 (south) or 3 (west) from this computer's location. Any numbers
greater than 3 are taken as the remainder of 4, so are interpreted as a
direction as well.

The second value on the stack is interpreted as an address. This address is the
split point - just before this address the computer's memory is split into two.
The first half remains in place. The second half goes into the neighboring
location in the direction given, unless that is full, in which case nothing
happens.

Processors remain in their parts. The processors on the second parts will have
all the addresses they may have on the stack screwed up, which is a problem for
them.

Computer resources are divided equally between the two halves.

If the address is out of range, nothing happens besides the values being
popped.

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

The processors on the merged part will have all the addresses they may have on
the stack screwed up, which is a problem for them.
