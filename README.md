# Klein Garter

> A terminal game "engine" and a proof-of-concept snake game, built in Rust.
> The project is built in an Object-Oriented architecture, which may has its strengths in clarity, but as I later learned, definitely has its downfalls when it comes to cache locality.

---

## Engine Architecture

The engine's main driver is an `Object` trait system, which provides a set of capabilities for an object to inherit. A good example is the `Stateful` trait, which gives an object the ability to be dynamic by announcing the `StateChanges` it's made. These changes are then collected, processed, and synced to the `SpatialGrid` and renderer.

The game specific logic happens through 3 different layers depending its complexity:
* **Object**: An object can handle simple collision logic and fire events.
* **Events**: Events handle more complex scenarios which would be outside the capabilities of an object.
* **Logic trait**: At last the `Logic<K>` trait provided by the engine provides full control of a tick.

The engine also provides a few other noteworthy capabilities:

* **Switching between Stages**: A `Stage` is made up of some `Logic` and a `Scene` which holds the objects. So if you have different stages like: `Level1`, `Level2`, etc., you can switch between them within the logic trait by returning `RuntimeCommand::SwitchStage(K)` from the update loop.

* **Replacing Logic or Scenes**: If you want to keep the same `Scene` but use different logic, you can use `RuntimeCommand::ReplaceLogic(Box<dyn Logic<K>>)`. Similarly, you can replace a scene with `RuntimeCommand::ReplaceScene(Box<Scene>)`. All done through the update loop.

---

## About the Downfalls

There's many performance downfalls in the engine design. The biggest one is cache locality on objects and state management. Using a `HashMap` to iterate through the different objects isn't the best solution as it suffers from cache misses. An ECS architecture instead of my Object-Oriented solution would've performed much better. Something I've yet to fully explore.

The state management for objects is a giant, beautiful mess. It obviously suffers from cache misses, but also its complexity of processing `StateChanges`. It would've been much simpler to make an infinite grid with a snapshot of state changes and just do a "simple" comparison between snapshots to determine a single source of truth. This could have also opened a nice door for concurrency and split the processing between multiple threads. Which brings me to the final downfall.

This was the first project I've built to learn rust. Learning Rust the last few months has been quite the eye opener. This means I didn't dabble in concurrent scenarios until much more recently, so the entire design was built for a single-threaded world, which misses out on a lot of performance benefits.

Another minor downfall is the event system, which is very likely store duplicated events. This could "easily" be fixed by implementing a more complex event system.

## Current state

The project is no longer actively developed on. Dread has the final say and I'm moving on for now.