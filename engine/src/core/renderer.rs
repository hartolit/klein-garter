
mod buffer;

use super::scene::Scene;
use crate::core::object::{state::StateChange};

use buffer::{Buffer, Operation};

pub struct Renderer {
    buffer: Buffer,
}

impl Renderer {
    pub fn new() -> Self {
        Self { buffer: Buffer::new() }
    }

    pub fn kill(&mut self) {
        self.buffer.kill();
    }

    pub fn full_render(&mut self, scene: &Scene) {
        self.buffer.clear();

        // Draw grid
        if let Some(grid) = &scene.spatial_grid {
            for y in 0..grid.full_height {
                for x in 0..grid.full_width {
                    let pos = crate::core::global::Position::new(x, y);
                    let index = (y * grid.full_width + x) as usize;
                    let glyph = &grid.cells[index].kind.appearance();
                    self.buffer.upsert(pos, Operation::Draw { glyph: *glyph, z_index: grid.cells[index].z_index });
                }
            }
        }

        // Draw objects
        for object in scene.objects.values() {
            for t_cell in object.t_cells() {
                self.buffer.upsert(t_cell.pos, Operation::Draw { glyph: t_cell.style, z_index: t_cell.z_index });
            }
        }

        self.buffer.flush();
    }

    pub fn partial_render(&mut self, scene: &Scene) {
        // TODO - Fix filtered states
        for state in scene.global_state.filtered.deletes.iter() {
            if let StateChange::Delete { init_pos, .. } = *state {
                if let Some(grid) = &scene.spatial_grid {
                    if let Some(cell) = grid.get_cell(&init_pos) {
                        let glyph = cell.kind.appearance();
                        self.buffer.upsert(init_pos, Operation::Draw { glyph, z_index: cell.z_index });
                    } else {
                        self.buffer.upsert(init_pos, Operation::Clear);
                    }
                } else {
                    self.buffer.upsert(init_pos, Operation::Clear);
                }
            }
        }

        for state in scene.global_state.filtered.updates.iter() {
            if let StateChange::Update { t_cell, init_pos } = *state {
                if t_cell.pos != init_pos {
                    if let Some(grid) = &scene.spatial_grid {
                        if let Some(cell) = grid.get_cell(&init_pos) {
                            let glyph = cell.kind.appearance();
                            self.buffer.upsert(init_pos, Operation::Draw { glyph, z_index: cell.z_index });
                            self.buffer.upsert(t_cell.pos, Operation::Draw { glyph: t_cell.style, z_index: t_cell.z_index });
                        } else {
                            self.buffer.upsert(init_pos, Operation::Clear);
                            self.buffer.upsert(t_cell.pos, Operation::Draw { glyph: t_cell.style, z_index: t_cell.z_index });
                        }
                    } else {
                        self.buffer.upsert(init_pos, Operation::Clear);
                        self.buffer.upsert(t_cell.pos, Operation::Draw { glyph: t_cell.style, z_index: t_cell.z_index });
                    }
                } else {
                    self.buffer.upsert(init_pos, Operation::Draw { glyph: t_cell.style, z_index: t_cell.z_index });
                }
            }
        }

        for state in scene.global_state.filtered.creates.iter() {
            if let StateChange::Create { new_t_cell } = *state {
                self.buffer.upsert(new_t_cell.pos, Operation::Draw { glyph: new_t_cell.style, z_index: new_t_cell.z_index });
            }
        }

        self.buffer.flush();
    }
}
