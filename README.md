# Single-rotation cellular automaton

This is a rust implementation of the ["single rotation" cellular automaton](http://dmishin.blogspot.com/2013/11/the-single-rotation-rule-remarkably.html).

## Installation and usage

First, clone and build this repository:

```sh
git clone https://github.com/adri326/single-rotation
cd single-rotation
cargo build --release
```

Then, run it:

```sh
./target/release/single-rotation
```

It will then listen for an [RLE-like](https://conwaylife.com/wiki/Run_Length_Encoded) formatted input string through the standard input (stdin).
You can type such a string yourself:

```
x = 0, y = 0
o$o2$o$o!
```

Then, hit enter and the simulation will begin.

The `x` and `y` parameters are optional and will default to `0`.
These are the parameters that are accepted:

- `x`, sets the `X` coordinate of the top-left corner (unlike classical RLE, where the `x` argument corresponds to the width of the pattern)
- `y`, sets the `Y` coordinate of the top-left corner (unlike classical RLE, where the `y` argument corresponds to the height of the pattern)
- `s`, which sets the number of simulation step between frames (default `4`)
- `i`, which sets the minimum time interval between frames, in milliseconds (default `100`)
- `u`, the "smoothing" factor for the interpolation in the rendered view; a value of `4` will visually hide the 4-step oscillator nature of isolated cells

When put together, it looks like this:

```
x = 0, y = 0, s = 41, i = 1000
4bobo2$b2ob3o!
```

You can also use shellscript notation to redirect stdin from a file:

```sh
./target/release/single-rotation < examples/lightest-slow.rle
```

By default, a CLI representation of the simulation will be displayed. You can switch to a window version instead by running it with the `-g` option:

```sh
./target/release/single-rotation -g < examples/lightest-slow.rle
```
