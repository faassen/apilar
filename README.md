# Apilar

## About Apilar

Apilar is a language and alife system.

Apilar is an stack-machine assembly language that supports self-assembly: a
replicator Apilar program can be written that makes copies of itself in memory
and in space. New programs then evolve.

The Apilar language is a stack-based assembly language. There's a virtual
machine implementation of this.

Here's a little example:

```
N2
N3
ADD
```

This adds two numbers to the stack, `2` and `3`, and then adds them together.
In the end the stack contains `5`. See the [Apilar language
reference](doc/language.md) for more information.

A computer has memory and one or more processors. Each processor has its own
stack.

A replicator program can be written in Apilar that causes it:

- to copy itself

- to spawn a new processor for its copy

Processors can disappear if:

- they run out of memory

- they run an explicit "END" instruction.

Computers exist in a 2d habitat. The habitat is a grid of locations, and each
location has resources and potentially a computer. Programs can cause a
computer to split into two pieces (into a neighbor), or merge a neighbor into
itself.

Repeated splitting of its memory would make a computer very small. So a
computer can also grow its memory. To do so it needs to eat resources.

When a computer has no more processors, it dies and its resources (including
that bound in its memory) are released to the environment.

So now we have reproduction. Computers may also die.

To introduce a process of evolution, once every while a random address in a
random computer's memory is mutated.

This is usually not very useful, but sometimes a mutation may help a replicator
survive or replicate better.

More than a single 2d habitat can exist, and each can have its own dimensions
and configuration. This is defined in a config JSON file.

Apilar is inspired by the famous alife simulation
[Tierra](<https://en.wikipedia.org/wiki/Tierra_(computer_simulation)>).

## How to build and use

You need to have a recent stable Rust installed. Then:

```
cargo run --release -- run config/simple-config.json
```

You also want to start the UI, for now this is:

```
cd ui
npm install
npm run dev
```

And then connect to http://localhost:3000

This creates a habitat, seeds it with a single hard-coded replicator, and then
lets it run. You can see the world evolve in the terminal, so you please make
your terminal window big enough.

The red squares on the map are computers. The blue is the free resources in a
location -- white is no resources, the more and darker blue the more.

Sometimes after starting a replicator growth stops due to back luck; you want
to restart Apilar then and reload the web page.

Sometimes growth is slow. Sometimes growth accelerates; it all depends on what
mutations occurred. Sometimes everything dies out after a long time.

If you want to see a new world, just `ctrl-C` to stop and run again.

There are also command line arguments to configure the simulation, see:

```
cargo run --release -- run -h
```

`--autosave` dumps save files on a regular basis that you can load from again:

```
cargo run --release -- load <mydumpfile>
```

What is going on in these worlds? It's a bit of a mystery without more careful
analysis. You can click on individual computers to see their disassembled
memory, and you can try to read what's going on. To understand the
instructions, see the [Apilar language reference](doc/language.md).

## sample code

You can find a few sample programs in the `sample_code` directory.

## sample configs

You can find a bunch of sample world configurations in the `config` directory.

## langjam 3

Apilar was originally created for [langjam
3](https://github.com/langjam/jam0003). The theme for this jam was "Beautiful
Assembly".
