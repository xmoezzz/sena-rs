use crate::animation::{
    PalSequenceAnimation, PalSequenceAnimationDesc, PalSheetAnimation, PalSheetAnimationDesc,
};
use crate::input::PalInputState;
use crate::sprite::SpriteSystem;

/// PAL task pool capacity. Matches the original fixed 512-slot pool at dword_103E1C8C.
pub const TASK_POOL_CAPACITY: usize = 512;

/// Blocking task flag (bit 16). When set on a task, normal processing of later siblings stops
/// after this task is processed. Only pending-free cleanup continues for remaining siblings.
/// PalWait and PalWaitTime use this flag.
pub const BLOCKING_FLAG: u32 = 0x10000;

fn debug_vm_wait_trace_enabled() -> bool {
    std::env::var("DEBUG_VM")
        .ok()
        .as_deref()
        .is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
}

/// Handle to a task node. Encodes pool index in low 16 bits and generation counter
/// in high 16 bits so stale handles to reused slots are detected.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TaskHandle(pub u32);

impl TaskHandle {
    fn pool_index(self) -> usize {
        (self.0 & 0xFFFF) as usize
    }

    fn generation(self) -> u16 {
        (self.0 >> 16) as u16
    }

    fn encode(index: usize, generation: u16) -> Self {
        debug_assert!(index < TASK_POOL_CAPACITY);
        Self((generation as u32) << 16 | index as u32)
    }
}

/// Life-cycle state of a task node. Matches original PAL values 0..3.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum TaskState {
    #[default]
    Free = 0,
    Active = 1,
    /// Marked for release; release callback runs on the next process() call.
    PendingFree = 2,
    ChildSentinel = 3,
}

