use std::collections::BTreeMap;
use std::sync::Arc;

use crate::msprite::MSpriteHandle;
use crate::scene::{
    DrawCommand, RectF, SceneTexture, SceneTextureFormat, SceneTextureId, SpriteDraw,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SpriteHandle(pub u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SpriteSurfaceId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RenderNodeId(pub u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SpriteTransitionHandle(pub u32);

#[derive(Debug, Default)]
pub struct SpriteSystem {
    next_handle: u32,
    next_render_node: u32,
    next_surface_id: u64,
    next_transition_handle: u32,
    next_fx_handle: u32,
    sprites: BTreeMap<SpriteHandle, PalSprite>,
    surfaces: BTreeMap<SpriteSurfaceId, SpriteSurface>,
    render_nodes: BTreeMap<RenderNodeId, RenderNode>,
    transitions: BTreeMap<SpriteTransitionHandle, SpriteTransition>,
    fx_effects: BTreeMap<SpriteFxHandle, SpriteFxEffect>,
    motion_entries: BTreeMap<SpriteHandle, SpriteMotionEntry>,
}

impl SpriteSystem {
    pub fn new() -> Self {
        Self {
            next_handle: 1,
            next_render_node: 1,
            next_surface_id: 1,
            next_transition_handle: 1,
            next_fx_handle: 1,
            sprites: BTreeMap::new(),
            surfaces: BTreeMap::new(),
            render_nodes: BTreeMap::new(),
            transitions: BTreeMap::new(),
            fx_effects: BTreeMap::new(),
            motion_entries: BTreeMap::new(),
        }
    }

    pub fn allocate_surface_id(&mut self) -> SpriteSurfaceId {
        let id = SpriteSurfaceId(self.next_surface_id);
        self.next_surface_id = self
            .next_surface_id
            .checked_add(1)
            .expect("sprite surface id space exhausted");
        id
    }

    pub fn insert_texture(&mut self, texture: SceneTexture) -> SpriteSurfaceId {
        self.insert_surface(SpriteSurface::from_scene_texture(texture))
    }

    pub fn insert_surface(&mut self, surface: SpriteSurface) -> SpriteSurfaceId {
        let id = surface.id;
        self.surfaces.insert(id, surface);
        id
    }

    pub fn surface(&self, id: SpriteSurfaceId) -> Option<&SpriteSurface> {
        self.surfaces.get(&id)
    }

    pub fn surface_mut(&mut self, id: SpriteSurfaceId) -> Option<&mut SpriteSurface> {
        self.surfaces.get_mut(&id)
    }

    pub fn textures(&self) -> impl Iterator<Item = SceneTexture> + '_ {
        self.surfaces.values().map(SpriteSurface::to_scene_texture)
    }

    pub fn create(&mut self, desc: SpriteDesc) -> SpriteHandle {
        let handle = self.allocate_handle();
        let render_node = self.allocate_render_node();
        let sprite = PalSprite::new(handle, render_node, desc);
        let priority = sprite.effective_priority();
        self.render_nodes
            .insert(render_node, RenderNode::new(render_node, handle, priority));
        self.sprites.insert(handle, sprite);
        handle
    }

    pub fn create_rgba_sprite(
        &mut self,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
        position: PalVec3,
        priority: i32,
        source_name: impl Into<String>,
    ) -> Option<SpriteHandle> {
        let width = width.max(1);
        let height = height.max(1);
        let surface_id = self.allocate_surface_id();
        let surface = SpriteSurface::rgba8(surface_id, 1, width, height, rgba).ok()?;
        self.insert_surface(surface);
        let mut desc = SpriteDesc::new(SceneTextureId(surface_id.0), width, height);
        desc.position = position;
        desc.base_priority = priority;
        desc.visible = true;
        desc.smooth_upscale = true;
        desc.source_name = source_name.into();
        Some(self.create(desc))
    }

    /// Copy a sprite's current surface pixels into a new sprite at the same
    /// position. Used by `get_backbuffer` / `PalSpriteBackBafferCopy`.
    pub fn copy_sprite_pixels(
        &mut self,
        source: SpriteHandle,
        priority: i32,
        source_name: impl Into<String>,
    ) -> Option<SpriteHandle> {
        let sprite = self.get(source)?;
        let position = sprite.position;
        let surface_id = sprite.surface;
        let texture = self.surface(surface_id)?.to_scene_texture();
        self.create_rgba_sprite(
            texture.width,
            texture.height,
            texture.pixels.to_vec(),
            position,
            priority,
            source_name,
        )
    }

    pub fn create_msprite(
        &mut self,
        decoder: MSpriteHandle,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
        position: PalVec3,
        priority: i32,
        source_name: impl Into<String>,
    ) -> Option<SpriteHandle> {
        let width = width.max(1);
        let height = height.max(1);
        let surface_id = self.allocate_surface_id();
        let surface = SpriteSurface::rgba8(surface_id, 1, width, height, rgba).ok()?;
        self.insert_surface(surface);
        let mut desc = SpriteDesc::new(SceneTextureId(surface_id.0), width, height);
        desc.kind = SpriteKind::MSprite { decoder };
        desc.position = position;
        desc.base_priority = priority;
        desc.visible = true;
        desc.source_name = source_name.into();
        Some(self.create(desc))
    }

    pub fn replace_sprite_surface(
        &mut self,
        handle: SpriteHandle,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
        source_name: impl Into<String>,
    ) -> bool {
        let width = width.max(1);
        let height = height.max(1);
        let Some(surface_id) = self.get(handle).map(|sprite| sprite.surface) else {
            return false;
        };
        // GUI rendering caches textures by id/generation. Native PAL frequently
        // replaces a sprite surface in-place with the same id and dimensions
        // for ADV typewriter text, face parts, and animation frames, so a
        // replacement must advance the generation even when the size is stable.
        let generation = self
            .surfaces
            .get(&surface_id)
            .map(|surface| surface.generation.saturating_add(1).max(1))
            .unwrap_or(1);
        let Ok(surface) = SpriteSurface::rgba8(surface_id, generation, width, height, rgba) else {
            return false;
        };
        self.surfaces.insert(surface_id, surface);
        if let Some(sprite) = self.get_mut(handle) {
            sprite.texture_size = PalSize::new(width, height);
            sprite.cell_size = PalSize::new(width, height);
            sprite.source_rect = PalRect::new(0, 0, width as i32, height as i32);
            sprite.source_name = source_name.into();
            return true;
        }
        false
    }

    pub fn replace_msprite_frame(
        &mut self,
        handle: SpriteHandle,
        decoder: MSpriteHandle,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
        source_name: impl Into<String>,
    ) -> bool {
        if !self.replace_sprite_surface(handle, width, height, rgba, source_name) {
            return false;
        }
        if let Some(sprite) = self.get_mut(handle) {
            sprite.kind = SpriteKind::MSprite { decoder };
            return true;
        }
        false
    }

    pub fn release(&mut self, handle: SpriteHandle) -> bool {
        let Some(sprite) = self.sprites.remove(&handle) else {
            return false;
        };
        let surface = sprite.surface;
        self.motion_entries.remove(&handle);
        if let Some(render_node) = sprite.render_node {
            self.render_nodes.remove(&render_node);
        }
        if !self.sprites.values().any(|other| other.surface == surface) {
            self.surfaces.remove(&surface);
        }
        true
    }

    pub fn create_transition_handle(&mut self) -> SpriteTransitionHandle {
        let handle = SpriteTransitionHandle(self.next_transition_handle);
        self.next_transition_handle = self
            .next_transition_handle
            .checked_add(1)
            .expect("sprite transition handle space exhausted");
        self.transitions.insert(handle, SpriteTransition::default());
        handle
    }

    pub fn release_transition_handle(&mut self, handle: SpriteTransitionHandle) -> bool {
        if self.transition_state(handle) != 3 {
            self.cancel_transition(handle);
        }
        self.transitions.remove(&handle).is_some()
    }

    pub fn set_transition(
        &mut self,
        handle: SpriteTransitionHandle,
        render_id: u32,
        from: Option<SpriteHandle>,
        to: Option<SpriteHandle>,
        transition_id: u32,
        duration_ms: u32,
        flags: u32,
    ) -> bool {
        if !self.transitions.contains_key(&handle) {
            return false;
        }
        if from.is_none() && to.is_none() {
            return false;
        }
        if self.transition_state(handle) != 3 {
            self.cancel_transition(handle);
        }
        if let Some(sprite) = from.and_then(|sprite| self.get_mut(sprite)) {
            sprite.transition_block = transition_id;
        }
        if let Some(sprite) = to.and_then(|sprite| self.get_mut(sprite)) {
            sprite.transition_block = transition_id;
        }
        let transition = self.transitions.get_mut(&handle).expect("checked above");
        *transition = SpriteTransition {
            render_id,
            state: 1,
            from,
            to,
            transition_id,
            duration_ms,
            flags,
            current_frame: -1,
        };
        true
    }

    pub fn cancel_transition(&mut self, handle: SpriteTransitionHandle) -> bool {
        let Some(transition) = self.transitions.get(&handle) else {
            return false;
        };
        if transition.state == 0 {
            return false;
        }
        let from = transition.from;
        let to = transition.to;
        if let Some(sprite) = from.and_then(|sprite| self.get_mut(sprite)) {
            sprite.transition_block = 0;
            if sprite.source_name.starts_with("screen-copy:") {
                sprite.visible = false;
            }
        }
        if let Some(sprite) = to.and_then(|sprite| self.get_mut(sprite)) {
            sprite.transition_block = 0;
            if sprite.source_name.starts_with("screen-copy:") {
                sprite.visible = false;
            }
        }
        if let Some(transition) = self.transitions.get_mut(&handle) {
            *transition = SpriteTransition::default();
        }
        true
    }

    pub fn transition_state(&self, handle: SpriteTransitionHandle) -> i32 {
        self.transitions
            .get(&handle)
            .map(|transition| {
                if transition.state != 0 {
                    transition.state
                } else {
                    3
                }
            })
            .unwrap_or(3)
    }

    pub fn advance_transitions(&mut self, delta_ms: u32) {
        let handles = self.transitions.keys().copied().collect::<Vec<_>>();
        for handle in handles {
            let Some(transition) = self.transitions.get_mut(&handle) else {
                continue;
            };
            if transition.state == 0 {
                continue;
            }
            let next = transition
                .current_frame
                .max(0)
                .saturating_add(delta_ms.min(i32::MAX as u32) as i32);
            transition.current_frame = next;
            if next as u32 >= transition.duration_ms.max(1) {
                self.cancel_transition(handle);
            }
        }
    }

    pub fn tween_pos_by(
        &mut self,
        handle: SpriteHandle,
        dx: f32,
        dy: f32,
        dz: f32,
        duration_ms: u32,
    ) -> bool {
        let Some(sprite) = self.get(handle) else {
            return false;
        };
        let initial_entry = SpriteMotionEntry::from_sprite(handle, sprite);
        if duration_ms == 0 {
            return self.move_pos(handle, dx, dy, dz);
        }
        let from = sprite.position;
        let to = PalVec3::from_f32(from.x + dx, from.y + dy, from.z + dz);
        let entry = self.motion_entries.entry(handle).or_insert(initial_entry);
        entry.pos_b = to;
        entry.anim_delta_pos = Some(SpritePositionTween {
            from,
            to,
            elapsed_ms: 0,
            duration_ms,
        });
        true
    }

    /// Advances the Game.exe `SpriteMotionEntry` analogue recovered in the
    /// latest IDB.  Native `VmTask_ApplyMotion` walks `gSpriteMotionList` and
    /// commits layered position/scale/color deltas into the wrapped PalSprite;
    /// the Rust port keeps only the currently implemented named-ANI lanes here.
    pub fn advance_motion_entries(&mut self, delta_ms: u32) {
        let handles = self.motion_entries.keys().copied().collect::<Vec<_>>();
        for handle in handles {
            let Some(mut entry) = self.motion_entries.remove(&handle) else {
                continue;
            };

            if let Some(mut tween) = entry.anim_delta_pos {
                tween.elapsed_ms = tween.elapsed_ms.saturating_add(delta_ms);
                let t = (tween.elapsed_ms as f32 / tween.duration_ms.max(1) as f32).clamp(0.0, 1.0);
                let x = tween.from.x + (tween.to.x - tween.from.x) * t;
                let y = tween.from.y + (tween.to.y - tween.from.y) * t;
                let z = tween.from.z + (tween.to.z - tween.from.z) * t;
                let _ = self.set_pos_float(handle, x, y, z);
                if tween.elapsed_ms < tween.duration_ms {
                    entry.anim_delta_pos = Some(tween);
                } else {
                    entry.pos_b = tween.to;
                    entry.anim_delta_pos = None;
                }
            }

            if let Some(mut tween) = entry.anim_delta_scale {
                tween.elapsed_ms = tween.elapsed_ms.saturating_add(delta_ms);
                let t = (tween.elapsed_ms as f32 / tween.duration_ms.max(1) as f32).clamp(0.0, 1.0);
                let scale = tween.from + (tween.to - tween.from) * t;
                let _ = self.set_scale_direct(handle, scale);
                if tween.elapsed_ms < tween.duration_ms {
                    entry.anim_delta_scale = Some(tween);
                } else {
                    entry.scale_b = tween.to;
                    entry.anim_delta_scale = None;
                }
            }

            if let Some(mut tween) = entry.anim_delta_color_a {
                tween.elapsed_ms = tween.elapsed_ms.saturating_add(delta_ms);
                let t = (tween.elapsed_ms as f32 / tween.duration_ms.max(1) as f32).clamp(0.0, 1.0);
                let alpha = tween.from as f32 + (tween.to as f32 - tween.from as f32) * t;
                let alpha = alpha.round().clamp(0.0, 255.0) as u8;
                let _ = self.set_alpha_direct(handle, alpha);
                if tween.elapsed_ms < tween.duration_ms {
                    entry.anim_delta_color_a = Some(tween);
                } else {
                    entry.color_a = tween.to;
                    entry.anim_delta_color_a = None;
                }
            }

            if entry.has_active_animation() && self.sprites.contains_key(&handle) {
                self.motion_entries.insert(handle, entry);
            }
        }
    }

    pub fn transition_commands(&self) -> Vec<DrawCommand> {
        let mut commands = Vec::new();
        for transition in self.transitions.values() {
            if transition.state == 0 {
                continue;
            }
            let progress =
                transition.current_frame.max(0) as f32 / transition.duration_ms.max(1) as f32;
            let progress = progress.clamp(0.0, 1.0);
            let priority = transition
                .from
                .and_then(|handle| self.get(handle))
                .or_else(|| transition.to.and_then(|handle| self.get(handle)))
                .map(PalSprite::effective_priority)
                .unwrap_or(0);
            if let Some(from) = transition.from.and_then(|handle| self.get(handle)) {
                if let Some(mut command) = from.draw_command_transition(self) {
                    if let DrawCommand::Sprite(ref mut sprite) = command {
                        sprite.priority = priority;
                        sprite.color[3] *= 1.0 - progress;
                    }
                    commands.push(command);
                }
            }
            if let Some(to) = transition.to.and_then(|handle| self.get(handle)) {
                if let Some(mut command) = to.draw_command_transition(self) {
                    if let DrawCommand::Sprite(ref mut sprite) = command {
                        sprite.priority = priority.saturating_add(1);
                        sprite.color[3] *= progress.max(0.001);
                    }
                    commands.push(command);
                }
            }
        }
        commands
    }

    pub fn is_screen_copy(&self, handle: SpriteHandle) -> bool {
        self.get(handle)
            .is_some_and(|sprite| sprite.source_name.starts_with("screen-copy:"))
    }

    pub fn set_sprite_fx_effect(
        &mut self,
        handle: SpriteHandle,
        effect_id: u32,
        action: u32,
        value: i32,
    ) -> bool {
        if !self.sprites.contains_key(&handle) {
            return false;
        }
        match action {
            0 => {
                if let Some(effect) = self.get(handle).and_then(|sprite| sprite.fx_effect) {
                    self.fx_effects.remove(&effect);
                }
                if let Some(sprite) = self.get_mut(handle) {
                    sprite.fx_effect = None;
                }
                true
            }
            1 => {
                if self
                    .get(handle)
                    .and_then(|sprite| sprite.fx_effect)
                    .and_then(|effect| self.fx_effects.get(&effect))
                    .is_some_and(|effect| effect.active)
                {
                    return false;
                }
                let effect = self
                    .get(handle)
                    .and_then(|sprite| sprite.fx_effect)
                    .unwrap_or_else(|| {
                        let effect = SpriteFxHandle(self.next_fx_handle);
                        self.next_fx_handle = self
                            .next_fx_handle
                            .checked_add(1)
                            .expect("sprite fx handle space exhausted");
                        effect
                    });
                self.fx_effects.insert(
                    effect,
                    SpriteFxEffect {
                        effect_id,
                        state: 1,
                        value,
                        active: true,
                    },
                );
                if let Some(sprite) = self.get_mut(handle) {
                    sprite.fx_effect = Some(effect);
                }
                true
            }
            2 => {
                if let Some(effect) = self.get(handle).and_then(|sprite| sprite.fx_effect) {
                    if let Some(effect) = self.fx_effects.get_mut(&effect) {
                        effect.value = value;
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub fn release_sprite_fx_effect(&mut self, handle: SpriteHandle) -> bool {
        let Some(effect) = self.get(handle).and_then(|sprite| sprite.fx_effect) else {
            return false;
        };
        self.fx_effects.remove(&effect);
        if let Some(sprite) = self.get_mut(handle) {
            sprite.fx_effect = None;
        }
        true
    }

    pub fn sprite_fx_effect_state(&self, handle: SpriteHandle) -> i32 {
        self.get(handle)
            .and_then(|sprite| sprite.fx_effect)
            .and_then(|effect| self.fx_effects.get(&effect))
            .map(|effect| if effect.active { effect.state } else { 0 })
            .unwrap_or(0)
    }

    pub fn get(&self, handle: SpriteHandle) -> Option<&PalSprite> {
        self.sprites.get(&handle)
    }

    pub fn get_mut(&mut self, handle: SpriteHandle) -> Option<&mut PalSprite> {
        self.sprites.get_mut(&handle)
    }

    pub fn view_ctrl(&mut self, handle: SpriteHandle, visible: bool) -> bool {
        match self.get_mut(handle) {
            Some(sprite) => {
                sprite.visible = visible;
                true
            }
            None => false,
        }
    }

    pub fn view_is(&self, handle: SpriteHandle) -> bool {
        self.get(handle).is_some_and(|sprite| sprite.visible)
    }

    pub fn set_color(&mut self, handle: SpriteHandle, color: PalColor) -> bool {
        match self.get_mut(handle) {
            Some(sprite) => {
                sprite.color = color;
                true
            }
            None => false,
        }
    }

    pub fn get_width(&self, handle: SpriteHandle) -> Option<u32> {
        self.get(handle).map(|sprite| sprite.cell_size.width)
    }

    pub fn get_height(&self, handle: SpriteHandle) -> Option<u32> {
        self.get(handle).map(|sprite| sprite.cell_size.height)
    }

    pub fn set_pos(&mut self, handle: SpriteHandle, x: i32, y: i32, z: i32) -> bool {
        if let Some(entry) = self.motion_entries.get_mut(&handle) {
            entry.anim_delta_pos = None;
            if !entry.has_active_animation() {
                self.motion_entries.remove(&handle);
            }
        }
        self.set_pos_float(handle, x as f32, y as f32, z as f32)
    }

    pub fn set_pos_float(&mut self, handle: SpriteHandle, x: f32, y: f32, z: f32) -> bool {
        let priority = match self.get_mut(handle) {
            Some(sprite) => {
                trace_sprite_position_invariant("set_pos", handle, sprite, x, y, z);
                sprite.position = PalVec3::from_f32(x, y, z);
                sprite.effective_priority()
            }
            None => return false,
        };
        self.update_render_node_priority(handle, priority);
        true
    }

    pub fn set_native_projection(
        &mut self,
        handle: SpriteHandle,
        projection: Option<(f32, f32)>,
    ) -> bool {
        match self.get_mut(handle) {
            Some(sprite) => {
                sprite.native_projection = projection;
                true
            }
            None => false,
        }
    }

    pub fn position(&self, handle: SpriteHandle) -> Option<PalVec3> {
        self.get(handle).map(|sprite| sprite.position)
    }

    pub fn move_pos(&mut self, handle: SpriteHandle, dx: f32, dy: f32, dz: f32) -> bool {
        let priority = match self.get_mut(handle) {
            Some(sprite) => {
                let new_x = sprite.position.x + dx;
                let new_y = sprite.position.y + dy;
                let new_z = sprite.position.z + dz;
                trace_sprite_position_invariant("move_pos", handle, sprite, new_x, new_y, new_z);
                sprite.position.x += dx;
                sprite.position.y += dy;
                sprite.position.z += dz;
                sprite.effective_priority()
            }
            None => return false,
        };
        self.update_render_node_priority(handle, priority);
        true
    }

    pub fn set_offset_pos(&mut self, handle: SpriteHandle, x: i32, y: i32) -> bool {
        match self.get_mut(handle) {
            Some(sprite) => {
                sprite.offset = PalPoint2 { x, y };
                true
            }
            None => false,
        }
    }

    pub fn move_offset_pos(&mut self, handle: SpriteHandle, dx: i32, dy: i32) -> bool {
        match self.get_mut(handle) {
            Some(sprite) => {
                sprite.offset.x = sprite.offset.x.saturating_add(dx);
                sprite.offset.y = sprite.offset.y.saturating_add(dy);
                true
            }
            None => false,
        }
    }

    pub fn set_center_offset(&mut self, handle: SpriteHandle, x: i32, y: i32) -> bool {
        match self.get_mut(handle) {
            Some(sprite) => {
                sprite.set_center_offset(x, y);
                true
            }
            None => false,
        }
    }

    pub fn set_rect(&mut self, handle: SpriteHandle, rect: Option<PalRect>) -> bool {
        match self.get_mut(handle) {
            Some(sprite) => {
                sprite.set_rect(rect);
                true
            }
            None => false,
        }
    }

    pub fn rect_set_pos(&mut self, handle: SpriteHandle, cell_x: u16, cell_y: u16) -> bool {
        match self.get_mut(handle) {
            Some(sprite) => {
                sprite.rect_set_pos(cell_x, cell_y);
                true
            }
            None => false,
        }
    }

    pub fn set_priority(&mut self, handle: SpriteHandle, priority: i32) -> bool {
        let effective_priority = match self.get_mut(handle) {
            Some(sprite) => {
                sprite.base_priority = priority;
                sprite.effective_priority()
            }
            None => return false,
        };
        self.update_render_node_priority(handle, effective_priority);
        true
    }

    pub fn change_priority(&mut self, handle: SpriteHandle, priority: i32) -> bool {
        let node_priority = match self.get_mut(handle) {
            Some(sprite) => {
                sprite.base_priority = priority;
                priority
            }
            None => return false,
        };
        self.update_render_node_priority(handle, node_priority);
        true
    }

    pub fn set_offset_rect(&mut self, handle: SpriteHandle, width: u32, height: u32) -> bool {
        match self.get_mut(handle) {
            Some(sprite) => {
                sprite.cell_size = PalSize::new(width.max(1), height.max(1));
                true
            }
            None => false,
        }
    }

    pub fn set_option(&mut self, handle: SpriteHandle, option_type: u32) -> bool {
        match self.get_mut(handle) {
            Some(sprite) => {
                sprite.option_type = option_type;
                if option_type == 0 {
                    sprite.option_aux = None;
                } else {
                    sprite.option_aux = Some(SpriteOptionAux {
                        raw_type: option_type,
                        raw_value: 0,
                    });
                }
                true
            }
            None => false,
        }
    }

    pub fn set_scale(&mut self, handle: SpriteHandle, scale: f32) -> bool {
        if let Some(entry) = self.motion_entries.get_mut(&handle) {
            entry.anim_delta_scale = None;
            if !entry.has_active_animation() {
                self.motion_entries.remove(&handle);
            }
        }
        self.set_scale_direct(handle, scale)
    }

    fn set_scale_direct(&mut self, handle: SpriteHandle, scale: f32) -> bool {
        match self.get_mut(handle) {
            Some(sprite) => {
                sprite.scale = scale;
                true
            }
            None => false,
        }
    }

    pub fn set_alpha(&mut self, handle: SpriteHandle, alpha: u8) -> bool {
        if let Some(entry) = self.motion_entries.get_mut(&handle) {
            entry.anim_delta_color_a = None;
            if !entry.has_active_animation() {
                self.motion_entries.remove(&handle);
            }
        }
        self.set_alpha_direct(handle, alpha)
    }

    fn set_alpha_direct(&mut self, handle: SpriteHandle, alpha: u8) -> bool {
        match self.get_mut(handle) {
            Some(sprite) => {
                let raw = sprite.color.0 & 0x00FF_FFFF;
                sprite.color = PalColor::from_argb(raw | ((alpha as u32) << 24));
                true
            }
            None => false,
        }
    }

    pub fn tween_scale_to(&mut self, handle: SpriteHandle, scale: f32, duration_ms: u32) -> bool {
        let Some(sprite) = self.get(handle) else {
            return false;
        };
        let initial_entry = SpriteMotionEntry::from_sprite(handle, sprite);
        let from = sprite.scale;
        if duration_ms == 0 {
            return self.set_scale(handle, scale);
        }
        let entry = self.motion_entries.entry(handle).or_insert(initial_entry);
        entry.scale_b = scale;
        entry.anim_delta_scale = Some(SpriteScaleTween {
            from,
            to: scale,
            elapsed_ms: 0,
            duration_ms,
        });
        true
    }

    pub fn tween_alpha_to(&mut self, handle: SpriteHandle, alpha: u8, duration_ms: u32) -> bool {
        let Some(sprite) = self.get(handle) else {
            return false;
        };
        let initial_entry = SpriteMotionEntry::from_sprite(handle, sprite);
        if duration_ms == 0 {
            return self.set_alpha(handle, alpha);
        }
        let from = sprite.color.alpha();
        let entry = self.motion_entries.entry(handle).or_insert(initial_entry);
        entry.color_a = alpha;
        entry.anim_delta_color_a = Some(SpriteAlphaTween {
            from,
            to: alpha,
            elapsed_ms: 0,
            duration_ms,
        });
        true
    }

    pub fn set_rotate(&mut self, handle: SpriteHandle, angle_z: f32) -> bool {
        match self.get_mut(handle) {
            Some(sprite) => {
                sprite.rotation.z = angle_z;
                true
            }
            None => false,
        }
    }

    pub fn set_rotate_ex(&mut self, handle: SpriteHandle, x: f32, y: f32, z: f32) -> bool {
        match self.get_mut(handle) {
            Some(sprite) => {
                sprite.rotation = PalVec3::from_f32(x, y, z);
                true
            }
            None => false,
        }
    }

    pub fn set_render_mode(&mut self, handle: SpriteHandle, render_mode: PalRenderMode) -> bool {
        match self.get_mut(handle) {
            Some(sprite) => {
                sprite.render_mode = render_mode;
                true
            }
            None => false,
        }
    }

    pub fn lock(&mut self, handle: SpriteHandle) -> Option<&mut SpriteSurface> {
        let surface = {
            let sprite = self.sprites.get_mut(&handle)?;
            sprite.locked = true;
            sprite.surface
        };
        self.surfaces.get_mut(&surface)
    }

    pub fn unlock(&mut self, handle: SpriteHandle) -> bool {
        match self.sprites.get_mut(&handle) {
            Some(sprite) => {
                sprite.locked = false;
                true
            }
            None => false,
        }
    }

    pub fn set_buffer(
        &mut self,
        handle: SpriteHandle,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> bool {
        let Some(surface_id) = self.get(handle).map(|sprite| sprite.surface) else {
            return false;
        };
        let Some(surface) = self.surface_mut(surface_id) else {
            return false;
        };
        let expected = width as usize * height as usize * 4;
        if rgba.len() < expected {
            return false;
        }
        blit_rgba_to_surface(surface, x, y, width, height, rgba, BlendMode::CopyRgba)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn composite_rgba_to_sprite(
        &mut self,
        handle: SpriteHandle,
        dst_x: i32,
        dst_y: i32,
        src_width: u32,
        src_height: u32,
        rgba: &[u8],
        src_x: i32,
        src_y: i32,
        width: u32,
        height: u32,
    ) -> bool {
        if rgba.len() < src_width as usize * src_height as usize * 4 {
            return false;
        }
        let Some(surface_id) = self.get(handle).map(|sprite| sprite.surface) else {
            return false;
        };
        let Some(surface) = self.surface_mut(surface_id) else {
            return false;
        };
        blit_surface_to_surface(
            surface,
            dst_x,
            dst_y,
            (src_width, src_height, rgba),
            src_x,
            src_y,
            width,
            height,
            BlendMode::SourceOver,
        )
    }

    pub fn paint(&mut self, handle: SpriteHandle, x: i32, y: i32, color: PalColor) -> bool {
        let Some(surface_id) = self.get(handle).map(|sprite| sprite.surface) else {
            return false;
        };
        let Some(surface) = self.surface_mut(surface_id) else {
            return false;
        };
        if x < 0 || y < 0 || x as u32 >= surface.width || y as u32 >= surface.height {
            return false;
        }
        let rgba = color.to_rgba_u8();
        let idx = ((y as u32 * surface.width + x as u32) * 4) as usize;
        surface.pixels_mut()[idx..idx + 4].copy_from_slice(&rgba);
        true
    }

    pub fn mask_alpha(
        &mut self,
        dst: SpriteHandle,
        dst_x: i32,
        dst_y: i32,
        width: u32,
        height: u32,
        mask: SpriteHandle,
        mask_x: i32,
        mask_y: i32,
    ) -> bool {
        let Some(dst_surface_id) = self.get(dst).map(|sprite| sprite.surface) else {
            return false;
        };
        let Some(mask_surface_id) = self.get(mask).map(|sprite| sprite.surface) else {
            return false;
        };
        let Some(mask_pixels) = self
            .surface(mask_surface_id)
            .map(|surface| (surface.width, surface.height, surface.pixels.clone()))
        else {
            return false;
        };
        let Some(dst_surface) = self.surface_mut(dst_surface_id) else {
            return false;
        };
        mask_alpha_surface(
            dst_surface,
            dst_x,
            dst_y,
            width,
            height,
            mask_pixels,
            mask_x,
            mask_y,
        )
    }

    pub fn copy_sprite_to_sprite_rgb(
        &mut self,
        dst: SpriteHandle,
        dst_x: i32,
        dst_y: i32,
        src: SpriteHandle,
        src_x: i32,
        src_y: i32,
        width: u32,
        height: u32,
    ) -> bool {
        self.blit_sprite_to_sprite(
            dst,
            dst_x,
            dst_y,
            src,
            src_x,
            src_y,
            width,
            height,
            BlendMode::CopyRgba,
        )
    }

    pub fn mix_sprite_to_sprite(
        &mut self,
        dst: SpriteHandle,
        dst_x: i32,
        dst_y: i32,
        src: SpriteHandle,
        src_x: i32,
        src_y: i32,
        width: u32,
        height: u32,
    ) -> bool {
        self.blit_sprite_to_sprite(
            dst,
            dst_x,
            dst_y,
            src,
            src_x,
            src_y,
            width,
            height,
            BlendMode::AlphaRgb,
        )
    }

    pub fn copy_sprite_rect_to_sprite_rgb(&mut self, dst: SpriteHandle, src: SpriteHandle) -> bool {
        let Some(src_sprite) = self.get(src) else {
            return false;
        };
        self.copy_sprite_to_sprite_rgb(
            dst,
            src_sprite.position.x as i32,
            src_sprite.position.y as i32,
            src,
            src_sprite.source_rect.left,
            src_sprite.source_rect.top,
            src_sprite.cell_size.width,
            src_sprite.cell_size.height,
        )
    }

    fn blit_sprite_to_sprite(
        &mut self,
        dst: SpriteHandle,
        dst_x: i32,
        dst_y: i32,
        src: SpriteHandle,
        src_x: i32,
        src_y: i32,
        width: u32,
        height: u32,
        mode: BlendMode,
    ) -> bool {
        let Some(dst_surface_id) = self.get(dst).map(|sprite| sprite.surface) else {
            return false;
        };
        let Some(src_surface_id) = self.get(src).map(|sprite| sprite.surface) else {
            return false;
        };
        let Some((src_width, src_height, src_pixels)) = self
            .surface(src_surface_id)
            .map(|surface| (surface.width, surface.height, surface.pixels.clone()))
        else {
            return false;
        };
        let Some(dst_surface) = self.surface_mut(dst_surface_id) else {
            return false;
        };
        blit_surface_to_surface(
            dst_surface,
            dst_x,
            dst_y,
            (src_width, src_height, &src_pixels),
            src_x,
            src_y,
            width,
            height,
            mode,
        )
    }

    pub fn commands(&self) -> Vec<DrawCommand> {
        let mut nodes = self.render_nodes.values().collect::<Vec<_>>();
        // PAL keeps same-priority sprites in creation/render-entry order; later
        // entries are drawn later.  The SYSTEM menu depends on this because it
        // creates SYS_BASE at z=0 over the older TITLE_BASE at z=0.
        nodes.sort_by_key(|node| (node.priority, node.id.0));
        nodes
            .into_iter()
            .filter_map(|node| {
                self.sprites
                    .get(&node.sprite)
                    .and_then(|sprite| sprite.draw_command(self))
            })
            .collect()
    }

    pub fn render_nodes(&self) -> impl Iterator<Item = &RenderNode> {
        self.render_nodes.values()
    }

    pub fn iter_sprites(&self) -> impl Iterator<Item = (&SpriteHandle, &PalSprite)> {
        self.sprites.iter()
    }

    pub fn surface_count(&self) -> usize {
        self.surfaces.len()
    }

    pub fn render_node_count(&self) -> usize {
        self.render_nodes.len()
    }

    pub fn len(&self) -> usize {
        self.sprites.len()
    }

    pub fn motion_entry_count(&self) -> usize {
        self.motion_entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sprites.is_empty()
    }

    fn allocate_handle(&mut self) -> SpriteHandle {
        let handle = SpriteHandle(self.next_handle);
        self.next_handle = self
            .next_handle
            .checked_add(1)
            .expect("sprite handle space exhausted");
        handle
    }

    fn allocate_render_node(&mut self) -> RenderNodeId {
        let id = RenderNodeId(self.next_render_node);
        self.next_render_node = self
            .next_render_node
            .checked_add(1)
            .expect("render node id space exhausted");
        id
    }

    fn update_render_node_priority(&mut self, handle: SpriteHandle, priority: i32) {
        let Some(sprite) = self.sprites.get(&handle) else {
            return;
        };
        if let Some(node_id) = sprite.render_node {
            if let Some(node) = self.render_nodes.get_mut(&node_id) {
                node.priority = priority;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BlendMode {
    CopyRgba,
    AlphaRgb,
    SourceOver,
}

fn blit_rgba_to_surface(
    dst: &mut SpriteSurface,
    dst_x: i32,
    dst_y: i32,
    src_width: u32,
    src_height: u32,
    src_pixels: &[u8],
    mode: BlendMode,
) -> bool {
    blit_surface_to_surface(
        dst,
        dst_x,
        dst_y,
        (src_width, src_height, src_pixels),
        0,
        0,
        src_width,
        src_height,
        mode,
    )
}

fn mask_alpha_surface(
    dst: &mut SpriteSurface,
    dst_x: i32,
    dst_y: i32,
    width: u32,
    height: u32,
    mask: (u32, u32, Arc<[u8]>),
    mask_x: i32,
    mask_y: i32,
) -> bool {
    let (mask_width, mask_height, mask_pixels) = mask;
    let Some(region) = ClipRegion::new(
        dst_x,
        dst_y,
        mask_x,
        mask_y,
        width,
        height,
        dst.width,
        dst.height,
        mask_width,
        mask_height,
    ) else {
        return false;
    };

    let dst_width = dst.width;
    let dst_pixels = dst.pixels_mut();
    for row in 0..region.height {
        let dst_row = region.dst_y + row;
        let mask_row = region.src_y + row;
        for col in 0..region.width {
            let dst_col = region.dst_x + col;
            let mask_col = region.src_x + col;
            let dst_idx = ((dst_row * dst_width + dst_col) * 4 + 3) as usize;
            let mask_idx = ((mask_row * mask_width + mask_col) * 4) as usize;
            dst_pixels[dst_idx] =
                (((u16::from(dst_pixels[dst_idx])) * u16::from(mask_pixels[mask_idx])) >> 8) as u8;
        }
    }
    true
}

fn blit_surface_to_surface(
    dst: &mut SpriteSurface,
    dst_x: i32,
    dst_y: i32,
    src: (u32, u32, &[u8]),
    src_x: i32,
    src_y: i32,
    width: u32,
    height: u32,
    mode: BlendMode,
) -> bool {
    let (src_width, src_height, src_pixels) = src;
    let Some(region) = ClipRegion::new(
        dst_x, dst_y, src_x, src_y, width, height, dst.width, dst.height, src_width, src_height,
    ) else {
        return false;
    };

    let dst_width = dst.width;
    let dst_pixels = dst.pixels_mut();
    for row in 0..region.height {
        let dst_row = region.dst_y + row;
        let src_row = region.src_y + row;
        for col in 0..region.width {
            let dst_col = region.dst_x + col;
            let src_col = region.src_x + col;
            let dst_idx = ((dst_row * dst_width + dst_col) * 4) as usize;
            let src_idx = ((src_row * src_width + src_col) * 4) as usize;
            blend_pixel(
                &mut dst_pixels[dst_idx..dst_idx + 4],
                &src_pixels[src_idx..src_idx + 4],
                mode,
            );
        }
    }
    true
}

fn blend_pixel(dst: &mut [u8], src: &[u8], mode: BlendMode) {
    match mode {
        BlendMode::CopyRgba => dst.copy_from_slice(src),
        BlendMode::AlphaRgb => {
            let alpha = u16::from(src[3]);
            if alpha == 0 {
                return;
            }
            for channel in 0..3 {
                let src_value = u16::from(src[channel]);
                let dst_value = u16::from(dst[channel]);
                dst[channel] = (((dst_value * (255 - alpha)) + (src_value * alpha)) >> 8) as u8;
            }
        }
        BlendMode::SourceOver => {
            let src_alpha = u32::from(src[3]);
            if src_alpha == 0 {
                return;
            }
            let dst_alpha = u32::from(dst[3]);
            let out_alpha = src_alpha + (dst_alpha * (255 - src_alpha) + 127) / 255;
            for channel in 0..3 {
                let numerator = u32::from(src[channel]) * src_alpha * 255
                    + u32::from(dst[channel]) * dst_alpha * (255 - src_alpha);
                dst[channel] = ((numerator + out_alpha * 127) / (out_alpha * 255)) as u8;
            }
            dst[3] = out_alpha as u8;
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ClipRegion {
    dst_x: u32,
    dst_y: u32,
    src_x: u32,
    src_y: u32,
    width: u32,
    height: u32,
}

impl ClipRegion {
    #[allow(clippy::too_many_arguments)]
    fn new(
        dst_x: i32,
        dst_y: i32,
        src_x: i32,
        src_y: i32,
        width: u32,
        height: u32,
        dst_width: u32,
        dst_height: u32,
        src_width: u32,
        src_height: u32,
    ) -> Option<Self> {
        let mut dst_x = i64::from(dst_x);
        let mut dst_y = i64::from(dst_y);
        let mut src_x = i64::from(src_x);
        let mut src_y = i64::from(src_y);
        let mut width = i64::from(width);
        let mut height = i64::from(height);

        if width <= 0 || height <= 0 {
            return None;
        }
        if src_x < 0 {
            dst_x -= src_x;
            width += src_x;
            src_x = 0;
        }
        if src_y < 0 {
            dst_y -= src_y;
            height += src_y;
            src_y = 0;
        }
        if dst_x < 0 {
            src_x -= dst_x;
            width += dst_x;
            dst_x = 0;
        }
        if dst_y < 0 {
            src_y -= dst_y;
            height += dst_y;
            dst_y = 0;
        }

        width = width
            .min(i64::from(dst_width).saturating_sub(dst_x))
            .min(i64::from(src_width).saturating_sub(src_x));
        height = height
            .min(i64::from(dst_height).saturating_sub(dst_y))
            .min(i64::from(src_height).saturating_sub(src_y));
        if width <= 0 || height <= 0 {
            return None;
        }

        Some(Self {
            dst_x: dst_x as u32,
            dst_y: dst_y as u32,
            src_x: src_x as u32,
            src_y: src_y as u32,
            width: width as u32,
            height: height as u32,
        })
    }
}

#[derive(Clone, Debug)]
pub struct SpriteDesc {
    pub kind: SpriteKind,
    pub texture_id: SceneTextureId,
    pub texture_width: u32,
    pub texture_height: u32,
    pub cell_width: u32,
    pub cell_height: u32,
    pub source_rect: Option<PalRect>,
    pub position: PalVec3,
    pub offset: PalPoint2,
    pub center_offset: PalPoint2,
    pub visible: bool,
    pub color: PalColor,
    pub base_priority: i32,
    pub scale: f32,
    pub center_scale: bool,
    pub rotation: PalVec3,
    pub render_mode: PalRenderMode,
    pub option_type: u32,
    pub info_extra: u32,
    pub source_name: String,
    pub native_projection: Option<(f32, f32)>,
    pub smooth_upscale: bool,
}

impl SpriteDesc {
    pub fn new(texture_id: SceneTextureId, texture_width: u32, texture_height: u32) -> Self {
        Self {
            kind: SpriteKind::Static,
            texture_id,
            texture_width,
            texture_height,
            cell_width: texture_width,
            cell_height: texture_height,
            source_rect: None,
            position: PalVec3::default(),
            offset: PalPoint2::default(),
            center_offset: PalPoint2::default(),
            visible: false,
            color: PalColor::WHITE,
            base_priority: 0,
            scale: 1.0,
            center_scale: true,
            rotation: PalVec3::default(),
            render_mode: PalRenderMode::DEFAULT,
            option_type: 0,
            info_extra: 0,
            source_name: String::new(),
            native_projection: None,
            smooth_upscale: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PalSprite {
    pub handle: SpriteHandle,
    pub kind: SpriteKind,
    pub surface: SpriteSurfaceId,
    pub render_node: Option<RenderNodeId>,
    pub option_aux: Option<SpriteOptionAux>,
    pub transition_block: u32,
    pub fx_effect: Option<SpriteFxHandle>,
    pub locked: bool,
    pub visible: bool,
    pub source_rect: PalRect,
    pub color: PalColor,
    pub position: PalVec3,
    pub offset: PalPoint2,
    pub texture_size: PalSize,
    pub cell_size: PalSize,
    pub center_offset: PalPoint2,
    pub base_priority: i32,
    pub scale: f32,
    pub center_scale: bool,
    pub rotation: PalVec3,
    pub render_mode: PalRenderMode,
    pub option_type: u32,
    pub info_extra: u32,
    pub source_name: String,
    pub native_projection: Option<(f32, f32)>,
    pub smooth_upscale: bool,
}

impl PalSprite {
    pub fn new(handle: SpriteHandle, render_node: RenderNodeId, desc: SpriteDesc) -> Self {
        let texture_size = PalSize::new(desc.texture_width.max(1), desc.texture_height.max(1));
        let cell_size = PalSize::new(desc.cell_width.max(1), desc.cell_height.max(1));
        let source_rect = desc
            .source_rect
            .unwrap_or_else(|| PalRect::new(0, 0, cell_size.width as i32, cell_size.height as i32));
        Self {
            handle,
            kind: desc.kind,
            surface: SpriteSurfaceId(desc.texture_id.0),
            render_node: Some(render_node),
            option_aux: None,
            transition_block: 0,
            fx_effect: None,
            locked: false,
            visible: desc.visible,
            source_rect,
            color: desc.color,
            position: desc.position,
            offset: desc.offset,
            texture_size,
            cell_size,
            center_offset: desc.center_offset,
            base_priority: desc.base_priority,
            scale: desc.scale,
            center_scale: desc.center_scale,
            rotation: desc.rotation,
            render_mode: desc.render_mode,
            option_type: desc.option_type,
            info_extra: desc.info_extra,
            source_name: desc.source_name,
            native_projection: desc.native_projection,
            smooth_upscale: desc.smooth_upscale,
        }
    }

    pub fn info(&self) -> PalSpriteInfo {
        PalSpriteInfo {
            visible: self.visible,
            color: self.color,
            position: self.position,
            offset: self.offset,
            texture_size: self.texture_size,
            cell_size: self.cell_size,
            center_offset: self.center_offset,
            priority: self.base_priority,
            scale: self.scale,
            center_scale: self.center_scale,
            rotation: self.rotation,
            render_mode: self.render_mode,
            extra: self.info_extra,
            source_name: self.source_name.clone(),
            native_projection: self.native_projection,
            smooth_upscale: self.smooth_upscale,
        }
    }

    pub fn set_info(&mut self, info: PalSpriteInfo) {
        self.visible = info.visible;
        self.color = info.color;
        self.position = info.position;
        self.offset = info.offset;
        self.texture_size = info.texture_size.clamped_nonzero();
        self.cell_size = info.cell_size.clamped_nonzero();
        self.center_offset = info.center_offset;
        self.base_priority = info.priority;
        self.scale = info.scale;
        self.center_scale = info.center_scale;
        self.rotation = info.rotation;
        self.render_mode = info.render_mode;
        self.info_extra = info.extra;
        self.source_name = info.source_name;
        self.native_projection = info.native_projection;
        self.smooth_upscale = info.smooth_upscale;
    }

    pub fn frame_count(&self, axis: PalAnimationAxis) -> u16 {
        let count = match axis {
            PalAnimationAxis::Horizontal => self.texture_size.width / self.cell_size.width.max(1),
            PalAnimationAxis::Vertical => self.texture_size.height / self.cell_size.height.max(1),
        };
        count.clamp(1, u16::MAX as u32) as u16
    }

    pub fn set_rect(&mut self, rect: Option<PalRect>) {
        self.source_rect = rect.unwrap_or_else(|| {
            PalRect::new(
                0,
                0,
                self.texture_size.width as i32,
                self.texture_size.height as i32,
            )
        });
    }

    pub fn rect_set_pos(&mut self, cell_x: u16, cell_y: u16) {
        let mut left = u32::from(cell_x).saturating_mul(self.cell_size.width);
        let mut top = u32::from(cell_y).saturating_mul(self.cell_size.height);
        if left >= self.texture_size.width {
            left = self.texture_size.width.saturating_sub(self.cell_size.width);
        }
        if top >= self.texture_size.height {
            top = self
                .texture_size
                .height
                .saturating_sub(self.cell_size.height);
        }
        self.source_rect = PalRect::new(
            left as i32,
            top as i32,
            left.saturating_add(self.cell_size.width) as i32,
            top.saturating_add(self.cell_size.height) as i32,
        );
    }

    pub fn set_center_offset(&mut self, x: i32, y: i32) {
        self.center_offset = PalPoint2 { x: -x, y: -y };
    }

    pub fn effective_position(&self) -> PalPoint2 {
        PalPoint2 {
            x: self.offset.x.saturating_add(self.position.x as i32),
            y: self.offset.y.saturating_add(self.position.y as i32),
        }
    }

    pub fn effective_priority(&self) -> i32 {
        self.base_priority.saturating_add(self.position.z as i32)
    }

    pub fn draw_command(&self, sprites: &SpriteSystem) -> Option<DrawCommand> {
        if self.transition_block != 0 || !self.visible || self.locked || self.color.alpha() == 0 {
            return None;
        }
        self.draw_command_inner(sprites)
    }

    fn draw_command_transition(&self, sprites: &SpriteSystem) -> Option<DrawCommand> {
        if !self.visible || self.locked || self.color.alpha() == 0 {
            return None;
        }
        self.draw_command_inner(sprites)
    }

    fn draw_command_inner(&self, sprites: &SpriteSystem) -> Option<DrawCommand> {
        let surface = sprites.surface(self.surface)?;
        let dst_pos = self.effective_position();
        let width = self.source_rect.width() as f32;
        let height = self.source_rect.height() as f32;
        let scale = self.scale.max(0.0);
        let rotation_z = self.rotation.z.rem_euclid(512.0);
        let draw_scale = pal_scratch_clamped_scale(width, height, scale);
        let mut dst = if self.kind == SpriteKind::SolidLayer {
            RectF::new(0.0, 0.0, surface.width as f32, surface.height as f32)
        } else if rotation_z != 0.0 {
            // PAL.dll `sub_1028D740` routes rotated sprites through a temporary
            // square work buffer. The work-buffer side is based on the source
            // diagonal times sprite.scale and is clamped to 4096 before the PAL
            // blit. `sub_1028EB30` samples the source into that square around
            // the source center; when rendering the visible source rectangle
            // directly, the equivalent content top-left is still the ordinary
            // centered rectangular scale offset.
            RectF::new(
                dst_pos.x as f32 - ((width * draw_scale) - width) * 0.5,
                dst_pos.y as f32 - ((height * draw_scale) - height) * 0.5,
                width * draw_scale,
                height * draw_scale,
            )
        } else if self.center_scale && (scale - 1.0).abs() > f32::EPSILON {
            // PAL.dll `sub_1028D740` does not scale from the top-left corner.
            // For scale-only sprites it creates a centered square scratch, but
            // the source pixels are centered within that scratch by
            // `sub_1028EB30`; drawing the visible source rect directly must use
            // the visible scaled width/height offset.
            RectF::new(
                dst_pos.x as f32 - ((width * draw_scale) - width) * 0.5,
                dst_pos.y as f32 - ((height * draw_scale) - height) * 0.5,
                width * draw_scale,
                height * draw_scale,
            )
        } else {
            RectF::new(
                dst_pos.x as f32,
                dst_pos.y as f32,
                width * draw_scale,
                height * draw_scale,
            )
        };
        if let Some((project_x, project_y)) = self.native_projection {
            dst = RectF::new(
                dst.x * project_x,
                dst.y * project_y,
                dst.w * project_x,
                dst.h * project_y,
            );
        }
        if !dst.is_drawable() {
            return None;
        }
        trace_sprite_draw_invariant(self, dst, surface.width(), surface.height());
        if self.source_name.starts_with("ST")
            || self.source_name.starts_with("BK")
            || self.source_name.starts_with("EV")
            || self.source_name.starts_with("FA")
        {
            log::debug!(
                "[trace-sprite-draw] handle={} name={} pos=({},{},{}) src={}x{} scale={:.3} native_projection={:?} dst=({:.1},{:.1},{:.1},{:.1}) alpha={} priority={}",
                self.handle.0,
                self.source_name,
                dst_pos.x,
                dst_pos.y,
                self.position.z,
                width,
                height,
                scale,
                self.native_projection,
                dst.x,
                dst.y,
                dst.w,
                dst.h,
                self.color.alpha(),
                self.effective_priority()
            );
        }
        let tex_w = surface.width().max(1) as f32;
        let tex_h = surface.height().max(1) as f32;
        let src = RectF::new(
            self.source_rect.left as f32 / tex_w,
            self.source_rect.top as f32 / tex_h,
            self.source_rect.width() as f32 / tex_w,
            self.source_rect.height() as f32 / tex_h,
        );
        Some(DrawCommand::Sprite(SpriteDraw {
            texture_id: surface.texture_id,
            smooth_upscale: self.smooth_upscale,
            priority: self.effective_priority(),
            dst,
            src,
            source_rect: [
                self.source_rect.left,
                self.source_rect.top,
                self.source_rect.right,
                self.source_rect.bottom,
            ],
            texture_size: [self.texture_size.width, self.texture_size.height],
            cell_size: [self.cell_size.width, self.cell_size.height],
            position: [self.position.x, self.position.y, self.position.z],
            offset: [self.offset.x, self.offset.y],
            color: self.color.to_rgba_f32(),
            scale: self.scale,
            rotation: [self.rotation.x, self.rotation.y, self.rotation.z],
            center_offset: [self.center_offset.x as f32, self.center_offset.y as f32],
            render_mode: self.render_mode.raw(),
        }))
    }
}

fn pal_scratch_clamped_scale(width: f32, height: f32, scale: f32) -> f32 {
    if scale <= 0.0 || !scale.is_finite() {
        return 0.0;
    }
    let diagonal = (width.mul_add(width, height * height)).sqrt();
    if diagonal <= f32::EPSILON || !diagonal.is_finite() {
        return scale;
    }
    let scaled_side = diagonal * scale;
    if scaled_side <= 4096.0 {
        scale
    } else {
        4096.0 / diagonal
    }
}

fn trace_sprite_position_invariant(
    op: &str,
    handle: SpriteHandle,
    sprite: &PalSprite,
    x: f32,
    y: f32,
    z: f32,
) {
    if x.abs() <= 4096.0 && y.abs() <= 4096.0 && sprite.scale > 0.0 && sprite.scale <= 8.0 {
        return;
    }
    log::warn!(
        "[trace-sprite-invariant] op={op} handle={} source={:?} old_pos=({:.0},{:.0},{:.0}) new_pos=({x:.0},{y:.0},{z:.0}) scale={:.3} src_rect=({},{},{},{}) tex={}x{} cell={}x{}",
        handle.0,
        sprite.source_name,
        sprite.position.x,
        sprite.position.y,
        sprite.position.z,
        sprite.scale,
        sprite.source_rect.left,
        sprite.source_rect.top,
        sprite.source_rect.right,
        sprite.source_rect.bottom,
        sprite.texture_size.width,
        sprite.texture_size.height,
        sprite.cell_size.width,
        sprite.cell_size.height,
    );
}

fn trace_sprite_draw_invariant(
    sprite: &PalSprite,
    dst: RectF,
    surface_width: u32,
    surface_height: u32,
) {
    let rect_outside = sprite.source_rect.left < 0
        || sprite.source_rect.top < 0
        || sprite.source_rect.right > surface_width as i32
        || sprite.source_rect.bottom > surface_height as i32;
    let bad_source =
        sprite.source_rect.width() <= 0 || sprite.source_rect.height() <= 0 || rect_outside;
    let bad_dst = dst.x.abs() > 4096.0 || dst.y.abs() > 4096.0 || dst.w <= 0.0 || dst.h <= 0.0;
    let bad_scale =
        sprite.kind != SpriteKind::SolidLayer && (sprite.scale <= 0.0 || sprite.scale > 8.0);
    if !(bad_source || bad_dst || bad_scale) {
        return;
    }
    log::warn!(
        "[trace-sprite-invariant] op=draw source={:?} dst=({:.0},{:.0},{:.0}x{:.0}) pos=({:.0},{:.0},{:.0}) offset=({},{}) scale={:.3} src_rect=({},{},{},{}) surface={}x{} tex={}x{} cell={}x{} bad_source={} bad_dst={} bad_scale={}",
        sprite.source_name,
        dst.x,
        dst.y,
        dst.w,
        dst.h,
        sprite.position.x,
        sprite.position.y,
        sprite.position.z,
        sprite.offset.x,
        sprite.offset.y,
        sprite.scale,
        sprite.source_rect.left,
        sprite.source_rect.top,
        sprite.source_rect.right,
        sprite.source_rect.bottom,
        surface_width,
        surface_height,
        sprite.texture_size.width,
        sprite.texture_size.height,
        sprite.cell_size.width,
        sprite.cell_size.height,
        bad_source,
        bad_dst,
        bad_scale,
    );
}

#[derive(Clone, Debug)]
pub struct PalSpriteInfo {
    pub visible: bool,
    pub color: PalColor,
    pub position: PalVec3,
    pub offset: PalPoint2,
    pub texture_size: PalSize,
    pub cell_size: PalSize,
    pub center_offset: PalPoint2,
    pub priority: i32,
    pub scale: f32,
    pub center_scale: bool,
    pub rotation: PalVec3,
    pub render_mode: PalRenderMode,
    pub extra: u32,
    pub source_name: String,
    pub native_projection: Option<(f32, f32)>,
    pub smooth_upscale: bool,
}

#[derive(Clone, Debug)]
pub struct SpriteSurface {
    pub id: SpriteSurfaceId,
    pub texture_id: SceneTextureId,
    pub generation: u64,
    pub width: u32,
    pub height: u32,
    pub format: SceneTextureFormat,
    pub pixels: Arc<[u8]>,
    pub dirty: bool,
}

impl SpriteSurface {
    pub fn from_scene_texture(texture: SceneTexture) -> Self {
        Self {
            id: SpriteSurfaceId(texture.id.0),
            texture_id: texture.id,
            generation: texture.generation,
            width: texture.width,
            height: texture.height,
            format: texture.format,
            pixels: texture.pixels,
            dirty: false,
        }
    }

    pub fn rgba8(
        id: SpriteSurfaceId,
        generation: u64,
        width: u32,
        height: u32,
        pixels: Vec<u8>,
    ) -> anyhow::Result<Self> {
        let expected = width as usize * height as usize * 4;
        if pixels.len() != expected {
            anyhow::bail!(
                "RGBA sprite surface {:?} has {} bytes, expected {} for {}x{}",
                id,
                pixels.len(),
                expected,
                width,
                height
            );
        }
        Ok(Self {
            id,
            texture_id: SceneTextureId(id.0),
            generation,
            width,
            height,
            format: SceneTextureFormat::Rgba8,
            pixels: Arc::from(pixels),
            dirty: false,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.generation = self.generation.saturating_add(1).max(1);
    }

    pub fn pixels_mut(&mut self) -> &mut [u8] {
        self.mark_dirty();
        Arc::make_mut(&mut self.pixels)
    }

    pub fn to_scene_texture(&self) -> SceneTexture {
        SceneTexture {
            id: self.texture_id,
            generation: self.generation,
            width: self.width,
            height: self.height,
            format: self.format,
            pixels: self.pixels.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RenderNode {
    pub id: RenderNodeId,
    pub sprite: SpriteHandle,
    pub priority: i32,
}

impl RenderNode {
    pub const fn new(id: RenderNodeId, sprite: SpriteHandle, priority: i32) -> Self {
        Self {
            id,
            sprite,
            priority,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpriteKind {
    Static,
    SolidLayer,
    MSprite { decoder: MSpriteHandle },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MSpriteDecoderHandle(pub u32);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpriteOptionAux {
    pub raw_type: u32,
    pub raw_value: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SpriteFxHandle(pub u32);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SpriteTransition {
    pub render_id: u32,
    pub state: i32,
    pub from: Option<SpriteHandle>,
    pub to: Option<SpriteHandle>,
    pub transition_id: u32,
    pub duration_ms: u32,
    pub flags: u32,
    pub current_frame: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SpriteFxEffect {
    pub effect_id: u32,
    pub state: i32,
    pub value: i32,
    pub active: bool,
}

/// Portable subset of Game.exe `SpriteMotionEntry`.
///
/// Latest IDB evidence declares the native entry as 688 bytes / 172 DWORDs:
/// `[0] PalSprite *sprite`, base/final position and rotation lanes, scale,
/// color/alpha lanes, an animation-delta block at byte offsets 376..451, and
/// `[171] next` for `gSpriteMotionList`.  This Rust struct intentionally
/// stores only the lanes currently implemented by named ANI playback.  Keeping
/// them together prevents the old port mistake of treating move/scale/alpha as
/// unrelated sprite-local tweens.
#[derive(Clone, Copy, Debug)]
pub struct SpriteMotionEntry {
    pub sprite: SpriteHandle,
    pub pos_b: PalVec3,
    pub scale_b: f32,
    pub color_a: u8,
    anim_delta_pos: Option<SpritePositionTween>,
    anim_delta_scale: Option<SpriteScaleTween>,
    anim_delta_color_a: Option<SpriteAlphaTween>,
}

impl SpriteMotionEntry {
    fn from_sprite(sprite: SpriteHandle, state: &PalSprite) -> Self {
        Self {
            sprite,
            pos_b: state.position,
            scale_b: state.scale,
            color_a: state.color.alpha(),
            anim_delta_pos: None,
            anim_delta_scale: None,
            anim_delta_color_a: None,
        }
    }

    fn has_active_animation(&self) -> bool {
        self.anim_delta_pos.is_some()
            || self.anim_delta_scale.is_some()
            || self.anim_delta_color_a.is_some()
    }
}

#[derive(Clone, Copy, Debug)]
struct SpritePositionTween {
    from: PalVec3,
    to: PalVec3,
    elapsed_ms: u32,
    duration_ms: u32,
}

#[derive(Clone, Copy, Debug)]
struct SpriteScaleTween {
    from: f32,
    to: f32,
    elapsed_ms: u32,
    duration_ms: u32,
}

#[derive(Clone, Copy, Debug)]
struct SpriteAlphaTween {
    from: u8,
    to: u8,
    elapsed_ms: u32,
    duration_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PalAnimationAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PalAnimationFlags(u8);

impl PalAnimationFlags {
    pub const HORIZONTAL: Self = Self(0x01);
    pub const VERTICAL: Self = Self(0x02);

    pub fn from_original(flags: u8) -> Self {
        let masked = flags & 0x03;
        if masked == 0 {
            Self::HORIZONTAL
        } else {
            Self(masked)
        }
    }

    pub fn axis(self) -> PalAnimationAxis {
        if (self.0 & Self::HORIZONTAL.0) != 0 {
            PalAnimationAxis::Horizontal
        } else if (self.0 & Self::VERTICAL.0) != 0 {
            PalAnimationAxis::Vertical
        } else {
            PalAnimationAxis::Horizontal
        }
    }

    pub fn raw(self) -> u8 {
        self.0
    }
}

impl From<PalAnimationAxis> for PalAnimationFlags {
    fn from(axis: PalAnimationAxis) -> Self {
        match axis {
            PalAnimationAxis::Horizontal => Self::HORIZONTAL,
            PalAnimationAxis::Vertical => Self::VERTICAL,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub struct PalPoint2 {
    pub x: i32,
    pub y: i32,
}

impl PalPoint2 {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct PalVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl PalVec3 {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
            z: z as f32,
        }
    }

    pub const fn from_f32(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

pub type PalPoint3 = PalVec3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PalSize {
    pub width: u32,
    pub height: u32,
}

impl PalSize {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn clamped_nonzero(self) -> Self {
        Self {
            width: self.width.max(1),
            height: self.height.max(1),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PalRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl PalRect {
    pub const fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn width(self) -> i32 {
        self.right.saturating_sub(self.left)
    }

    pub fn height(self) -> i32 {
        self.bottom.saturating_sub(self.top)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PalColor(pub u32);

impl PalColor {
    pub const WHITE: Self = Self(0xFFFF_FFFF);

    pub const fn from_argb(raw: u32) -> Self {
        Self(raw)
    }

    pub fn to_rgba_f32(self) -> [f32; 4] {
        let a = ((self.0 >> 24) & 0xFF) as f32 / 255.0;
        let r = ((self.0 >> 16) & 0xFF) as f32 / 255.0;
        let g = ((self.0 >> 8) & 0xFF) as f32 / 255.0;
        let b = (self.0 & 0xFF) as f32 / 255.0;
        [r, g, b, a]
    }

    pub fn to_rgba_u8(self) -> [u8; 4] {
        [
            ((self.0 >> 16) & 0xFF) as u8,
            ((self.0 >> 8) & 0xFF) as u8,
            (self.0 & 0xFF) as u8,
            ((self.0 >> 24) & 0xFF) as u8,
        ]
    }

    pub const fn alpha(self) -> u8 {
        ((self.0 >> 24) & 0xFF) as u8
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PalRenderMode(u32);

impl PalRenderMode {
    pub const DEFAULT: Self = Self(1);

    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_sprite_pixels_duplicates_the_source_surface() {
        let mut sprites = SpriteSystem::new();
        let pixels = vec![9, 8, 7, 255, 1, 2, 3, 128];
        let source = sprites
            .create_rgba_sprite(
                2,
                1,
                pixels.clone(),
                PalVec3::from_f32(12.0, 34.0, 1.0),
                4,
                "source",
            )
            .expect("source sprite");
        let copied = sprites
            .copy_sprite_pixels(source, i32::MIN / 2, "backbuffer")
            .expect("copied sprite");
        let copied_sprite = sprites.get(copied).expect("copied handle");
        assert_eq!(copied_sprite.position.x, 12.0);
        assert_eq!(copied_sprite.position.y, 34.0);
        let texture = sprites
            .surface(copied_sprite.surface)
            .expect("copied surface")
            .to_scene_texture();
        assert_eq!(texture.pixels.as_ref(), pixels.as_slice());
        assert_ne!(copied, source);
    }
}
