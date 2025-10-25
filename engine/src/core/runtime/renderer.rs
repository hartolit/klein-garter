pub mod frame_thread;

use std::{sync::mpsc::{sync_channel, SyncSender, TrySendError}, thread};
use rustc_hash::{FxHashMap, FxHashSet};
use crate::prelude::{ObjectIndex, Position, Scene, StateChange};
use frame_thread::{FrameThread, Operation};

pub enum RenderCommand {
    Draw(FxHashMap<Position, Operation>, bool),
    Kill
}

pub struct Renderer {
    tx: SyncSender<RenderCommand>,
    frame_buffer: FxHashMap<Position, Operation>,
    render_handle: Option<thread::JoinHandle<()>>,
}

impl Renderer {
    pub fn new() -> Self {
        let (tx, rx) = sync_channel::<RenderCommand>(1);

        let render_handle = thread::spawn(move || {
            let mut frame_thread = FrameThread::new();
            frame_thread.run(rx);
        });

        Self {
            tx,
            frame_buffer: FxHashMap::default(),
            render_handle: Some(render_handle),
        }
    }

    pub fn kill(&mut self) {
        if self.tx.send(RenderCommand::Kill).is_ok() {
            if let Some(handle) = self.render_handle.take() {
                handle.join().unwrap();
            }
        } else {
            panic!("Failed to send kill to renderer");
        }
    }

    pub fn upsert(&mut self, pos: Position, new_op: Operation) {
        use std::collections::hash_map::Entry;

        let new_z = match &new_op {
            Operation::Clear => 0,
            Operation::Draw { z_index, .. } => *z_index,
        };

        match self.frame_buffer.entry(pos) {
            Entry::Vacant(entry) => {
                entry.insert(new_op);
            }
            Entry::Occupied(mut entry) => {
                let existing_op = entry.get_mut();
                let existing_z = match existing_op {
                    Operation::Clear => 0,
                    Operation::Draw { z_index, .. } => *z_index,
                };

                if new_z > existing_z {
                    *existing_op = new_op;
                }
            }
        }
    }



    pub fn full_render(&mut self, scene: &Scene) {
        self.frame_buffer.clear();

        // Draws grid and spatial objects
        if let Some(grid) = &scene.spatial_grid {
            // Draws border
            for (pos, glyph) in grid.get_border() {
                self.upsert(
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
                        self.upsert(
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
        let empty_set = FxHashSet::default();
        let spatial_ids = scene
            .indexes
            .get(&ObjectIndex::Spatial)
            .unwrap_or(&empty_set);
        for (id, object) in &scene.objects {
            if spatial_ids.contains(id) {
                continue;
            }

            for t_cell in object.t_cells() {
                self.upsert(
                    t_cell.pos,
                    Operation::Draw {
                        glyph: t_cell.style,
                        z_index: t_cell.z_index,
                    },
                );
            }
        }

        self.send_frame(true);
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
                            self.upsert(
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
                                self.upsert(
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
                            self.upsert(
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
                            self.upsert(
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
                    self.upsert(*init_pos, Operation::Clear);
                }
                StateChange::Update { t_cell, init_pos } => {
                    if t_cell.pos != *init_pos {
                        self.upsert(*init_pos, Operation::Clear);
                    }
                    self.upsert(
                        t_cell.pos,
                        Operation::Draw {
                            glyph: t_cell.style,
                            z_index: t_cell.z_index,
                        },
                    );
                }
                StateChange::Create { new_t_cell } => {
                    self.upsert(
                        new_t_cell.pos,
                        Operation::Draw {
                            glyph: new_t_cell.style,
                            z_index: new_t_cell.z_index,
                        },
                    );
                }
            }
        }

        self.send_frame(false);
    }

    fn send_frame(&mut self, is_full_render: bool) {
        if !self.frame_buffer.is_empty() {
            let frame = std::mem::take(&mut self.frame_buffer);

            if let Err(e) = self.tx.try_send(RenderCommand::Draw(frame, is_full_render)) {
                match e {
                    TrySendError::Full(RenderCommand::Draw(frame_buffer, .. )) => {
                        // Render thread is busy, drop the frame.
                        // Puts the map back so we can reuse its allocation.
                        self.frame_buffer = frame_buffer;
                    },
                    TrySendError::Disconnected(_) => {
                        eprintln!("Render thread disconnected: {}", e);
                    },
                    _ => {}
                }
            }
        }
    }
}