/// Known task payload types. Each variant mirrors the original PAL task data layout.
pub enum TaskKind {
    /// Slot is free; no payload.
    Free,
    /// Sheet (sprite-strip) animation. Corresponds to PalAnimation task data (0x1C bytes).
    AnimationSheet(PalSheetAnimation),
    /// Sequence (frame-record) animation. Corresponds to PalAnimationEx task data (0x18 bytes).
    AnimationSequence(PalSequenceAnimation),
    /// Frame-count wait. `remaining == -1` means wait forever.
    /// Corresponds to PalWait task data (4 bytes, remaining_count field).
    WaitFrame { remaining: i32 },
    /// Time-based wait. Corresponds to PalWaitTime task data (8 bytes).
    WaitTime { duration_ms: u32, start_ms: u32 },
    /// Input-push wait. Frees itself when any key or mouse button is pushed.
    WaitClick,
    /// Input-push wait with a timeout. Mirrors wait_click(duration >= 0).
    /// `voice_gated` marks the ADV auto-mode hold: the timeout only finishes
    /// the wait once voice playback has also ended (koikake.exe 0x42F9D0
    /// requires the PalSoundPlayIs voice check before advancing).
    WaitClickOrTime {
        duration_ms: u32,
        start_ms: u32,
        voice_gated: bool,
    },
    /// Generic PAL task node with modeled metadata but no Rust callback body yet.
    Raw,
    /// A task type not yet mapped to a Rust payload. Logs a warning each update and
    /// never frees itself. Callers must free it explicitly.
    Unsupported { name: &'static str },
}

impl Default for TaskKind {
    fn default() -> Self {
        TaskKind::Free
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TaskUpdateOutcome {
    Continue,
    FreeSelf,
}

struct TaskNode {
    state: TaskState,
    kind: TaskKind,
    flags: u32,
    process: i32,
    end_process: i32,
    message: i32,
    task_data: Vec<u8>,
    /// Incremented each time the slot is reused, to invalidate stale handles.
    generation: u16,
    /// Ordered list of child task pool indices.
    children: Vec<usize>,
    /// Index of parent task in pool. None = root list.
    parent: Option<usize>,
    name: &'static str,
}

impl Default for TaskNode {
    fn default() -> Self {
        Self {
            state: TaskState::Free,
            kind: TaskKind::Free,
            flags: 0,
            process: 0,
            end_process: 0,
            message: 0,
            task_data: Vec::new(),
            generation: 0,
            children: Vec::new(),
            parent: None,
            name: "",
        }
    }
}

/// PAL task scheduler. Owns a fixed pool of 512 task nodes and a root-level task list.
///
/// Processing is depth-first (children before siblings), matching sub_10264230.
/// Tasks with BLOCKING_FLAG stop normal sibling processing after they are handled;
/// only pending-free cleanup continues for remaining siblings.
pub struct TaskSystem {
    nodes: Vec<TaskNode>,
    root_children: Vec<usize>,
    /// PAL cached time in milliseconds. Updated once per engine frame by set_pal_time().
    pub pal_time_ms: u32,
    /// Next pool scan start index (circular scan for allocation).
    scan_start: usize,
}

impl TaskSystem {
    pub fn new() -> Self {
        let mut nodes = Vec::with_capacity(TASK_POOL_CAPACITY);
        for _ in 0..TASK_POOL_CAPACITY {
            nodes.push(TaskNode::default());
        }
        Self {
            nodes,
            root_children: Vec::new(),
            pal_time_ms: 0,
            scan_start: 0,
        }
    }

    /// Update the PAL cached time. Must be called once per frame before process().
    pub fn set_pal_time(&mut self, ms: u32) {
        self.pal_time_ms = ms;
    }

    /// True if the handle refers to a currently live (active or pending-free) task.
    pub fn is_alive(&self, handle: TaskHandle) -> bool {
        let idx = handle.pool_index();
        if idx >= TASK_POOL_CAPACITY {
            return false;
        }
        let node = &self.nodes[idx];
        node.state != TaskState::Free && node.generation == handle.generation()
    }

    /// Allocate a free slot from the pool. Returns None if all 512 slots are occupied.
    fn alloc_node(&mut self) -> Option<usize> {
        let start = self.scan_start % TASK_POOL_CAPACITY;
        for offset in 0..TASK_POOL_CAPACITY {
            let idx = (start + offset) % TASK_POOL_CAPACITY;
            if self.nodes[idx].state == TaskState::Free {
                self.scan_start = (idx + 1) % TASK_POOL_CAPACITY;
                return Some(idx);
            }
        }
        None
    }

    fn insert_task(&mut self, idx: usize, flags: u32, parent: Option<usize>, name: &'static str) {
        self.nodes[idx].state = TaskState::Active;
        self.nodes[idx].flags = flags;
        if self.nodes[idx].process == 0 {
            self.nodes[idx].process = 1;
        }
        self.nodes[idx].parent = parent;
        self.nodes[idx].name = name;
        self.nodes[idx].children.clear();
        match parent {
            None => self.root_children.push(idx),
            Some(p) => self.nodes[p].children.push(idx),
        }
    }

    /// Create a raw PAL task node with zeroed task data. This mirrors the generic
    /// PalTaskCreate path for callers whose callback body is not modeled yet.
    pub fn create_raw_task(
        &mut self,
        process: i32,
        end_process: i32,
        data_size: usize,
        flags: u32,
        parent: Option<TaskHandle>,
        name: &'static str,
    ) -> Option<TaskHandle> {
        if process == 0 {
            return None;
        }
        let idx = self.alloc_node()?;
        let parent_idx = parent.filter(|h| self.is_alive(*h)).map(|h| h.pool_index());
        let gen = self.nodes[idx].generation;
        self.nodes[idx].kind = TaskKind::Raw;
        self.nodes[idx].process = process;
        self.nodes[idx].end_process = end_process;
        self.nodes[idx].task_data = vec![0; data_size];
        self.nodes[idx].message = 0;
        self.insert_task(idx, flags, parent_idx, name);
        Some(TaskHandle::encode(idx, gen))
    }

    /// Create a sheet animation task. `parent` = None means root list.
    pub fn create_animation_sheet(
        &mut self,
        sprites: &mut SpriteSystem,
        desc: PalSheetAnimationDesc,
        parent: Option<TaskHandle>,
    ) -> anyhow::Result<TaskHandle> {
        let idx = self.alloc_node().ok_or_else(|| {
            anyhow::anyhow!(
                "task pool full (capacity {}); cannot create sheet animation",
                TASK_POOL_CAPACITY
            )
        })?;
        let parent_idx = parent.filter(|h| self.is_alive(*h)).map(|h| h.pool_index());
        let anim = PalSheetAnimation::create(sprites, desc, self.pal_time_ms)?;
        let gen = self.nodes[idx].generation;
        self.nodes[idx].kind = TaskKind::AnimationSheet(anim);
        self.insert_task(idx, 0, parent_idx, "Animation");
        Ok(TaskHandle::encode(idx, gen))
    }

    /// Create a sequence animation task. `parent` = None means root list.
    pub fn create_animation_sequence(
        &mut self,
        sprites: &mut SpriteSystem,
        desc: PalSequenceAnimationDesc,
        parent: Option<TaskHandle>,
    ) -> anyhow::Result<TaskHandle> {
        let idx = self.alloc_node().ok_or_else(|| {
            anyhow::anyhow!(
                "task pool full (capacity {}); cannot create sequence animation",
                TASK_POOL_CAPACITY
            )
        })?;
        let parent_idx = parent.filter(|h| self.is_alive(*h)).map(|h| h.pool_index());
        let anim = PalSequenceAnimation::create(sprites, desc, self.pal_time_ms)?;
        let gen = self.nodes[idx].generation;
        self.nodes[idx].kind = TaskKind::AnimationSequence(anim);
        self.insert_task(idx, 0, parent_idx, "PalAnimationEx");
        Ok(TaskHandle::encode(idx, gen))
    }

    /// Create a WaitFrame blocking task. `remaining == -1` waits forever.
    /// Returns None if the pool is full.
    pub fn create_wait_frame(&mut self, remaining: i32) -> Option<TaskHandle> {
        let idx = self.alloc_node()?;
        let gen = self.nodes[idx].generation;
        self.nodes[idx].kind = TaskKind::WaitFrame { remaining };
        self.nodes[idx].task_data = remaining.to_le_bytes().to_vec();
        self.insert_task(idx, BLOCKING_FLAG, None, "PalWait");
        let handle = TaskHandle::encode(idx, gen);
        if debug_vm_wait_trace_enabled() {
            log::debug!("[trace-wait] create_wait_frame handle={handle:?} remaining={remaining}");
        }
        Some(handle)
    }

    /// Create a WaitTime blocking task. Uses the cached PAL clock as its start time.
    /// Returns None if the pool is full.
    pub fn create_wait_time(&mut self, duration_ms: u32) -> Option<TaskHandle> {
        let idx = self.alloc_node()?;
        let gen = self.nodes[idx].generation;
        self.nodes[idx].kind = TaskKind::WaitTime {
            duration_ms,
            start_ms: self.pal_time_ms,
        };
        let mut task_data = Vec::with_capacity(8);
        task_data.extend_from_slice(&duration_ms.to_le_bytes());
        task_data.extend_from_slice(&self.pal_time_ms.to_le_bytes());
        self.nodes[idx].task_data = task_data;
        self.insert_task(idx, BLOCKING_FLAG, None, "PalWaitTime");
        let handle = TaskHandle::encode(idx, gen);
        if debug_vm_wait_trace_enabled() {
            log::debug!(
                "[trace-wait] create_wait_time handle={handle:?} duration_ms={duration_ms} start_ms={}",
                self.pal_time_ms
            );
        }
        Some(handle)
    }

    /// Create a WaitClick blocking task. Frees itself when any input push is detected.
    /// Returns None if the pool is full.
    pub fn create_wait_click(&mut self) -> Option<TaskHandle> {
        let idx = self.alloc_node()?;
        let gen = self.nodes[idx].generation;
        self.nodes[idx].kind = TaskKind::WaitClick;
        self.nodes[idx].task_data.clear();
        self.insert_task(idx, BLOCKING_FLAG, None, "PalWaitClick");
        let handle = TaskHandle::encode(idx, gen);
        log::debug!("[trace-wait] create_wait_click handle={handle:?}");
        Some(handle)
    }

    /// Create the PAL wait_click(duration) task: input ends it early, otherwise timeout ends it.
    /// Returns None if the pool is full.
    pub fn create_wait_click_or_time(&mut self, duration_ms: u32) -> Option<TaskHandle> {
        self.create_wait_click_or_time_gated(duration_ms, false)
    }

    /// `voice_gated` variant of `create_wait_click_or_time` used for ADV
    /// auto-mode holds: the timeout finishes the wait only after voice
    /// playback has ended as well.
    pub fn create_wait_click_or_time_gated(
        &mut self,
        duration_ms: u32,
        voice_gated: bool,
    ) -> Option<TaskHandle> {
        let idx = self.alloc_node()?;
        let gen = self.nodes[idx].generation;
        self.nodes[idx].kind = TaskKind::WaitClickOrTime {
            duration_ms: duration_ms.max(1),
            start_ms: self.pal_time_ms,
            voice_gated,
        };
        let mut task_data = Vec::with_capacity(8);
        task_data.extend_from_slice(&duration_ms.max(1).to_le_bytes());
        task_data.extend_from_slice(&self.pal_time_ms.to_le_bytes());
        self.nodes[idx].task_data = task_data;
        self.insert_task(idx, BLOCKING_FLAG, None, "PalWaitClickTime");
        let handle = TaskHandle::encode(idx, gen);
        log::debug!(
            "[trace-wait] create_wait_click_or_time handle={handle:?} duration_ms={} start_ms={}",
            duration_ms.max(1),
            self.pal_time_ms
        );
        Some(handle)
    }

    /// Mark a task as pending-free. All children are also marked pending-free.
    /// The task and its children will be released on the next process() call.
    /// Matches PalTaskFree_0 behavior (does not free immediately).
    pub fn free(&mut self, handle: TaskHandle) -> bool {
        if !self.is_alive(handle) {
            return false;
        }
        let idx = handle.pool_index();
        self.nodes[idx].process = self.nodes[idx].end_process;
        self.mark_pending_free(idx);
        true
    }

    fn mark_pending_free(&mut self, idx: usize) {
        if self.nodes[idx].state == TaskState::Free {
            return;
        }
        self.nodes[idx].state = TaskState::PendingFree;
        let children = self.nodes[idx].children.clone();
        for child in children {
            self.mark_pending_free(child);
        }
    }

    /// Process all tasks depth-first. Animation callbacks update sprite source rects.
    /// Wait tasks check input state and free themselves when their condition is met.
    /// `skip_active` mirrors the native per-frame GetSkipState poll: wait-click
    /// style tasks complete immediately while any skip mode is active.
    /// `voice_active` gates ADV auto-mode hold timeouts on voice playback.
    pub fn process(
        &mut self,
        sprites: &mut SpriteSystem,
        input: &PalInputState,
        skip_active: bool,
        voice_active: bool,
    ) {
        let root = std::mem::take(&mut self.root_children);
        let new_root = Self::process_list(
            &mut self.nodes,
            &root,
            sprites,
            input,
            self.pal_time_ms,
            false,
            skip_active,
            voice_active,
        );
        self.root_children = new_root;
    }

    /// Returns the number of live (active + pending-free) task slots.
    pub fn active_task_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|n| n.state != TaskState::Free)
            .count()
    }

