pub mod buffer;

use std::{collections::HashSet};

use crate::prelude::{Scene, StateChange, ObjectIndex};

use buffer::{Buffer, Operation};

pub struct Renderer {
    buffer: Buffer,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
        }
    }

    pub fn kill(&mut self) {
        self.buffer.kill();
    }

    pub fn full_render(&mut self, scene: &Scene) {
        self.buffer.clear();

        // Draws grid and spatial objects
        if let Some(grid) = &scene.spatial_grid {
            // Draws border
            for (pos, glyph) in grid.get_border() {
                self.buffer.upsert(
                    pos,
                    Operation::Draw {
                        glyph,
                        z_index: 255,
                    },
                );
            }

            // Draws grid cells and their occupants
            for y in 0..grid.height {
                for x in 0..grid.width {
                    let grid_pos = crate::core::global::Position::new(x, y);
                    let world_pos = grid.pos_to_world(grid_pos);
                    if let Some(cell) = grid.get_cell(&world_pos) {
                        let (glyph, z_index) = cell.top_glyph_and_z();
                        self.buffer.upsert(
                            world_pos,
                            Operation::Draw {
                                glyph: *glyph,
                                z_index,
                            },
                        );
                    }
                }
            }
        }

        // Draws non-spatial objects (like UI)
        let empty_set = HashSet::new();
        let spatial_ids = scene.indexes.get(&ObjectIndex::Spatial).unwrap_or(&empty_set);
        for (id, object) in &scene.objects {
            if spatial_ids.contains(id) {
                continue;
            }

            for t_cell in object.t_cells() {
                self.buffer.upsert(t_cell.pos, Operation::Draw { glyph: t_cell.style, z_index: t_cell.z_index });
            }
        }

        self.buffer.flush();
    }

    pub fn partial_render(&mut self, scene: &Scene) {
        // Spatial changes are synced with the grid and hides behind:
        // `cell.top_glyph_and_z()`, which gets the most prominent cell
        // e.g: 'Terrain' or 'TCell'.
        // The logic behind this sync is located at 'scene.rs'.

        // Spatial draws
        for state in scene.global_state.filtered.spatial.iter() {
            if let Some(grid) = &scene.spatial_grid {
                match state {
                    StateChange::Delete { init_pos, .. } => {
                        if let Some(cell) = grid.get_cell(init_pos) {
                            let (glyph, z_index) = cell.top_glyph_and_z();
                            self.buffer.upsert(
                                *init_pos,
                                Operation::Draw {
                                    glyph: *glyph,
                                    z_index,
                                },
                            );
                        }
                    }
                    StateChange::Update { t_cell, init_pos } => {
                        if t_cell.pos != *init_pos {
                            if let Some(cell) = grid.get_cell(init_pos) {
                                let (glyph, z_index) = cell.top_glyph_and_z();
                                self.buffer.upsert(
                                    *init_pos,
                                    Operation::Draw {
                                        glyph: *glyph,
                                        z_index,
                                    },
                                );
                            }
                        }

                        if let Some(cell) = grid.get_cell(&t_cell.pos) {
                            let (glyph, z_index) = cell.top_glyph_and_z();
                            self.buffer.upsert(
                                t_cell.pos,
                                Operation::Draw {
                                    glyph: *glyph,
                                    z_index,
                                },
                            );
                        }
                    }
                    StateChange::Create { new_t_cell } => {
                        if let Some(cell) = grid.get_cell(&new_t_cell.pos) {
                            let (glyph, z_index) = cell.top_glyph_and_z();
                            self.buffer.upsert(
                                new_t_cell.pos,
                                Operation::Draw {
                                    glyph: *glyph,
                                    z_index,
                                },
                            );
                        }
                    }
                }
            }
        }

        // Non-spatial draws
        for state in scene.global_state.filtered.non_spatial.iter() {
            match state {
                StateChange::Delete { init_pos, .. } => {
                    self.buffer.upsert(*init_pos, Operation::Clear);
                }
                StateChange::Update { t_cell, init_pos } => {
                    if t_cell.pos != *init_pos {
                        self.buffer.upsert(*init_pos, Operation::Clear);
                    }
                    self.buffer.upsert(
                        t_cell.pos,
                        Operation::Draw {
                            glyph: t_cell.style,
                            z_index: t_cell.z_index,
                        },
                    );
                }
                StateChange::Create { new_t_cell } => {
                    self.buffer.upsert(
                        new_t_cell.pos,
                        Operation::Draw {
                            glyph: new_t_cell.style,
                            z_index: new_t_cell.z_index,
                        },
                    );
                }
            }
        }

        self.buffer.flush();
    }
}
