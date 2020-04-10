# Dathos game engine

This is a simple, extensible 2D game engine written in Rust for use with a
combination of Rust and Ruby.
Dathos supports writing extensions in Rust that are exposed as modules and
classes to the Ruby runtime.
These modules also have hooks into the phases of the game loop:

- `init` - runs once at the beginning of the game
- `pre_update` - runs before the game objects are updated and before coroutines are executed
- `update` - considered undefined whether this runs before or after game object updates
- `post_update` - runs after the game objects are updated and coroutines are executed. Indeterminant whether this is pre- or post-rendering.

## Example

See [Melody Madness](https://github.com/BrianMWest/melody-madness) for a simple
example of this engine used to create a Slack game, including custom modules
and usage of coroutines.

## Rendering

Rendering can be called from anywhere in the Ruby code.
The engine exposes a `Draw` module to the Ruby runtime.
This module allows for several drawing functions, including sprites, primitives,
and text.

`Draw.text!`

`Draw.rect!` / `Draw.rectangle!`

`Draw.circle!`

`Draw.line!` (WIP)

`Draw.sprite!`

`Draw.arc!`

Fonts and spritesheets can be loaded asynchronously, although there is currently
no way to check whether the loading is finished.
Draw commands for a font or spritesheet that has not been loaded yet will simply
be ignored.

Drawing a sprite involves aliasing a Ruby symbol (e.g. `:my_sprite`) to a
spritesheet with an offset and a size using `Draw.create_sprite`.

`Draw.load_font`

`Draw.load_spritesheet`

`Draw.create_sprite`

## Creating game objects

Game objects can be created with `Game.create! MyObject.new`.
To store a reference to the object, you can create it first:

```ruby
@child = MyObject.new
Game.create! @child
```

You can delete an object later with `Game.delete! @child`.

**Important!** all game objects must inherit from `GameObject`!

## Coroutines

Most scripted games today include coroutines, which are methods that execute for
your game object asynchronously over several frames.
Dathos supports two kinds, the second being a convenience method for a common
coroutine use case. Both methods are defined in and inherited from the
`GameObject` super class.

`run!` - takes a block or proc accepting a single parameter `wait` which allows
you to control how long the coroutine should wait before continuing.

```ruby
run! do |wait|
  puts 'first'
  wait.for_seconds(1.9)
  puts 'second'
  wait.for_frames(5)
  puts 'third'
  wait.next_frame
  puts 'fourth'
end
```

`run_for!` - takes a duration and block or proc accepting two parameters,
`elapsed` and `duration`. This runs the provided block of code every frame for
the provided duration. Useful for easing functions, for example.

```ruby
run_for!(2.75) do |elapsed, duration|
  puts "Percent complete: #{elapsed / duration}"
end
```