    fn process_list(
        nodes: &mut Vec<TaskNode>,
        list: &[usize],
        sprites: &mut SpriteSystem,
        input: &PalInputState,
        pal_time_ms: u32,
        pending_only: bool,
        skip_active: bool,
        voice_active: bool,
    ) -> Vec<usize> {
        let mut new_list = Vec::with_capacity(list.len());
        let mut blocking_triggered = false;

        for &idx in list {
            // After a blocking task: only clean up pending-free nodes, skip active ones.
            if blocking_triggered || pending_only {
                match nodes[idx].state {
                    TaskState::PendingFree => {
                        Self::do_release(nodes, idx, sprites);
                    }
                    TaskState::Free => {}
                    TaskState::Active | TaskState::ChildSentinel => {
                        new_list.push(idx);
                    }
                }
                continue;
            }

            // Depth-first: process this task's children before the task itself.
            let children = std::mem::take(&mut nodes[idx].children);
            let new_children = Self::process_list(
                nodes,
                &children,
                sprites,
                input,
                pal_time_ms,
                false,
                skip_active,
                voice_active,
            );
            nodes[idx].children = new_children;

            let state = nodes[idx].state;
            match state {
                TaskState::PendingFree => {
                    Self::do_release(nodes, idx, sprites);
                }
                TaskState::Active | TaskState::ChildSentinel => {
                    let outcome =
                        Self::do_update(nodes, idx, sprites, input, pal_time_ms, skip_active, voice_active);
                    if outcome == TaskUpdateOutcome::FreeSelf {
                        nodes[idx].state = TaskState::PendingFree;
                    }
                    // Task stays in list (pending-free nodes are removed on the next process call).
                    new_list.push(idx);
                    // Check blocking flag after the task is processed.
                    if nodes[idx].flags & BLOCKING_FLAG != 0 && nodes[idx].state != TaskState::Free
                    {
                        blocking_triggered = true;
                    }
                }
                TaskState::Free => {}
            }
        }
        new_list
    }

