use engine::core::Runtime;

mod game;
mod ui;

fn main() {
    let runtime = Runtime::new(logic, spatial_grid, tick_rate);
}
