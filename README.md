# Klein Garter

> A terminal game "engine" and a proof-of-concept snake game, built in Rust (for learning).
> The project is built in an Object-Oriented architecture, which may has its strengths in clarity, but as I later learned, definitely has its downfalls when it comes to cache locality.

---

https://github.com/user-attachments/assets/930cfabd-54cc-487c-948f-c0bf3c6f3100

---

### Wan't to try it out?
Prerequisites:
* **Rust**: Install the Rust toolchain using [rustup](https://www.rust-lang.org/tools/install).
* **Git**: Required to clone the project repository. You can get it from [git-scm.com](https://git-scm.com/).

Follow these instructions to get the project up and running:
1.  **Clone the repository:**
    ```sh
    git clone [https://github.com/hartolit/klein-garter.git](https://github.com/hartolit/klein-garter.git)
    cd klein-garter
    ```

2.  **Build the project:**
    ```sh
    cargo build --release
    ```

3.  **Run the game:**
    ```sh
    cargo run --release
    ```

---

## Engine Architecture

The engine is operating from an `Object` trait system, which provides a set of capability traits for an object to inherit. The idea was to have a flexible way for the engine to efficiently distinguish and process objects of certain capabilities.

### Objects, TCells, and StateChanges
At its core, everything is an `Object`. An object's physical form in the terminal is made up of one or more `TCell` (Terminal Cells). A `TCell` is just a single character with some properties: `Position`, `Glyph` (its symbol and its colors), and a z_index for layering.

An `Object` is by default static but can be made dynamic by giving it the `Stateful` capability trait. Instead of changing an object's properties directly, you record the `StateChange(Create, Update, or Delete)` for one of its TCells. The engine then collects these changes and tells the renderer which cells to redraw (this is to avoid re-rendering an entire objects state). If the object also had the `Spatial` capability, the engine would sync these changes to the spatial grid to make it collidable.
The cabilities include: `Stateful`, `Destructible`, `Active`, `Spatial` and `Movable`.

### Game Logic Layers
The game logic can happen through 3 different layers depending on its complexity:
* **Object-level**: An object can handle simple logic like changing its state, react to collisions and fire events.
* **Events**: Events handle more complex interactions like handling specific collisions which are outside the capabilities of an object.
* **Logic trait**: At last the `Logic<K>` trait handles the high-level game flow, like player input, spawning new objects, and managing the overall game state.

### Other
The engine also provides a few other noteworthy capabilities:

* **Switching between Stages**: A `Stage` is made up of some `Logic` and a `Scene` which holds the objects. So if you have different stages like: `Level1`, `Level2`, etc., you can switch between them within the logic trait by returning `RuntimeCommand::SwitchStage(K)` from the update loop.

* **Hot-swapping Logic/Scenes**: If you want to keep the same `Scene` but use a different logic, you can use `RuntimeCommand::ReplaceLogic(Box<dyn Logic<K>>)`. Similarly, you can replace a scene with `RuntimeCommand::ReplaceScene(Box<Scene>)`. All done through the update loop. 
(The proof-of-concept implements the Logic swap by swapping from main SnakeGame Logic to DeathLogic via key input.)

---

## About the Downfalls

There's many performance downfalls in the engine design. The biggest one is cache locality on objects and state management. Using a `HashMap` to iterate through the different objects isn't the best solution as it suffers from cache misses. An ECS architecture instead of my Object-Oriented solution would've performed much better. Something I've yet to fully explore.

The state management for objects is a giant, beautiful mess. It obviously suffers from cache misses, but also its complexity of processing a `StateChange`. It would've been much simpler to make an infinite grid with a snapshot of state changes and just do a "simple" comparison between snapshots to determine a single source of truth. This could have also opened a nice door for concurrency and split processing between multiple threads. Which brings me to the final downfall.

This was the first project I've built to learn rust. Learning Rust the last few months has been quite the eye opener. This means I didn't dabble in concurrent scenarios until much more recently, so the entire design was built for a single-threaded world, which misses out on a lot of performance benefits.

Another minor downfall is the event system, which is very likely store duplicated events. This could "easily" be fixed by implementing a more complex event system.

---

### TODO's

Small list for future me:
* **Unit and integration tests! :D**
* **Asset generation** from pictures and gifs
* **Make grid bounds toggle**
* **Make terrain collidable**
* **Better event system**
* **Food ghost object**

### Current state

This project is no longer actively developed on. Dread has the final say and I'm moving on for now.