    fn do_release(nodes: &mut Vec<TaskNode>, idx: usize, _sprites: &mut SpriteSystem) {
        nodes[idx].state = TaskState::Free;
        nodes[idx].kind = TaskKind::Free;
        nodes[idx].generation = nodes[idx].generation.wrapping_add(1);
        nodes[idx].children.clear();
        nodes[idx].parent = None;
        nodes[idx].flags = 0;
        nodes[idx].process = 0;
        nodes[idx].end_process = 0;
        nodes[idx].message = 0;
        nodes[idx].task_data.clear();
    }

    fn do_update(
        nodes: &mut Vec<TaskNode>,
        idx: usize,
        sprites: &mut SpriteSystem,
        input: &PalInputState,
        pal_time_ms: u32,
        skip_active: bool,
        voice_active: bool,
    ) -> TaskUpdateOutcome {
        // Take the kind out to avoid aliasing with the nodes slice during the update.
        let mut kind = std::mem::replace(&mut nodes[idx].kind, TaskKind::Free);
        let outcome = match &mut kind {
            TaskKind::AnimationSheet(anim) => anim.step_once(sprites, pal_time_ms),
            TaskKind::AnimationSequence(anim) => anim.step_once(sprites, pal_time_ms),
            TaskKind::WaitFrame { remaining } => {
                if *remaining < 0 {
                    // remaining == -1: permanent wait (forever).
                    TaskUpdateOutcome::Continue
                } else {
                    *remaining -= 1;
                    if nodes[idx].task_data.len() >= 4 {
                        nodes[idx].task_data[0..4].copy_from_slice(&remaining.to_le_bytes());
                    }
                    if *remaining <= 0 {
                        TaskUpdateOutcome::FreeSelf
                    } else {
                        TaskUpdateOutcome::Continue
                    }
                }
            }
            TaskKind::WaitTime {
                duration_ms,
                start_ms,
            } => {
                let elapsed = pal_time_ms.wrapping_sub(*start_ms);
                if elapsed >= *duration_ms {
                    TaskUpdateOutcome::FreeSelf
                } else {
                    TaskUpdateOutcome::Continue
                }
            }
            TaskKind::WaitClick => {
                // Native wait-click consumers poll GetSkipState every frame:
                // any latched skip mode (skip byte, scene skip, held Ctrl)
                // completes the wait just like a click push. Only the native
                // click set (Space/Return/left mouse/wheel up) qualifies;
                // Escape, arrows and F-keys must not advance text.
                if input.click_push() || skip_active {
                    log::debug!(
                        "[trace-wait] wait_click complete idx={idx} click_push={} skip_active={skip_active} pal_time_ms={pal_time_ms}",
                        input.click_push()
                    );
                    TaskUpdateOutcome::FreeSelf
                } else {
                    TaskUpdateOutcome::Continue
                }
            }
            TaskKind::WaitClickOrTime {
                duration_ms,
                start_ms,
                voice_gated,
            } => {
                let elapsed = pal_time_ms.wrapping_sub(*start_ms);
                // Auto-mode holds (voice_gated) finish by timeout only once
                // voice playback has ended; skip state and input always
                // complete immediately (koikake.exe 0x42F9D0/0x4326DF).
                let timed_out = elapsed >= *duration_ms && !(*voice_gated && voice_active);
                if input.click_push() || skip_active || timed_out {
                    log::debug!(
                        "[trace-wait] wait_click_or_time complete idx={idx} click_push={} skip_active={skip_active} elapsed={elapsed} duration_ms={duration_ms} voice_gated={voice_gated} voice_active={voice_active}",
                        input.click_push()
                    );
                    TaskUpdateOutcome::FreeSelf
                } else {
                    TaskUpdateOutcome::Continue
                }
            }
            TaskKind::Unsupported { name } => {
                log::warn!(
                    "TaskSystem: unsupported task '{}' encountered; it will never self-complete",
                    name
                );
                TaskUpdateOutcome::Continue
            }
            TaskKind::Raw => TaskUpdateOutcome::Continue,
            TaskKind::Free => TaskUpdateOutcome::FreeSelf,
        };
        nodes[idx].kind = kind;
        outcome
    }

    // ---- Animation accessors ----

    /// Mark an animation task as pending-free.
    pub fn animation_release(&mut self, handle: TaskHandle) -> bool {
        self.free(handle)
    }

    /// Reset a sheet animation to frame 0 and update last_time_ms.
    pub fn animation_reset(&mut self, handle: TaskHandle, sprites: &mut SpriteSystem) -> bool {
        if !self.is_alive(handle) {
            return false;
        }
        let idx = handle.pool_index();
        let t = self.pal_time_ms;
        match &mut self.nodes[idx].kind {
            TaskKind::AnimationSheet(anim) => anim.reset(sprites, t).is_ok(),
            TaskKind::AnimationSequence(anim) => anim.reset(sprites, t).is_ok(),
            _ => false,
        }
    }

    /// Reset a sequence animation with a new frame table and restart it.
    pub fn animation_reset_sequence_with_frames(
        &mut self,
        handle: TaskHandle,
        sprites: &mut SpriteSystem,
        frames: Vec<crate::animation::PalAnimationFrameRecord>,
    ) -> bool {
        if !self.is_alive(handle) {
            return false;
        }
        let idx = handle.pool_index();
        let t = self.pal_time_ms;
        match &mut self.nodes[idx].kind {
            TaskKind::AnimationSequence(anim) => anim.reset_with_frames(sprites, frames, t).is_ok(),
            _ => false,
        }
    }

    pub fn animation_start(&mut self, handle: TaskHandle) -> bool {
        if !self.is_alive(handle) {
            return false;
        }
        let idx = handle.pool_index();
        match &mut self.nodes[idx].kind {
            TaskKind::AnimationSheet(anim) => {
                anim.running = true;
                true
            }
            TaskKind::AnimationSequence(anim) => {
                anim.running = true;
                true
            }
            _ => false,
        }
    }

    pub fn animation_stop(&mut self, handle: TaskHandle) -> bool {
        if !self.is_alive(handle) {
            return false;
        }
        let idx = handle.pool_index();
        match &mut self.nodes[idx].kind {
            TaskKind::AnimationSheet(anim) => {
                anim.running = false;
                true
            }
            TaskKind::AnimationSequence(anim) => {
                anim.running = false;
                true
            }
            _ => false,
        }
    }

    /// Set frame delay and reset last_time_ms. Only valid for sheet animations.
    pub fn animation_set_time(&mut self, handle: TaskHandle, frame_delay_ms: u32) -> bool {
        if !self.is_alive(handle) {
            return false;
        }
        let idx = handle.pool_index();
        let t = self.pal_time_ms;
        match &mut self.nodes[idx].kind {
            TaskKind::AnimationSheet(anim) => {
                anim.frame_delay_ms = frame_delay_ms;
                anim.last_time_ms = t;
                true
            }
            _ => false,
        }
    }

    /// Returns frame_delay_ms for a sheet animation handle.
    /// For sequence handles, returns None (original PalAnimationGetTime_0 reads task_data+0x04
    /// which is the frame-record table pointer for Ex, not a time value).
    pub fn animation_get_time(&self, handle: TaskHandle) -> Option<u32> {
        if !self.is_alive(handle) {
            return None;
        }
        match &self.nodes[handle.pool_index()].kind {
            TaskKind::AnimationSheet(anim) => Some(anim.frame_delay_ms),
            _ => None,
        }
    }

    /// Returns the raw value at task_data+0x10.
    /// For Ex (sequence), this is current_frame. For sheet, this is the flags byte (as dword).
    pub fn animation_get_point(&self, handle: TaskHandle) -> Option<i32> {
        if !self.is_alive(handle) {
            return None;
        }
        match &self.nodes[handle.pool_index()].kind {
            TaskKind::AnimationSheet(anim) => Some(i32::from(anim.flags.raw())),
            TaskKind::AnimationSequence(anim) => Some(anim.current_frame),
            _ => None,
        }
    }

    /// Consume and return the looped flag for a sheet animation (one-shot read).
    pub fn animation_take_looped(&mut self, handle: TaskHandle) -> bool {
        if !self.is_alive(handle) {
            return false;
        }
        let idx = handle.pool_index();
        match &mut self.nodes[idx].kind {
            TaskKind::AnimationSheet(anim) => std::mem::take(&mut anim.looped_since_last_query),
            _ => false,
        }
    }

    pub fn task_state(&self, handle: TaskHandle) -> Option<TaskState> {
        if !self.is_alive(handle) {
            return None;
        }
        let node = &self.nodes[handle.pool_index()];
        if node.process == node.end_process {
            Some(TaskState::PendingFree)
        } else {
            Some(node.state)
        }
    }

    pub fn task_state_raw(&self, handle: TaskHandle) -> i32 {
        self.task_state(handle)
            .map(|state| state as i32)
            .unwrap_or(0)
    }

    pub fn change_state(&mut self, handle: TaskHandle, state: TaskState) -> bool {
        if !self.is_alive(handle) {
            return false;
        }
        self.nodes[handle.pool_index()].state = state;
        true
    }

    pub fn set_message(&mut self, handle: TaskHandle, message: i32) -> bool {
        if !self.is_alive(handle) {
            return false;
        }
        self.nodes[handle.pool_index()].message = message;
        true
    }

    pub fn take_message(&mut self, handle: TaskHandle) -> i32 {
        if !self.is_alive(handle) {
            return 0;
        }
        std::mem::take(&mut self.nodes[handle.pool_index()].message)
    }

    pub fn task_data(&self, handle: TaskHandle) -> Option<&[u8]> {
        if !self.is_alive(handle) {
            return None;
        }
        Some(&self.nodes[handle.pool_index()].task_data)
    }

    pub fn task_data_mut(&mut self, handle: TaskHandle) -> Option<&mut [u8]> {
        if !self.is_alive(handle) {
            return None;
        }
        Some(&mut self.nodes[handle.pool_index()].task_data)
    }

    pub fn set_task_data(&mut self, handle: TaskHandle, data: Vec<u8>) -> bool {
        if !self.is_alive(handle) {
            return false;
        }
        self.nodes[handle.pool_index()].task_data = data;
        true
    }

    pub fn change_next(&mut self, handle: TaskHandle, process: i32) -> bool {
        if !self.is_alive(handle) {
            return false;
        }
        self.nodes[handle.pool_index()].process = process;
        true
    }

    pub fn get_animation_sheet(&self, handle: TaskHandle) -> Option<&PalSheetAnimation> {
        if !self.is_alive(handle) {
            return None;
        }
        match &self.nodes[handle.pool_index()].kind {
            TaskKind::AnimationSheet(anim) => Some(anim),
            _ => None,
        }
    }

    pub fn get_animation_sequence(&self, handle: TaskHandle) -> Option<&PalSequenceAnimation> {
        if !self.is_alive(handle) {
            return None;
        }
        match &self.nodes[handle.pool_index()].kind {
            TaskKind::AnimationSequence(anim) => Some(anim),
            _ => None,
        }
    }

    /// Returns the count of all non-free task slots.
    pub fn active_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|n| n.state != TaskState::Free)
            .count()
    }

    /// Returns a dump of all active (non-free) task slots for diagnostic output.
    pub fn dump_entries(&self) -> Vec<TaskDumpEntry> {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.state != TaskState::Free)
            .map(|(idx, n)| TaskDumpEntry {
                handle: TaskHandle::encode(idx, n.generation),
                kind_name: n.name,
                state: n.state,
                flags: n.flags,
                parent_index: n.parent,
                children_count: n.children.len(),
                is_blocking: (n.flags & BLOCKING_FLAG) != 0,
                is_pending_free: n.state == TaskState::PendingFree,
            })
            .collect()
    }

    /// Returns whether any task in the pool has BLOCKING_FLAG set and is Active.
    pub fn has_blocking_task(&self) -> bool {
        self.nodes
            .iter()
            .any(|n| n.state == TaskState::Active && (n.flags & BLOCKING_FLAG) != 0)
    }
}

impl Default for TaskSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct TaskDumpEntry {
    pub handle: TaskHandle,
    pub kind_name: &'static str,
    pub state: TaskState,
    pub flags: u32,
    pub parent_index: Option<usize>,
    pub children_count: usize,
    pub is_blocking: bool,
    pub is_pending_free: bool,
}

/// Type alias: animation handles are task handles.
pub type AnimationHandle = TaskHandle;
