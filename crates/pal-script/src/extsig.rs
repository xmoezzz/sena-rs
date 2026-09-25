//! Shared semantic table for Game.exe script extcalls.
//!
//! This table is intentionally evidence-oriented: it is not just a pretty
//! decompiler signature list.  The VM, decompiler, and writeup should all agree
//! on these records before an extcall is treated as implemented.

#[path = "extsig_auto.rs"]
mod extsig_auto;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParamKind {
    Integer,
    Slot,
    SpriteSlot,
    ButtonSlot,
    SoundSlot,
    ResourceName,
    TextId,
    TextString,
    FileDatString,
    DynamicString,
    IniSection,
    IniKey,
    IniFilename,
    PointId,
    CoordinateX,
    CoordinateY,
    CoordinateZ,
    DurationMs,
    Volume,
    Color,
    Alpha,
    Flag,
    Mode,
    Handle,
    BufferPointer,
    Unknown,
    /// Compatibility alias used by the current decompiler renderer.
    ResourceStringFromFileDat,
    /// Compatibility alias used by the current decompiler renderer.
    TextStringFromTextDat,
    /// Compatibility alias used by the current decompiler renderer.
    Coordinate,
    /// Compatibility alias used by the current decompiler renderer.
    Duration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReturnKind {
    Void,
    Integer,
    Bool,
    Handle,
    StringId,
    Status,
    PointId,
    UnsupportedUnknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SideEffect {
    CreatesSprite,
    MutatesSprite,
    DeletesSprite,
    CreatesTask,
    BlocksScript,
    ReadsIni,
    ReadsFile,
    WritesVmMemory,
    CreatesSound,
    PlaysSound,
    StopsSound,
    ChangesTextState,
    ChangesSelectState,
    ChangesSaveState,
    ChangesHistoryState,
    ChangesRunState,
    CreatesMovie,
    MutatesWindow,
    UnknownSideEffect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImplStatus {
    Verified,
    Partial,
    Blocked,
    StackDisciplineOnly,
    Stub,
    Unknown,
    WrongOrSuspicious,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvidenceKind {
    GameSqlite,
    PalSqlite,
    RuntimeTrace,
    Writeup,
    Disassembly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Evidence {
    pub kind: EvidenceKind,
    pub reference: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParamSpec {
    pub name: &'static str,
    pub kind: ParamKind,
    pub meaning: &'static str,
}

/// One parameter in display order. `pop_idx` indexes the runtime pop-order
/// argument vector: 0 is the last pushed value and matches `pop_ext_args()[0]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Param {
    pub name: &'static str,
    pub pop_idx: usize,
    pub kind: ParamKind,
    pub meaning: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExtSig {
    pub category: u16,
    pub index: u16,
    pub canonical_name: &'static str,
    /// Compatibility field for existing decompiler code.
    pub name: &'static str,
    pub game_handler_ea: Option<&'static str>,
    pub pal_export_name: Option<&'static str>,
    pub pal_export_ea: Option<&'static str>,
    pub pop_count: usize,
    pub pop_order: &'static [ParamSpec],
    pub display_order: &'static [usize],
    /// Compatibility field: display-order parameters with pop-order indices.
    pub params: &'static [Param],
    pub return_kind: ReturnKind,
    /// Compatibility field for existing decompiler code.
    pub returns: bool,
    pub side_effects: &'static [SideEffect],
    pub purpose: &'static str,
    pub evidence: &'static [Evidence],
    pub implementation_status: ImplStatus,
    pub decompiler_status: ImplStatus,
}

/// Backward-compatible name used by pal-decompiler.
pub type ExtCallSig = ExtSig;

macro_rules! evidence {
    () => {
        &[]
    };
    ($($kind:ident : $reference:literal),+ $(,)?) => {
        &[$(Evidence { kind: EvidenceKind::$kind, reference: $reference }),+]
    };
}

macro_rules! sidefx {
    () => {
        &[]
    };
    ($($effect:ident),+ $(,)?) => {
        &[$(SideEffect::$effect),+]
    };
}

macro_rules! params {
    () => {
        &[]
    };
    ($($name:literal : $idx:literal = $kind:ident => $meaning:literal),+ $(,)?) => {
        &[$(Param {
            name: $name,
            pop_idx: $idx,
            kind: ParamKind::$kind,
            meaning: $meaning,
        }),+]
    };
    ($($name:literal : $idx:literal = $kind:ident),+ $(,)?) => {
        &[$(Param {
            name: $name,
            pop_idx: $idx,
            kind: ParamKind::$kind,
            meaning: "",
        }),+]
    };
}

macro_rules! pop_order_from_params {
    ($params:expr) => {
        &[]
    };
}

macro_rules! display_order {
    () => {
        &[]
    };
    ($($idx:literal),+ $(,)?) => {
        &[$($idx),+]
    };
}

macro_rules! sig {
    (
        $cat:expr, $idx_val:expr, $name:literal,
        pop=$pop:expr,
        params=[$($p:tt)*],
        return=$ret:ident,
        effects=[$($fx:ident),* $(,)?],
        purpose=$purpose:literal,
        status=$status:ident,
        decompiler=$decompiler:ident,
        evidence=[$($ev_kind:ident : $ev_ref:literal),* $(,)?]
        $(, game=$game:expr)?
        $(, pal=$pal_name:expr => $pal_ea:expr)?
    ) => {{
        const PARAMS: &[Param] = params!($($p)*);
        ExtSig {
            category: $cat,
            index: $idx_val,
            canonical_name: $name,
            name: $name,
            game_handler_ea: sig!(@opt None $(, $game)?),
            pal_export_name: sig!(@opt None $(, $pal_name)?),
            pal_export_ea: sig!(@opt None $(, $pal_ea)?),
            pop_count: $pop,
            pop_order: pop_order_from_params!(PARAMS),
            display_order: display_order!(),
            params: PARAMS,
            return_kind: ReturnKind::$ret,
            returns: !matches!(ReturnKind::$ret, ReturnKind::Void),
            side_effects: sidefx!($($fx),*),
            purpose: $purpose,
            evidence: evidence!($($ev_kind : $ev_ref),*),
            implementation_status: ImplStatus::$status,
            decompiler_status: ImplStatus::$decompiler,
        }
    }};
    (@opt $default:expr) => { $default };
    (@opt $default:expr, $value:expr) => { Some($value) };
}

// Category 2 - text / ADV message state.

/// category 2 index 2: text
///
/// Purpose: Set current ADV text line and channel without blocking the VM.
/// Caller is responsible for following with wait_click or text_w.
///
/// VM arguments (display order: mode, text_id, name_or_aux_id, voice_or_aux_id):
/// - pop[0]: voice_or_aux_id (TextId) — voice/auxiliary id, or -1.
/// - pop[1]: name_or_aux_id (TextId) — speaker name Text.dat id, or -1.
/// - pop[2]: text_id (TextId) — Text.dat byte offset for message body.
/// - pop[3]: mode (Mode) — ADV channel/mode selector.
///
/// Return: void.
///
/// Side effects: ChangesTextState.
///
/// Evidence:
/// - Writeup: docs/writeup.md 24.17
/// - RuntimeTrace: text args=[mode,text_id,name,voice]
///
/// Engine: Blocked — queues text display; does not block.
///
/// Decompiler: Blocked — renders four args in display order.
static SIG_TEXT: ExtSig = sig!(2, 2, "text", pop=4,
    params=[
        "mode":3=Mode=>"text mode / ADV channel",
        "text_id":2=TextId=>"Text.dat byte offset or dynamic string id",
        "name_or_aux_id":1=TextId=>"speaker/name text id or -1",
        "voice_or_aux_id":0=TextId=>"voice/aux id or -1"
    ],
    return=Void, effects=[ChangesTextState], purpose="Set current ADV text without script blocking.",
    status=Blocked, decompiler=Blocked,
    evidence=[Writeup:"docs/writeup.md 24.17", RuntimeTrace:"text args=[mode,text_id,name,voice]"]);
/// category 2 index 3: text_hide
///
/// Purpose: Hide the ADV text window and associated UI state immediately.
///
/// VM arguments:
/// - pop[0]: redraw_flag (Flag) — non-zero requests text redraw/update flag.
///
/// Return: void.
///
/// Side effects: ChangesTextState.
///
/// Evidence:
/// - Game.sqlite: sub_43F340 pops one flag, clears visible text sprite colors,
///   calls sub_439C10(...,0), and sets ctx[201021] when flag is non-zero.
/// - Writeup: docs/writeup.md 24.17
///
/// Engine: Blocked — hides text window without releasing the native text sprites.
///
/// Decompiler: Blocked — renders as text_hide(redraw_flag).
static SIG_TEXT_HIDE: ExtSig = sig!(2, 3, "text_hide", pop=1,
    params=["redraw_flag":0=Flag=>"non-zero requests native redraw/update flag"],
    return=Void, effects=[ChangesTextState], purpose="Hide the ADV text window/state.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_43F340 pops one flag, hides text sprite colors, and updates ctx[201021]", Writeup:"docs/writeup.md 24.17"],
    game="0x0043F340");
/// category 2 index 4: text_show
///
/// Purpose: Show the ADV text window.  The visible flag is observed in script
/// wrappers but actual semantics of 0 vs non-zero are engine-defined.
///
/// VM arguments:
/// - pop[0]: visible (Flag) — non-zero to show; 0 to hide (exact semantics
///   engine-dependent; most call sites pass 1).
///
/// Return: void.
///
/// Side effects: ChangesTextState.
///
/// Evidence:
/// - Writeup: docs/writeup.md 24.17
///
/// Engine: Blocked — shows text window/state.
///
/// Decompiler: Blocked — renders as text_show(visible).
static SIG_TEXT_SHOW: ExtSig = sig!(2, 4, "text_show", pop=1,
    params=["visible":0=Flag=>"show flag observed in script wrappers"],
    return=Void, effects=[ChangesTextState], purpose="Show the ADV text window/state.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md 24.17"]);
/// category 2 index 5: text_set_btn
///
/// Purpose: Select the active text-window button id used by the ADV message
/// UI. Native clears the selected button's per-window reaction slot after
/// storing the id in the active text context.
///
/// VM arguments:
/// - pop[0]: button_id (Integer) — text button/window id.
///
/// Return: status integer.
///
/// Side effects: ChangesTextState.
///
/// Evidence:
/// - Game.sqlite: sub_43F1A0 pops one value, stores it at text context +44,
///   clears `4512 * id + ctx + 12980`, and logs `text_set_btn %d`.
///
/// Engine: Verified — pal-vm dispatch_text_stub index 5 pops one id and stores
/// it in TextSubsystemState.
///
/// Decompiler: Verified — renders as text_set_btn(button_id).
static SIG_TEXT_SET_BTN: ExtSig = sig!(2, 5, "text_set_btn", pop=1,
    params=["button_id":0=Integer=>"text button/window id"],
    return=Integer, effects=[ChangesTextState], purpose="Select active ADV text-window button id.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_43F1A0 pops button id and clears its text reaction slot"],
    game="0x0043F1A0");
/// category 2 index 8: text_clear
///
/// Purpose: Clear the current text line and schedule body/name sprite release
/// on the next text synchronization step.
///
/// VM arguments: none.
///
/// Return: void.
///
/// Side effects: ChangesTextState, DeletesSprite.
///
/// Evidence:
/// - Writeup: docs/writeup.md 24.17
///
/// Engine: Blocked — clears text state and queues sprite release.
///
/// Decompiler: Blocked — renders as text_clear().
static SIG_TEXT_CLEAR: ExtSig = sig!(2, 8, "text_clear", pop=0, params=[],
    return=Void, effects=[ChangesTextState, DeletesSprite], purpose="Clear current text and release body/name sprites on the next text sync.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md 24.17"]);
/// category 2 index 9: text_clear_ex
///
/// Purpose: Zero-argument text cleanup/repaint hook adjacent to `text_clear` in
/// the native category-2 dispatch table.  It is reachable from title system
/// menu setup and must not fall through to raw `ext_0002_0009`.
///
/// VM arguments: none.
///
/// Return: void.
///
/// Side effects: ChangesTextState.
///
/// Evidence: runtime reachability at PCs 0x00039584/0x0003A960/0x0003AE8C;
/// Game.sqlite confirms the neighboring `sub_43F0B0 text_clear` zero-argument
/// cleanup/paint behavior. Exact handler EA for index 9 remains blocked.
static SIG_TEXT_CLEAR_EX: ExtSig = sig!(2, 9, "text_clear_ex", pop=0, params=[],
    return=Void, effects=[ChangesTextState], purpose="Zero-argument text cleanup/repaint hook adjacent to text_clear.",
    status=Blocked, decompiler=Blocked,
    evidence=[RuntimeTrace:"reachable extcall 0002:0009 at PCs 0x00039584/0x0003A960/0x0003AE8C", GameSqlite:"reverse/Game.sqlite neighboring sub_43F0B0 text_clear"]);
/// category 2 index 10: text_get_time
///
/// Purpose: Return the current PAL text/task time counter.  Native does not pop
/// any script arguments; it writes `*(PalTaskGetTaskData(0)+32)` into the
/// extcall destination slot.
///
/// VM arguments: none.
///
/// Return: integer task time.
///
/// Side effects: none beyond VM return writeback.
///
/// Evidence:
/// - Game.sqlite: sub_43F010 has no stack-pop sequence and stores the task-time
///   dword to `dst_slot`.
///
/// Engine: Verified — pal-vm pops zero arguments and returns elapsed text task
/// time through ExtCallOutcome.
///
/// Decompiler: Verified — renders destination assignment when dst_slot is
/// non-zero, e.g. `v6 = text_get_time()`.
static SIG_TEXT_GET_TIME: ExtSig = sig!(2, 10, "text_get_time", pop=0,
    params=[],
    return=Integer, effects=[WritesVmMemory], purpose="Return the current text/task time counter.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_43F010 writes PalTaskGetTaskData(0)+32 to dst_slot and pops no args"],
    game="0x0043F010");
/// category 2 index 14: text_set_icon_animation_time
///
/// Purpose: Set the animation time for one of the four ADV text-window icon
/// animations.
///
/// VM arguments:
/// - pop[0]: icon_slot (Integer) — icon animation slot, native accepts 0..3.
/// - pop[1]: duration_ms (DurationMs) — animation time passed to
///   PalAnimationSetTime.
///
/// Return: status integer.
///
/// Side effects: ChangesTextState.
///
/// Evidence:
/// - Game.sqlite: sub_43ED40 pops icon slot then duration and calls
///   PalAnimationSetTime(text_icon_animation[slot], duration).
/// - PAL.sqlite: PalAnimationSetTime is the PAL-side animation timer API.
///
/// Engine: Verified — pal-vm consumes the same two arguments and updates text
/// icon timing state used by the renderer.
///
/// Decompiler: Verified — renders text_set_icon_animation_time(icon_slot,
/// duration_ms).
static SIG_TEXT_SET_ICON_ANIMATION_TIME: ExtSig = sig!(2, 14, "text_set_icon_animation_time", pop=2,
    params=[
        "icon_slot":0=Integer=>"text icon animation slot, 0..3",
        "duration_ms":1=DurationMs=>"animation time passed to PalAnimationSetTime"
    ],
    return=Integer, effects=[ChangesTextState], purpose="Set ADV text icon animation time.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_43ED40 pops icon slot and duration then calls PalAnimationSetTime", PalSqlite:"reverse/PAL.sqlite PalAnimationSetTime 0x10120110 -> PalAnimationSetTime_0 0x10239C40 writes duration/start time"],
    game="0x0043ED40", pal="PalAnimationSetTime" => "0x10120110");
/// category 2 index 15: text_w
///
/// Purpose: Set current ADV line and start the character-by-character visual
/// reveal animation.  Does NOT block the VM; script typically follows with an
/// explicit wait_click to wait for player input.
///
/// VM arguments (same layout as text/text_a):
/// - pop[0]: voice_or_aux_id (TextId) — voice/aux id, or -1.
/// - pop[1]: name_or_aux_id (TextId) — speaker name id, or -1.
/// - pop[2]: text_id (TextId) — Text.dat byte offset.
/// - pop[3]: duration_ms (DurationMs) — explicit reveal duration; 0 uses native text-speed timing.
///
/// Return: void.
///
/// Side effects: ChangesTextState.
///
/// Evidence:
/// - Writeup: docs/writeup.md 24.17
/// - GameSqlite: sub_43FFC0 writes pop[3] to text_ctx+4196; if zero it derives timing from task time and text speed.
/// - RuntimeTrace: text_wait index=15 followed by explicit wait_click
///
/// Engine: Blocked — queues reveal-mode text; reveal proceeds independently.
///
/// Decompiler: Blocked — renders four args in display order.
static SIG_TEXT_W: ExtSig = sig!(2, 15, "text_w", pop=4,
    params=[
        "duration_ms":3=DurationMs=>"explicit text reveal duration; 0 uses native text-speed timing",
        "text_id":2=TextId=>"Text.dat byte offset or dynamic string id",
        "name_or_aux_id":1=TextId=>"speaker/name text id or -1",
        "voice_or_aux_id":0=TextId=>"voice/aux id or -1"
    ],
    return=Void, effects=[ChangesTextState], purpose="Set current ADV line and start visual reveal; does not block VM.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_43FFC0 writes pop[3] to text_ctx+4196 and computes reveal timing when it is zero", RuntimeTrace:"text_wait index=15 followed by explicit wait_click"]);
/// category 2 index 16: text_a
///
/// Purpose: Set current ADV line in fully-visible (no reveal) mode.  Does not
/// block the VM.
///
/// VM arguments (same layout as text/text_w):
/// - pop[0]: voice_or_aux_id (TextId) — voice/aux id, or -1.
/// - pop[1]: name_or_aux_id (TextId) — speaker name id, or -1.
/// - pop[2]: text_id (TextId) — Text.dat byte offset.
/// - pop[3]: duration_ms (DurationMs) — timing/mode lane stored with the ADV line.
///
/// Return: void.
///
/// Side effects: ChangesTextState.
///
/// Evidence:
/// - Writeup: docs/writeup.md 24.17
///
/// Engine: Blocked — shows text fully; no per-character reveal.
///
/// Decompiler: Blocked — renders four args in display order.
static SIG_TEXT_A: ExtSig = sig!(2, 16, "text_a", pop=4,
    params=["duration_ms":3=DurationMs, "text_id":2=TextId, "name_or_aux_id":1=TextId, "voice_or_aux_id":0=TextId],
    return=Void, effects=[ChangesTextState], purpose="Set current ADV line fully visible; does not block VM.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md 24.17"]);
/// category 2 index 17: text_wa
///
/// Purpose: Combined text_w + text_a variant; starts reveal and does not block.
/// Exact distinction from text_w is engine-internal.
///
/// VM arguments (same layout as text/text_w/text_a):
/// - pop[0]: voice_or_aux_id (TextId) — voice/aux id, or -1.
/// - pop[1]: name_or_aux_id (TextId) — speaker name id, or -1.
/// - pop[2]: text_id (TextId) — Text.dat byte offset.
/// - pop[3]: duration_ms (DurationMs) — explicit reveal duration/timing lane.
///
/// Return: void.
///
/// Side effects: ChangesTextState.
///
/// Evidence:
/// - Writeup: docs/writeup.md 24.17
///
/// Engine: Blocked — dispatched through dispatch_text_stub.
///
/// Decompiler: Blocked — renders four args in display order.
static SIG_TEXT_WA: ExtSig = sig!(2, 17, "text_wa", pop=4,
    params=["duration_ms":3=DurationMs, "text_id":2=TextId, "name_or_aux_id":1=TextId, "voice_or_aux_id":0=TextId],
    return=Void, effects=[ChangesTextState], purpose="Set current ADV line and start visual reveal; does not block VM.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md 24.17"]);
/// category 2 index 22: text_set_base
///
/// Purpose: Select the ADV message-window base image from File.dat.  The main
/// route uses this after checking the text-window style flag and passes one of
/// the File.dat ids 1418/1419/1420.
///
/// VM arguments (display order: mode, x, y, base_resource):
/// - pop[0]: base_resource (ResourceStringFromFileDat) — File.dat id for the
///   text window skin/base image.
/// - pop[1]: y (CoordinateY) — optional override; 0x0fffffff means default.
/// - pop[2]: x (CoordinateX) — optional override; 0x0fffffff means default.
/// - pop[3]: mode (Mode) — text base/window mode.
///
/// Return: status integer.
///
/// Side effects: ChangesTextState.
///
/// Evidence:
/// - Disassembly: docs/dis.txt 000346D8..00034760 pushes four operands before
///   ext_0002_0016.
/// - RuntimeTrace: text_set_base args=[0,268435455,268435455,1418].
///
/// Engine: Blocked — loads the selected base image when syncing ADV text.
///
/// Decompiler: Blocked — renders all four args in display order.
static SIG_TEXT_SET_BASE: ExtSig = sig!(2, 22, "text_set_base", pop=4,
    params=[
        "mode":3=Mode=>"text base/window mode",
        "x":2=CoordinateX=>"optional base x override; 0x0fffffff means default",
        "y":1=CoordinateY=>"optional base y override; 0x0fffffff means default",
        "base_resource":0=ResourceStringFromFileDat=>"File.dat id for ADV text-window base image"
    ],
    return=Integer, effects=[ChangesTextState], purpose="Select ADV message-window base image/resource.",
    status=Blocked, decompiler=Blocked,
    evidence=[Disassembly:"docs/dis.txt 000346D8..00034760", RuntimeTrace:"text_set_base args=[0,268435455,268435455,1418]"]);
/// category 2 index 25: texttimecheckset
///
/// Purpose: Create the native AdvTextTimeCheckTask sprite that measures/renders
/// a timing-check text block at a requested rectangle.
///
/// VM arguments:
/// - pop[0]: text_id (TextId) — string id resolved by sub_44B120.
/// - pop[1]: x (CoordinateX) — task sprite x coordinate.
/// - pop[2]: y (CoordinateY) — task sprite y coordinate.
/// - pop[3]: width (CoordinateX) — requested text/check width.
/// - pop[4]: height (CoordinateY) — requested text/check height.
///
/// Return: status integer.
///
/// Side effects: CreatesTask, CreatesSprite, ChangesTextState.
///
/// Evidence:
/// - Game.sqlite: sub_43E590 pops five values, resolves the first as a string,
///   then stores `sub_43A470(string, x, y, width, height)` in text task state.
/// - Game.sqlite: sub_43A470 calls PalTaskCreate(sub_43A420), PalSpriteCreate,
///   PalSpriteSetPos, and PalSpriteViewCtrl.
///
/// Engine: Blocked — pal-vm now consumes and traces all five parameters, but
/// the temporary native check sprite/task lifecycle is not yet equivalent.
///
/// Decompiler: Verified — renders all five arguments instead of the previous
/// erroneous empty call.
static SIG_TEXT_TIME_CHECK_SET: ExtSig = sig!(2, 25, "texttimecheckset", pop=5,
    params=[
        "text_id":0=TextId=>"string id resolved by sub_44B120",
        "x":1=CoordinateX=>"check sprite x coordinate",
        "y":2=CoordinateY=>"check sprite y coordinate",
        "width":3=CoordinateX=>"check text width",
        "height":4=CoordinateY=>"check text height"
    ],
    return=Integer, effects=[CreatesTask, CreatesSprite, ChangesTextState],
    purpose="Create native ADV text timing-check task/sprite.",
    status=Blocked, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_43E590 pops five args and calls sub_43A470", GameSqlite:"reverse/Game.sqlite sub_43A470 creates AdvTextTimeCheckTask and PalSpriteCreate/PalSpriteSetPos"],
    game="0x0043E590");
/// category 2 index 28: text_set_color
///
/// Purpose: Configure one entry in the ADV text color table, keyed by a label
/// string, with separate glyph and effect colors.
///
/// VM arguments:
/// - pop[0]: color_slot (Integer) — native color table slot; valid range < 16.
/// - pop[1]: label_string (TextId) — string copied with sub_44B120.
/// - pop[2]: text_color (Color) — primary glyph color.
/// - pop[3]: effect_color (Color) — outline/effect color.
///
/// Return: status integer.
///
/// Side effects: ChangesTextState.
///
/// Evidence:
/// - Game.sqlite: sub_43E2E0 pops four values, rejects slots >=16, resolves the
///   label string, and stores enable/text/effect/length/string fields in the
///   PAL task text color table.
///
/// Engine: Blocked — pal-vm consumes the correct four arguments and applies
/// the active color pair, but does not yet model all 16 native label entries.
///
/// Decompiler: Verified — renders text_set_color(color_slot, label_string,
/// text_color, effect_color).
static SIG_TEXT_SET_COLOR: ExtSig = sig!(2, 28, "text_set_color", pop=4,
    params=[
        "color_slot":0=Integer=>"native text color table slot, valid when <16",
        "label_string":1=TextId=>"label string id copied with sub_44B120",
        "text_color":2=Color=>"primary glyph color",
        "effect_color":3=Color=>"outline/effect color"
    ],
    return=Integer, effects=[ChangesTextState], purpose="Configure ADV text color-table entry.",
    status=Blocked, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_43E2E0 pops slot,label,text_color,effect_color and writes the text color table"],
    game="0x0043E2E0");

// Category 3 - sprite wrappers.

/// category 3 index 2: sp_set
///
/// Purpose: Load a sprite into slot from a File.dat resource name and position
/// it at (x, y).  Creates or replaces the slot.
///
/// VM arguments (display order: slot, name, x, y):
/// - pop[0]: name (ResourceStringFromFileDat) — File.dat slot string, e.g. "BG01".
/// - pop[1]: slot (SpriteSlot) — sprite slot index.
/// - pop[2]: x (CoordinateX) — horizontal position.
/// - pop[3]: y (CoordinateY) — vertical position.
///
/// Return: void.
///
/// Side effects: CreatesSprite, MutatesSprite.
///
/// Evidence:
/// - Writeup: docs/writeup.md sprite sections
///
/// Engine: Blocked — loads resource and sets sprite position.
///
/// Decompiler: Blocked — renders as sp_set(slot, name, x, y).
static SIG_SP_SET: ExtSig = sig!(3, 2, "sp_set", pop=4,
    params=["slot":1=SpriteSlot, "name":0=ResourceStringFromFileDat, "x":2=CoordinateX, "y":3=CoordinateY],
    return=Void, effects=[CreatesSprite, MutatesSprite], purpose="Load/set a sprite slot from a File.dat resource name.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md sprite sections"]);
/// category 3 index 3: sp_set_ex
///
/// Purpose: Load a sprite into slot with explicit z/priority in addition to
/// (x, y) position.  Extended form of sp_set.
///
/// VM arguments (display order: slot, name, x, y, z):
/// - pop[0]: name (ResourceStringFromFileDat) — File.dat slot string, e.g. "#FFFFFFFF".
/// - pop[1]: slot (SpriteSlot) — sprite slot index.
/// - pop[2]: x (CoordinateX) — horizontal position.
/// - pop[3]: y (CoordinateY) — vertical position.
/// - pop[4]: z (CoordinateZ) — depth/priority layer.
///
/// Return: void.
///
/// Side effects: CreatesSprite, MutatesSprite.
///
/// Evidence:
/// - Writeup: docs/writeup.md 24.13
/// - RuntimeTrace: sample sp_set_ex(40, "#FFFFFFFF", 0, 0, 0)
///
/// Engine: Blocked — loads resource and sets slot with z priority.
///
/// Decompiler: Blocked — renders as sp_set_ex(slot, name, x, y, z).
static SIG_SP_SET_EX: ExtSig = sig!(3, 3, "sp_set_ex", pop=5,
    params=["slot":1=SpriteSlot, "name":0=ResourceStringFromFileDat, "x":2=CoordinateX, "y":3=CoordinateY, "z":4=CoordinateZ],
    return=Void, effects=[CreatesSprite, MutatesSprite], purpose="Load/set a sprite slot with explicit z/priority.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md 24.13"]);
/// category 3 index 4: sp_set_pos
///
/// Purpose: Immediately move a sprite slot to (x, y, z) without animation.
///
/// VM arguments (display order: slot, x, y, z):
/// - pop[0]: slot (SpriteSlot) — sprite slot to reposition.
/// - pop[1]: x (CoordinateX) — new horizontal position.
/// - pop[2]: y (CoordinateY) — new vertical position.
/// - pop[3]: z (CoordinateZ) — new depth/priority.
///
/// Return: void.
///
/// Side effects: MutatesSprite.
///
/// Evidence:
/// - Writeup: docs/writeup.md sprite sections
///
/// Engine: Blocked — sets sprite position immediately via PAL sprite API.
///
/// Decompiler: Blocked — renders as sp_set_pos(slot, x, y, z).
static SIG_SP_SET_POS: ExtSig = sig!(3, 4, "sp_set_pos", pop=4,
    params=["slot":0=SpriteSlot, "x":1=CoordinateX, "y":2=CoordinateY, "z":3=CoordinateZ],
    return=Void, effects=[MutatesSprite], purpose="Move a sprite slot immediately.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md sprite sections"]);
/// category 3 index 5: sp_cls
///
/// Purpose: Release/clear a sprite slot.  slot=-1 clears all sprites.
///
/// VM arguments:
/// - pop[0]: slot (SpriteSlot) — sprite slot index, or -1 for wildcard.
///
/// Return: void.
///
/// Side effects: DeletesSprite.
///
/// Evidence:
/// - Writeup: docs/writeup.md sprite sections
/// - RuntimeTrace: pal-vm ext_sp_cls pops 1; slot=-1 branch clears all
///
/// Engine: Blocked — releases slot via PalSpriteRelease; -1 walks all slots.
///
/// Decompiler: Blocked — renders as sp_cls(slot).
static SIG_SP_CLS: ExtSig = sig!(3, 5, "sp_cls", pop=1,
    params=["slot":0=SpriteSlot], return=Void, effects=[DeletesSprite], purpose="Release/clear a sprite slot.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md sprite sections"]);
/// category 3 index 6: sp_set_alpha
///
/// Purpose: Set the transparency of a sprite slot by updating the high alpha
/// byte in the Game sprite wrapper color lane.
///
/// VM arguments:
/// - pop[0]: slot (SpriteSlot) — sprite slot index.
/// - pop[1]: alpha (Alpha) — alpha value (0=transparent, 255=opaque).
///
/// Return: status integer.
///
/// Side effects: MutatesSprite.
///
/// Evidence:
/// - Game.sqlite: sub_4281D0 pops slot, alpha; resolves wrapper with
///   sub_449120; writes `(alpha << 24) | (color & 0x00FFFFFF)` to wrapper+0x5C
///   and clears WORD wrapper+0x60.
/// - PAL.sqlite: PalSpriteSetColor_0 writes PalSprite+0x3C color; shared commit
///   path sub_448600 applies dirty flag 0x80000 through PalSpriteSetColor.
///
/// Engine: Verified — normal sprite slots and PAL encoded button layers update
/// alpha through the shared sprite/button color path; pending alpha is queued
/// until sprite creation when script emits alpha before creation.
///
/// Decompiler: Verified — renders as sp_set_alpha(slot, alpha).
static SIG_SP_SET_ALPHA: ExtSig = sig!(3, 6, "sp_set_alpha", pop=2,
    params=[
        "slot":0=SpriteSlot=>"sprite slot or encoded PAL layer slot such as 0x010000NN",
        "alpha":1=Alpha=>"0 transparent, 255 opaque"
    ],
    return=Integer, effects=[MutatesSprite],
    purpose="Set the sprite/button alpha lane by replacing the high byte of the native color field.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_4281D0 pops slot,alpha and writes wrapper color alpha high byte", PalSqlite:"reverse/PAL.sqlite PalSpriteSetColor_0 writes PalSprite+0x3C color; sub_448600 applies dirty 0x80000"],
    game="0x004281D0", pal="PalSpriteSetColor" => "0x1025FFC0");
/// category 3 index 7: set_priority
///
/// Purpose: Advance the native sprite priority cursor by one priority lane.
static SIG_SP_SET_PRIORITY_LANE: ExtSig = sig!(3, 7, "set_priority", pop=1,
    params=["lane":0=Integer=>"priority lane delta; native adds 134*lane to ctx+804232"],
    return=Integer, effects=[MutatesSprite],
    purpose="Advance the Game sprite priority cursor.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_427780 pops lane and updates VM offset +804232 by 134*lane"],
    game="0x00427780");
/// category 3 index 9: sp_set_center
///
/// Purpose: Set a sprite's PAL center offset.
static SIG_SP_SET_CENTER: ExtSig = sig!(3, 9, "sp_set_center", pop=3,
    params=["slot":0=SpriteSlot, "center_x":1=CoordinateX, "center_y":2=CoordinateY],
    return=Integer, effects=[MutatesSprite],
    purpose="Set PalSprite center offset for a sprite slot.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_4282B0 pops slot/center_x/center_y and calls PalSpriteSetCenterOffset"],
    game="0x004282B0");
/// category 3 index 12: set_filter
///
/// Purpose: Set a sprite render/filter mode flag.  The script uses this as a
/// render-pipeline selector; the VM accepts and models it as stack-disciplined
/// state without interpreting the mode value.
///
/// VM arguments:
/// - pop[0]: slot (SpriteSlot) — sprite slot index.
/// - pop[1]: filter_mode (Mode) — render pipeline mode integer.
///
/// Return: void.
///
/// Side effects: MutatesSprite.
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite decompilation search for set_filter wrapper
/// - PAL.sqlite: PalSpriteSetRenderMode 0x1011B4F8 / Game import thunk 0x4506BE
/// - RuntimeTrace: pal-vm dispatch_sprite_ext index 12 pops 2
///
/// Engine: Blocked — pops 2 args; filter_mode passed to PalSpriteSetRenderMode.
///
/// Decompiler: Blocked — renders as set_filter(slot, filter_mode).
static SIG_SP_SET_FILTER: ExtSig = sig!(3, 12, "set_filter", pop=2,
    params=["slot":0=SpriteSlot, "filter_mode":1=Mode],
    return=Void, effects=[MutatesSprite],
    purpose="Set a sprite render/filter mode; current VM treats it as stack-disciplined state because the observed script mainly uses it as a render pipeline flag.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite decompilation search for set_filter wrapper", PalSqlite:"PalSpriteSetRenderMode 0x1011B4F8 / Game import thunk 0x4506BE", RuntimeTrace:"pal-vm dispatch_sprite_ext index 12 pops 2"]);
/// category 3 index 11: sp_cls_ex
///
/// Purpose: Clear a consecutive range of sprite slots.
///
/// VM arguments:
/// - pop[0]: first_slot (SpriteSlot) — first sprite slot to release.
/// - pop[1]: count (Integer) — number of consecutive slots.
///
/// Return: void.
///
/// Side effects: DeletesSprite.
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite sprite dispatch shares 3:0005/000B/000D clear path
/// - PAL.sqlite: PalSpriteRelease 0x10120D45 / Game import thunk 0x45049E
/// - Koikake Script.src: `sp_cls_ex(126, 2)` releases the popup canvas and
///   its adjacent frame sprite after a choice is made.
///
/// Engine: Blocked — releases each slot via PalSpriteRelease.
///
/// Decompiler: Blocked — renders as sp_cls_ex(first_slot, count).
static SIG_SP_CLS_EX: ExtSig = sig!(3, 11, "sp_cls_ex", pop=2,
    params=["first_slot":0=SpriteSlot, "count":1=Integer],
    return=Void, effects=[DeletesSprite],
    purpose="Clear count consecutive sprite slots from first_slot.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite sprite dispatch shares 3:0005/000B/000D clear path", PalSqlite:"PalSpriteRelease 0x10120D45 / Game import thunk 0x45049E", RuntimeTrace:"Koikake Script.src popup teardown pushes count 2 then first_slot 126"]);
/// category 3 index 17: sp_set_scale
///
/// Purpose: Set the display scale lane of a sprite slot.
///
/// VM arguments:
/// - pop[0]: slot (SpriteSlot) — sprite slot index.
/// - pop[1]: scale (Integer) — raw percent scale, 100 = 1.0.
///
/// Return: status integer.
///
/// Side effects: MutatesSprite.
///
/// Evidence:
/// - Game.sqlite: sub_428000 pops slot, raw_scale; resolves wrapper with
///   sub_449120 and stores raw_scale / 100.0 into wrapper offset +84.  It does
///   not mutate x/y placement.
/// - PAL.sqlite: PalSpriteSetScale_0 stores its scale argument into PalSprite
///   offset +112 and returns 1.
///
/// Engine: Verified — stores native raw scale and applies it to the sprite scale
/// lane without recomputing placement.
///
/// Decompiler: Verified — renders as sp_set_scale(slot, scale).
static SIG_SP_SET_SCALE: ExtSig = sig!(3, 17, "sp_set_scale", pop=2,
    params=[
        "slot":0=SpriteSlot=>"sprite slot resolved through sub_449120",
        "scale":1=Integer=>"raw percent; native wrapper stores scale / 100.0"
    ],
    return=Integer, effects=[MutatesSprite],
    purpose="Set the sprite scale lane; native handler does not recompute x/y placement.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_428000 pops slot,scale and stores scale/100.0 into wrapper+84", PalSqlite:"reverse/PAL.sqlite PalSpriteSetScale_0 writes PalSprite+112 and returns 1"],
    game="0x00428000", pal="PalSpriteSetScale" => "0x10260DF0");
/// category 3 index 15: sp_set_rect_pos
static SIG_SP_SET_RECT_POS: ExtSig = sig!(3, 15, "sp_set_rect_pos", pop=3,
    params=["slot":0=SpriteSlot, "cell_x":1=Integer, "cell_y":2=Integer],
    return=Integer, effects=[MutatesSprite],
    purpose="Set the active sprite cell/rect position.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_4283B0 pops slot/cell_x/cell_y and calls PalSpriteRectSetPos"],
    game="0x004283B0");
/// category 3 index 18: sp_set_rotate
static SIG_SP_SET_ROTATE: ExtSig = sig!(3, 18, "sp_set_rotate", pop=4,
    params=["slot":0=SpriteSlot, "x":1=Integer, "y":2=Integer, "z":3=Integer],
    return=Integer, effects=[MutatesSprite],
    purpose="Set sprite x/y/z rotation lanes.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_427E90 pops slot/x/y/z and writes rotation fields"],
    game="0x00427E90");
/// category 3 index 21: sp_get_color
static SIG_SP_GET_COLOR: ExtSig = sig!(3, 21, "sp_get_color", pop=1,
    params=["slot":0=SpriteSlot],
    return=Integer, effects=[WritesVmMemory],
    purpose="Return a sprite's RGB color, or -1 when no image exists.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_427A20 pops slot and writes wrapper color & 0xFFFFFF to return slot"],
    game="0x00427A20");
/// category 3 index 25: sp_set_pos_move
///
/// Purpose: Move a sprite slot by a relative (dx, dy, dz) delta.  When an
/// action duration is active, the movement is interpolated over that duration;
/// script may follow with a wait if blocking is needed.
///
/// VM arguments (display order: slot, dx, dy, dz):
/// - pop[0]: slot (SpriteSlot) — sprite slot to animate.
/// - pop[1]: dx (CoordinateX) — relative x delta in configured logical coordinates.
/// - pop[2]: dy (CoordinateY) — relative y delta in configured logical coordinates.
/// - pop[3]: dz (CoordinateZ) — relative depth delta.
///
/// Return: void.
///
/// Side effects: MutatesSprite, CreatesTask.
///
/// Evidence:
/// - IDB: reverse/STRUCTS_RE.md shows VmExtcall_SpSetPosMove adds deltas to position/velocity fields.
/// - Writeup: docs/writeup.md 24.13
///
/// Engine: Blocked — mutates or tweens sprite position by relative delta.
///
/// Decompiler: Blocked — renders as sp_set_pos_move(slot, dx, dy, dz).
static SIG_SP_SET_POS_MOVE: ExtSig = sig!(3, 25, "sp_set_pos_move", pop=4,
    params=["slot":0=SpriteSlot, "dx":1=CoordinateX, "dy":2=CoordinateY, "dz":3=CoordinateZ],
    return=Void, effects=[MutatesSprite, CreatesTask], purpose="Move or tween a sprite by a relative delta.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"reverse/STRUCTS_RE.md VmExtcall_SpSetPosMove adds delta fields", Writeup:"docs/writeup.md 24.13"]);
/// category 3 index 24: sp_set_rect
static SIG_SP_SET_RECT: ExtSig = sig!(3, 24, "sp_set_rect", pop=5,
    params=["slot":0=SpriteSlot, "left":1=CoordinateX, "top":2=CoordinateY, "right":3=CoordinateX, "bottom":4=CoordinateY],
    return=Integer, effects=[MutatesSprite],
    purpose="Set a sprite source rectangle.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_4284D0 pops slot/left/top/right/bottom and writes rect fields"],
    game="0x004284D0");
/// category 3 index 26: sp_get_alpha
static SIG_SP_GET_ALPHA: ExtSig = sig!(3, 26, "sp_get_alpha", pop=1,
    params=["slot":0=SpriteSlot],
    return=Integer, effects=[WritesVmMemory],
    purpose="Return a sprite alpha value, or -1 when no image exists.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_427AE0 pops slot and writes wrapper alpha byte to return slot"],
    game="0x00427AE0");
/// category 3 index 27: sp_get_rotate
static SIG_SP_GET_ROTATE: ExtSig = sig!(3, 27, "sp_get_rotate", pop=2,
    params=["slot":0=SpriteSlot=>"sprite slot", "axis":1=Mode=>"0=x, 1=y, 2=z; other returns slot in native"],
    return=Integer, effects=[WritesVmMemory],
    purpose="Return one sprite rotation axis.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_427810 pops slot/axis and returns x/y/z rotation lane; debug string says sp_get_rotate"],
    game="0x00427810");
/// category 3 index 29: sp_get_width
///
/// Purpose: Return the PAL logical cell width of a script sprite.  PAL export
/// `PalSpriteGetWidth` reads sprite offset +0x5C, which is the cell/logical
/// width, not necessarily the backing texture width.
///
/// VM arguments:
/// - pop[0]: slot (SpriteSlot) — script sprite slot.
///
/// Return: integer width in logical pixels.
///
/// Side effects: WritesVmMemory.
///
/// Evidence:
/// - PAL.sqlite: PalSpriteGetWidth 0x1011EB2B -> PalSpriteGetWidth_0 returns
///   sprite +0x5C.
/// - RuntimeTrace: pal-vm dispatch_sprite_ext index 29 pops one slot and
///   returns SpriteSystem::get_width().
///
/// Engine: Verified — returns the sprite cell/logical width from the portable
/// sprite object.
///
/// Decompiler: Verified — renders sp_get_width(slot) instead of ext_0003_001D.
static SIG_SP_GET_WIDTH: ExtSig = sig!(3, 29, "sp_get_width", pop=1,
    params=["slot":0=SpriteSlot=>"script sprite slot"],
    return=Integer, effects=[WritesVmMemory],
    purpose="Return PAL logical/cell sprite width.",
    status=Verified, decompiler=Verified,
    evidence=[PalSqlite:"reverse/PAL.sqlite PalSpriteGetWidth 0x1011EB2B -> PalSpriteGetWidth_0 returns sprite+0x5C", RuntimeTrace:"pal-vm dispatch_sprite_ext index 29 pops slot and returns SpriteSystem::get_width"],
    pal="PalSpriteGetWidth" => "0x1011EB2B");
/// category 3 index 30: sp_get_height
///
/// Purpose: Return the PAL logical cell height of a script sprite.  PAL export
/// `PalSpriteGetHeight` reads sprite offset +0x60.
///
/// VM arguments:
/// - pop[0]: slot (SpriteSlot) — script sprite slot.
///
/// Return: integer height in logical pixels.
///
/// Side effects: WritesVmMemory.
///
/// Evidence:
/// - PAL.sqlite: PalSpriteGetHeight 0x1011E46E -> PalSpriteGetHeight_0 returns
///   sprite +0x60.
/// - RuntimeTrace: pal-vm dispatch_sprite_ext index 30 pops one slot and
///   returns SpriteSystem::get_height().
///
/// Engine: Verified — returns the sprite cell/logical height from the portable
/// sprite object.
///
/// Decompiler: Verified — renders sp_get_height(slot) instead of ext_0003_001E.
static SIG_SP_GET_HEIGHT: ExtSig = sig!(3, 30, "sp_get_height", pop=1,
    params=["slot":0=SpriteSlot=>"script sprite slot"],
    return=Integer, effects=[WritesVmMemory],
    purpose="Return PAL logical/cell sprite height.",
    status=Verified, decompiler=Verified,
    evidence=[PalSqlite:"reverse/PAL.sqlite PalSpriteGetHeight 0x1011E46E -> PalSpriteGetHeight_0 returns sprite+0x60", RuntimeTrace:"pal-vm dispatch_sprite_ext index 30 pops slot and returns SpriteSystem::get_height"],
    pal="PalSpriteGetHeight" => "0x1011E46E");
/// category 3 index 36: sp_get_scale
static SIG_SP_GET_SCALE: ExtSig = sig!(3, 36, "sp_get_scale", pop=1,
    params=["slot":0=SpriteSlot],
    return=Integer, effects=[WritesVmMemory],
    purpose="Return a sprite scale as a PAL percent value, or -1 when no image exists.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_427940 pops slot and writes summed native scale lanes * 100 to return slot"],
    game="0x00427940");
/// category 3 index 37: sp_set_color
static SIG_SP_SET_COLOR_SPRITE: ExtSig = sig!(3, 37, "sp_set_color", pop=2,
    params=["slot":0=SpriteSlot, "rgb":1=Color],
    return=Integer, effects=[MutatesSprite],
    purpose="Set a sprite RGB color while preserving alpha.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_428120 pops slot/rgb and writes low 24 bits of the color field"],
    game="0x00428120");
/// category 3 index 44: sp_set_vis_clip
static SIG_SP_SET_VIS_CLIP: ExtSig = sig!(3, 44, "sp_set_vis_clip", pop=1,
    params=["slot":0=SpriteSlot],
    return=Integer, effects=[MutatesSprite],
    purpose="Enable the native sprite vis-clip bit.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_424FD0 pops slot and ORs wrapper flag 0x1000000"],
    game="0x00424FD0");
/// category 3 index 49: sp_set_child
static SIG_SP_SET_CHILD: ExtSig = sig!(3, 49, "sp_set_child", pop=5,
    params=["parent_slot":0=SpriteSlot, "child_slot":1=SpriteSlot, "offset_x":2=CoordinateX, "offset_y":3=CoordinateY, "child_index":4=Integer],
    return=Integer, effects=[MutatesSprite],
    purpose="Attach a child sprite lane to a parent sprite.",
    status=Blocked, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_424A20 pops parent_slot/child_slot/offset_x/offset_y/child_index and writes child-lane fields"],
    game="0x00424A20");
/// category 3 index 50: sp_set_transition
static SIG_SP_SET_TRANSITION_HOT: ExtSig = sig!(3, 50, "sp_set_transition", pop=4,
    params=["slot":1=SpriteSlot=>"sprite slot whose image is replaced", "name":0=ResourceStringFromFileDat=>"new File.dat image resource", "transition_id":2=Integer=>"native transition table id", "duration_ms":3=DurationMs=>"transition duration"],
    return=Integer, effects=[CreatesSprite, MutatesSprite, CreatesTask],
    purpose="Replace a sprite image and queue a PAL transition from the old image to the new image.",
    status=Blocked, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_428F10 pops name/slot/transition/duration, saves old sprite pointer, loads new image, and queues 0x30032 render action", PalSqlite:"PalSpriteSetTransition 0x1011E851 is the PAL transition primitive"],
    game="0x00428F10", pal="PalSpriteSetTransition" => "0x1011E851");
/// category 3 index 51: sp_copy_image
static SIG_SP_COPY_IMAGE_HOT: ExtSig = sig!(3, 51, "sp_copy_image", pop=1,
    params=["slot":0=SpriteSlot=>"sprite slot whose current image is moved to the transition source lane"],
    return=Integer, effects=[MutatesSprite],
    purpose="Move the current sprite image into the wrapper's transition source/copy lane.",
    status=Blocked, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_424940 pops slot, cancels active transition, stores current sprite pointer into wrapper old-image lane, and zeros the live pointer"],
    game="0x00424940");
/// category 3 index 52: sp_transition
static SIG_SP_TRANSITION_HOT: ExtSig = sig!(3, 52, "sp_transition", pop=3,
    params=["slot":0=SpriteSlot=>"sprite slot to transition", "transition_id":1=Integer=>"native transition table id", "duration_ms":2=DurationMs=>"transition duration"],
    return=Integer, effects=[MutatesSprite, CreatesTask],
    purpose="Start a PAL transition for the current sprite, using any saved source image lane.",
    status=Blocked, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_424790 pops slot/transition/duration, creates transition handle, and queues 0x30032 or skip action", PalSqlite:"PalSpriteSetTransition 0x1011E851 and PalSpriteCancelTransition 0x1011B309"],
    game="0x00424790", pal="PalSpriteSetTransition" => "0x1011E851");
/// category 3 index 54: get_backbuffer
static SIG_SP_GET_BACKBUFFER: ExtSig = sig!(3, 54, "get_backbuffer", pop=1,
    params=["slot":0=SpriteSlot],
    return=Integer, effects=[MutatesSprite],
    purpose="Copy a sprite image into PAL's active backbuffer.",
    status=Blocked, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_424620 pops slot and calls PalSpriteBackBafferCopy"],
    game="0x00424620");
/// category 3 index 55: sp_set_mask
///
/// Purpose: Apply a file-backed alpha mask to an existing PAL sprite surface.
/// Game.exe stores the nine values in a wrapper mask lane and immediately calls
/// sub_422FD0, which resolves `mask_resource + ".tga"` and invokes
/// PalSpriteMaskAlpha(live_sprite, dst_x, dst_y, width, height, mask, mask_x,
/// mask_y). Coordinates are target/mask surface pixels, not screen coordinates.
static SIG_SP_SET_MOTION: ExtSig = sig!(3, 55, "sp_set_mask", pop=9,
    params=["slot":0=SpriteSlot, "dst_x":1=CoordinateX, "dst_y":2=CoordinateY, "width":3=Integer, "height":4=Integer, "mask_resource":5=ResourceStringFromFileDat, "mask_x":6=CoordinateX, "mask_y":7=CoordinateY, "lane":8=Integer],
    return=Integer, effects=[MutatesSprite, ReadsFile],
    purpose="Apply an alpha mask resource to a live sprite surface.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_425EF0 pops slot/dst_x/dst_y/width/height/resource/mask_x/mask_y/lane, writes mask lane fields, and calls sub_422FD0; sub_422FD0 calls PalSpriteMaskAlpha", PalSqlite:"PAL.dll PalSpriteMaskAlpha is the target alpha-mask primitive"],
    game="0x00425EF0");
/// category 3 index 8: sp_get_filename
///
/// Purpose: Copy a sprite slot's current resource filename into a dynamic
/// string destination and return the native resource id.  Used by title/menu
/// table helpers to inspect sprite resources.
static SIG_SP_GET_FILENAME: ExtSig = sig!(3, 8, "sp_get_filename", pop=2,
    params=["slot":0=SpriteSlot=>"script sprite slot", "dst_slot":1=BufferPointer=>"dynamic string destination"],
    return=Integer, effects=[WritesVmMemory],
    purpose="Copy sprite resource filename/name to dynamic string storage.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite VmExtcall_SpSet2/sub_4244E0 pops slot and dst string slot, copies sprite filename, returns resource id"],
    game="0x004244E0");
/// category 3 index 16: sp_set_render_mode
///
/// Purpose: Set the PAL render mode for a sprite.  Native passes mode+1 to
/// PalSpriteSetRenderMode.
static SIG_SP_SET_RENDER_MODE: ExtSig = sig!(3, 16, "sp_set_render_mode", pop=2,
    params=["slot":0=SpriteSlot=>"script sprite slot", "mode":1=Mode=>"native render mode before +1 adjustment"],
    return=Integer, effects=[MutatesSprite],
    purpose="Set sprite render/blend mode.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_427530 pops slot/mode and calls PalSpriteSetRenderMode(mode+1)"],
    game="0x00427530");
/// category 3 index 28: sp_get_pos_to_mem
///
/// Purpose: Write a sprite's composite x/y/z position to up to three Mem.dat
/// destination indexes; -1 disables an individual destination.
static SIG_SP_GET_POS_TO_MEM: ExtSig = sig!(3, 28, "sp_get_pos_to_mem", pop=4,
    params=["slot":0=SpriteSlot=>"script sprite slot", "dst_x":1=BufferPointer=>"Mem.dat destination for x or -1", "dst_y":2=BufferPointer=>"Mem.dat destination for y or -1", "dst_z":3=BufferPointer=>"Mem.dat destination for z or -1"],
    return=Integer, effects=[WritesVmMemory],
    purpose="Write sprite x/y/z position into Mem.dat destinations.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_427D10 pops slot/dst_x/dst_y/dst_z and writes composite position to Mem.dat when dst != -1"],
    game="0x00427D10");
/// category 3 index 31: sp_set_anim2
///
/// Purpose: Configure a nine-argument sprite animation/mix-file lane.  Native
/// stores the lane into sprite state and may call the sprite mix loader.
static SIG_SP_SET_ANIM2: ExtSig = sig!(3, 31, "sp_set_anim2", pop=9,
    params=["slot":0=SpriteSlot, "arg1":1=Integer, "arg2":2=Integer, "arg3":3=Integer, "arg4":4=Integer, "resource":5=ResourceStringFromFileDat, "arg6":6=Integer, "arg7":7=Integer, "lane":8=Integer],
    return=Integer, effects=[MutatesSprite, ReadsFile],
    purpose="Configure native sprite mix/animation lane with nine parameters.",
    status=Blocked, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_4266C0 pops nine values and writes sprite lane fields before sub_422F10"],
    game="0x004266C0");
/// category 3 index 19: face_init
///
/// Purpose: Initialize one native face table entry.  The latest IDB names the
/// handler `VmExtcall_SpSetPosEx`, but the decompiled body at 0x004273C0 logs
/// `face_init` and writes VM offsets +710420..+710432: mapped sprite slot,
/// priority lane, center x, and center y.
static SIG_FACE_INIT: ExtSig = sig!(3, 19, "face_init", pop=5,
    params=["face_id":0=Integer=>"face table index", "sprite_slot":1=SpriteSlot=>"script sprite slot used for this face part", "center_x":2=Coordinate=>"face anchor x or -1", "center_y":3=Coordinate=>"face anchor y or -1", "priority_lane":4=Integer=>"native priority lane stored as -134*lane"],
    return=Integer, effects=[MutatesSprite],
    purpose="Initialize a face/part slot mapping and anchor table entry.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_4273C0 pops face_id/sprite_slot/center_x/center_y/priority_lane, writes ctx+710420..710432, and logs face_init"],
    game="0x004273C0");
/// category 3 index 20: face_set
///
/// Purpose: Replace the image part for a face table entry.  Native pops a
/// resource id, face id, dx, dy; releases the mapped old sprite; loads the new
/// PGD; positions it from the face anchor/default image offset; and commits it
/// through the PAL sprite path.
static SIG_FACE_SET: ExtSig = sig!(3, 20, "face_set", pop=4,
    params=["resource":0=ResourceStringFromFileDat=>"PGD/part resource or 0x0fffffff clear sentinel", "face_id":1=Integer=>"face table index", "dx":2=Coordinate=>"x offset from anchor/default", "dy":3=Coordinate=>"y offset from anchor/default"],
    return=Integer, effects=[MutatesSprite, ReadsFile],
    purpose="Load or clear a face/standing-picture part through the face table.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_426E50 pops resource/face_id/dx/dy, resolves ctx+710420 face table, releases old sprite, loads resource, computes position, and logs face_set"],
    game="0x00426E50");
/// category 3 index 23: face_cls
///
/// Purpose: Clear a face table entry's mapped sprite slot.
static SIG_FACE_CLS: ExtSig = sig!(3, 23, "face_cls", pop=1,
    params=["face_id":0=Integer=>"face table index"],
    return=Integer, effects=[MutatesSprite],
    purpose="Release the sprite currently mapped to a face table entry.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_426DB0 pops face_id, reads ctx+710420[face_id], calls sub_4232E0 on that sprite wrapper, and logs face_cls"],
    game="0x00426DB0");
/// category 3 index 34: sp_set_anim_param
///
/// Purpose: Store a native per-sprite integer parameter later returned by
/// sp_get_anim_param.
static SIG_SP_SET_ANIM_PARAM: ExtSig = sig!(3, 34, "sp_set_anim_param", pop=2,
    params=["slot":0=SpriteSlot, "value":1=Integer],
    return=Integer, effects=[MutatesSprite],
    purpose="Store native sprite parameter at ctx+666396.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_425870 pops slot/value and stores value at ctx+666396 for that slot"],
    game="0x00425870");
/// category 3 index 35: sp_get_anim_param
///
/// Purpose: Return the per-sprite integer parameter stored by index 34.
static SIG_SP_GET_ANIM_PARAM: ExtSig = sig!(3, 35, "sp_get_anim_param", pop=1,
    params=["slot":0=SpriteSlot],
    return=Integer, effects=[WritesVmMemory],
    purpose="Return native sprite parameter at ctx+666396.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_4257F0 pops slot and writes ctx+666396 slot value to return"],
    game="0x004257F0");
/// category 3 index 48: is_sp
///
/// Purpose: Return whether a script sprite or motion slot is currently live.
static SIG_IS_SP: ExtSig = sig!(3, 48, "is_sp", pop=1,
    params=["slot":0=SpriteSlot],
    return=Bool, effects=[WritesVmMemory],
    purpose="Test whether a sprite slot currently exists.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_424CE0 pops slot and returns whether script/motion sprite storage is non-null"],
    game="0x00424CE0");
/// category 3 index 56: sp_set_motion_pos
///
/// Purpose: Configure a nine-argument sprite bitblt/motion lane.  Native stores
/// rectangle/resource/lane fields and calls sub_422E10.
static SIG_SP_SET_MOTION_POS: ExtSig = sig!(3, 56, "sp_set_motion_pos", pop=9,
    params=["slot":0=SpriteSlot, "arg1":1=Integer, "arg2":2=Integer, "arg3":3=Integer, "arg4":4=Integer, "resource":5=ResourceStringFromFileDat, "arg6":6=Integer, "arg7":7=Integer, "lane":8=Integer],
    return=Integer, effects=[MutatesSprite, ReadsFile],
    purpose="Configure native sprite motion/bitblt lane with nine parameters.",
    status=Blocked, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_426180 pops nine values and writes sprite lane fields before sub_422E10"],
    game="0x00426180");
/// category 3 index 41: sp_set_anim
///
/// Purpose: Attach a named sheet or ANI animation resource to a sprite slot
/// and begin playback.
///
/// VM arguments (display order: slot, name, flag):
/// - pop[0]: slot (SpriteSlot) — sprite slot to animate.
/// - pop[1]: name (ResourceStringFromFileDat) — animation resource name.
/// - pop[2]: flag (Flag) — playback control flag (loop/oneshot).
///
/// Return: void.
///
/// Side effects: MutatesSprite, CreatesTask.
///
/// Evidence:
/// - Writeup: docs/writeup.md 24.13
///
/// Engine: Blocked — loads animation and attaches to sprite slot.
///
/// Decompiler: Blocked — renders as sp_set_anim(slot, name, flag).
static SIG_SP_SET_ANIM_41: ExtSig = sig!(3, 41, "sp_set_anim", pop=3,
    params=["slot":0=SpriteSlot, "name":1=ResourceStringFromFileDat, "flag":2=Flag],
    return=Void, effects=[MutatesSprite, CreatesTask], purpose="Attach sheet or named ANI animation to a sprite slot.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md 24.13"]);
/// category 3 index 57: sp_set_anim (alias)
///
/// Purpose: Alias/wrapper for sp_set_anim at index 41; identical arity and
/// argument layout.  Exact behavioral difference between indices 41 and 57
/// is not confirmed.
///
/// VM arguments: identical to sp_set_anim (3,41).
/// - pop[0]: slot (SpriteSlot)
/// - pop[1]: name (ResourceStringFromFileDat)
/// - pop[2]: flag (Flag)
///
/// Return: void.
///
/// Side effects: MutatesSprite, CreatesTask.
///
/// Evidence:
/// - Writeup: docs/writeup.md 24.13
///
/// Engine: Blocked — shares implementation path with index 41.
///
/// Decompiler: Blocked — renders as sp_set_anim(slot, name, flag).
///
/// Open points: Exact difference from index 41 not reversed.
static SIG_SP_SET_ANIM_57: ExtSig = sig!(3, 57, "sp_set_anim", pop=3,
    params=["slot":0=SpriteSlot, "name":1=ResourceStringFromFileDat, "flag":2=Flag],
    return=Void, effects=[MutatesSprite, CreatesTask], purpose="Alias/wrapper for sprite animation setup.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md 24.13"]);
/// category 3 index 53: set_aspect_position_type
///
/// Purpose: Store the sprite's PAL aspect-position conversion type.  Native
/// Game.exe writes a small table value into sprite wrapper field 26; the shared
/// sprite commit path later passes that value to PalSpriteConvertAspectPosition.
///
/// VM arguments:
/// - pop[0]: slot (SpriteSlot) — sprite slot looked up with sub_449120.
/// - pop[1]: type_index (Mode) — index into xmmword_4A3FC0.  In this build the
///   table bytes are `[0, 1, 2, 3]`.
///
/// Return: status integer.
///
/// Side effects: MutatesSprite.
///
/// Evidence:
/// - Game.sqlite: sub_4246C0 pops slot then type_index, writes
///   dword_4C02A8[189 * wrapper_index] = xmmword_4A3FC0[type_index].
/// - PAL.sqlite: PalSpriteConvertAspectPosition only adjusts x/y when PAL
///   aspect-position conversion is enabled and global aspect mode is 3.
///
/// Engine: Partial — stores the per-slot type and routes through a centralized
/// conversion hook; exact PAL letterbox rectangle math is still a no-op until the
/// portable renderer exposes those rectangles.
///
/// Decompiler: Verified — renders both parameters instead of the old zero-arg
/// fallback.
static SIG_SP_SET_ASPECT_POSITION_TYPE: ExtSig = sig!(3, 53, "set_aspect_position_type", pop=2,
    params=[
        "slot":0=SpriteSlot=>"sprite slot looked up with sub_449120",
        "type_index":1=Mode=>"index into native table xmmword_4A3FC0 = [0,1,2,3]"
    ],
    return=Integer, effects=[MutatesSprite],
    purpose="Store PAL aspect-position conversion type for a sprite slot.",
    status=Partial, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_4246C0 pops slot,type_index and writes wrapper field 26", PalSqlite:"reverse/PAL.sqlite PalSpriteConvertAspectPosition_0 consumes the stored type"],
    game="0x004246C0", pal="PalSpriteConvertAspectPosition" => "0x101215A1");
/// category 3 index 39: sp_set_shake
///
/// Purpose: Apply a PAL-style shake (vibration) animation to a sprite slot.
/// The VM models it as alternating offset motion.
///
/// VM arguments (display order: slot, amplitude, count, axis):
/// - pop[0]: slot (SpriteSlot) — sprite slot to shake.
/// - pop[1]: amplitude (Integer) — shake displacement magnitude in pixels.
/// - pop[2]: count (Integer) — number of shake oscillations.
/// - pop[3]: axis (Mode) — shake axis: 0=x, 1=y, 2=both (engine-defined).
///
/// Return: void.
///
/// Side effects: MutatesSprite, CreatesTask.
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite sprite handler path around sub_425640/sub_4275F0 stack pops
/// - RuntimeTrace: pal-vm ext_sp_set_shake pops 4; operand_fixture slot/amplitude/count/axis
/// - PAL.sqlite: PalSpriteSetOffsetPos 0x1011C65A / PalSpriteMoveOffsetPos 0x1011E022
///
/// Engine: Blocked — spawns shake-animation task using offset-position APIs.
///
/// Decompiler: Blocked — renders as sp_set_shake(slot, amplitude, count, axis).
static SIG_SP_SET_SHAKE: ExtSig = sig!(3, 39, "sp_set_shake", pop=4,
    params=["slot":0=SpriteSlot, "amplitude":1=Integer, "count":2=Integer, "axis":3=Mode],
    return=Void, effects=[MutatesSprite, CreatesTask],
    purpose="Apply PAL-style shake animation to a sprite slot; VM currently models it as alternating offset motion.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite sprite handler path around sub_425640/sub_4275F0 stack pops", RuntimeTrace:"pal-vm ext_sp_set_shake pops 4; operand_fixture slot/amplitude/count/axis", PalSqlite:"PalSpriteSetOffsetPos 0x1011C65A / PalSpriteMoveOffsetPos 0x1011E022"]);
/// category 3 index 46: sp_show
///
/// Purpose: Make a sprite slot visible.  Calls PalSpriteViewCtrl(slot, true).
/// Pop count corrected from 2 to 1: runtime pops only the slot; there is no
/// separate visible flag argument.
///
/// VM arguments:
/// - pop[0]: slot (SpriteSlot) — sprite slot to show.
///
/// Return: void.
///
/// Side effects: MutatesSprite.
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite sprite view-control handler
/// - PAL.sqlite: PalSpriteViewCtrl 0x1011B859 / Game import thunk 0x4503BE
/// - RuntimeTrace: pal-vm ext_sp_view_ctrl pops 1 (slot only)
///
/// Engine: Blocked — delegates to PalSpriteViewCtrl(slot, true).
///
/// Decompiler: Blocked — renders as sp_show(slot).
static SIG_SP_SHOW: ExtSig = sig!(3, 46, "sp_show", pop=1,
    params=["slot":0=SpriteSlot],
    return=Void, effects=[MutatesSprite],
    purpose="Show sprite; VM calls PalSpriteViewCtrl(slot, true) after popping only the slot.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite sprite view-control handler", PalSqlite:"PalSpriteViewCtrl 0x1011B859 / Game import thunk 0x4503BE", RuntimeTrace:"pal-vm ext_sp_view_ctrl pops 1 (slot only)"]);
/// category 3 index 47: sp_hide
///
/// Purpose: Make a sprite slot invisible.  Calls PalSpriteViewCtrl(slot, false).
/// Pop count corrected from 2 to 1: runtime pops only the slot.
///
/// VM arguments:
/// - pop[0]: slot (SpriteSlot) — sprite slot to hide.
///
/// Return: void.
///
/// Side effects: MutatesSprite.
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite sprite view-control handler
/// - PAL.sqlite: PalSpriteViewCtrl 0x1011B859 / Game import thunk 0x4503BE
/// - RuntimeTrace: pal-vm ext_sp_view_ctrl pops 1 (slot only)
///
/// Engine: Blocked — delegates to PalSpriteViewCtrl(slot, false).
///
/// Decompiler: Blocked — renders as sp_hide(slot).
static SIG_SP_HIDE: ExtSig = sig!(3, 47, "sp_hide", pop=1,
    params=["slot":0=SpriteSlot],
    return=Void, effects=[MutatesSprite],
    purpose="Hide sprite; VM calls PalSpriteViewCtrl(slot, false) after popping only the slot.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite sprite view-control handler", PalSqlite:"PalSpriteViewCtrl 0x1011B859 / Game import thunk 0x4503BE", RuntimeTrace:"pal-vm ext_sp_view_ctrl pops 1 (slot only)"]);
/// category 3 index 22: sptext
///
/// Purpose: Create a transparent text sprite through the PAL font/text surface
/// path (AdvCommandSpText).  Renders a Text.dat string directly onto a named
/// sprite slot surface.
///
/// VM arguments (display order: slot, text_id, x, y, mode):
/// - pop[0]: slot (SpriteSlot) — target sprite slot.
/// - pop[1]: text_id (TextId) — Text.dat byte offset for the string.
/// - pop[2]: x (CoordinateX) — render position x.
/// - pop[3]: y (CoordinateY) — render position y.
/// - pop[4]: mode (Mode) — text render mode/channel.
///
/// Return: void.
///
/// Side effects: CreatesSprite, ChangesTextState.
///
/// Evidence:
/// - Writeup: docs/writeup.md AdvCommandSpText
/// - RuntimeTrace: AdvCommandSpText 5-arg path
///
/// Engine: Blocked — renders text surface into sprite slot.
///
/// Decompiler: Blocked — renders as sptext(slot, text_id, x, y, mode).
static SIG_SPTEXT: ExtSig = sig!(3, 22, "sptext", pop=5,
    params=["slot":0=SpriteSlot, "text_id":1=TextId, "x":2=CoordinateX, "y":3=CoordinateY, "mode":4=Mode],
    return=Void, effects=[CreatesSprite, ChangesTextState], purpose="Create a transparent text sprite through the PAL font/text surface path.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md AdvCommandSpText", RuntimeTrace:"AdvCommandSpText 5-arg path"]);

// Category 4/5/13 - audio.

/// category 4 index 0: bgm_play
///
/// Purpose: Load and play a BGM track into a slot with volume and flags.
///
/// VM arguments (display order: slot, unknown, name, flags, volume, unknown2, unknown3):
/// - pop[0]: slot (SoundSlot) — BGM slot index.
/// - pop[1]: unknown (Unknown) — purpose not reversed; likely loop mode.
/// - pop[2]: name (ResourceStringFromFileDat) — File.dat resource name.
/// - pop[3]: flags (Flag) — playback flags (loop/fade).
/// - pop[4]: volume (Volume) — initial volume.
/// - pop[5]: unknown2 (Unknown) — purpose not reversed.
/// - pop[6]: unknown3 (Unknown) — purpose not reversed.
///
/// Return: void.
///
/// Side effects: CreatesSound, PlaysSound.
///
/// Evidence:
/// - Writeup: docs/writeup.md audio
///
/// Engine: Blocked — loads BGM resource and begins playback.
///
/// Decompiler: Blocked — renders 7 args; unknown2/unknown3 meanings TBD.
///
/// Open points: Exact semantics of pop[1], pop[5], pop[6] not reversed.
static SIG_BGM_PLAY: ExtSig = sig!(4, 0, "bgm_play", pop=7,
    params=["slot":0=SoundSlot, "unknown":1=Unknown, "name":2=ResourceStringFromFileDat, "flags":3=Flag, "volume":4=Volume, "unknown2":5=Unknown, "unknown3":6=Unknown],
    return=Void, effects=[CreatesSound, PlaysSound], purpose="Load/play BGM with slot and volume parameters.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md audio"]);
/// category 4 index 1: bgm_stop
///
/// Purpose: Stop a BGM slot, optionally with a fade-out duration.
///
/// VM arguments:
/// - pop[0]: slot (SoundSlot) — BGM slot to stop.
/// - pop[1]: fade_ms (DurationMs) — fade duration in milliseconds; 0 = immediate.
///
/// Return: void.
///
/// Side effects: StopsSound.
///
/// Evidence:
/// - Writeup: docs/writeup.md audio
///
/// Engine: Blocked — stops slot with optional fade.
///
/// Decompiler: Blocked — renders as bgm_stop(slot, fade_ms).
static SIG_BGM_STOP: ExtSig = sig!(4, 1, "bgm_stop", pop=2,
    params=["slot":0=SoundSlot, "fade_ms":1=DurationMs], return=Void, effects=[StopsSound], purpose="Stop BGM slot, optionally with fade.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md audio"]);
/// category 4 index 2: bgm_set_volume
///
/// Purpose: Set the global BGM volume level.  Pop count corrected from 2 to 1:
/// runtime pops only volume; there is no per-slot argument.
///
/// VM arguments:
/// - pop[0]: volume (Volume) — new BGM volume (0–100).
///
/// Return: void.
///
/// Side effects: PlaysSound.
///
/// Evidence:
/// - Writeup: docs/writeup.md audio
/// - RuntimeTrace: pal-vm ext_bgm_set_volume pops 1
///
/// Engine: Blocked — sets global BGM volume.
///
/// Decompiler: Blocked — renders as bgm_set_volume(volume).
static SIG_BGM_SET_VOLUME: ExtSig = sig!(4, 2, "bgm_set_volume", pop=1,
    params=["volume":0=Volume], return=Void, effects=[PlaysSound],
    purpose="Set global BGM volume; runtime pops 1 arg (volume only, no per-slot).",
    status=Blocked, decompiler=Blocked,
    evidence=[Writeup:"docs/writeup.md audio", RuntimeTrace:"pal-vm ext_bgm_set_volume pops 1"]);
/// category 4 index 3: bgm_get_volume
///
/// Purpose: Read the current BGM volume level.
///
/// VM arguments:
/// - pop[0]: slot (SoundSlot) — BGM slot to query (runtime currently ignores
///   this and returns a cached/constant value).
///
/// Return: Integer — current BGM volume (0–100).  Runtime returns 100 as
/// placeholder until BGM is properly tracked.
///
/// Side effects: none.
///
/// Evidence:
/// - Writeup: docs/writeup.md audio
/// - RuntimeTrace: dispatch_bgm_ext(3) returns Value(100) without consuming slot
///
/// Engine: Blocked — returns constant 100; does not query real PAL state.
/// NOTE: runtime has a stack-leak bug: pop[0] is NOT consumed.
///
/// Decompiler: Blocked — renders as bgm_get_volume(slot).
///
/// Open points: Runtime must pop slot arg before returning (stack-leak bug at
/// dispatch_bgm_ext index 3).
static SIG_BGM_GET_VOLUME: ExtSig = sig!(4, 3, "bgm_get_volume", pop=1,
    params=["slot":0=SoundSlot], return=Integer, effects=[], purpose="Read BGM volume state.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md audio"]);
/// category 4 index 6: bgm_set_auto_volume
///
/// Purpose: Configure the BGM automatic ducking/auto-volume latch.
///
/// VM arguments:
/// - pop[0]: enabled (Flag) — non-zero enables auto volume behavior.
/// - pop[1]: volume (Volume) — auto-volume percent cached by Game.exe.
///
/// Return: void.
///
/// Side effects: ChangesSoundState.
///
/// Evidence:
/// - GameSqlite: `sub_40BFB0` pops two args, stores ctx[165003]/ctx[165004],
///   and restores normal BGM group volume when disabled.
static SIG_BGM_SET_AUTO_VOLUME: ExtSig = sig!(4, 6, "bgm_set_auto_volume", pop=2,
    params=["enabled":0=Flag, "volume":1=Volume], return=Void, effects=[PlaysSound],
    purpose="Configure BGM automatic volume/ducking state.",
    status=Blocked, decompiler=Blocked, evidence=[GameSqlite:"reverse/Game.sqlite sub_40BFB0"]);
/// category 4 index 11: set_master_volume
///
/// Purpose: Set the primary/master volume.
///
/// VM arguments:
/// - pop[0]: volume (Volume) — script percent, converted to PAL raw `* 100`.
///
/// Return: void.
///
/// Evidence: Game.sqlite `sub_40BD40`.
static SIG_SET_MASTER_VOLUME: ExtSig = sig!(4, 11, "set_master_volume", pop=1,
    params=["volume":0=Volume], return=Void, effects=[PlaysSound],
    purpose="Set primary/master volume percent.",
    status=Blocked, decompiler=Blocked, evidence=[GameSqlite:"reverse/Game.sqlite sub_40BD40"]);
/// category 4 index 13: mute_master_volume
///
/// Purpose: Gate the primary/master volume without changing the configured
/// slider value.
///
/// VM arguments:
/// - pop[0]: muted (Flag) — non-zero mutes, zero restores.
///
/// Return: void.
///
/// Evidence: Game.sqlite `sub_40BC70`.
static SIG_MUTE_MASTER_VOLUME: ExtSig = sig!(4, 13, "mute_master_volume", pop=1,
    params=["muted":0=Flag], return=Void, effects=[PlaysSound],
    purpose="Mute/unmute primary volume while preserving configured percent.",
    status=Blocked, decompiler=Blocked, evidence=[GameSqlite:"reverse/Game.sqlite sub_40BC70"]);
/// category 4 index 14: bgm_mute
///
/// Purpose: Gate BGM group volume without changing the configured BGM slider.
///
/// VM arguments:
/// - pop[0]: muted (Flag) — non-zero mutes, zero restores.
///
/// Evidence: Game.sqlite `sub_40C230`.
static SIG_BGM_MUTE: ExtSig = sig!(4, 14, "bgm_mute", pop=1,
    params=["muted":0=Flag], return=Void, effects=[PlaysSound],
    purpose="Mute/unmute BGM group while preserving configured percent.",
    status=Blocked, decompiler=Blocked, evidence=[GameSqlite:"reverse/Game.sqlite sub_40C230"]);
/// category 4 index 15: mute_bgm_auto_volume
///
/// Purpose: Toggle BGM auto-volume mute/enable latch.
///
/// VM arguments:
/// - pop[0]: muted (Flag) — non-zero disables/gates auto volume.
///
/// Evidence: Game.sqlite `sub_40C060`.
static SIG_MUTE_BGM_AUTO_VOLUME: ExtSig = sig!(4, 15, "mute_bgm_auto_volume", pop=1,
    params=["muted":0=Flag], return=Void, effects=[ChangesRunState],
    purpose="Toggle BGM automatic volume mute latch.",
    status=Blocked, decompiler=Blocked, evidence=[GameSqlite:"reverse/Game.sqlite sub_40C060"]);
/// category 4 index 9: bgm_load
///
/// Purpose: Preload a BGM resource into a slot without starting playback.
///
/// VM arguments (display order: slot, name, unknown, unknown2, unknown3):
/// - pop[0]: slot (SoundSlot) — destination BGM slot.
/// - pop[1]: name (ResourceStringFromFileDat) — File.dat resource name.
/// - pop[2]: unknown (Unknown) — purpose not reversed.
/// - pop[3]: unknown2 (Unknown) — purpose not reversed.
/// - pop[4]: unknown3 (Unknown) — purpose not reversed.
///
/// Return: void.
///
/// Side effects: CreatesSound.
///
/// Evidence:
/// - Writeup: docs/writeup.md audio
///
/// Engine: Blocked — preloads BGM resource into slot.
///
/// Decompiler: Blocked — renders 5 args; unknown meanings TBD.
///
/// Open points: Exact semantics of pop[2..4] not reversed.
static SIG_BGM_LOAD: ExtSig = sig!(4, 9, "bgm_load", pop=5,
    params=["slot":0=SoundSlot, "name":1=ResourceStringFromFileDat, "unknown":2=Unknown, "unknown2":3=Unknown, "unknown3":4=Unknown],
    return=Void, effects=[CreatesSound], purpose="Preload BGM resource into a slot.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md audio"]);
/// category 5 index 1: se_play
///
/// Purpose: Load and play a sound effect resource.
///
/// VM arguments (display order: slot, name, unknown, flags, volume):
/// - pop[0]: slot (SoundSlot) — SE slot index.
/// - pop[1]: name (ResourceStringFromFileDat) — File.dat resource name.
/// - pop[2]: unknown (Unknown) — purpose not reversed; likely pan/channel.
/// - pop[3]: flags (Flag) — playback flags.
/// - pop[4]: volume (Volume) — playback volume.
///
/// Return: void.
///
/// Side effects: CreatesSound, PlaysSound.
///
/// Evidence:
/// - Writeup: docs/writeup.md audio
///
/// Engine: Blocked — pops 5 args; loads and plays SE resource.
///
/// Decompiler: Blocked — renders 5 args.
static SIG_SE_PLAY: ExtSig = sig!(5, 1, "se_play", pop=5,
    params=["slot":0=SoundSlot, "name":1=ResourceStringFromFileDat, "unknown":2=Unknown, "flags":3=Flag, "volume":4=Volume],
    return=Void, effects=[CreatesSound, PlaysSound], purpose="Play SE resource.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md audio"]);
/// category 5 index 2: se_play_ex
///
/// Purpose: Play a sound effect; extended variant observed before first text_w
/// in many scenes.  Shares the same 5-arg layout as se_play.
///
/// VM arguments (same as se_play):
/// - pop[0]: slot (SoundSlot)
/// - pop[1]: name (ResourceStringFromFileDat)
/// - pop[2]: unknown (Unknown)
/// - pop[3]: flags (Flag)
/// - pop[4]: volume (Volume)
///
/// Return: void.
///
/// Side effects: CreatesSound, PlaysSound.
///
/// Evidence:
/// - Writeup: docs/writeup.md 24.1
///
/// Engine: Blocked — shares se_play dispatch path.
///
/// Decompiler: Blocked — renders 5 args.
///
/// Open points: Exact behavioral difference from se_play (index 1) not reversed.
static SIG_SE_PLAY_EX: ExtSig = sig!(5, 2, "se_play_ex", pop=5,
    params=["slot":0=SoundSlot, "name":1=ResourceStringFromFileDat, "unknown":2=Unknown, "flags":3=Flag, "volume":4=Volume],
    return=Void, effects=[CreatesSound, PlaysSound], purpose="Play SE resource; observed before first text_w.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md 24.1"]);
/// category 5 index 0: se_load
///
/// Purpose: Preload a sound effect resource into a slot without playing it.
///
/// VM arguments:
/// - pop[0]: slot (SoundSlot) — destination SE slot.
/// - pop[1]: name (ResourceStringFromFileDat) — File.dat resource name.
///
/// Return: void.
///
/// Side effects: CreatesSound.
///
/// Evidence:
/// - Writeup: docs/writeup.md audio
///
/// Engine: Blocked — dispatched through ext_se_play (index 0 path).
///
/// Decompiler: Blocked — renders as se_load(slot, name).
static SIG_SE_LOAD: ExtSig = sig!(5, 0, "se_load", pop=2,
    params=["slot":0=SoundSlot, "name":1=ResourceStringFromFileDat], return=Void, effects=[CreatesSound],
    purpose="Preload SE resource.", status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md audio"]);
/// category 5 index 3: se_stop
///
/// Purpose: Stop a sound effect slot, optionally with a fade.
///
/// VM arguments:
/// - pop[0]: slot (SoundSlot) — SE slot to stop.
/// - pop[1]: fade_ms (DurationMs) — fade duration in ms; 0 = immediate.
///
/// Return: void.
///
/// Side effects: StopsSound.
///
/// Evidence:
/// - Writeup: docs/writeup.md audio
///
/// Engine: Blocked — stops slot with optional fade.
///
/// Decompiler: Blocked — renders as se_stop(slot, fade_ms).
static SIG_SE_STOP: ExtSig = sig!(5, 3, "se_stop", pop=2,
    params=["slot":0=SoundSlot, "fade_ms":1=DurationMs], return=Void, effects=[StopsSound],
    purpose="Stop SE slot.", status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md audio"]);
/// category 5 index 4: se_set_volume
///
/// Purpose: Set the volume for a sound effect lane.
///
/// VM arguments:
/// - pop[0]: volume (Volume) — new volume (0–100).
/// - pop[1]: slot (SoundSlot) — SE lane index.
///
/// Return: void.
///
/// Side effects: PlaysSound.
///
/// Evidence:
/// - Game.sqlite: `sub_434C50` pops `v4` then `v6`, stores
///   `cached_volume[v6] = v4 * 100`, and writes through `dword_49C264[v6]`.
///
/// Engine: Verified — updates the lane cache and projected portable SE group.
///
/// Decompiler: Verified — renders as se_set_volume(volume, slot).
static SIG_SE_SET_VOLUME: ExtSig = sig!(5, 4, "se_set_volume", pop=2,
    params=["volume":0=Volume, "slot":1=SoundSlot], return=Void, effects=[PlaysSound],
    purpose="Set SE lane volume.", status=Verified, decompiler=Verified, evidence=[GameSqlite:"reverse/Game.sqlite sub_434C50"]);
/// category 5 index 5: se_get_volume
///
/// Purpose: Read the current volume of a sound effect slot.
///
/// VM arguments:
/// - pop[0]: slot (SoundSlot) — SE slot to query.
///
/// Return: Integer — current SE volume (0–100).
///
/// Side effects: none.
///
/// Evidence:
/// - Writeup: docs/writeup.md audio
///
/// Engine: Blocked — returns cached SE volume for slot.
///
/// Decompiler: Blocked — renders as se_get_volume(slot).
static SIG_SE_GET_VOLUME: ExtSig = sig!(5, 5, "se_get_volume", pop=1,
    params=["slot":0=SoundSlot], return=Integer, effects=[],
    purpose="Read SE volume.", status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md audio"]);
/// category 5 index 14: se_mute
///
/// Purpose: Mute/unmute an SE slot or SE group lane.
///
/// VM arguments:
/// - pop[0]: muted (Flag) — non-zero mutes, zero restores.
/// - pop[1]: slot (SoundSlot) — SE slot/lane.
///
/// Return: void.
///
/// Evidence: Game.sqlite `sub_434D00` pops `v9` then `v4`, logs
/// `se_mute %d, %d`, stores `enabled = (v9 == 0)` in the lane latch, and
/// calls `PalSoundSetVolume(enabled * cached_volume[v4], dword_49C264[v4])`.
static SIG_SE_MUTE: ExtSig = sig!(5, 14, "se_mute", pop=2,
    params=["muted":0=Flag, "slot":1=SoundSlot], return=Void, effects=[PlaysSound],
    purpose="Mute/unmute SE lane while preserving cached SE volume.",
    status=Verified, decompiler=Verified, evidence=[GameSqlite:"reverse/Game.sqlite sub_434D00"]);
/// category 13 index 0: voice_play
///
/// Purpose: Play a voice/dialogue audio resource.
///
/// VM arguments (display order: slot, name, flags, volume):
/// - pop[0]: slot (SoundSlot) — voice slot index.
/// - pop[1]: name (ResourceStringFromFileDat) — File.dat resource name.
/// - pop[2]: flags (Flag) — playback flags.
/// - pop[3]: volume (Volume) — playback volume.
///
/// Return: void.
///
/// Side effects: CreatesSound, PlaysSound.
///
/// Evidence:
/// - Writeup: docs/writeup.md audio
///
/// Engine: Blocked — loads and plays voice resource.
///
/// Decompiler: Blocked — renders as voice_play(slot, name, flags, volume).
static SIG_VOICE_PLAY: ExtSig = sig!(13, 0, "voice_play", pop=4,
    params=["slot":0=SoundSlot, "name":1=ResourceStringFromFileDat, "flags":2=Flag, "volume":3=Volume],
    return=Void, effects=[CreatesSound, PlaysSound], purpose="Play voice resource.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md audio"]);
/// category 13 index 1: voice_stop
///
/// Purpose: Stop a voice slot, optionally with fade.
///
/// VM arguments:
/// - pop[0]: slot (SoundSlot) — voice slot to stop.
/// - pop[1]: fade_ms (DurationMs) — fade duration in ms; 0 = immediate.
///
/// Return: void.
///
/// Side effects: StopsSound.
///
/// Evidence:
/// - Writeup: docs/writeup.md audio
///
/// Engine: Blocked — stops voice slot.
///
/// Decompiler: Blocked — renders as voice_stop(slot, fade_ms).
static SIG_VOICE_STOP: ExtSig = sig!(13, 1, "voice_stop", pop=2,
    params=["slot":0=SoundSlot, "fade_ms":1=DurationMs], return=Void, effects=[StopsSound],
    purpose="Stop voice slot.", status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md audio"]);
/// category 13 index 2: voice_set_volume
///
/// Purpose: Set the global voice volume.  Pop count corrected from 2 to 1:
/// runtime pops only volume; there is no per-slot argument.
///
/// VM arguments:
/// - pop[0]: volume (Volume) — new voice volume (0–100).
///
/// Return: void.
///
/// Side effects: PlaysSound.
///
/// Evidence:
/// - Writeup: docs/writeup.md audio
/// - RuntimeTrace: pal-vm dispatch_voice_ext index 2 pops 1
///
/// Engine: Blocked — sets global voice volume.
///
/// Decompiler: Blocked — renders as voice_set_volume(volume).
static SIG_VOICE_SET_VOLUME: ExtSig = sig!(13, 2, "voice_set_volume", pop=1,
    params=["volume":0=Volume], return=Void, effects=[PlaysSound],
    purpose="Set global voice volume; runtime pops 1 arg (volume only, no per-slot).",
    status=Blocked, decompiler=Blocked,
    evidence=[Writeup:"docs/writeup.md audio", RuntimeTrace:"pal-vm dispatch_voice_ext index 2 pops 1"]);
/// category 13 index 3: voice_get_volume
///
/// Purpose: Read the current voice volume.
///
/// VM arguments: none.
///
/// Return: Integer — current voice volume (0–100).
///
/// Side effects: none.
///
/// Evidence:
/// - GameSqlite: sub_4445A0 writes ctx[ctx[184629] + 177710] and performs no
///   stack-top decrement.
///
/// Engine: Blocked — returns cached modeled volume; native derives it from the
/// PAL-scaled ctx voice volume field.
///
/// Decompiler: Blocked — renders as voice_get_volume().
static SIG_VOICE_GET_VOLUME: ExtSig = sig!(13, 3, "voice_get_volume", pop=0,
    params=[], return=Integer, effects=[],
    purpose="Read global voice volume.", status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_4445A0"]);
/// category 13 index 7: voice_play_fade
///
/// Purpose: Set the fade/scheduling value consumed by the following voice-play
/// setup path.
///
/// VM arguments:
/// - pop[0]: fade_ms (DurationMs) — native stores this at ctx+660028.
///
/// Return: status integer.
///
/// Evidence: Game.sqlite `sub_444190` pops one value and stores it at
/// ctx+660028. `reverse/GAME_VM_RE.md` maps category 13 index 7 to
/// `VmExtcall_VoicePlayFade`; category 13 index 24 is the real
/// `VmExtcall_VoiceMute`.
static SIG_VOICE_PLAY_FADE: ExtSig = sig!(13, 7, "voice_play_fade", pop=1,
    params=["fade_ms":0=DurationMs], return=Integer, effects=[PlaysSound],
    purpose="Set native voice-play fade/scheduling latch.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_444190 pops one value and writes ctx+660028", Writeup:"reverse/GAME_VM_RE.md category 13 index 7 = VmExtcall_VoicePlayFade"],
    game="0x00444190");
/// category 13 index 18: voice_wait
///
/// Purpose: Block the VM until the specified voice slot finishes playback.
///
/// VM arguments:
/// - pop[0]: slot (SoundSlot) — voice slot to wait on.
///
/// Return: void.
///
/// Side effects: BlocksScript.
///
/// Evidence:
/// - GameSqlite: sub_443800 pops slot only on first pass, ORs ctx[163819],
///   stores timing fields, and rewinds ctx[201015] by 12 while not skipped.
///
/// Engine: Blocked — runtime models the native first-pass pop plus PC rewind
/// polling against the portable audio backend.
///
/// Decompiler: Blocked — renders as voice_wait(slot).
///
/// Open points: Runtime must implement actual blocking via WaitRequest::Frame
/// polling the voice slot completion state.
static SIG_VOICE_WAIT: ExtSig = sig!(13, 18, "voice_wait", pop=1,
    params=["slot":0=SoundSlot], return=Void, effects=[BlocksScript],
    purpose="Wait for voice slot completion.", status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_443800"]);
/// category 13 index 16: set_voice_autopan_size_over
///
/// Purpose: Configure one voice auto-pan entry used by Game.exe for positional
/// voice panning.
///
/// VM arguments (display order: slot, target, name, mode):
/// - pop[0]: slot (Integer) — auto-pan table slot.
/// - pop[1]: target (Integer) — sprite slot/wrapper target, or -1 for clear.
/// - pop[2]: name (TextStringFromTextDat) — <=32 byte voice/autopan label.
/// - pop[3]: mode (Integer) — entry mode; target=-1 and mode=0 clears slot.
///
/// Return: void.
///
/// Side effects: PlaysSound (modifies pan configuration).
///
/// Evidence:
/// - Game.sqlite: handler 0x004438D0 `VmExtcall_VoiceAutopanSizeOver` pops
///   slot, target, name, mode; writes ctx+658964+40*slot or clears the 40-byte
///   entry when target=-1 and mode=0.
/// - RuntimeTrace: longrun-newgame-ctrl-60000 executes this helper 4991 times.
///
/// Engine: Verified — pal-vm stores/clears the portable autopan table and
/// enforces the native 32-byte string guard.
///
/// Decompiler: Verified — renders 4 args in display order.
static SIG_VOICE_AUTOPAN_SIZE_OVER: ExtSig = sig!(13, 16, "set_voice_autopan_size_over", pop=4,
    params=["slot":0=Integer, "target":1=Integer, "name":2=TextStringFromTextDat, "mode":3=Integer],
    return=Void, effects=[PlaysSound],
    purpose="Configure or clear one native voice auto-pan table entry.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"0x004438D0 VmExtcall_VoiceAutopanSizeOver pops slot,target,name,mode and mutates ctx+658964+40*slot", RuntimeTrace:"longrun-newgame-ctrl-60000 executes set_voice_autopan_size_over 4991 times"],
    game="0x004438D0");

// Category 7 - waits. These block only via TaskSystem, not through text calls.

/// category 7 index 0: wait
///
/// Purpose: Block the script for a fixed duration.  When skip_cancel is
/// non-zero the player can advance the wait by clicking.
///
/// VM arguments:
/// - pop[0]: duration_ms (DurationMs) — wait duration in milliseconds.
/// - pop[1]: skip_cancel (Flag) — non-zero allows click/key to skip.
///
/// Return: void.
///
/// Side effects: CreatesTask, BlocksScript.
///
/// Evidence:
/// - Game.sqlite: sub_444F40 pops duration then skip/cancel flag, records
///   start time with PaltimeGetTime, rewinds PC until time/cancel completes,
///   and returns 1.
/// - RuntimeTrace: pal-vm dispatch_wait_ext index 0 pops 2 and emits
///   WaitRequest::Time.
///
/// Engine: Verified — emits a timed wait request and traces the decoded
/// skip/cancel flag.
///
/// Decompiler: Verified — renders as wait(duration_ms, skip_cancel).
static SIG_WAIT: ExtSig = sig!(7, 0, "wait", pop=2,
    params=["duration_ms":0=DurationMs, "skip_cancel":1=Flag], return=Void, effects=[CreatesTask, BlocksScript],
    purpose="Timed script wait with native cancel/skip flag.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_444F40 pops duration/skip flag and blocks by rewinding PC until timeout", RuntimeTrace:"pal-vm dispatch_wait_ext index 0 pops duration/skip flag and returns WaitRequest::Time"],
    game="0x00444F40");
/// category 7 index 1: wait_click
///
/// Purpose: Wait for player input or advance a native text task.  duration_ms
/// -1 is a special text-task completion shortcut; 0 is normalized to a short
/// wait; positive values create a click-or-timeout wait.
///
/// VM arguments:
/// - pop[0]: duration_ms (DurationMs) — -1 completes the active text task,
///   0 becomes a one-tick wait, positive values are timeout ms.
///
/// Return: void.
///
/// Side effects: CreatesTask, BlocksScript, ChangesTextState.
///
/// Evidence:
/// - Game.sqlite: sub_444DE0 pops duration; runtime traces through the title
///   values record a click-or-timeout wait, rewind PC, and return 1.  The -1
///   branch clears the text flag at ctx[1050] or calls sub_43A860, then calls
///   sub_44A010 and returns without rewinding.
/// - RuntimeTrace: pal-vm dispatch_wait_ext index 1 pops duration and gates
///   the portable text reveal/click path.
///
/// Engine: Verified — completes text reveal for -1 and emits ClickOrTime for
/// non-negative durations.
///
/// Decompiler: Verified — renders as wait_click(duration_ms).
static SIG_WAIT_CLICK: ExtSig = sig!(7, 1, "wait_click", pop=1,
    params=["duration_ms":0=DurationMs], return=Void, effects=[CreatesTask, BlocksScript, ChangesTextState],
    purpose="-1 completes the active text task; non-negative values are click-or-time waits.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_444DE0 pops duration; -1 clears/completes text task via ctx[1050]/sub_43A860/sub_44A010; non-negative values rewind until click/timeout", RuntimeTrace:"pal-vm dispatch_wait_ext index 1 gates the portable text reveal/click path"],
    game="0x00444DE0");
/// category 7 index 2: wait_sync_begin
///
/// Purpose: Begin a PAL wait-sync timing scope.  Marks the start of a timed
/// section; use wait_sync_release or wait_sync_step to fence.
///
/// VM arguments: none.
///
/// Return: void.
///
/// Side effects: CreatesTask.
///
/// Evidence:
/// - Game.sqlite/writeup: recovered wait-sync fields at VM offsets
///   +655244/+655248.
/// - RuntimeTrace: pal-vm stores cached PAL time on index 2.
///
/// Engine: Verified — records sync start timestamp and clears a pending
/// release wait.
///
/// Decompiler: Verified — renders as wait_sync_begin().
static SIG_WAIT_SYNC_BEGIN: ExtSig = sig!(7, 2, "wait_sync_begin", pop=0, params=[],
    return=Void, effects=[CreatesTask], purpose="Begin PAL wait-sync timing scope.",
    status=Verified, decompiler=Verified, evidence=[GameSqlite:"docs/writeup.md wait-sync offsets +655244/+655248 from Game.sqlite", RuntimeTrace:"pal-vm dispatch_wait_ext index 2 stores pal_time_ms"]);
/// category 7 index 3: wait_sync_release
///
/// Purpose: End a wait-sync scope and block until the elapsed time since
/// wait_sync_begin reaches duration_ms.
///
/// VM arguments:
/// - pop[0]: duration_ms (DurationMs) — minimum total scope duration.
///
/// Return: void.
///
/// Side effects: CreatesTask, BlocksScript.
///
/// Evidence:
/// - Game.sqlite/writeup: recovered wait-sync duration/start fields at
///   +655244/+655248.
/// - RuntimeTrace: pal-vm emits a timed WaitRequest and resumes after elapsed
///   duration.
///
/// Engine: Verified — computes remaining time and emits a timed WaitRequest.
///
/// Decompiler: Verified — renders as wait_sync_release(duration_ms).
static SIG_WAIT_SYNC_RELEASE: ExtSig = sig!(7, 3, "wait_sync_release", pop=1,
    params=["duration_ms":0=DurationMs], return=Void, effects=[CreatesTask, BlocksScript],
    purpose="Release wait-sync with timed block.", status=Verified, decompiler=Verified, evidence=[GameSqlite:"docs/writeup.md wait-sync offsets +655244/+655248 from Game.sqlite", RuntimeTrace:"pal-vm wait_sync_release blocks until elapsed duration"]);
/// category 7 index 4: wait_sync_end
///
/// Purpose: Close a wait-sync timing scope without blocking.
///
/// VM arguments: none.
///
/// Return: void.
///
/// Side effects: none (timing scope metadata only).
///
/// Evidence:
/// - Game.sqlite/writeup: recovered wait-sync fields at VM offsets
///   +655244/+655248.
///
/// Engine: Verified — resets sync timestamp and pending release state.
///
/// Decompiler: Verified — renders as wait_sync_end().
static SIG_WAIT_SYNC_END: ExtSig = sig!(7, 4, "wait_sync_end", pop=0, params=[],
    return=Void, effects=[], purpose="End wait-sync timing scope.", status=Verified, decompiler=Verified, evidence=[GameSqlite:"docs/writeup.md wait-sync offsets +655244/+655248 from Game.sqlite", RuntimeTrace:"pal-vm dispatch_wait_ext index 4 clears wait-sync state"]);
/// category 7 index 5: wait_sync_step
///
/// Purpose: One-frame wait/work fence used inside script animation loops.  The
/// Game handler only marks the native sync-step flag; the portable runtime
/// yields one rendered frame for the same observable script pacing.
///
/// VM arguments: none.
///
/// Return: void.
///
/// Side effects: CreatesTask, BlocksScript.
///
/// Evidence: Game.sqlite sub_444AC0 sets the native field at ctx+804084 to 1
/// and returns success without popping arguments or resetting the sync timer.
static SIG_WAIT_SYNC_STEP: ExtSig = sig!(7, 5, "wait_sync_step", pop=0, params=[],
    return=Void, effects=[CreatesTask, BlocksScript], purpose="One-frame wait/work fence used by script animation loops.",
    status=Verified, decompiler=Verified, evidence=[GameSqlite:"reverse/Game.sqlite sub_444AC0 sets ctx+804084=1 and returns 1"],
    game="0x00444AC0");
/// category 7 index 7: wait_click_no_anim
///
/// Purpose: Input wait without triggering the wait-icon animation; otherwise
/// identical to wait_click.
///
/// VM arguments:
/// - pop[0]: duration_ms (DurationMs) — timeout in ms, or <= 0 for click-only.
///
/// Return: void.
///
/// Side effects: CreatesTask, BlocksScript.
///
/// Evidence:
/// - Game.sqlite: sub_444C90 pops duration, shares wait_click timeout
///   semantics, but logs wait_click_no_anim and bypasses wait-icon animation.
/// - RuntimeTrace: pal-vm dispatch_wait_ext index 7 pops duration and emits
///   Click or ClickOrTime wait without wait-icon side effects.
///
/// Engine: Verified — emits the same blocking wait as wait_click while
/// avoiding wait-icon animation state.
///
/// Decompiler: Verified — renders as wait_click_no_anim(duration_ms).
static SIG_WAIT_CLICK_NO_ANIM: ExtSig = sig!(7, 7, "wait_click_no_anim", pop=1,
    params=["duration_ms":0=DurationMs], return=Void, effects=[CreatesTask, BlocksScript],
    purpose="Input wait without wait-icon animation side effects; non-positive durations are click-only.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_444C90 implements wait_click_no_anim duration semantics", RuntimeTrace:"pal-vm dispatch_wait_ext index 7 returns Click for <=0 or ClickOrTime for positive waits"],
    game="0x00444C90");
/// category 7 index 8: wait_sync_get_time
///
/// Purpose: Read elapsed PAL wait-sync time since the last wait_sync_begin.
///
/// VM arguments: none.
///
/// Return: Integer — elapsed time in milliseconds.
///
/// Side effects: none.
///
/// Evidence:
/// - Game.sqlite/writeup: recovered begin timestamp field at +655248.
/// - RuntimeTrace: pal-vm returns cached PAL time delta.
///
/// Engine: Verified — returns elapsed ms since last sync_begin.
///
/// Decompiler: Verified — renders as wait_sync_get_time().
static SIG_WAIT_SYNC_GET_TIME: ExtSig = sig!(7, 8, "wait_sync_get_time", pop=0, params=[],
    return=Integer, effects=[], purpose="Read elapsed PAL wait-sync time.", status=Verified, decompiler=Verified, evidence=[GameSqlite:"docs/writeup.md wait-sync begin timestamp +655248 from Game.sqlite", RuntimeTrace:"pal-vm dispatch_wait_ext index 8 returns pal_time_ms - wait_sync_begin_ms"]);
/// category 7 index 9: wait_time_push
///
/// Purpose: Push current PAL wait-time state onto an internal stack, allowing
/// nested wait-time scopes.
///
/// VM arguments: none.
///
/// Return: void.
///
/// Side effects: none (wait-time stack mutation only).
///
/// Evidence:
/// - Writeup: docs/writeup.md wait
///
/// Engine: Blocked — saves wait-time context.
///
/// Decompiler: Blocked — renders as wait_time_push().
static SIG_WAIT_TIME_PUSH: ExtSig = sig!(7, 9, "wait_time_push", pop=0, params=[],
    return=Void, effects=[], purpose="Push PAL wait time state.", status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md wait"]);
/// category 7 index 10: wait_time_pop
///
/// Purpose: Restore previously pushed PAL wait-time state.
///
/// VM arguments: none.
///
/// Return: void.
///
/// Side effects: none (wait-time stack mutation only).
///
/// Evidence:
/// - Writeup: docs/writeup.md wait
///
/// Engine: Blocked — restores wait-time context.
///
/// Decompiler: Blocked — renders as wait_time_pop().
static SIG_WAIT_TIME_POP: ExtSig = sig!(7, 10, "wait_time_pop", pop=0, params=[],
    return=Void, effects=[], purpose="Pop PAL wait time state.", status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md wait"]);

// Category 6 - select/menu choice state.

/// category 6 index 0: select_init
///
/// Purpose: Initialize the select/menu system with a prompt text and color
/// scheme for normal, hovered, and disabled button states.
///
/// VM arguments (display order: text_id, normal_color, hover_color, disabled_color):
/// - pop[0]: text_id (TextId) — prompt/header text id, or -1.
/// - pop[1]: normal_color (Color) — default option color.
/// - pop[2]: hover_color (Color) — highlighted option color.
/// - pop[3]: disabled_color (Color) — color for disabled options.
///
/// Return: void.
///
/// Side effects: ChangesSelectState.
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite select handler; pal-vm dispatch_select_stub index 0 pops 4
///
/// Engine: Blocked — stores select layout and color scheme.
///
/// Decompiler: Blocked — renders 4 args in display order.
static SIG_SELECT_INIT: ExtSig = sig!(6, 0, "select_init", pop=4,
    params=["text_id":0=TextId, "normal_color":1=Color, "hover_color":2=Color, "disabled_color":3=Color],
    return=Void, effects=[ChangesSelectState],
    purpose="Initialize select/menu layout and colors.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite select handler; pal-vm dispatch_select_stub index 0 pops 4"]);
/// category 6 index 2: select_set
///
/// Purpose: Register a single selectable menu option with its display text,
/// jump target, position, color, and flags.
///
/// VM arguments (display order: text_id, target, x, y, color, flags):
/// - pop[0]: text_id (TextId) — option label text id.
/// - pop[1]: target (PointId) — script jump target when selected.
/// - pop[2]: x (CoordinateX) — display position x.
/// - pop[3]: y (CoordinateY) — display position y.
/// - pop[4]: color (Color) — option-specific color override.
/// - pop[5]: flags (Flag) — option state flags (enabled/disabled/etc).
///
/// Return: void.
///
/// Side effects: ChangesSelectState.
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite select handler; pal-vm dispatch_select_stub index 2 pops 6
/// - RuntimeTrace: select_option text/target/pos/color/flags
///
/// Engine: Blocked — appends option entry to select state.
///
/// Decompiler: Blocked — renders 6 args in display order.
static SIG_SELECT_SET: ExtSig = sig!(6, 2, "select_set", pop=6,
    params=["text_id":0=TextId, "target":1=PointId, "x":2=CoordinateX, "y":3=CoordinateY, "color":4=Color, "flags":5=Flag],
    return=Void, effects=[ChangesSelectState],
    purpose="Register one selectable option with target/process metadata.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite select handler; pal-vm dispatch_select_stub index 2 pops 6", RuntimeTrace:"select_option text/target/pos/color/flags"]);
/// category 6 index 3: select_commit
///
/// Purpose: Commit/refresh the current select option list.  Native consumes no
/// arguments; later select input/query handlers use the current option table.
///
/// VM arguments: none.
///
/// Return: void.
///
/// Side effects: ChangesSelectState.
///
/// Evidence:
/// - RuntimeTrace: reachable 0006:0003 follows select_set sequences.
/// - pal-vm: dispatch_select_stub index 3 pops zero args and logs option count.
///
/// Engine: Blocked — records the commit boundary for select state.
///
/// Decompiler: Blocked — renders as select_commit().
static SIG_SELECT_COMMIT: ExtSig = sig!(6, 3, "select_commit", pop=0,
    params=[],
    return=Void, effects=[ChangesSelectState],
    purpose="Commit/refresh the current select option list.",
    status=Blocked, decompiler=Blocked,
    evidence=[RuntimeTrace:"reachable 0006:0003 follows select_set sequences", Disassembly:"out/extcall_report.json reachable extcall 0006:0003"]);
/// category 6 index 4: select_clear
///
/// Purpose: Clear all registered select/menu options and reset menu state.
///
/// VM arguments: none.
///
/// Return: void.
///
/// Side effects: ChangesSelectState.
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite select handler; pal-vm dispatch_select_stub index 4 pops 0
///
/// Engine: Blocked — clears option list.
///
/// Decompiler: Blocked — renders as select_clear().
static SIG_SELECT_CLEAR: ExtSig = sig!(6, 4, "select_clear", pop=0,
    params=[],
    return=Void, effects=[ChangesSelectState],
    purpose="Clear all select/menu state.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite select handler; pal-vm dispatch_select_stub index 4 pops 0"]);

// Category 8 - title buttons.

/// category 8 index 0: btn_init
///
/// Purpose: Initialize a button group for use.  Pop count corrected from 1
/// to 3: runtime pops group + 2 additional configuration args whose meaning
/// has not been reversed.
///
/// VM arguments:
/// - pop[0]: group (ButtonSlot) — button group index to initialize.
/// - pop[1]: normal_image (ResourceStringFromFileDat) — optional normal group
///   image resource, or 0x0FFFFFFF sentinel.
/// - pop[2]: hover_image (ResourceStringFromFileDat) — optional hover group
///   image resource, or 0x0FFFFFFF sentinel.
///
/// Return: void.
///
/// Side effects: ChangesSelectState.
///
/// Evidence:
/// - Disassembly: docs/dis.txt category 8
/// - RuntimeTrace: pal-vm ext_btn_init pops 3
///
/// Engine: Verified — stores a portable button-group record with normal/hover
/// image ids and clears stale queued reactions for the group.
///
/// Decompiler: Verified — renders as btn_init(group, normal_image, hover_image).
static SIG_BTN_INIT: ExtSig = sig!(8, 0, "btn_init", pop=3,
    params=["group":0=ButtonSlot=>"button group", "normal_image":1=ResourceStringFromFileDat=>"normal group image or sentinel", "hover_image":2=ResourceStringFromFileDat=>"hover group image or sentinel"],
    return=Integer, effects=[ChangesSelectState],
    purpose="Initialize a native PAL button group with normal/hover image resources.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_40FC60 pops group/normal_image/hover_image and calls PalButtonCreateEx", RuntimeTrace:"pal-vm ext_btn_init stores button group state"],
    game="0x0040FC60");
/// category 8 index 1: btn_uninit
///
/// Purpose: Release all buttons belonging to the specified group.
///
/// VM arguments:
/// - pop[0]: group (ButtonSlot) — button group to uninitialize.
///
/// Return: void.
///
/// Side effects: DeletesSprite.
///
/// Evidence:
/// - Game.sqlite: sub_40DB20 walks active button groups, releases their
///   sprites, calls PalButtonRelease, and clears native group storage.
/// - PAL.sqlite: PalButtonRelease 0x1011D72B / PalButtonDelete 0x1011ACF1.
/// - RuntimeTrace: pal-vm ext_btn_uninit pops one group argument and returns 1.
///
/// Engine: Verified — compatibility runtime filters by group, accepts -1 as
/// all-groups wildcard, clears queued reactions, and releases sprite handles.
///
/// Decompiler: Verified — renders as btn_uninit(group).
static SIG_BTN_UNINIT: ExtSig = sig!(8, 1, "btn_uninit", pop=1,
    params=["group":0=ButtonSlot], return=Void, effects=[DeletesSprite],
    purpose="Release a button group; group=-1 is treated as all groups by the compatibility runtime.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_40DB20 releases active button sprites and calls PalButtonRelease", PalSqlite:"PalButtonRelease 0x1011D72B / PalButtonDelete 0x1011ACF1", RuntimeTrace:"pal-vm ext_btn_uninit pops group and returns 1"],
    game="0x0040DB20", pal="PalButtonRelease" => "0x1011D72B");
/// category 8 index 3: btn_set
///
/// Purpose: Create or register a button entry within a group, binding four
/// sprite resource names (normal, hover, push, disabled states) and an entry
/// flag.
///
/// VM arguments (display order: group, index, normal, hover, push, disabled, entry_flag):
/// - pop[0]: group (ButtonSlot) — button group index.
/// - pop[1]: index (ButtonSlot) — button index within group.
/// - pop[2]: normal (ResourceStringFromFileDat) — normal-state sprite.
/// - pop[3]: hover (ResourceStringFromFileDat) — hover/focus-state sprite.
/// - pop[4]: push (ResourceStringFromFileDat) — pressed-state sprite.
/// - pop[5]: disabled (ResourceStringFromFileDat) — disabled-state sprite.
/// - pop[6]: entry_flag (Flag) — button state/type flags.
///
/// Return: void.
///
/// Side effects: CreatesSprite.
///
/// Evidence:
/// - Writeup: docs/writeup.md 24.10
///
/// Engine: Blocked — loads sprites for each button state.
///
/// Decompiler: Blocked — renders 7 args in display order.
static SIG_BTN_SET: ExtSig = sig!(8, 3, "btn_set", pop=7,
    params=["group":0=ButtonSlot, "index":1=ButtonSlot, "normal":2=ResourceStringFromFileDat, "hover":3=ResourceStringFromFileDat, "push":4=ResourceStringFromFileDat, "disabled":5=ResourceStringFromFileDat, "entry_flag":6=Flag],
    return=Void, effects=[CreatesSprite], purpose="Create/register a button entry and its sprite states.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md 24.10"]);
/// category 8 index 4: btn_hide
///
/// Purpose: Hide an already registered button entry by submitting a native
/// button sprite visibility command.
///
/// VM arguments:
/// - pop[0]: group (ButtonSlot) — button group index.
/// - pop[1]: index (ButtonSlot) — button index within group.
///
/// Return: void/status 1.
///
/// Side effects: MutatesSprite, ChangesSelectState.
///
/// Evidence:
/// - Game.sqlite: sub_40F290 pops group/index, builds command 0x80004, and
///   logs btn_hide.
/// - RuntimeTrace: pal-vm ext_btn_view_ctrl(false) pops group/index and
///   hides matching sprite handles.
///
/// Engine: Verified — toggles matching button sprites invisible.
///
/// Decompiler: Verified — renders as btn_hide(group, index).
static SIG_BTN_HIDE: ExtSig = sig!(8, 4, "btn_hide", pop=2,
    params=["group":0=ButtonSlot, "index":1=ButtonSlot],
    return=Void, effects=[MutatesSprite, ChangesSelectState],
    purpose="Hide a registered button entry.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_40F290 pops group/index and queues native btn_hide command", RuntimeTrace:"pal-vm ext_btn_view_ctrl(false) pops group/index and hides matching sprites"],
    game="0x0040F290");
/// category 8 index 5: btn_show
///
/// Purpose: Show an already registered button entry by submitting a native
/// button sprite visibility command.
///
/// VM arguments:
/// - pop[0]: group (ButtonSlot) — button group index.
/// - pop[1]: index (ButtonSlot) — button index within group.
///
/// Return: void/status 1.
///
/// Side effects: MutatesSprite, ChangesSelectState.
///
/// Evidence:
/// - Game.sqlite: sub_40F1B0 pops group/index, builds command 0x80005, and
///   logs btn_show.
/// - RuntimeTrace: pal-vm ext_btn_view_ctrl(true) pops group/index and shows
///   matching sprite handles.
///
/// Engine: Verified — toggles matching button sprites visible.
///
/// Decompiler: Verified — renders as btn_show(group, index).
static SIG_BTN_SHOW: ExtSig = sig!(8, 5, "btn_show", pop=2,
    params=["group":0=ButtonSlot, "index":1=ButtonSlot],
    return=Void, effects=[MutatesSprite, ChangesSelectState],
    purpose="Show a registered button entry.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_40F1B0 pops group/index and queues native btn_show command", RuntimeTrace:"pal-vm ext_btn_view_ctrl(true) pops group/index and shows matching sprites"],
    game="0x0040F1B0");
/// category 8 index 6: btn_set_pos
///
/// Purpose: Move a button entry's sprite to (x, y).
///
/// VM arguments (display order: group, index, x, y):
/// - pop[0]: group (ButtonSlot) — button group index.
/// - pop[1]: index (ButtonSlot) — button index within group.
/// - pop[2]: x (CoordinateX) — new horizontal position.
/// - pop[3]: y (CoordinateY) — new vertical position.
///
/// Return: void.
///
/// Side effects: MutatesSprite.
///
/// Evidence:
/// - Writeup: docs/writeup.md 24.10
///
/// Engine: Blocked — repositions button sprite.
///
/// Decompiler: Blocked — renders as btn_set_pos(group, index, x, y).
static SIG_BTN_SET_POS: ExtSig = sig!(8, 6, "btn_set_pos", pop=4,
    params=["group":0=ButtonSlot, "index":1=ButtonSlot, "x":2=CoordinateX, "y":3=CoordinateY],
    return=Void, effects=[MutatesSprite], purpose="Position a button entry.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md 24.10"]);
/// category 8 index 8: btn_release
///
/// Purpose: Release one or more button entries.  index=-1 releases all
/// entries in the group.
///
/// VM arguments:
/// - pop[0]: group (ButtonSlot) — button group index.
/// - pop[1]: index (ButtonSlot) — button index within group, or -1 for all.
///
/// Return: void.
///
/// Side effects: DeletesSprite.
///
/// Evidence:
/// - Writeup: docs/writeup.md 24.10
///
/// Engine: Blocked — releases button sprite entries.
///
/// Decompiler: Blocked — renders as btn_release(group, index).
static SIG_BTN_RELEASE: ExtSig = sig!(8, 8, "btn_release", pop=2,
    params=["group":0=ButtonSlot, "index":1=ButtonSlot], return=Void, effects=[DeletesSprite],
    purpose="Release one or more button entries.", status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md 24.10"]);
/// category 8 index 9: btn_slider_get
///
/// Purpose: Query/update a slider button offset from the current mouse
/// position.  Used by the SYSTEM/SOUND menu for text-window alpha and audio
/// volume sliders.
///
/// VM arguments:
/// - pop[0]: group (ButtonSlot) — button group index.
/// - pop[1]: index (ButtonSlot) — slider button index within group.
/// - pop[2]: max_offset (Integer) — maximum pixel offset for the slider.
/// - pop[3]: axis (Mode) — 0=horizontal, 1=vertical.
/// - pop[4]: snap_to_100 (Flag) — non-zero enables native hundred-step snapping.
///
/// Return: Integer — current slider offset in pixels.
///
/// Side effects: MutatesSprite, ChangesSelectState.
///
/// Evidence:
/// - Game.sqlite: sub_40EC10 pops five values, reads PalInputGetMouseX/Y, clamps
///   to the supplied max offset, stores the sprite offset, and writes dst.
/// - RuntimeTrace: setting menu calls ext_0008_0009 with five pushed args before
///   converting the result to alpha/volume percentages.
///
/// Engine: Verified — compatible runtime computes/stores slider offset and
/// returns it.
///
/// Decompiler: Verified — renders as btn_slider_get(group,index,max_offset,axis,snap_to_100).
static SIG_BTN_SLIDER_GET: ExtSig = sig!(8, 9, "btn_slider_get", pop=5,
    params=["group":0=ButtonSlot, "index":1=ButtonSlot, "max_offset":2=Integer, "axis":3=Mode, "snap_to_100":4=Flag],
    return=Integer, effects=[MutatesSprite, ChangesSelectState],
    purpose="Query/update slider button offset from mouse position for settings sliders.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_40EC10 pops group/index/max_offset/axis/snap_to_100 and writes slider offset to dst", RuntimeTrace:"docs/dis.txt 00030D20 and 00044124 push five args before ext_0008_0009"],
    game="0x0040EC10");
/// category 8 index 10: btn_slider_set
///
/// Purpose: Initialize/update a slider button offset and enabled state.
///
/// VM arguments:
/// - pop[0]: group (ButtonSlot)
/// - pop[1]: index (ButtonSlot)
/// - pop[2]: offset (Integer)
/// - pop[3]: axis (Mode)
/// - pop[4]: enabled (Flag)
///
/// Return: void/status 1.
///
/// Side effects: MutatesSprite, ChangesSelectState.
///
/// Evidence: docs/dis.txt 00030B80 and related settings procedures push five
/// arguments to ext_0008_000A before entering the slider poll loop.
static SIG_BTN_SLIDER_SET: ExtSig = sig!(8, 10, "btn_slider_set", pop=5,
    params=["group":0=ButtonSlot, "index":1=ButtonSlot, "offset":2=Integer, "axis":3=Mode, "enabled":4=Flag],
    return=Void, effects=[MutatesSprite, ChangesSelectState],
    purpose="Initialize/update compatible slider offset and button state.",
    status=Verified, decompiler=Verified,
    evidence=[Disassembly:"docs/dis.txt 00030B80 and 000337C4 push five args before ext_0008_000A", RuntimeTrace:"setting submenu initializes slider before polling"]);
/// category 8 index 11: btn_slider_begin
///
/// Purpose: Arm a slider drag loop for the specified group/index.
///
/// VM arguments: group, index.
///
/// Return: void/status 1.
///
/// Evidence: settings procedures push group/index only before ext_0008_000B;
/// previous auto pop_count=3 consumed unrelated stack values.
static SIG_BTN_SLIDER_BEGIN: ExtSig = sig!(8, 11, "btn_slider_begin", pop=2,
    params=["group":0=ButtonSlot, "index":1=ButtonSlot],
    return=Void, effects=[ChangesSelectState],
    purpose="Arm native slider polling for a button entry.",
    status=Verified, decompiler=Verified,
    evidence=[Disassembly:"docs/dis.txt 00030CD4/000440CC push group,index before ext_0008_000B", RuntimeTrace:"runtime stack pollution stopped by two-arg pop"]);
/// category 8 index 12: btn_on_check
///
/// Purpose: Return whether a concrete button is currently reacting/under mouse.
///
/// VM arguments: group, index.
///
/// Return: Bool.
///
/// Evidence: Game.sqlite sub_410CE0 pops group/index, calls
/// PalButtonGetReaction, and writes a boolean result to the extcall dst.
static SIG_BTN_ON_CHECK: ExtSig = sig!(8, 12, "btn_on_check", pop=2,
    params=["group":0=ButtonSlot, "index":1=ButtonSlot],
    return=Bool, effects=[ChangesSelectState],
    purpose="Query current native reaction state for one button.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_410CE0 pops group/index and writes PalButtonGetReaction != 0 to dst", PalSqlite:"PalButtonGetReaction 0x1011E1D0", RuntimeTrace:"settings slider loops call ext_0008_000C with group/index"],
    game="0x00410CE0", pal="PalButtonGetReaction" => "0x1011E1D0");
/// category 8 index 13: btn_set_toggle
///
/// Purpose: Set the visual toggle/selection state for a button.  Used to
/// indicate the currently selected option in a button group.
///
/// VM arguments (display order: group, index, toggle):
/// - pop[0]: group (ButtonSlot) — button group index.
/// - pop[1]: index (ButtonSlot) — button index within group.
/// - pop[2]: toggle (Flag) — 1=selected/on, 0=unselected/off.
///
/// Return: void.
///
/// Side effects: MutatesSprite, ChangesSelectState.
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite button dispatch uses PalButtonMode/PalSpriteRectSetPos path
/// - PAL.sqlite: PalButtonMode 0x101191D5 / PalSpriteRectSetPos 0x1011E865
/// - RuntimeTrace: pal-vm ext_btn_set_toggle pops 3
///
/// Engine: Verified — updates compatible toggle state and moves the button
/// sprite rect to the selected/unselected cell row.
///
/// Decompiler: Verified — renders as btn_set_toggle(group, index, toggle).
static SIG_BTN_SET_TOGGLE: ExtSig = sig!(8, 13, "btn_set_toggle", pop=3,
    params=["group":0=ButtonSlot, "index":1=ButtonSlot, "toggle":2=Flag],
    return=Void, effects=[MutatesSprite, ChangesSelectState],
    purpose="Select the button visual/toggle state for a group/index.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_40E830 pops group/index/toggle and queues native btn_set_toggle render command", PalSqlite:"PalSpriteRectSetPos 0x1011E865 is used by the native button render command pipeline", RuntimeTrace:"pal-vm ext_btn_set_toggle pops 3 and updates rect cell"],
    game="0x0040E830", pal="PalSpriteRectSetPos" => "0x1011E865");
/// Category 8 index 14 queries a button's x/y position into caller-selected
/// temporary-memory slots. Both games read slots 0/1 after this call and use
/// the coordinates to place another button.
static SIG_BTN_GET_POS: ExtSig = sig!(8, 14, "btn_get_pos", pop=4,
    params=["group":0=ButtonSlot, "index":1=ButtonSlot, "x_slot":2=Slot, "y_slot":3=Slot],
    return=Status, effects=[WritesVmMemory],
    purpose="Write the button position to two temporary-memory slots.",
    status=Partial, decompiler=Partial,
    evidence=[Disassembly:"Totsulover Script.src 0xE5C90/0xE5D7C and Koikake Script.src 0x324AC read temp slots 0/1 after this call", RuntimeTrace:"Totsulover POP_YES/POP_NO remain at their original coordinates until these slots are written"]);
/// Totsulover's index 29 carries the four-argument button control/state call.
static SIG_BTN_SET_STATE: ExtSig = sig!(8, 29, "btn_set_state", pop=4,
    params=["group":0=ButtonSlot, "index":1=ButtonSlot, "ctrl":2=Mode, "state":3=Mode],
    return=Status, effects=[MutatesSprite, ChangesSelectState],
    purpose="Set button control mode and visual cell/state for menus.",
    status=Partial, decompiler=Partial,
    evidence=[Disassembly:"Totsulover Script.src 0x990AC and 0x94490 call this with group/index/ctrl/state", RuntimeTrace:"pal-vm ext_btn_set_state consumes four arguments"]);
/// category 8 index 15: btn_enable
///
/// Purpose: Enable or disable a button's input and visual state.  index=-1
/// applies to all entries in the group.
///
/// VM arguments (display order: group, index, enabled):
/// - pop[0]: group (ButtonSlot) — button group index.
/// - pop[1]: index (ButtonSlot) — button index within group, or -1 for all.
/// - pop[2]: enabled (Flag) — 1=enable, 0=disable.
///
/// Return: void.
///
/// Side effects: MutatesSprite, ChangesSelectState.
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite button dispatch around PalButtonCtrl
/// - PAL.sqlite: PalButtonCtrl 0x1011AE09 / Game import thunk 0x45034E
/// - RuntimeTrace: pal-vm ext_btn_enable pops 3
///
/// Engine: Verified — updates compatible enable/disabled state and applies the
/// corresponding sprite cell/alpha behavior.
///
/// Decompiler: Verified — renders as btn_enable(group, index, enabled).
static SIG_BTN_ENABLE: ExtSig = sig!(8, 15, "btn_enable", pop=3,
    params=["group":0=ButtonSlot, "index":1=ButtonSlot, "enabled":2=Flag],
    return=Void, effects=[MutatesSprite, ChangesSelectState],
    purpose="Enable or disable button input/visual state for one entry or a group wildcard.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_40E5B0 pops group/index/enabled, calls PalButtonCtrl, and updates disabled flag bits", PalSqlite:"PalButtonCtrl 0x1011AE09 / Game import thunk 0x45034E", RuntimeTrace:"pal-vm ext_btn_enable pops group/index/enabled and updates compatible button state"],
    game="0x0040E5B0", pal="PalButtonCtrl" => "0x1011AE09");
/// category 8 index 16: btn_set_alpha_0x
///
/// Purpose: Set the alpha transparency of a button's sprite.  index=-1
/// applies to all entries in the group.
///
/// VM arguments (display order: group, index, alpha):
/// - pop[0]: group (ButtonSlot) — button group index.
/// - pop[1]: index (ButtonSlot) — button index within group, or -1 for all.
/// - pop[2]: alpha (Alpha) — transparency value (0=transparent, 255=opaque).
///
/// Return: void.
///
/// Side effects: MutatesSprite.
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite button dispatch alpha path
/// - PAL.sqlite: PalSpriteSetColor 0x10119103
/// - RuntimeTrace: pal-vm ext_btn_set_alpha pops 3
///
/// Engine: Verified — writes alpha into compatible button entries and sprite
/// colors.
///
/// Decompiler: Verified — renders as btn_set_alpha_0x(group, index, alpha).
static SIG_BTN_SET_ALPHA: ExtSig = sig!(8, 16, "btn_set_alpha_0x", pop=3,
    params=["group":0=ButtonSlot, "index":1=ButtonSlot, "alpha":2=Alpha],
    return=Void, effects=[MutatesSprite],
    purpose="Set button sprite alpha for one entry or a group wildcard.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_40E4B0 pops group/index/alpha and writes alpha into button sprite color field", PalSqlite:"PalSpriteSetColor 0x10119103 is the PAL color primitive used by sprite alpha paths", RuntimeTrace:"pal-vm ext_btn_set_alpha pops 3 and updates compatible sprite alpha"],
    game="0x0040E4B0", pal="PalSpriteSetColor" => "0x10119103");
static SIG_BTN_GET_ALPHA: ExtSig = sig!(8, 37, "btn_get_alpha_0x", pop=2,
    params=["group":0=ButtonSlot, "index":1=ButtonSlot],
    return=Integer, effects=[],
    purpose="Read the current button sprite alpha.",
    status=Partial, decompiler=Partial,
    evidence=[Disassembly:"Totsulover Script.src 0xA96CC and 0xA9878 query group/index alpha before menu animation"]);
/// The SE channel metadata call is still native-opaque, but its four pushed
/// arguments are explicit in Totsulover's Script.src.
static SIG_CHANNEL_ERROR_SET_SE_INFO: ExtSig = sig!(5, 8, "channel_error_set_se_info", pop=4,
    params=["arg0":0=Unknown, "arg1":1=Unknown, "arg2":2=Unknown, "arg3":3=Unknown],
    return=Status, effects=[UnknownSideEffect],
    purpose="Preserve the SE channel metadata call's stack contract.",
    status=StackDisciplineOnly, decompiler=Partial,
    evidence=[Disassembly:"Totsulover Script.src 0x157AF8 and 0x157B24 each push four arguments"]);
/// category 8 index 21: btn_set_anim
///
/// Purpose: Bind/play an animation resource for an already-registered button
/// sprite.  Native resolves the resource string and attaches the resulting ANI
/// task to the button sprite wrapper.
///
/// VM arguments (display order: group, index, anim_id, state):
/// - pop[0]: group (ButtonSlot) — button group index.
/// - pop[1]: index (ButtonSlot) — button index within group.
/// - pop[2]: anim_resource (ResourceStringFromFileDat) — ANI/File.dat resource id.
/// - pop[3]: play_flag (Flag) — non-zero queues animation update work.
///
/// Return: void.
///
/// Side effects: MutatesSprite, ChangesSelectState.
///
/// Evidence:
/// - Game.sqlite: sub_40DE40 pops group/index/resource/play_flag, resolves the
///   resource through sub_44B1E0, calls sub_447440, and queues animation work
///   when play_flag is non-zero.
///
/// Engine: Blocked — binds supported named ANI resources to portable button sprites.
///
/// Decompiler: Blocked — renders as btn_set_anim(group, index, anim_resource, play_flag).
static SIG_BTN_SET_ANIM: ExtSig = sig!(8, 21, "btn_set_anim", pop=4,
    params=["group":0=ButtonSlot, "index":1=ButtonSlot, "anim_resource":2=ResourceStringFromFileDat, "play_flag":3=Flag],
    return=Void, effects=[MutatesSprite, ChangesSelectState],
    purpose="Bind/play button animation resource for an already registered button.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_40DE40 pops group/index/resource/play_flag, resolves the resource with sub_44B1E0, calls sub_447440, and queues animation work"],
    game="0x0040DE40");
/// category 8 index 22: btn_set_hit
///
/// Purpose: Bind/update the native PAL reaction target for a concrete button
/// entry.  Game.exe pops group then index, looks up the stored PAL button cell,
/// and calls PalButtonSetReaction(native_group, *entry).
///
/// VM arguments:
/// - pop[0]: group (ButtonSlot) — button group index.
/// - pop[1]: index (ButtonSlot) — button index within group.
///
/// Return: void/status 1.
///
/// Side effects: ChangesSelectState.
///
/// Evidence:
/// - Game.sqlite: sub_40DD90 pops group/index and calls PalButtonSetReaction.
/// - PAL.sqlite: PalButtonSetReaction 0x101186DB.
/// - RuntimeTrace: pal-vm ext_btn_set_hit pops 2 and returns 1.
///
/// Engine: Verified — portable runtime clears the compatibility hit override
/// for matching entries so native-style reaction hit testing is used.
///
/// Decompiler: Verified — renders as btn_set_hit(group, index).
static SIG_BTN_SET_HIT: ExtSig = sig!(8, 22, "btn_set_hit", pop=2,
    params=["group":0=ButtonSlot, "index":1=ButtonSlot],
    return=Void, effects=[ChangesSelectState],
    purpose="Bind/update the PAL reaction target for an existing button entry.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_40DD90 pops group/index and calls PalButtonSetReaction", PalSqlite:"PalButtonSetReaction 0x101186DB", RuntimeTrace:"pal-vm ext_btn_set_hit pops group/index and returns 1"],
    game="0x0040DD90", pal="PalButtonSetReaction" => "0x101186DB");
/// category 8 index 17: btn_get_push
///
/// Purpose: Consume the latched "pushed button" index for a group.  Returns
/// the index of the last activated button, then clears the latch.  The script
/// polls this to determine which choice the player made.
///
/// VM arguments:
/// - pop[0]: group (ButtonSlot) — button group to query.
///
/// Return: Integer — pushed button index, or 0 when no push/reaction is pending
/// in the portable runtime path.
///
/// Side effects: ChangesSelectState.  Native PalButtonGetReaction can update
/// the active script/message reaction path; the portable runtime consumes a
/// latched click queue or performs mouse hit testing.
///
/// Evidence:
/// - Game.sqlite: sub_40CDE0 polls PalButtonGetReaction and mutates script
///   reaction state when a button is active.
/// - PAL.sqlite: PalButtonGetReaction 0x1011E1D0 / Game import thunk 0x45029E.
/// - RuntimeTrace: pal-vm ext_btn_get_push pops group, consumes compatible
///   push queue state, and returns 0 for no push.
///
/// Engine: Verified — models the native reaction poll with mouse hit testing
/// plus a latched push queue used by the portable input layer.
///
/// Decompiler: Verified — renders as btn_get_push(group).
static SIG_BTN_GET_PUSH: ExtSig = sig!(8, 17, "btn_get_push", pop=1,
    params=["group":0=ButtonSlot], return=Integer, effects=[ChangesSelectState],
    purpose="Poll/consume the native button reaction for a group; compatible no-push sentinel is 0.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_40CDE0 calls PalButtonGetReaction and mutates script reaction state", PalSqlite:"PalButtonGetReaction 0x1011E1D0 / Game import thunk 0x45029E", RuntimeTrace:"pal-vm ext_btn_get_push pops group and returns clicked index or 0"],
    game="0x0040CDE0", pal="PalButtonGetReaction" => "0x1011E1D0");
/// category 8 index 23: btn_get_onmouse
///
/// Purpose: Return the current hover/onmouse button index for a group.
///
/// VM arguments:
/// - pop[0]: group (ButtonSlot) — button group to query.
///
/// Return: Integer — hovered button index, or 0 when none.
///
/// Side effects: ChangesSelectState — native returns a cached group field;
/// portable runtime refreshes it from live hit testing.
///
/// Evidence:
/// - Game.sqlite: sub_40E3B0 pops one group and returns
///   `*(ctx + 4512 * group + 12996)`.
/// - RuntimeTrace: pal-vm ext_btn_get_onmouse pops group and returns the live
///   hit-test/onmouse cache.
///
/// Engine: Verified — computes and caches group hover index.
///
/// Decompiler: Verified — renders as btn_get_onmouse(group).
static SIG_BTN_GET_ONMOUSE: ExtSig = sig!(8, 23, "btn_get_onmouse", pop=1,
    params=["group":0=ButtonSlot=>"button group"],
    return=Integer, effects=[ChangesSelectState],
    purpose="Return current hover/onmouse button index for a group.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_40E3B0 pops group and returns ctx + 4512*group + 12996", RuntimeTrace:"pal-vm ext_btn_get_onmouse pops group and hit-tests current mouse"],
    game="0x0040E3B0");
/// category 8 index 19: btn_lock
///
/// Purpose: Lock input for a button group and optionally start a native lock
/// timer/duration.  This is group-scoped in Game.exe, not per-entry.
///
/// VM arguments:
/// - pop[0]: group (ButtonSlot) — button group index.
/// - pop[1]: duration_ms (DurationMs) — non-zero lock duration/timer value.
///
/// Return: void.
///
/// Side effects: ChangesSelectState.
///
/// Evidence:
/// - Game.sqlite: sub_40E0C0 pops group/duration, sets group lock fields, and
///   records PaltimeGetTime when duration is non-zero.
/// - RuntimeTrace: pal-vm ext_btn_lock pops group/duration; duration 0 is a
///   redraw gate and does not permanently disable compatible entries.
///
/// Engine: Verified — non-zero durations lock compatible entries; duration 0
/// only records the native redraw gate so SYSTEM/SOUND tab groups remain
/// clickable after their page rebuild.
///
/// Decompiler: Verified — renders as btn_lock(group, duration_ms).
static SIG_BTN_LOCK: ExtSig = sig!(8, 19, "btn_lock", pop=2,
    params=["group":0=ButtonSlot, "duration_ms":1=DurationMs],
    return=Void, effects=[ChangesSelectState],
    purpose="Lock input for a button group; second argument is the native duration/timer value.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_40E0C0 pops group/duration, sets group lock timer fields, and stores start time only when duration is non-zero", RuntimeTrace:"pal-vm ext_btn_lock pops group/duration; duration 0 no longer locks matching group entries"],
    game="0x0040E0C0");
/// category 8 index 20: btn_unlock
///
/// Purpose: Unlock all button inputs for a group, re-enabling activation.
///
/// VM arguments:
/// - pop[0]: group (ButtonSlot) — button group to unlock.
///
/// Return: void.
///
/// Side effects: ChangesSelectState.
///
/// Evidence:
/// - Game.sqlite: sub_40E040 pops group and clears the native group lock timer
///   fields.
/// - RuntimeTrace: pal-vm ext_btn_unlock pops group and unlocks the compatible
///   group.
///
/// Engine: Verified — unlocks all compatible entries in the group.
///
/// Decompiler: Verified — renders as btn_unlock(group).
static SIG_BTN_UNLOCK: ExtSig = sig!(8, 20, "btn_unlock", pop=1,
    params=["group":0=ButtonSlot], return=Void, effects=[ChangesSelectState],
    purpose="Unlock button input for a group.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_40E040 pops group and clears native button lock fields", RuntimeTrace:"pal-vm ext_btn_unlock pops group and unlocks matching entries"],
    game="0x0040E040");

/// category 9 index 0: skip_set
///
/// Purpose: Set the ADV skip latch at text context offset +804248 and update
/// the native skip button row when a text window is bound.
static SIG_SKIP_SET: ExtSig = sig!(9, 0, "skip_set", pop=1,
    params=["enabled":0=Flag=>"nonzero enables native ADV skip latch"],
    return=Integer, effects=[ChangesTextState, ChangesSelectState],
    purpose="Set the ADV text skip latch.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_438C40 pops enabled, writes byte ctx+804248, updates skip button row through PalButtonSetPos"],
    game="0x00438C40");
/// category 9 index 1: skip_is
///
/// Purpose: Return the current ADV skip latch stored at text context +804248.
static SIG_SKIP_IS: ExtSig = sig!(9, 1, "skip_is", pop=0,
    params=[], return=Integer, effects=[WritesVmMemory],
    purpose="Return the ADV text skip latch.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_438C00 writes unsigned byte ctx+804248 to the extcall destination"],
    game="0x00438C00");
/// category 9 index 2: auto_set
///
/// Purpose: Set the ADV auto latch at text context offset +804252 and update
/// the native auto button row when a text window is bound.
static SIG_AUTO_SET: ExtSig = sig!(9, 2, "auto_set", pop=1,
    params=["enabled":0=Flag=>"nonzero enables auto-advance after text reveal"],
    return=Integer, effects=[ChangesTextState, ChangesSelectState],
    purpose="Set the ADV text auto latch.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_438A90 pops enabled, writes ctx+804252, updates auto button row through PalButtonSetPos"],
    game="0x00438A90");
/// category 9 index 3: auto_is
///
/// Purpose: Return the current ADV auto latch stored at text context +804252.
static SIG_AUTO_IS: ExtSig = sig!(9, 3, "auto_is", pop=0,
    params=[], return=Integer, effects=[WritesVmMemory],
    purpose="Return the ADV text auto latch.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_438A50 writes ctx+804252 to the extcall destination"],
    game="0x00438A50");

/// category 9 index 4: auto_set_speed
///
/// Purpose: Store the PAL text auto/typewriter speed percentage in the global
/// task data block.  Native clamps values above 100 before writing offset +28.
///
/// VM arguments:
/// - pop[0]: speed_percent (Integer) — text timing percent clamped to 0..100.
///
/// Return: status integer.
///
/// Side effects: ChangesTextState.
///
/// Evidence:
/// - Game.sqlite: sub_4389F0 pops one value, clamps it to 100, and writes
///   PalTaskGetTaskData(0)+28.
/// - RuntimeTrace/Disassembly: callsites push config values before 9:4.
///
/// Engine: Verified — pops one value and records the clamped speed.
///
/// Decompiler: Verified — renders as auto_set_speed(speed_percent).
static SIG_AUTO_SET_SPEED: ExtSig = sig!(9, 4, "auto_set_speed", pop=1,
    params=["speed_percent":0=Integer=>"text timing percent clamped to 0..100"],
    return=Integer, effects=[ChangesTextState],
    purpose="Set PAL text auto/typewriter speed percent.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_4389F0 pops speed, clamps >100, writes PalTaskGetTaskData(0)+28", Disassembly:"docs/dis.txt 00039628 pushes config value before ext_0009_0004"],
    game="0x004389F0");
/// category 9 index 6: window_change_mode
///
/// Purpose: Request a PAL window mode change and cache the selected mode in
/// task data offset +8.
///
/// VM arguments:
/// - pop[0]: mode (Mode) — PAL window mode posted through PalWindowChangeMode.
///
/// Return: status integer.
///
/// Side effects: MutatesWindow.
///
/// Evidence:
/// - Game.sqlite: sub_438950 pops one mode, calls PalWindowChangeMode(mode),
///   and stores mode at PalTaskGetTaskData(0)+8.
/// - PAL.sqlite: PalWindowChangeMode posts the portable-equivalent window
///   request message.
static SIG_WINDOW_CHANGE_MODE: ExtSig = sig!(9, 6, "window_change_mode", pop=1,
    params=["mode":0=Mode=>"PAL window mode"],
    return=Integer, effects=[MutatesWindow],
    purpose="Request a PAL window mode change.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_438950 calls PalWindowChangeMode(mode)", PalSqlite:"reverse/PAL.sqlite PalWindowChangeMode 0x101194A5 -> PalWindowChangeMode_0 posts message 0x8065"],
    game="0x00438950", pal="PalWindowChangeMode" => "0x101194A5");
/// category 9 index 7: window_set_mode_cache
///
/// Purpose: Store the current logical window mode in task data offset +12
/// without posting a PAL window-change message.
///
/// VM arguments:
/// - pop[0]: mode (Mode) — cached window/layout mode.
///
/// Return: status integer.
///
/// Side effects: MutatesWindow.
///
/// Evidence: Game.sqlite sub_4388F0 pops one value and stores it at
/// PalTaskGetTaskData(0)+12.
static SIG_WINDOW_SET_MODE_CACHE: ExtSig = sig!(9, 7, "window_set_mode_cache", pop=1,
    params=["mode":0=Mode=>"cached window/layout mode"],
    return=Integer, effects=[MutatesWindow],
    purpose="Cache logical window mode without posting a window-change message.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_4388F0 pops mode and writes PalTaskGetTaskData(0)+12"],
    game="0x004388F0");
/// category 9 index 8: effect_enable
///
/// Purpose: Enable or disable PAL visual effects and store the same flag in task
/// data offset +16.
///
/// VM arguments:
/// - pop[0]: enabled (Flag) — non-zero enables PAL effects.
///
/// Return: status integer.
///
/// Side effects: ChangesRunState, MutatesWindow.
///
/// Evidence:
/// - Game.sqlite: sub_438890 pops enabled, calls PalEffectEnable(enabled), and
///   writes task data +16.
/// - PAL.sqlite: PalEffectEnable_0 writes Block[277].
static SIG_EFFECT_ENABLE: ExtSig = sig!(9, 8, "effect_enable", pop=1,
    params=["enabled":0=Flag=>"non-zero enables PAL effects"],
    return=Integer, effects=[ChangesRunState, MutatesWindow],
    purpose="Enable or disable PAL visual effects.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_438890 pops flag, calls PalEffectEnable, writes PalTaskGetTaskData(0)+16", PalSqlite:"reverse/PAL.sqlite PalEffectEnable 0x1011C119 -> PalEffectEnable_0 writes Block[277]"],
    game="0x00438890", pal="PalEffectEnable" => "0x1011C119");
/// category 9 index 9: effect_enable_is
///
/// Purpose: Query PAL effect-enable state for script conditionals.
///
/// VM arguments: none.
///
/// Return: bool integer from PalEffectEnableIs.
///
/// Side effects: none.
///
/// Evidence:
/// - Game.sqlite: sub_438810 writes PalEffectEnableIs() into the extcall return
///   destination and pops no arguments.
/// - PAL.sqlite: PalEffectEnableIs_0 returns Block[277].
static SIG_EFFECT_ENABLE_IS: ExtSig = sig!(9, 9, "effect_enable_is", pop=0,
    params=[],
    return=Bool, effects=[],
    purpose="Query whether PAL visual effects are enabled.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_438810 returns PalEffectEnableIs with no pops", PalSqlite:"reverse/PAL.sqlite PalEffectEnableIs 0x101195A9 -> PalEffectEnableIs_0 returns Block[277]"],
    game="0x00438810", pal="PalEffectEnableIs" => "0x101195A9");
/// category 9 index 10: window_get_mode_cache
///
/// Purpose: Return the cached logical window/layout mode stored by
/// window_set_mode_cache.
///
/// VM arguments: none.
///
/// Return: integer mode.
///
/// Side effects: WritesVmMemory.
///
/// Evidence: Game.sqlite sub_438830 returns PalTaskGetTaskData(0)+12.
static SIG_WINDOW_GET_MODE_CACHE: ExtSig = sig!(9, 10, "window_get_mode_cache", pop=0,
    params=[],
    return=Integer, effects=[WritesVmMemory],
    purpose="Return cached logical window/layout mode.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_438830 writes PalTaskGetTaskData(0)+12 to dst_slot"],
    game="0x00438830");
/// category 9 index 18: input_key_cancel
///
/// Purpose: Cancel/clear selected PAL input key masks.
///
/// VM arguments:
/// - pop[0]: key_mask (Integer) — keyboard mask.
/// - pop[1]: mouse_mask (Integer) — mouse mask.
///
/// Return: status integer.
///
/// Side effects: ChangesSelectState.
///
/// Evidence:
/// - Game.sqlite: sub_437E70 pops two values and calls
///   PalInputKeyCancel(key_mask, mouse_mask).
/// - PAL.sqlite: PalInputKeyCancel_0 removes those masks from PAL input state.
static SIG_INPUT_KEY_CANCEL: ExtSig = sig!(9, 18, "input_key_cancel", pop=2,
    params=["key_mask":0=Integer=>"keyboard mask", "mouse_mask":1=Integer=>"mouse mask"],
    return=Integer, effects=[ChangesSelectState],
    purpose="Cancel selected PAL input key/mouse masks.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_437E70 pops key_mask,mouse_mask and calls PalInputKeyCancel", PalSqlite:"reverse/PAL.sqlite PalInputKeyCancel 0x1011F56C -> PalInputKeyCancel_0 masks PAL input state"],
    game="0x00437E70", pal="PalInputKeyCancel" => "0x1011F56C");
/// category 9 index 17: set_language
///
/// Purpose: Set PAL font type/language and rebuild cached text sprites when
/// native font resources are available.
///
/// VM arguments:
/// - pop[0]: font_type (Mode) — PAL font type passed to PalFontSetType.
///
/// Return: status integer.
///
/// Side effects: MutatesWindow.
///
/// Evidence: Game.sqlite sub_438090 pops one value, calls PalFontSetType, then
/// conditionally repaints cached text resources; PAL.sqlite PalFontSetType_0
/// stores Block[232].
static SIG_SET_LANGUAGE: ExtSig = sig!(9, 17, "set_language", pop=1,
    params=["font_type":0=Mode=>"PAL font type/language id"],
    return=Integer, effects=[MutatesWindow],
    purpose="Set PAL font type/language.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_438090 pops font_type and calls PalFontSetType", PalSqlite:"reverse/PAL.sqlite PalFontSetType 0x1011C4A2 -> PalFontSetType_0 stores Block[232]"],
    game="0x00438090", pal="PalFontSetType" => "0x1011C4A2");
/// category 9 index 19: set_font_color
///
/// Purpose: Set PAL shared UI font primary/effect colors.
///
/// VM arguments:
/// - pop[0]: text_color (Color) — primary color.
/// - pop[1]: effect_color (Color) — effect/outline color.
///
/// Return: status integer.
///
/// Side effects: MutatesWindow.
///
/// Evidence: Game.sqlite sub_437F30 pops two values and calls PalFontSetColor;
/// PAL.sqlite PalFontSetColor_0 stores Block[113]/Block[114].
static SIG_SET_FONT_COLOR: ExtSig = sig!(9, 19, "set_font_color", pop=2,
    params=["text_color":0=Color=>"primary font color", "effect_color":1=Color=>"effect/outline color"],
    return=Integer, effects=[MutatesWindow],
    purpose="Set PAL shared UI font colors.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_437F30 pops text_color,effect_color and calls PalFontSetColor", PalSqlite:"reverse/PAL.sqlite PalFontSetColor 0x101214F7 -> PalFontSetColor_0 stores Block[113]/Block[114]"],
    game="0x00437F30", pal="PalFontSetColor" => "0x101214F7");
/// category 9 index 20: load_font_ex
///
/// Purpose: Load an extended font TGA resource by File.dat string id.
///
/// VM arguments:
/// - pop[0]: font_resource (ResourceStringFromFileDat) — resource base name.
///
/// Return: status integer.
///
/// Side effects: MutatesWindow.
///
/// Evidence: Game.sqlite sub_4382D0 pops one File.dat string id, appends
/// ".tga", unloads the old extended font, then calls PalExFontLoad.
static SIG_LOAD_FONT_EX: ExtSig = sig!(9, 20, "load_font_ex", pop=1,
    params=["font_resource":0=ResourceStringFromFileDat=>"extended font resource base name"],
    return=Integer, effects=[MutatesWindow],
    purpose="Load an extended font TGA resource.",
    status=Blocked, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_4382D0 pops font string, appends .tga, calls PalExFontUnload/PalExFontLoad"],
    game="0x004382D0");
/// category 9 index 27: set_font_size
///
/// Purpose: Set PAL shared UI font size and return the previous size.
///
/// VM arguments:
/// - pop[0]: font_size (Integer) — PAL font size.
///
/// Return: previous font size.
///
/// Evidence: Game.sqlite sub_437B00 pops size, reads PalFontGetFontSize, calls
/// PalFontSetFontSize(size), then writes the old size to dst_slot.
static SIG_SET_FONT_SIZE: ExtSig = sig!(9, 27, "set_font_size", pop=1,
    params=["font_size":0=Integer=>"PAL font size"],
    return=Integer, effects=[MutatesWindow],
    purpose="Set PAL shared UI font size and return previous size.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_437B00 pops size, calls PalFontGetFontSize and PalFontSetFontSize", PalSqlite:"reverse/PAL.sqlite PalFontSetFontSize 0x1011E734 / PalFontGetFontSize 0x1011C59C"],
    game="0x00437B00", pal="PalFontSetFontSize" => "0x1011E734");
/// category 9 index 28: get_font_size
///
/// Purpose: Return PAL shared UI font size.
static SIG_GET_FONT_SIZE: ExtSig = sig!(9, 28, "get_font_size", pop=0,
    params=[],
    return=Integer, effects=[WritesVmMemory],
    purpose="Return PAL shared UI font size.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_437AC0 writes PalFontGetFontSize to dst_slot", PalSqlite:"reverse/PAL.sqlite PalFontGetFontSize 0x1011C59C"],
    game="0x00437AC0", pal="PalFontGetFontSize" => "0x1011C59C");
/// category 9 index 29: get_font_type
///
/// Purpose: Return PAL shared UI font type/language.
static SIG_GET_FONT_TYPE: ExtSig = sig!(9, 29, "get_font_type", pop=0,
    params=[],
    return=Integer, effects=[WritesVmMemory],
    purpose="Return PAL shared UI font type/language.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_438050 writes PalFontGetType to dst_slot", PalSqlite:"reverse/PAL.sqlite PalFontGetType 0x1011CA2E"],
    game="0x00438050", pal="PalFontGetType" => "0x1011CA2E");
/// category 9 index 30: set_font_effect
///
/// Purpose: Set PAL shared UI font effect mode.
static SIG_SET_FONT_EFFECT: ExtSig = sig!(9, 30, "set_font_effect", pop=1,
    params=["effect":0=Mode=>"PAL font effect mode"],
    return=Integer, effects=[MutatesWindow],
    purpose="Set PAL shared UI font effect mode.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_437A60 pops effect and calls PalFontSetEffect"],
    game="0x00437A60");
/// category 9 index 31: get_font_effect
///
/// Purpose: Return PAL shared UI font effect mode.
static SIG_GET_FONT_EFFECT: ExtSig = sig!(9, 31, "get_font_effect", pop=0,
    params=[],
    return=Integer, effects=[WritesVmMemory],
    purpose="Return PAL shared UI font effect mode.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_437A20 writes PalFontGetEffect to dst_slot"],
    game="0x00437A20");
/// category 9 index 21: memory_stack_push
///
/// Purpose: Snapshot native VM memory bank 715956..732339 onto a 32-entry
/// memory stack.  Scripts use it around menu/system overlay procedures; this
/// does not include Mem.dat, so page-request values such as memdat[158] survive
/// a pop.
///
/// VM arguments: none.
///
/// Return: status integer.
///
/// Side effects: WritesVmMemory.
///
/// Evidence: Game.sqlite sub_437E10 copies 0x4000 bytes from ctx+715956 to the
/// next stack slot at ctx+738524 and increments the memory-stack depth.
static SIG_MEMORY_STACK_PUSH: ExtSig = sig!(9, 21, "memory_stack_push", pop=0,
    params=[],
    return=Integer, effects=[WritesVmMemory],
    purpose="Snapshot the native script memory bank onto the memory stack.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_437E10 copies 0x4000 bytes from ctx+715956 to ctx+738524+(depth*0x4000)"],
    game="0x00437E10");
/// category 9 index 22: memory_stack_pop
///
/// Purpose: Restore the most recent native VM memory-bank snapshot.  Mem.dat is
/// intentionally not restored by the native handler.
///
/// VM arguments: none.
///
/// Return: status integer.
///
/// Side effects: WritesVmMemory.
///
/// Evidence: Game.sqlite sub_437D90 restores 0x4000 bytes from the current
/// stack slot to ctx+715956 and clears that stack slot.
static SIG_MEMORY_STACK_POP: ExtSig = sig!(9, 22, "memory_stack_pop", pop=0,
    params=[],
    return=Integer, effects=[WritesVmMemory],
    purpose="Restore the last native script memory-bank snapshot.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_437D90 restores 0x4000 bytes from ctx+738524+(depth*0x4000) to ctx+715956"],
    game="0x00437D90");
/// category 9 index 23: list_stack_push_point
///
/// Purpose: Push a script point/address onto the native list stack.  The stored
/// value is the current process base plus the resolved point entry.
///
/// VM arguments:
/// - pop[0]: point_id (PointId) — point operand whose address is pushed.
///
/// Return: status integer.
///
/// Side effects: ChangesRunState.
///
/// Evidence: Game.sqlite sub_437CD0 pops one point id, resolves it through the
/// point table, adds process base +201016, and PalListPushes an 8-byte entry.
static SIG_LIST_STACK_PUSH_POINT: ExtSig = sig!(9, 23, "list_stack_push_point", pop=1,
    params=["point_id":0=PointId=>"script point id whose resolved address is pushed"],
    return=Integer, effects=[ChangesRunState],
    purpose="Push a resolved script point onto the native list stack.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_437CD0 pops point id, resolves point table, and PalListPushes the address"],
    game="0x00437CD0");
/// category 9 index 24: list_stack_pop_count
///
/// Purpose: Pop one or more entries from the native list stack; zero means one
/// entry, otherwise the popped count is used after subtracting one for a control
/// flag.
///
/// VM arguments:
/// - pop[0]: count_mode (Integer) — native count/control argument.
///
/// Return: status integer.
///
/// Side effects: ChangesRunState.
///
/// Evidence: Game.sqlite sub_437C00 pops one count/control value and PalListPops
/// up to the requested number of entries.
static SIG_LIST_STACK_POP_COUNT: ExtSig = sig!(9, 24, "list_stack_pop_count", pop=1,
    params=["count_mode":0=Integer=>"native list-pop count/control value"],
    return=Integer, effects=[ChangesRunState],
    purpose="Pop native list-stack continuation entries.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_437C00 pops count/control and PalListPops entries"],
    game="0x00437C00");
/// category 9 index 53: list_stack_get_count
///
/// Purpose: Return the number of entries currently stored in the native list
/// stack with tag 2.  Cleanup scripts compare this against a saved depth and
/// call list_stack_pop_count(delta), so returning 0 leaves old menu/text/button
/// overlays alive.
///
/// VM arguments: none.
///
/// Return: integer list-stack depth.
///
/// Side effects: WritesVmMemory.
///
/// Evidence: Game.sqlite sub_437BC0 calls PalListGetDataCount(ctx+804228, 2),
/// writes the result to the extcall destination, logs "[%d]lsize", and returns
/// 1.  IDB dump maps category 9 index 53 to 0x00437BC0.
static SIG_LIST_STACK_GET_COUNT: ExtSig = sig!(9, 53, "list_stack_get_count", pop=0,
    params=[],
    return=Integer, effects=[WritesVmMemory],
    purpose="Return native list-stack depth for tag 2.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_437BC0 returns PalListGetDataCount(ctx+804228, 2)", Disassembly:"target/diagnostics/game_idb_extcalls.json maps category 9 index 53 to 0x00437BC0"],
    game="0x00437BC0");

/// category 15 index 4: system_window_overlay_set
///
/// Purpose: Configure a native system/debug/window overlay record used by early
/// boot and menu procedures. Reachable callsites pass a text/resource id plus
/// two mode/sentinel arguments.
///
/// VM arguments:
/// - pop[0]: text_id (TextId) — text/resource id or sentinel.
/// - pop[1]: value (Integer) — per-callsite value.
/// - pop[2]: mode (Mode) — mode or sentinel 0x0FFFFFFF.
///
/// Return: status integer.
///
/// Side effects: MutatesWindow.
///
/// Evidence: Koikake's three-argument form is recorded in docs/dis.txt;
/// Totsulover pushes eight arguments, including live layout values. The
/// baseline signature below describes Koikake. The runtime selects the
/// eight-argument form when that extension set is detected.
static SIG_SYSTEM_WINDOW_OVERLAY_SET: ExtSig = sig!(15, 4, "system_window_overlay_set", pop=3,
    params=[
        "text_id":0=TextId=>"text/resource id or sentinel",
        "value":1=Integer=>"per-callsite value",
        "mode":2=Mode=>"mode or sentinel 0x0FFFFFFF"
    ],
    return=Integer, effects=[MutatesWindow],
    purpose="Configure native system/window overlay record.",
    status=Blocked, decompiler=Partial,
    evidence=[Disassembly:"Koikake docs/dis.txt reachable ext_000F_0004 callsites push three values; Totsulover Script.src 0x102088 pushes eight values", RuntimeTrace:"Totsulover startup leaves five stack values per call if only three are popped"]);
/// category 15 index 5: debug_window_set
///
/// Purpose: Set PAL's debug-window state and return the previous state.
///
/// VM arguments:
/// - pop[0]: state (Mode) — new PalDebugWindow state.
///
/// Return: previous debug-window state.
///
/// Side effects: MutatesWindow.
///
/// Evidence: Game.sqlite sub_410F80 pops one state argument, calls
/// PalDebugWindowGetState, PalDebugWindowSetState(new_state), writes the old
/// state to the extcall destination slot, and returns 1.
static SIG_DEBUG_WINDOW_SET: ExtSig = sig!(15, 5, "debug_window_set", pop=1,
    params=["state":0=Mode=>"new PalDebugWindow state"],
    return=Integer, effects=[MutatesWindow],
    purpose="Set PAL debug-window state and return previous state.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_410F80 pops state, calls PalDebugWindowGetState/SetState, and returns old state"],
    game="0x00410F80");

// Category 12 - system buttons.

/// category 12 index 0: system_btn_set
///
/// Purpose: Configure a native system/menu button (e.g. Save, Load, Auto)
/// in Game.exe's window button table with a sprite resource and a state code.
///
/// VM arguments (display order: index, image, state):
/// - pop[0]: index (ButtonSlot) — system button slot index.
/// - pop[1]: image (ResourceStringFromFileDat) — sprite resource name.
/// - pop[2]: state (Mode) — button state/visibility code.
///
/// Return: void.
///
/// Side effects: ChangesSelectState, MutatesWindow.
///
/// Evidence:
/// - Game.sqlite: sub_439270 pops index/image/state, stores the sprite point
///   and enabled/default state in the system button table, then returns 1.
/// - RuntimeTrace: pal-vm dispatch_system_button_stub index 0 pops 3.
///
/// Engine: Verified — pops and records the system button configuration in the
/// compatible system-button table placeholder.
///
/// Decompiler: Verified — renders as system_btn_set(index, image, state).
static SIG_SYSTEM_BTN_SET: ExtSig = sig!(12, 0, "system_btn_set", pop=3,
    params=["index":0=ButtonSlot, "image":1=ResourceStringFromFileDat, "state":2=Mode],
    return=Void, effects=[ChangesSelectState, MutatesWindow],
    purpose="Configure native system/menu button state in Game.exe's window button table.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_439270 pops index/image/state and updates system button table", RuntimeTrace:"pal-vm dispatch_system_button_stub index 0 pops index/image/state"],
    game="0x00439270");
/// category 12 index 1: system_btn_release
///
/// Purpose: Release/remove a system button entry from the window button table.
///
/// VM arguments:
/// - pop[0]: index (ButtonSlot) — system button slot to release.
///
/// Return: void.
///
/// Side effects: ChangesSelectState.
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite system button handler; pal-vm dispatch_system_button_stub index 1 pops 1
///
/// Engine: Blocked — removes system button entry.
///
/// Decompiler: Blocked — renders as system_btn_release(index).
static SIG_SYSTEM_BTN_RELEASE: ExtSig = sig!(12, 1, "system_btn_release", pop=1,
    params=["index":0=ButtonSlot],
    return=Void, effects=[ChangesSelectState],
    purpose="Release a system/menu button entry.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite system button handler; pal-vm dispatch_system_button_stub index 1 pops 1"]);
/// category 12 index 2: system_btn_enable
///
/// Purpose: Enable or disable a system button's input and visual state.
///
/// VM arguments:
/// - pop[0]: index (ButtonSlot) — system button slot.
/// - pop[1]: enabled (Flag) — 1=enable, 0=disable.
///
/// Return: void.
///
/// Side effects: ChangesSelectState.
///
/// Evidence:
/// - Game.sqlite: sub_439100 pops index/enabled, writes enabled to one slot or
///   all 32 slots when index==0xFFFF, and returns 1.
/// - RuntimeTrace: pal-vm dispatch_system_button_stub index 2 pops 2.
///
/// Engine: Verified — pops the native two-argument shape and preserves status
/// return 1; exact platform window-button drawing remains intentionally
/// portable.
///
/// Decompiler: Verified — renders as system_btn_enable(index, enabled).
static SIG_SYSTEM_BTN_ENABLE: ExtSig = sig!(12, 2, "system_btn_enable", pop=2,
    params=["index":0=ButtonSlot, "enabled":1=Flag],
    return=Void, effects=[ChangesSelectState],
    purpose="Enable/disable a system/menu button entry.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_439100 pops index/enabled and supports index 0xFFFF wildcard", RuntimeTrace:"pal-vm dispatch_system_button_stub index 2 pops index/enabled"],
    game="0x00439100");

// Category 11 - MSprite/movie wrappers. The script table uses category 0x000B;
// PAL itself stores the decoder handle inside sprite object offset +0x14.

/// category 11 index 0: movie_play
///
/// Purpose: Open and begin playing a full-screen or layer movie/MSprite
/// resource.  PAL stores the decoder handle at sprite object offset +0x14.
///
/// VM arguments:
/// - pop[0]: layer (SpriteSlot) — target sprite layer/slot for the movie.
/// - pop[1]: name (ResourceStringFromFileDat) — File.dat resource name.
///
/// Return: void.
///
/// Side effects: CreatesMovie, CreatesSprite.
///
/// Evidence:
/// - Disassembly: docs/dis.txt 000B:0000
/// - PAL.sqlite: PalMSpriteLoad 0x1011B39A
/// - Writeup: docs/writeup.md MSprite
///
/// Engine: Blocked — calls PalMSpriteLoad.
///
/// Decompiler: Blocked — renders as movie_play(layer, name).
static SIG_MOVIE_PLAY: ExtSig = sig!(11, 0, "movie_play", pop=2,
    params=["layer":0=SpriteSlot, "name":1=ResourceStringFromFileDat],
    return=Void, effects=[CreatesMovie, CreatesSprite], purpose="Open/play a full-screen or layer movie resource.",
    status=Blocked, decompiler=Blocked, evidence=[Disassembly:"docs/dis.txt 000B:0000", PalSqlite:"PalMSpriteLoad 0x1011B39A", Writeup:"docs/writeup.md MSprite"],
    pal="PalMSpriteLoad" => "0x1011B39A");
/// category 11 index 1: msp_set_loop_sp_ep
///
/// Purpose: Load a movie/MSprite into a sprite slot and configure loop
/// mode, loop start point, loop end point, and position.
///
/// VM arguments (display order: slot, name, loop_mode, loop_start, loop_end, x, y, z):
/// - pop[0]: slot (SpriteSlot) — target sprite slot.
/// - pop[1]: name (ResourceStringFromFileDat) — File.dat resource name.
/// - pop[2]: loop_mode (Mode) — 0=no loop, 1=loop, etc.
/// - pop[3]: loop_start (Integer) — loop start frame.
/// - pop[4]: loop_end (Integer) — loop end frame.
/// - pop[5]: x (CoordinateX) — display position x.
/// - pop[6]: y (CoordinateY) — display position y.
/// - pop[7]: z (CoordinateZ) — depth/priority.
///
/// Return: void.
///
/// Side effects: CreatesMovie, CreatesSprite, MutatesSprite.
///
/// Evidence:
/// - Disassembly: docs/dis.txt 000B:0001
/// - PAL.sqlite: PalMSpriteSetLoopPoint 0x1011B61A / PalMSpriteSetLoop 0x1011E923
/// - Writeup: docs/writeup.md MSprite
///
/// Engine: Blocked — calls PalMSpriteLoad + PalMSpriteSetLoopPoint.
///
/// Decompiler: Blocked — renders 8 args in display order.
static SIG_MSP_SET_LOOP_SP_EP: ExtSig = sig!(11, 1, "msp_set_loop_sp_ep", pop=8,
    params=["slot":0=SpriteSlot, "name":1=ResourceStringFromFileDat, "loop_mode":2=Mode, "loop_start":3=Integer, "loop_end":4=Integer, "x":5=CoordinateX, "y":6=CoordinateY, "z":7=CoordinateZ],
    return=Void, effects=[CreatesMovie, CreatesSprite, MutatesSprite], purpose="Load an MSprite/movie into a sprite slot and configure loop/start/end points.",
    status=Blocked, decompiler=Blocked, evidence=[Disassembly:"docs/dis.txt 000B:0001", PalSqlite:"PalMSpriteSetLoopPoint 0x1011B61A / PalMSpriteSetLoop 0x1011E923", Writeup:"docs/writeup.md MSprite"],
    pal="PalMSpriteSetLoopPoint" => "0x1011B61A");
/// category 11 index 2: msp_cls
///
/// Purpose: Stop and release an MSprite/movie slot.
///
/// VM arguments:
/// - pop[0]: slot (SpriteSlot) — movie slot to release.
///
/// Return: void.
///
/// Side effects: DeletesSprite.
///
/// Evidence:
/// - Disassembly: docs/dis.txt 000B:0002
/// - PAL.sqlite: PalMSpriteStop 0x10119C7F
/// - Writeup: docs/writeup.md MSprite
///
/// Engine: Blocked — calls PalMSpriteStop.
///
/// Decompiler: Blocked — renders as msp_cls(slot).
static SIG_MSP_CLS: ExtSig = sig!(11, 2, "msp_cls", pop=1,
    params=["slot":0=SpriteSlot], return=Void, effects=[DeletesSprite], purpose="Release an MSprite/movie slot.",
    status=Blocked, decompiler=Blocked, evidence=[Disassembly:"docs/dis.txt 000B:0002", PalSqlite:"PalMSpriteStop 0x10119C7F", Writeup:"docs/writeup.md MSprite"],
    pal="PalMSpriteStop" => "0x10119C7F");
/// category 11 index 3: msp_wait
///
/// Purpose: Block the script until the MSprite/movie slot finishes (native
/// "finished" bit set in PalMSpriteGetState).
///
/// VM arguments:
/// - pop[0]: slot (SpriteSlot) — movie slot to wait on.
///
/// Return: void.
///
/// Side effects: BlocksScript.
///
/// Evidence:
/// - GameSqlite: sub_42E240 stores the pending MSprite slot pointer, disables
///   loop with PalMSpriteSetLoop(handle, 0), and rewinds ctx[201015] by 12 until
///   PalMSpriteGetState(handle) has bit 4 or skip/auto is active.
/// - Disassembly: docs/dis.txt 000B:0003
/// - PAL.sqlite: PalMSpriteGetState 0x10120BD3
/// - Writeup: docs/writeup.md MSprite
///
/// Engine: Blocked — portable runtime mirrors first-pass pop plus PC rewind
/// polling against MSpriteSystem state.
///
/// Decompiler: Blocked — renders as msp_wait(slot).
static SIG_MSP_WAIT: ExtSig = sig!(11, 3, "msp_wait", pop=1,
    params=["slot":0=SpriteSlot], return=Void, effects=[BlocksScript], purpose="Wait until an MSprite/movie slot reaches the native finished bit.",
    status=Blocked, decompiler=Blocked, evidence=[GameSqlite:"reverse/Game.sqlite sub_42E240", Disassembly:"docs/dis.txt 000B:0003", PalSqlite:"PalMSpriteGetState 0x10120BD3", Writeup:"docs/writeup.md MSprite"],
    pal="PalMSpriteGetState" => "0x10120BD3");
/// category 11 index 4: msp_lock
///
/// Purpose: Pause/lock frame advancement for an MSprite/movie slot.
///
/// VM arguments:
/// - pop[0]: slot (SpriteSlot) — movie slot to lock.
///
/// Return: void.
///
/// Side effects: MutatesSprite.
///
/// Evidence:
/// - Disassembly: docs/dis.txt 000B:0004
/// - PAL.sqlite: PalMSpriteLock 0x1011A86E
/// - Writeup: docs/writeup.md MSprite
///
/// Engine: Blocked — calls PalMSpriteLock.
///
/// Decompiler: Blocked — renders as msp_lock(slot).
static SIG_MSP_LOCK: ExtSig = sig!(11, 4, "msp_lock", pop=1,
    params=["slot":0=SpriteSlot], return=Void, effects=[MutatesSprite], purpose="Lock/pause decoder frame advancement.",
    status=Blocked, decompiler=Blocked, evidence=[Disassembly:"docs/dis.txt 000B:0004", PalSqlite:"PalMSpriteLock 0x1011A86E", Writeup:"docs/writeup.md MSprite"],
    pal="PalMSpriteLock" => "0x1011A86E");
/// category 11 index 5: msp_unlock
///
/// Purpose: Resume/unlock frame advancement for an MSprite/movie slot.
///
/// VM arguments:
/// - pop[0]: slot (SpriteSlot) — movie slot to unlock.
///
/// Return: void.
///
/// Side effects: MutatesSprite.
///
/// Evidence:
/// - Disassembly: docs/dis.txt 000B:0005
/// - PAL.sqlite: PalMSpriteUnlock 0x10120CD2
/// - Writeup: docs/writeup.md MSprite
///
/// Engine: Blocked — calls PalMSpriteUnlock.
///
/// Decompiler: Blocked — renders as msp_unlock(slot).
static SIG_MSP_UNLOCK: ExtSig = sig!(11, 5, "msp_unlock", pop=1,
    params=["slot":0=SpriteSlot], return=Void, effects=[MutatesSprite], purpose="Unlock/resume decoder frame advancement.",
    status=Blocked, decompiler=Blocked, evidence=[Disassembly:"docs/dis.txt 000B:0005", PalSqlite:"PalMSpriteUnlock 0x10120CD2", Writeup:"docs/writeup.md MSprite"],
    pal="PalMSpriteUnlock" => "0x10120CD2");

// Category 17 - action/tween scheduler.

/// category 17 index 0: action_run_count_over
///
/// Purpose: Start or poll an asynchronous action counter without blocking the
/// VM.  Returns non-zero if the counter has elapsed.
///
/// VM arguments:
/// - pop[0]: action_id (Integer) — action slot/identifier, or -1 for active.
/// - pop[1]: duration_ms (DurationMs) — counter duration.
///
/// Return: Status — native handler returns 1 after scheduling, or 1 for
/// invalid action ids after logging.
///
/// Side effects: CreatesTask.
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite action handler decompilation around action_run_count_over
/// - RuntimeTrace: pal-vm dispatch_action_stub index 0 pops 2
///
/// Engine: Verified — starts a nonblocking compatible action timer, stores the
/// active action id, and returns native status 1.
///
/// Decompiler: Verified — renders as action_run_count_over(action_id, duration_ms).
static SIG_ACTION_RUN_COUNT_OVER: ExtSig = sig!(17, 0, "action_run_count_over", pop=2,
    params=["action_id":0=Integer, "duration_ms":1=DurationMs],
    return=Status, effects=[CreatesTask],
    purpose="Start/check an asynchronous action counter without blocking.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_40B6E0 pops action_id then duration, schedules action state, and returns 1", RuntimeTrace:"pal-vm dispatch_action_stub index 0 pops 2 and returns native status 1"],
    game="0x0040B6E0");
/// category 17 index 1: action_sync_run_count_over
///
/// Purpose: Start an action counter and block the script until the duration
/// elapses.  Synchronous variant of action_run_count_over.
///
/// VM arguments:
/// - pop[0]: action_id (Integer) — action slot/identifier, or -1 for active.
/// - pop[1]: duration_ms (DurationMs) — counter duration.
///
/// Return: Bool — non-zero on completion.
///
/// Side effects: CreatesTask, BlocksScript.
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite action handler decompilation around action_sync_run_count_over
/// - RuntimeTrace: pal-vm dispatch_action_stub index 1 pops 2
///
/// Engine: Verified — schedules the compatible action timer and returns a VM
/// wait request for the decoded duration.
///
/// Decompiler: Verified — renders as action_sync_run_count_over(action_id, duration_ms).
static SIG_ACTION_SYNC_RUN_COUNT_OVER: ExtSig = sig!(17, 1, "action_sync_run_count_over", pop=2,
    params=["action_id":0=Integer, "duration_ms":1=DurationMs],
    return=Bool, effects=[CreatesTask, BlocksScript],
    purpose="Start an action counter and block until the duration elapses.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_40B5C0 pops two values, schedules action state, sets blocking flag, and logs action_sync_run", RuntimeTrace:"pal-vm dispatch_action_stub index 1 pops 2 and returns WaitRequest::Time"],
    game="0x0040B5C0");
/// category 17 index 3: action_clear_count_over
///
/// Purpose: Clear/reset an action counter.  action_id=-1 clears the current
/// active counter; otherwise clears the indexed counter.
///
/// VM arguments:
/// - pop[0]: action_id (Integer) — action id, or -1 for current.
///
/// Return: void.
///
/// Side effects: CreatesTask (counter teardown).
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite sub_40B430 pseudocode "action_clear count over" and one stack pop
/// - RuntimeTrace: pal-vm dispatch_action_stub index 3 pops 1
///
/// Engine: Verified — clears the current compatible action counter when
/// action_id is -1 and preserves native status return 1.
///
/// Decompiler: Verified — renders as action_clear_count_over(action_id).
static SIG_ACTION_CLEAR_COUNT_OVER: ExtSig = sig!(17, 3, "action_clear_count_over", pop=1,
    params=["action_id":0=Integer],
    return=Void, effects=[CreatesTask],
    purpose="-1 clears the current action counter; otherwise clear/check the indexed action counter.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_40B430 pops one action id, resolves -1 to active id, clears action memory, and returns 1", RuntimeTrace:"pal-vm dispatch_action_stub index 3 pops 1 and clears compatible action state"],
    game="0x0040B430");
/// category 17 index 5: action_timeline_entry
///
/// Purpose: Append a timed sprite-position delta section to an action line.
/// Native stores action bytecode section type 0; the runner applies the delta
/// over `duration_ms` and commits the final position lane when complete.
///
/// VM arguments (display/pop order):
/// - pop[0]: line_id (Integer) — action line id, max 0x3f.
/// - pop[1]: sprite_slot (SpriteSlot) — Game sprite slot resolved through
///   `sub_449120`.
/// - pop[2]: delta_x (CoordinateX) — script-space x delta; the native action
///   parser multiplies by 1.5 before committing to the 1920-wide wrapper.
/// - pop[3]: delta_y (CoordinateY) — script-space y delta; native multiplies
///   by 1.5 before committing to the 1080-high wrapper.
/// - pop[4]: delta_z (Integer) — z/priority delta.
/// - pop[5]: duration_ms (DurationMs) — section duration; zero coerces to 1.
///
/// Return: void.
///
/// Side effects: CreatesTask, MutatesSprite.
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite `sub_40ADB0` pops line_id plus
///   sprite_slot/dx/dy/dz/duration, writes section type 0, and resolves the
///   wrapper with `sub_449120`.
/// - Game.sqlite: reverse/Game.sqlite `sub_446650` case 0 builds a timed
///   `sub_403490` position action and `sub_403430` commits final x/y lanes.
/// - RuntimeTrace: New Game path uses `action_timeline_entry(0, 12, 0, -30,
///   0, 1000)` after `sp_set_pos_move(12, 0, 30, 0)` to slide the standing
///   sprite back while fading it in.
///
/// Engine: Verified — pal-vm maps section type 0 to a PalSprite position
/// tween.
///
/// Decompiler: Verified — renders line/slot/delta/duration directly.
static SIG_ACTION_TIMELINE_5: ExtSig = sig!(17, 5, "action_timeline_entry", pop=6,
    params=["line_id":0=Integer, "sprite_slot":1=SpriteSlot, "delta_x":2=CoordinateX, "delta_y":3=CoordinateY, "delta_z":4=Integer, "duration_ms":5=DurationMs],
    return=Void, effects=[CreatesTask, MutatesSprite],
    purpose="Append a timed sprite-position delta action section.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_40ADB0 pops line_id plus sprite_slot/dx/dy/dz/duration, writes section type 0, and resolves sprite_slot via sub_449120", GameSqlite:"reverse/Game.sqlite sub_446650 case 0 builds sub_403490 timed position delta and sub_403430 commits final position lanes", RuntimeTrace:"New Game path uses action_timeline_entry(0,12,0,-30,0,1000) to undo a temporary sp_set_pos_move offset"]);
/// category 17 index 6: action_timeline_entry_ex
///
/// Purpose: Append a six-argument Game.exe action/tween entry.  Shares the
/// runtime stack layout with index 5, but native assigns a different internal
/// section type.
///
/// VM arguments: same layout as `action_timeline_entry`.
///
/// Return: void.
///
/// Side effects: CreatesTask, MutatesSprite.
///
/// Evidence: Game.sqlite action dispatch group 5/6/10/14/15/29 consumes six
/// values and appends an action section; runtime dispatch index 6 pops six.
static SIG_ACTION_TIMELINE_6: ExtSig = sig!(17, 6, "action_timeline_entry_ex", pop=6,
    params=["action_id":0=Integer, "duration_ms":1=DurationMs, "from":2=Integer, "to":3=Integer, "mode":4=Mode, "flags":5=Flag],
    return=Void, effects=[CreatesTask, MutatesSprite],
    purpose="Append a six-argument Game.exe action/tween entry with a distinct native section type.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite action dispatch group 5/6/10/14/15/29 pops 6", RuntimeTrace:"pal-vm dispatch_action_stub index 6 pops 6"]);
/// category 17 index 7: action_timeline4_entry
///
/// Purpose: Append a four-argument Game.exe action/tween entry.  Native shares
/// the same family as index 11 with a different internal section type.
static SIG_ACTION_TIMELINE_7: ExtSig = sig!(17, 7, "action_timeline4_entry", pop=4,
    params=["duration_ms":0=DurationMs, "action_id":1=Integer, "arg2":2=Integer, "arg3":3=Integer],
    return=Void, effects=[CreatesTask, MutatesSprite],
    purpose="Append a four-argument Game.exe action/tween entry.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite action dispatch index 7/11 pops 4", RuntimeTrace:"pal-vm dispatch_action_stub index 7 pops 4"]);
/// category 17 index 8: action_alpha_delta
///
/// Purpose: Configure a timed alpha-delta action for a sprite slot.  The
/// shared title/system transition helper at point[3042] uses this to fade the
/// temporary slot-64 layer before clearing it.
///
/// VM arguments (display/source order: action_id, sprite_slot, alpha_delta, duration_ms):
/// - pop[0]: action_id (Integer) — action slot, usually 2 for wrapper fades.
/// - pop[1]: sprite_slot (Integer) — Game sprite slot to mutate.
/// - pop[2]: alpha_delta (Integer) — signed delta added to current alpha.
/// - pop[3]: duration_ms (DurationMs) — tween duration.
///
/// Return: void.
///
/// Side effects: CreatesTask, MutatesSprite.
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite `sub_446650` action bytecode case 3
///   constructs the `sub_402F30` timed color-lane delta, and
///   `sub_4494D0`/`sub_4498D0` clamp the lane into PalSpriteSetColor.
/// - PAL.sqlite: PalSpriteSetColor 0x10119103.
/// - Disassembly: docs/dis.txt point[809] pushes formal arg[-2],
///   `-255`, formal arg[-1], and `2`; after PAL arg-area reversal this pops
///   as `[action_id, sprite_slot, delta, duration]`.
/// - RuntimeTrace: the corrected arg-area maps the EV104AA fade to slot 21 and
///   duration 1500, eliminating the bogus slot=1500 clear.
///
/// Engine: Verified — schedules the action timer and tweens compatible sprite
/// alpha from current to current+delta.
///
/// Decompiler: Verified — renders explicit action id, sprite slot, alpha delta,
/// and duration arguments.
static SIG_ACTION_TIMELINE_8: ExtSig = sig!(17, 8, "action_alpha_delta", pop=4,
    params=["action_id":0=Integer, "sprite_slot":1=SpriteSlot, "alpha_delta":2=Integer, "duration_ms":3=DurationMs],
    return=Void, effects=[CreatesTask, MutatesSprite],
    purpose="Configure a timed sprite alpha-delta action.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_446650 case 3 builds sub_402F30 timed alpha delta and sub_4494D0/sub_4498D0 commit through PalSpriteSetColor", PalSqlite:"PalSpriteSetColor 0x10119103", Disassembly:"docs/dis.txt point[809] formal args plus pack_args reversal produce pop order action_id,slot,delta,duration", RuntimeTrace:"EV104AA path decodes slot=21 duration=1500 and clears slot=21"]);
/// category 17 index 23: set_active_action
///
/// Purpose: Set the active action id that is used by subsequent action
/// query/clear calls when they receive action_id=-1.
///
/// VM arguments:
/// - pop[0]: action_id (Integer) — action id to set as current.
///
/// Return: void.
///
/// Side effects: CreatesTask (updates active action state).
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite action handler set_active_action path
/// - RuntimeTrace: pal-vm dispatch_action_stub index 23 pops 1
///
/// Engine: Verified — stores action_id as current active action for later
/// -1 resolution in action clear/query calls.
///
/// Decompiler: Verified — renders as set_active_action(action_id).
static SIG_ACTION_SET_ACTIVE: ExtSig = sig!(17, 23, "set_active_action", pop=1,
    params=["action_id":0=Integer],
    return=Void, effects=[CreatesTask],
    purpose="Set the active action id used by later action query/clear calls.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_4095C0 pops action_id, validates <16, stores old/current active ids, and logs set_active_action", RuntimeTrace:"pal-vm dispatch_action_stub index 23 pops action_id and updates active action"],
    game="0x004095C0");
/// category 17 index 9: action_line_wait
///
/// Purpose: Append a wait/timing section to a Game.exe action line.  Native
/// records section type 3 and stores the second parameter in the section
/// duration field.
///
/// VM arguments:
/// - pop[0]: line_id (Integer) — action line id, max 0x3f.
/// - pop[1]: duration_ms (DurationMs) — section duration; native coerces zero
///   to 1.
///
/// Return: void.
///
/// Side effects: CreatesTask.
///
/// Evidence:
/// - Game.sqlite / IDB: category 17 dispatch index 9 resolves to sub_40A390.
///   It pops one action line id from the VM stack, calls sub_44B050 once for
///   duration, stores section type 3, stores duration in section[19], and
///   increments the line section count.
///
/// Engine: Verified — pal-vm pops line_id/duration, schedules the compatible
/// action timer, and no longer treats duration as active_action_id.
///
/// Decompiler: Verified — renders as action_line_wait(line_id, duration_ms).
static SIG_ACTION_TIMELINE_9: ExtSig = sig!(17, 9, "action_line_wait", pop=2,
    params=["line_id":0=Integer, "duration_ms":1=DurationMs],
    return=Void, effects=[CreatesTask],
    purpose="Append a wait/timing section to a Game.exe action line.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_40A390 pops line id and duration, writes action section type 3 and duration field", RuntimeTrace:"pal-vm dispatch_action_stub index 9 pops line_id/duration and schedules duration"],
    game="0x0040A390");
/// category 17 index 10: action_timeline_entry_ease
///
/// Purpose: Append a six-argument action/tween entry in the same family as
/// index 5, with native interpolation/easing section type selected by index 10.
static SIG_ACTION_TIMELINE_10: ExtSig = sig!(17, 10, "action_timeline_entry_ease", pop=6,
    params=["action_id":0=Integer, "duration_ms":1=DurationMs, "from":2=Integer, "to":3=Integer, "mode":4=Mode, "flags":5=Flag],
    return=Void, effects=[CreatesTask, MutatesSprite],
    purpose="Append a six-argument Game.exe action/tween entry with easing section type.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite action dispatch group 5/6/10/14/15/29 pops 6", RuntimeTrace:"pal-vm dispatch_action_stub index 10 pops 6"]);
/// category 17 index 11: action_timeline4_entry_ex
///
/// Purpose: Four-argument action/tween helper sharing the index-7 dispatch
/// path with a different native section type.
///
/// VM arguments: same layout as ext_0011_0008 (index 8).
/// - pop[0]: duration_ms (DurationMs)
/// - pop[1]: action_id (Integer)
/// - pop[2]: arg2 (Integer)
/// - pop[3]: arg3 (Integer)
///
/// Return: void.
///
/// Side effects: CreatesTask, MutatesSprite.
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite action dispatch index 11 pops 4
/// - RuntimeTrace: pal-vm dispatch_action_stub index 11 pops 4
///
/// Engine: Blocked — stores 4-arg tween entry.
///
/// Decompiler: Blocked — renders 4 args.
static SIG_ACTION_TIMELINE_11: ExtSig = sig!(17, 11, "action_timeline4_entry_ex", pop=4,
    params=["duration_ms":0=DurationMs, "action_id":1=Integer, "arg2":2=Integer, "arg3":3=Integer],
    return=Void, effects=[CreatesTask, MutatesSprite],
    purpose="Four-argument action/tween helper sharing the index-7 path.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite action dispatch index 11 pops 4", RuntimeTrace:"pal-vm dispatch_action_stub index 11 pops 4"]);
/// category 17 index 14: action_timeline_entry_log
///
/// Purpose: Append a six-argument action/tween entry; IDB helper family
/// includes log/ease interpolation variants.
static SIG_ACTION_TIMELINE_14: ExtSig = sig!(17, 14, "action_timeline_entry_log", pop=6,
    params=["action_id":0=Integer, "duration_ms":1=DurationMs, "from":2=Integer, "to":3=Integer, "mode":4=Mode, "flags":5=Flag],
    return=Void, effects=[CreatesTask, MutatesSprite],
    purpose="Append a six-argument action/tween entry using a native log/ease section variant.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite action dispatch group 5/6/10/14/15/29 pops 6", RuntimeTrace:"pal-vm dispatch_action_stub index 14 pops 6"]);
/// category 17 index 15: action_timeline_entry_sine
///
/// Purpose: Append a six-argument action/tween entry; IDB helper family
/// includes sine interpolation variants.
static SIG_ACTION_TIMELINE_15: ExtSig = sig!(17, 15, "action_timeline_entry_sine", pop=6,
    params=["action_id":0=Integer, "duration_ms":1=DurationMs, "from":2=Integer, "to":3=Integer, "mode":4=Mode, "flags":5=Flag],
    return=Void, effects=[CreatesTask, MutatesSprite],
    purpose="Append a six-argument action/tween entry using a native sine section variant.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite action dispatch group 5/6/10/14/15/29 pops 6", RuntimeTrace:"pal-vm dispatch_action_stub index 15 pops 6"]);
/// category 17 index 20: action_timeline_rect
///
/// Purpose: Append a five-argument rect/source-rect action section.  Native
/// writes action section type 20 in `sub_4097A0`.
static SIG_ACTION_TIMELINE_20: ExtSig = sig!(17, 20, "action_timeline_rect", pop=5,
    params=["line_id":0=Integer, "left":1=CoordinateX, "top":2=CoordinateY, "right":3=CoordinateX, "bottom":4=CoordinateY],
    return=Void, effects=[CreatesTask, MutatesSprite],
    purpose="Append a source-rect action section to an action line.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_4097A0 pops line id, reads four action args via sub_44B050, writes section type 20", RuntimeTrace:"pal-vm dispatch_action_stub index 20 pops 5"],
    game="0x004097A0");
/// category 17 index 21: action_push
///
/// Purpose: Push the current native action context onto the Game action stack.
/// Native copies the active action block into a heap backup and clears the
/// current block.
static SIG_ACTION_PUSH: ExtSig = sig!(17, 21, "action_push", pop=0,
    params=[], return=Void, effects=[CreatesTask],
    purpose="Push current Game action context onto the action stack.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_4096D0 action_push copies current action block to heap backup"]);
/// category 17 index 22: action_pop
///
/// Purpose: Restore the last action context saved by `action_push`.
static SIG_ACTION_POP: ExtSig = sig!(17, 22, "action_pop", pop=0,
    params=[], return=Void, effects=[CreatesTask],
    purpose="Restore the previous Game action context from the action stack.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_409660 action_pop restores heap-saved action block"]);
/// category 17 index 29: action_timeline_entry2
///
/// Purpose: Append a timed sprite-position delta section to the active Game
/// action line.  This is the common ADV standing-sprite slide helper.
///
/// VM arguments (pop/display order):
/// - pop[0]: line_id (Integer) — action line id, max 0x3f.
/// - pop[1]: sprite_slot (SpriteSlot) — Game sprite slot resolved through
///   `sub_449120`.
/// - pop[2]: delta_x (CoordinateX) — script-space x delta; native multiplies
///   by 1.5 before writing the 1920-wide action section.
/// - pop[3]: delta_y (CoordinateY) — script-space y delta; native multiplies
///   by 1.5 before writing the 1080-high action section.
/// - pop[4]: delta_z (Integer) — z/priority delta.
/// - pop[5]: duration_ms (DurationMs) — section duration; zero coerces to 1.
///
/// Return: void.
///
/// Side effects: CreatesTask, MutatesSprite.
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite `sub_40AC70` pops line id, then calls
///   `sub_44B050` for sprite slot, x/y/z delta, and duration.  It stores
///   section type 15 and resolves the sprite slot with `sub_449120`.
/// - RuntimeTrace: ADV New Game path uses this immediately after
///   `sp_set_pos_move(slot, 0, -500, 0)` to slide standing sprites back into
///   position.
///
/// Engine: Verified — pal-vm maps the section to a PalSprite position tween.
///
/// Decompiler: Verified — renders complete line/slot/delta/duration args.
static SIG_ACTION_TIMELINE_29: ExtSig = sig!(17, 29, "action_timeline_entry2", pop=6,
    params=["line_id":0=Integer, "sprite_slot":1=SpriteSlot, "delta_x":2=CoordinateX, "delta_y":3=CoordinateY, "delta_z":4=Integer, "duration_ms":5=DurationMs],
    return=Void, effects=[CreatesTask, MutatesSprite],
    purpose="Append a timed sprite-position delta action section.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_40AC70 pops line_id plus sprite_slot/dx/dy/dz/duration via sub_44B050, stores section type 15, and resolves sprite_slot via sub_449120", RuntimeTrace:"New Game ST05A path: sp_set_pos_move(slot=12,dy=-500) followed by action_timeline_entry2 tween restoring the standing sprite"]);
/// category 17 index 30: set_action_clear
///
/// Purpose: Clear/reset an action entry.  action_id=-1 clears the current
/// active action state; otherwise sets/clears the indexed action flag.
///
/// VM arguments:
/// - pop[0]: action_id (Integer) — action id, or -1 for current.
///
/// Return: void.
///
/// Side effects: CreatesTask.
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite sub_40B3A0 debug string "set_action_clear" and one stack pop
/// - RuntimeTrace: pal-vm dispatch_action_stub index 30 pops 1
///
/// Engine: Verified — resolves -1 to the active action id and sets the clear
/// marker for that action in the compatible action state.
///
/// Decompiler: Verified — renders as set_action_clear(action_id).
static SIG_ACTION_SET_CLEAR: ExtSig = sig!(17, 30, "set_action_clear", pop=1,
    params=["action_id":0=Integer],
    return=Void, effects=[CreatesTask],
    purpose="-1 clears current action state; otherwise sets/clears the indexed action flag.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_40B3A0 pops one action id, resolves -1 to active id, marks action clear, and returns 1", RuntimeTrace:"pal-vm dispatch_action_stub index 30 pops 1 and updates active action clear state"],
    game="0x0040B3A0");

// Category 18 - INI/file startup path.

/// category 18 index 1: app_exec
///
/// Purpose: Launch or open a resolved application/path.  Native calls
/// ShellExecuteA after resolving the script string and resetting CWD to the PAL
/// file root.
///
/// VM arguments:
/// - pop[0]: path (TextStringFromTextDat/DynamicString) — executable or file
///   path to pass to ShellExecuteA.
///
/// Return: status integer, 1 after dispatch.
///
/// Side effects: ReadsFile, MutatesWindow.
///
/// Evidence:
/// - Game.sqlite: sub_41AB90 pops one string, calls PalGetFilePath(),
///   SetCurrentDirectoryA(), then ShellExecuteA("open", path).
/// - Disassembly: docs/dis.txt 00059EA0 pushes the wsprint result before
///   ext_0012_0001.
///
/// Engine: Blocked — portable runtime deliberately does not launch host
/// programs, but it resolves and logs the target and preserves stack/return.
///
/// Decompiler: Verified — renders app_exec(path) with the real argument.
static SIG_APP_EXEC: ExtSig = sig!(18, 1, "app_exec", pop=1,
    params=["path":0=TextStringFromTextDat=>"executable/file path"],
    return=Integer, effects=[ReadsFile, MutatesWindow],
    purpose="Open a resolved path through the OS shell; portable runtime records but does not launch.",
    status=Blocked, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_41AB90 pops one string and calls ShellExecuteA", Disassembly:"docs/dis.txt 00059EA0 pushes path before ext_0012_0001"],
    game="0x0041AB90");

/// category 18 index 9: wsprint
///
/// Purpose: Format a script string into a dynamic string buffer.  The native
/// handler is used heavily by file/table scan loops and menu/debug overlays;
/// a wrong pop count here leaves dynamic strings empty and corrupts the VM
/// stack for following strlenf/string compare calls.
///
/// VM arguments:
/// - pop[0]: dst (DynamicString) — destination `0x10000000 | slot` buffer.
/// - pop[1]: format (TextStringFromTextDat/DynamicString) — printf-like format.
/// - pop[2..9]: value0..value7 (Integer/TextStringFromTextDat/DynamicString)
///   — formatting operands. `%f`/`%s` resolve strings, `%d`/`%i` resolve ints.
///
/// Return: status integer.
///
/// Side effects: WritesVmMemory.
///
/// Evidence:
/// - Game.sqlite: sub_419560 pops destination, format, then eight values,
///   resolves the format through sub_44B120/sub_44B1E0, expands printf-like
///   tokens, and copies the result to `ctx+771296+2047*slot`.
/// - RuntimeTrace: New Game path reaches repeated strlenf loops immediately
///   after wsprint/string allocation; zero-pop fallback made every length 0.
///
/// Engine: Verified — runtime pops ten values and writes a formatted dynamic
/// string for `%f`/`%s`/`%d`/`%i`/`%x` tokens.
///
/// Decompiler: Verified — renders wsprint(dst, format, value0..value7).
static SIG_WSPRINT: ExtSig = sig!(18, 9, "wsprint", pop=10,
    params=[
        "dst":0=BufferPointer=>"destination dynamic string handle",
        "format":1=TextStringFromTextDat=>"printf-like format string",
        "value0":2=Integer=>"format operand 0",
        "value1":3=Integer=>"format operand 1",
        "value2":4=Integer=>"format operand 2",
        "value3":5=Integer=>"format operand 3",
        "value4":6=Integer=>"format operand 4",
        "value5":7=Integer=>"format operand 5",
        "value6":8=Integer=>"format operand 6",
        "value7":9=Integer=>"format operand 7"
    ],
    return=Integer, effects=[WritesVmMemory],
    purpose="Format a script string into a dynamic string buffer.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"0x00419560 sub_419560 pops dst/format/eight values and writes formatted bytes to dynamic string storage", RuntimeTrace:"codex-sprite-transition-new showed strlenf on freshly allocated handles staying 0 under the old zero-pop fallback"],
    game="0x00419560");

/// category 18 index 3: string_not_equal
///
/// Purpose: Compare two script strings case-insensitively and return whether
/// they differ.  Used by table/file scan loops to stop when the currently read
/// label does not match the requested label.
///
/// VM arguments:
/// - pop[0]: lhs (TextStringFromTextDat/DynamicString) — first string id.
/// - pop[1]: rhs (TextStringFromTextDat/DynamicString) — second string id.
///
/// Return: bool integer, 1 when strings differ, 0 when equal.
///
/// Side effects: WritesVmMemory.
///
/// Evidence:
/// - Game.sqlite: sub_41EBD0 pops two string ids, resolves both with
///   sub_44B120, lowercases both strings, strcmp()s them, and writes
///   `(strcmp != 0)` to the extcall return slot.
/// - Disassembly: docs/dis.txt callsites push two values before ext_0012_0003
///   and branch on the returned bool.
///
/// Engine: Verified — runtime pops two arguments and compares resolved strings
/// case-insensitively.
///
/// Decompiler: Verified — renders string_not_equal(lhs, rhs) instead of the
/// raw ext_0012_0003 name.
static SIG_STRING_NOT_EQUAL: ExtSig = sig!(18, 3, "string_not_equal", pop=2,
    params=["lhs":0=TextStringFromTextDat=>"first script/dynamic string", "rhs":1=TextStringFromTextDat=>"second script/dynamic string"],
    return=Bool, effects=[WritesVmMemory],
    purpose="Case-insensitive string inequality test used by table scan loops.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_41EBD0 pops two strings, lowercases both, strcmp()s, and returns strcmp != 0", Disassembly:"docs/dis.txt 00004170/0000F470/000637D0 push two args before ext_0012_0003"],
    game="0x0041EBD0");
/// category 18 index 5: strgetcf
///
/// Purpose: Read one character code or parse a digit-only substring from a
/// script/dynamic string.  This is used by CSV/table loops after `file_string`
/// and by `%02d` resource-name builders; documenting it as favorite/bookmark
/// access was an old reverse mistake.
///
/// VM arguments:
/// - pop[0]: text (TextStringFromTextDat) — source string handle/id.
/// - pop[1]: offset (Integer) — byte offset inside the resolved string.
/// - pop[2]: length (Integer) — zero returns one byte; non-zero parses that
///   many ASCII digits as an integer.
///
/// Return: Integer — character byte when length is zero, parsed integer for a
/// digit-only span, or 0 for out-of-range/non-digit spans.
///
/// Side effects: WritesVmMemory (native extcall return slot).
///
/// Evidence:
/// - Game.sqlite/IDB: sub_41A6B0 pops string, offset, length; resolves with
///   sub_44B120; returns byte-at-offset when length is zero, otherwise copies
///   a bounded substring and validates digits before atoi.
/// - RuntimeTrace: table/resource-name setup calls ext_0012_0005 immediately
///   after file_string/strlenf and expects numeric character fields.
///
/// Engine: Verified — pal-vm `ext_str_get_char_or_int` implements the same
/// three-argument return behavior.
///
/// Decompiler: Verified — renders strgetcf(text, offset, length).
static SIG_STRGETCF: ExtSig = sig!(18, 5, "strgetcf", pop=3,
    params=["text":0=TextStringFromTextDat=>"source script/dynamic string", "offset":1=Integer=>"byte offset", "length":2=Integer=>"0 for one byte, non-zero for digit substring"],
    return=Integer, effects=[WritesVmMemory],
    purpose="Read a character byte or parse a numeric substring from a resolved string.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite/IDB sub_41A6B0 pops string, offset, length and returns byte or parsed digit substring", RuntimeTrace:"file_string/strlenf table loops call ext_0012_0005 for character/numeric fields"],
    game="0x0041A6B0");
/// category 18 index 8: file_exist
///
/// Purpose: Test whether a resolved PAL resource/path exists.
///
/// VM arguments:
/// - pop[0]: path (TextStringFromTextDat/DynamicString) — file/resource path.
///
/// Return: bool integer.
///
/// Side effects: ReadsFile.
///
/// Evidence:
/// - Game.sqlite: sub_41A260 pops one string, resolves it through sub_44B120,
///   calls PalFileCreate/CloseHandle, and writes a boolean result.
/// - Disassembly: docs/dis.txt 00067BF0 and 00068390 push a formatted path
///   before ext_0012_0008.
///
/// Engine: Verified — portable runtime checks ResourceManager/PAC and loose
/// root paths without opening native Win32 handles.
///
/// Decompiler: Verified — renders file_exist(path).
static SIG_FILE_EXIST: ExtSig = sig!(18, 8, "file_exist", pop=1,
    params=["path":0=TextStringFromTextDat=>"resource or filesystem path"],
    return=Bool, effects=[ReadsFile],
    purpose="Return whether a resolved resource/path exists.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_41A260 pops one string and calls PalFileCreate", Disassembly:"docs/dis.txt 00067BF0/00068390 push path before file_exist"],
    game="0x0041A260");
/// category 18 index 10: check_disc
///
/// Purpose: Check whether the requested disc/volume label is mounted.  Native
/// bypasses the check in debug mode, otherwise scans CD-ROM drives.
///
/// VM arguments:
/// - pop[0]: volume_label (TextStringFromTextDat/DynamicString) — requested
///   disc volume label.
///
/// Return: bool integer.
///
/// Side effects: ReadsFile.
///
/// Evidence:
/// - Game.sqlite: sub_419370 pops one string; if PalDebugIs() it returns 1,
///   otherwise enumerates logical drives and compares CD-ROM volume labels.
/// - Disassembly: docs/dis.txt 0002D03C and 0002D168 push the label before
///   ext_0012_000A.
///
/// Engine: Verified — portable runtime follows the debug/compat path and
/// returns true so non-Windows/headless runs do not fail a physical-disc check.
///
/// Decompiler: Verified — renders check_disc(volume_label).
static SIG_CHECK_DISC: ExtSig = sig!(18, 10, "check_disc", pop=1,
    params=["volume_label":0=TextStringFromTextDat=>"requested CD/DVD volume label"],
    return=Bool, effects=[ReadsFile],
    purpose="Check mounted disc volume label; portable runtime accepts the check.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_419370 pops one label and scans CD-ROM volume labels, with debug return 1", Disassembly:"docs/dis.txt 0002D03C/0002D168 push label before check_disc"],
    game="0x00419370");
/// category 18 index 6: arg_get
///
/// Purpose: Read a value from the current Game.exe extcall argument stack by
/// one-based index.  Used by wrapper scripts that need to re-inspect the
/// argument block of the calling extcall without re-popping it.
///
/// VM arguments:
/// - pop[0]: ignored (Integer) — native pops one value but does not use it.
///
/// Return: DynamicString handle — `slot | 0x10000000`.
///
/// Side effects: WritesVmMemory.
///
/// Evidence:
/// - Game.sqlite: handler 0x0041AAF0 pops one value, uses ctx[192823] as a
///   rotating 16-slot string-buffer cursor, clears `ctx+771296+2047*slot`,
///   advances the cursor, and writes `slot | 0x10000000` to the extcall dst.
/// - RuntimeTrace: longrun-newgame-ctrl-60000 executes this helper 1519 times
///   in file/string setup loops.
///
/// Engine: Verified — pal-vm keeps the same 16-slot rotating dynamic-string
/// handle table and clears the selected slot.
///
/// Decompiler: Verified — renders as string_alloc(ignored).
static SIG_ARG_GET: ExtSig = sig!(18, 6, "string_alloc", pop=1,
    params=["ignored":0=Integer],
    return=StringId, effects=[WritesVmMemory],
    purpose="Allocate/clear one native rotating dynamic string buffer and return its 0x10000000-tagged handle.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"0x0041AAF0 pops one ignored value, clears 2047-byte dynamic string slot ctx[192823], advances modulo 16, and writes slot|0x10000000", RuntimeTrace:"longrun-newgame-ctrl-60000 executes ext_0012_0006 1519 times"],
    game="0x0041AAF0");
/// category 18 index 12: strlen
///
/// Purpose: Return the byte length of a dynamic/script string.  Native only
/// counts dynamic string handles in the recovered handler; the portable VM also
/// resolves Text.dat ids so decompiled/resource-driven scripts remain readable.
///
/// VM arguments:
/// - pop[0]: value (TextStringFromTextDat/DynamicString) — string id/handle.
///
/// Return: integer byte length.
///
/// Side effects: WritesVmMemory.
///
/// Evidence:
/// - Game.sqlite: sub_41EAC0 pops one value, checks the dynamic string tag,
///   copies the string into a local buffer, strlen()s it, and writes the length.
/// - Disassembly: docs/dis.txt 0005D420 and later callsites use the returned
///   value as a loop/condition result.
/// - Koikake runtime trace: some callsites push seven `0x0FFF_FFFF` fillers
///   immediately before the value. Its runtime consumes that padded form so
///   the fillers cannot displace the caller's argument-frame marker.
///
/// Engine: Verified — pops one value, resolves it, and returns byte length.
///
/// Decompiler: Verified — renders strlen(value) instead of ext_0012_000C.
static SIG_STRING_LENGTH: ExtSig = sig!(18, 12, "strlen", pop=1,
    params=["value":0=TextStringFromTextDat=>"script/dynamic string handle"],
    return=Integer, effects=[WritesVmMemory],
    purpose="Return byte length of a script/dynamic string.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_41EAC0 pops one dynamic string handle and returns strlen", Disassembly:"docs/dis.txt 0005D420 uses ext_0012_000C return as a condition"],
    game="0x0041EAC0");
/// category 18 index 13: process_checkpoint_set
///
/// Purpose: Store a script/process checkpoint id used by the native return/list
/// scheduler.  Reachable story procedures call it before save thumbnail,
/// transition, or chapter-entry setup.
///
/// VM arguments:
/// - pop[0]: checkpoint_id (PointId) — script point/process id.
///
/// Return: status integer.
///
/// Side effects: ChangesRunState.
///
/// Evidence:
/// - Game.sqlite: sub_41B8D0 pops one value, PalListPushes the current process
///   point with tag 3, and stores the popped id in dword_54CE38.
/// - Disassembly: docs/dis.txt 0011A054/0011A068/0011A07C call index 13 with
///   point ids 345/346/347 before save/transition setup.
///
/// Engine: Blocked — full native PalList scheduler tags are not byte-for-byte
/// modeled yet; runtime records no visible state beyond stack discipline.
///
/// Decompiler: Blocked — renders process_checkpoint_set(checkpoint_id).
static SIG_PROCESS_CHECKPOINT_SET: ExtSig = sig!(18, 13, "process_checkpoint_set", pop=1,
    params=["checkpoint_id":0=PointId=>"script/process checkpoint id"],
    return=Integer, effects=[ChangesRunState],
    purpose="Store native script/process checkpoint id for the return/list scheduler.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_41B8D0 pops one id, pushes current process point with PalList tag 3, and stores dword_54CE38", Disassembly:"docs/dis.txt 0011A054/0011A068/0011A07C pass point ids to ext_0012_000D"],
    game="0x0041B8D0");
/// category 18 index 14: update_access
///
/// Purpose: Mark an access/read flag by resource name or numeric entry id.
///
/// VM arguments:
/// - pop[0]: entry (Integer/TextStringFromTextDat) — access id or resource
///   name.
///
/// Return: status integer.
///
/// Side effects: WritesVmMemory, ReadsFile.
///
/// Evidence:
/// - Game.sqlite: sub_417C20 pops one value. String handles are resolved and
///   looked up in the native access table; numeric ids update packed access
///   flag bits under PalTaskGetTaskData(0).
/// - Disassembly: docs/dis.txt 0005B828, 00063410, and 00064C48 push one
///   entry before ext_0012_000E.
///
/// Engine: Blocked — portable runtime records the access update; the full
/// packed native access bitmap is not yet modeled.
///
/// Decompiler: Verified — renders update_access(entry).
static SIG_UPDATE_ACCESS: ExtSig = sig!(18, 14, "update_access", pop=1,
    params=["entry":0=TextStringFromTextDat=>"access flag id or resource string"],
    return=Integer, effects=[WritesVmMemory, ReadsFile],
    purpose="Mark native access/read flag entry.",
    status=Blocked, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_417C20 pops one id/string and mutates native access bits", Disassembly:"docs/dis.txt 0005B828/00063410/00064C48 push entry before update_access"],
    game="0x00417C20");
/// category 18 index 21: strlenf
///
/// Purpose: Resolve a script/dynamic string and return the native byte length.
/// Used by file/table scan loops before string comparison.
///
/// VM arguments:
/// - pop[0]: value (TextStringFromTextDat/DynamicString) — string id/handle.
///
/// Return: integer byte length.
///
/// Side effects: WritesVmMemory.
///
/// Evidence:
/// - Game.sqlite: handler 0x0041A5F0 `sub_41A5F0` pops one string handle,
///   calls `sub_44B1E0(Str1, value, 0)`, computes `strlen(Str1)`, and writes
///   that length to `ctx[dst + 177710]`.
/// - RuntimeTrace: longrun-newgame-ctrl-60000 executes this helper 48839 times
///   from PC 0x00004128/0x000043D0 in the string/file loop.
///
/// Engine: Verified — pal-vm resolves the script string and returns its
/// original NLS byte length, matching native strlen semantics.
///
/// Decompiler: Verified — renders strlenf(value).
static SIG_STRING_NON_EMPTY: ExtSig = sig!(18, 21, "strlenf", pop=1,
    params=["value":0=TextStringFromTextDat=>"script/dynamic string handle"],
    return=Integer, effects=[WritesVmMemory],
    purpose="Return native strlen byte length for a resolved script/dynamic string.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"0x0041A5F0 sub_41A5F0 pops one string handle, calls sub_44B1E0, strlen(Str1), and writes length to ctx[dst + 177710]", RuntimeTrace:"longrun-newgame-ctrl-60000 executes ext_0012_0015 48839 times before file/string loop branches"],
    game="0x0041A5F0");
/// category 18 index 28: attach_work_process
///
/// Purpose: Attach the native Game.exe background work-process pump.  This is
/// used by menu/system scripts to enable a PAL-thread work callback while the
/// script waits for UI/system state.
///
/// VM arguments: none.
///
/// Return: status integer.
///
/// Side effects: CreatesTask, WritesVmMemory, ChangesRunState.
///
/// Evidence:
/// - Game.sqlite / IDB: category 18 dispatch index 28 resolves to sub_417A50.
///   sub_417A50 has no VM stack-pop sequence, sets VM flags at +804088 and
///   +804084, then calls
///   PalAttachWorkProcess(sub_44A080, PalTaskGetTaskData(0)+824).
/// - PAL.sqlite: PalAttachWorkProcess 0x1011CCA9 dispatches to
///   PalAttachWorkProcess_0 0x10237F80, which posts a PAL work-thread message.
///
/// Engine: Verified — pal-vm pops zero arguments and records the compatible
/// single-threaded attach flag/state dirty signal.
///
/// Decompiler: Verified — renders attach_work_process() with no bogus
/// arg_base/value parameter.
static SIG_ATTACH_WORK_PROCESS: ExtSig = sig!(18, 28, "attach_work_process", pop=0,
    params=[],
    return=Integer, effects=[CreatesTask, WritesVmMemory, ChangesRunState],
    purpose="Attach the Game.exe/PAL background work-process pump.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_417A50 pops no args, sets +804088/+804084, calls PalAttachWorkProcess(sub_44A080, PalTaskGetTaskData(0)+824)", PalSqlite:"reverse/PAL.sqlite PalAttachWorkProcess 0x1011CCA9 -> PalAttachWorkProcess_0 0x10237F80 posts PAL work-thread message"],
    game="0x00417A50", pal="PalAttachWorkProcess" => "0x1011CCA9");
/// category 18 index 29: detach_work_process
///
/// Purpose: Clear the native background work-process attached flag.
///
/// VM arguments: none.
///
/// Return: status integer.
///
/// Side effects: WritesVmMemory, ChangesRunState.
///
/// Evidence:
/// - Game.sqlite / IDB: category 18 dispatch index 29 resolves to sub_417A30.
///   sub_417A30 clears VM flag +804088 and returns 1; it has no VM pop.
///
/// Engine: Verified — pal-vm pops zero arguments and clears the portable attach
/// flag.
///
/// Decompiler: Verified — renders detach_work_process().
static SIG_DETACH_WORK_PROCESS: ExtSig = sig!(18, 29, "detach_work_process", pop=0,
    params=[],
    return=Integer, effects=[WritesVmMemory, ChangesRunState],
    purpose="Detach the Game.exe/PAL background work-process pump flag.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_417A30 pops no args and clears +804088"],
    game="0x00417A30");
/// category 18 index 30: openfile
///
/// Purpose: Open a PAC-resident resource file by its resolved script string
/// name.  Returns an integer handle used by read_file / set_file_pointer.
/// Primary use is font loading (FONT*.DAT) during engine startup.
///
/// VM arguments:
/// - pop[0]: name (TextStringFromTextDat) — NUL-terminated resource filename
///   resolved from Text.dat.
///
/// Return: Handle — non-zero file handle on success; 0 on failure.
///
/// Side effects: ReadsFile — opens a resource entry in the PAC archive.
///
/// Evidence:
/// - Writeup: docs/writeup.md 24.9
/// - RuntimeTrace: pal-vm ext_openfile pops 1, opens PAC entry, returns handle
///
/// Engine: Blocked — opens PAC-resident resource; handle is opaque integer.
///
/// Decompiler: Blocked — renders as openfile(name).
///
/// Open points: Handle lifetime / close mechanism not reversed.
static SIG_OPENFILE: ExtSig = sig!(18, 30, "openfile", pop=1,
    params=["name":0=TextStringFromTextDat], return=Handle, effects=[ReadsFile],
    purpose="Open resource/file by resolved script string; returns runtime file handle or 0.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md 24.9"]);
/// category 18 index 31: read_file
///
/// Purpose: Read bytes from an open resource handle into the VM's temporary
/// memory buffer.  Used by the font loader after openfile to pull glyph data.
///
/// VM arguments (display order: handle, temp_offset, count):
/// - pop[0]: handle (Handle) — open file handle from openfile.
/// - pop[1]: temp_offset (BufferPointer) — destination offset in VM temp buffer.
/// - pop[2]: count (Integer) — number of bytes to read.
///
/// Return: Integer — bytes actually read, or -1 on error.
///
/// Side effects: ReadsFile, WritesVmMemory — reads from resource and writes
/// into internal VM temp/scratch buffer.
///
/// Evidence:
/// - Writeup: docs/writeup.md 24.9
/// - RuntimeTrace: pal-vm ext_read_file pops 3
///
/// Engine: Blocked — reads from open handle into vm scratch buffer.
///
/// Decompiler: Blocked — renders as read_file(handle, temp_offset, count).
static SIG_READ_FILE: ExtSig = sig!(18, 31, "read_file", pop=3,
    params=["handle":0=Handle, "temp_offset":1=BufferPointer, "count":2=Integer],
    return=Integer, effects=[ReadsFile, WritesVmMemory], purpose="Read file/table bytes into VM temp memory.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md 24.9"]);
/// category 18 index 32: close_file_not_handle
///
/// Purpose: Close and delete a native open_file table handle.  The historical
/// name is misleading: the Game.exe handler pops the handle pointer, removes it
/// from the PAL list, frees the table buffer, then frees the handle object.
///
/// VM arguments:
/// - pop[0]: handle (Handle) — handle returned by openfile.
///
/// Return: status integer, always 1 after attempting cleanup.
///
/// Side effects: ReadsFile — releases an open runtime file/table handle.
///
/// Evidence:
/// - Game.sqlite: sub_4170C0 pops one handle, calls PalListDelete, PalMemoryFree
///   on the table buffer and handle object, and returns 1.
/// - Disassembly: docs/dis.txt 000633E0 pushes the openfile handle immediately
///   before ext_0012_0020.close_file_not_handle.
///
/// Engine: Verified — runtime pops one handle and releases the portable table
/// handle slot.
///
/// Decompiler: Verified — renders close_file_not_handle(handle).
static SIG_CLOSE_FILE_NOT_HANDLE: ExtSig = sig!(18, 32, "close_file_not_handle", pop=1,
    params=["handle":0=Handle=>"openfile handle"],
    return=Integer, effects=[ReadsFile],
    purpose="Close and release an open_file table/resource handle.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_4170C0 pops one handle, PalListDelete()s it, frees buffer and object, returns 1", Disassembly:"docs/dis.txt 000633E0 pushes handle before ext_0012_0020"],
    game="0x004170C0");
/// category 18 index 33: set_file_pointer
///
/// Purpose: Seek an open resource handle to a given position.  Mirror of
/// Win32 SetFilePointer; origin constants match: 0=begin, 1=current, 2=end.
///
/// VM arguments (display order: handle, offset, origin):
/// - pop[0]: handle (Handle) — open file handle from openfile.
/// - pop[1]: offset (Integer) — seek offset (signed).
/// - pop[2]: origin (Mode) — 0=FILE_BEGIN, 1=FILE_CURRENT, 2=FILE_END.
///
/// Return: Integer — new file position from the beginning, or -1 on error.
///
/// Side effects: ReadsFile — mutates seek position of open handle.
///
/// Evidence:
/// - Writeup: docs/writeup.md 24.9
/// - RuntimeTrace: pal-vm ext_set_file_pointer pops 3
///
/// Engine: Blocked — delegates to internal seek on PAC resource handle.
///
/// Decompiler: Blocked — renders as set_file_pointer(handle, offset, origin).
static SIG_SET_FILE_POINTER: ExtSig = sig!(18, 33, "set_file_pointer", pop=3,
    params=["handle":0=Handle, "offset":1=Integer, "origin":2=Mode],
    return=Integer, effects=[ReadsFile], purpose="Seek an opened runtime file handle.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md 24.9"]);
/// category 18 index 34: file_string
///
/// Purpose: Copy a string payload from an open_file parsed table entry into a
/// dynamic string slot.  File table string entries are encoded as
/// `0x80000000 | offset`; the native handler masks the high bits, reads a
/// u16 byte length at `handle->buffer + string_base + offset`, and copies the
/// bytes into the destination dynamic-string storage.
///
/// VM arguments:
/// - pop[0]: handle (Handle) — handle returned by openfile.
/// - pop[1]: entry (Integer) — encoded table string entry/offset.
/// - pop[2]: dst_slot (BufferPointer) — dynamic string slot/index.
///
/// Return: status/string id.  Native writes destination storage and returns 1;
/// the portable runtime returns the dynamic string handle used by later calls.
///
/// Side effects: WritesVmMemory — mutates dynamic string storage.
///
/// Evidence:
/// - Game.sqlite: sub_416EC0 pops handle pointer first, then entry/offset, then
///   destination slot; it masks entry with 0x7FFFFFFF and destination with
///   0xEFFFFFFF before copying a length-prefixed string into
///   `a1 + 771296 + 2047 * dst`.
/// - Disassembly: docs/dis.txt table readers leave handle on top of the stack
///   before ext_0012_0022.file_string.
///
/// Engine: Verified — runtime resolves the parsed table string and updates or
/// allocates the dynamic string slot.
///
/// Decompiler: Verified — renders file_string(handle, dst_slot, entry).
static SIG_FILE_STRING: ExtSig = sig!(18, 34, "file_string", pop=3,
    params=["handle":0=Handle=>"openfile handle", "entry":1=Integer=>"encoded string table entry", "dst_slot":2=BufferPointer=>"dynamic string destination"],
    return=StringId, effects=[WritesVmMemory],
    purpose="Copy a parsed open_file table string entry into dynamic string storage.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_416EC0 pops handle/entry/dst, masks entry, and copies u16-length string into dynamic string storage", Disassembly:"docs/dis.txt ext_0012_0022 callsites push three values"],
    game="0x00416EC0");
/// category 18 index 35: set_last_process
///
/// Purpose: Resolve a script point id to a process address and store it in the
/// VM's `last_process` field.  Used by native flow-control/menu helpers.
///
/// VM arguments:
/// - pop[0]: point_id (PointId) — script process/point id, or 0 to clear.
///
/// Return: status integer, 1.
///
/// Side effects: ChangesRunState — updates native VM process bookkeeping.
///
/// Evidence:
/// - Game.sqlite: sub_4179B0 pops one point id, resolves it through the script
///   point table when non-zero, stores the resolved address at a1[201019], and
///   returns 1.
///
/// Engine: Blocked — portable VM records the id for trace/diagnostics; full
/// native point-address caching is unnecessary until a caller consumes it.
///
/// Decompiler: Verified — renders set_last_process(point_id).
static SIG_SET_LAST_PROCESS: ExtSig = sig!(18, 35, "set_last_process", pop=1,
    params=["point_id":0=PointId=>"script/process point id or zero"],
    return=Integer, effects=[ChangesRunState],
    purpose="Store native last-process point bookkeeping.",
    status=Blocked, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_4179B0 pops one point id and stores resolved process address in a1[201019]"],
    game="0x004179B0");
/// category 18 index 36: sz_buf
///
/// Purpose: Read an INI string value into the VM's dynamic string store.
/// Mirrors Win32 GetPrivateProfileString.  Non-.ini filename arguments are
/// normalized to SYSTEM.INI by ini_filename_or_system().
///
/// VM arguments (display order: section, key, default, filename, buffer_or_size):
/// - pop[0]: section (TextStringFromTextDat) — INI section name.
/// - pop[1]: key (TextStringFromTextDat) — INI key name.
/// - pop[2]: default (TextStringFromTextDat) — value returned when key absent.
/// - pop[3]: filename (IniFilename) — INI filename; non-.ini → SYSTEM.INI at
///   runtime.  Decompiler annotates with --[[runtime uses SYSTEM.INI: not *.ini]].
/// - pop[4]: buffer_or_size (BufferPointer) — destination dynamic string slot
///   or size hint.
///
/// Return: StringId — dynamic string id holding the result, or 0.
///
/// Side effects: ReadsIni, WritesVmMemory.
///
/// Evidence:
/// - Writeup: docs/writeup.md INI
/// - RuntimeTrace: pal-vm ext_sz_buf pops 5
///
/// Engine: Blocked — reads SYSTEM.INI (or redirected file) and stores result
/// in dynamic string buffer.
///
/// Decompiler: Blocked — IniFilename param renders with SYSTEM.INI annotation.
static SIG_SZ_BUF: ExtSig = sig!(18, 36, "sz_buf", pop=5,
    params=["section":0=TextStringFromTextDat, "key":1=TextStringFromTextDat, "default":2=TextStringFromTextDat, "filename":3=IniFilename, "buffer_or_size":4=BufferPointer],
    return=StringId, effects=[ReadsIni], purpose="Read INI string into dynamic string storage.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md INI", RuntimeTrace:"pal-vm ext_sz_buf pops 5"]);
/// category 18 index 37: getprivateprofileint
///
/// Purpose: Read an integer value from SYSTEM.INI (or a redirected .ini file).
/// PRIMARY RUNTIME BLOCKER: this extcall must return real SYSTEM.INI values
/// for Game.exe to proceed past the startup configuration phase.
/// Non-.ini filename arguments are normalized to SYSTEM.INI by
/// ini_filename_or_system() in the runtime.
///
/// VM arguments (display order: section, key, default, filename):
/// - pop[0]: section (TextStringFromTextDat) — INI section, e.g. "Config".
/// - pop[1]: key (TextStringFromTextDat) — INI key, e.g. "VoiceVolume".
/// - pop[2]: default (Integer) — fallback value when key is absent.
/// - pop[3]: filename (IniFilename) — INI filename; non-.ini strings such as
///   "HIP_BACK_G" are silently remapped to SYSTEM.INI at runtime.  Decompiler
///   annotates with --[[runtime uses SYSTEM.INI: not *.ini]].
///
/// Return: Integer — parsed integer from INI entry, or default.
///
/// Side effects: ReadsIni.
///
/// Evidence:
/// - Writeup: docs/writeup.md 24.9
/// - RuntimeTrace: pal-vm ext_get_private_profile_int pops 4, uses ini_filename_or_system()
///
/// Engine: Blocked — reads live SYSTEM.INI via Rust ini parser; filename
/// normalization is implemented.
///
/// Decompiler: Blocked — IniFilename param renders with SYSTEM.INI annotation
/// when applicable.
static SIG_GETPRIVATEPROFILEINT: ExtSig = sig!(18, 37, "getprivateprofileint", pop=4,
    params=["section":0=TextStringFromTextDat, "key":1=TextStringFromTextDat, "default":2=Integer, "filename":3=IniFilename],
    return=Integer, effects=[ReadsIni], purpose="Read an integer from SYSTEM.INI or requested INI.",
    status=Blocked, decompiler=Blocked, evidence=[Writeup:"docs/writeup.md 24.9"]);

// Category 10 - save/load UI state.

/// category 10 index 12: save_thumbnail_mosaic_set
///
/// Purpose: Set a flag controlling whether the save-slot thumbnail is rendered
/// with mosaic/pixelation processing during save UI display.
///
/// VM arguments:
/// - pop[0]: enabled (Flag) — 1 to enable mosaic on thumbnail, 0 to disable.
///
/// Return: void.
///
/// Side effects: ChangesSaveState.
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite save dispatch index 12 pops 1
/// - RuntimeTrace: pal-vm dispatch_save_stub index 12 pops 1
///
/// Engine: Blocked — stores mosaic flag for save UI rendering.
///
/// Decompiler: Blocked — renders as save_thumbnail_mosaic_set(enabled).
static SIG_SAVE_THUMBNAIL_MOSAIC_SET: ExtSig = sig!(10, 12, "save_thumbnail_mosaic_set", pop=1,
    params=["enabled":0=Flag],
    return=Void, effects=[ChangesSaveState],
    purpose="Set save thumbnail mosaic/processing flag used by save UI rendering.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite save dispatch index 12 pops 1", RuntimeTrace:"pal-vm dispatch_save_stub index 12 pops 1"]);

/// category 10 index 13: savetimedraw
///
/// Purpose: Draw a save-file modification time string into a Game-managed
/// sprite slot used by the save/load menu.
///
/// VM arguments (native pop order):
/// - pop[0]: sprite_slot (SpriteSlot) — text sprite slot to create or update.
/// - pop[1]: save_slot (SaveSlot) — save number, or -1 for continue.dat.
/// - pop[2]: x (CoordinateX) — text sprite x coordinate.
/// - pop[3]: y (CoordinateY) — text sprite y coordinate.
/// - pop[4]: format_mode (Mode) — 0/2 draw colon time, 1/3 draw compact time.
///
/// Return: Integer — native handler returns 1 whether the save data exists or
/// not; missing files only skip text creation.
///
/// Side effects: CreatesSprite, ReadsFile.
///
/// Evidence:
/// - Game.sqlite/IDB: sub_431C70 logs "AdvCommandSaveTimeDraw", pops five
///   values into v22/v25/v27/v26/v23, builds continue.dat or save%03d.dat,
///   reads file mtime, formats HH:MM[:SS] or HHMM[SS], then calls
///   PalSpriteCreateText/PalSpriteCreateTextEx and queues render entry
///   0x000A000D.
///
/// Engine: Blocked — portable runtime now reads loose save-file metadata and
/// creates a text sprite, but PAL's native sprite-text object and local-time
/// conversion are approximated.
///
/// Decompiler: Verified — renders as savetimedraw(sprite_slot, save_slot, x,
/// y, format_mode) with no raw fallback.
static SIG_SAVE_TIME_DRAW: ExtSig = sig!(10, 13, "savetimedraw", pop=5,
    params=["sprite_slot":0=SpriteSlot, "save_slot":1=Integer, "x":2=CoordinateX, "y":3=CoordinateY, "format_mode":4=Mode],
    return=Integer, effects=[CreatesSprite, ReadsFile],
    purpose="Draw save-file modification time text into a save/load UI sprite slot.",
    status=Blocked, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite/IDB sub_431C70 AdvCommandSaveTimeDraw pops sprite/save/x/y/format and calls PalSpriteCreateText"]);

// Category 16 - window/effect helpers.

/// category 16 index 1: screen_shake
///
/// Purpose: Start a native screen-shake effect.  Game.exe records the shake
/// amplitudes, duration, and phase/mode into the effect state and marks the PAL
/// effect pipeline active.
///
/// VM arguments:
/// - pop[0]: x_amp (Integer) — horizontal shake amplitude.
/// - pop[1]: y_amp (Integer) — vertical shake amplitude.
/// - pop[2]: duration_ms (DurationMs) — shake duration; -1 disables/short-circuits.
/// - pop[3]: phase (Mode) — fourth native shake parameter.
///
/// Return: void.
///
/// Side effects: MutatesWindow, ChangesRunState.
///
/// Evidence:
/// - Game.sqlite / IDB: category 16 index 1 resolves to sub_4122A0.  It pops
///   four values into ctx[201085..201088], treats ctx[201087] as duration,
///   sets timing/dirty flags, calls sub_44A010, and logs "shake %d,%d,%d,%d".
/// - PAL.sqlite: PalEffectEnableIs guards the native effect pipeline.
///
/// Engine: Blocked — pal-vm now records the shake timer/state, but renderer-wide
/// viewport offset still needs native-equivalent integration.
///
/// Decompiler: Verified — renders as screen_shake(x_amp, y_amp, duration_ms,
/// phase).
static SIG_WINDOW_EFFECT_1: ExtSig = sig!(16, 1, "screen_shake", pop=4,
    params=["x_amp":0=Integer, "y_amp":1=Integer, "duration_ms":2=DurationMs, "phase":3=Mode],
    return=Void, effects=[MutatesWindow, ChangesRunState],
    purpose="Start a native screen-shake effect.",
    status=Blocked, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_4122A0 pops four shake args, stores ctx[201085..201088], and logs shake", PalSqlite:"reverse/PAL.sqlite PalEffectEnableIs 0x101195A9 guards effect availability"],
    game="0x004122A0", pal="PalEffectEnableIs" => "0x101195A9");
/// category 16 index 2: screen_flash
///
/// Purpose: Create a full-screen flash/fill overlay.  Native creates a
/// full-screen PAL sprite, paints it with RGB, records duration/start time, and
/// marks the effect pipeline active.
///
/// VM arguments:
/// - pop[0]: red (Color) — red component, 0..255.
/// - pop[1]: green (Color) — green component, 0..255.
/// - pop[2]: blue (Color) — blue component, 0..255.
/// - pop[3]: duration_ms (DurationMs) — fade/hold duration.
///
/// Return: void.
///
/// Side effects: MutatesWindow, CreatesSprite, ChangesRunState.
///
/// Evidence:
/// - Game.sqlite / IDB: category 16 index 2 resolves to sub_4120C0.  It pops
///   duration/R/G/B, creates a 1920x1080 PAL sprite on the active render target,
///   calls PalSpritePaint with the packed RGB color, records duration/start
///   time, and logs "flush 0x%06X".
/// - PAL.sqlite: PalEffectEnableIs guards the native effect pipeline.
///
/// Engine: Verified — pal-vm renders a logical full-screen colored quad with
/// the same timing, using configured logical coordinates instead of a hard-coded
/// native 1920x1080 surface.
///
/// Decompiler: Verified — renders as screen_flash(red, green, blue,
/// duration_ms).
static SIG_WINDOW_EFFECT_2: ExtSig = sig!(16, 2, "screen_flash", pop=4,
    params=["red":0=Color, "green":1=Color, "blue":2=Color, "duration_ms":3=DurationMs],
    return=Void, effects=[MutatesWindow, CreatesSprite, ChangesRunState],
    purpose="Create a full-screen PAL flash/fill overlay.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_4120C0 creates full-screen sprite, PalSpritePaints RGB, stores duration/start time, and logs flush", PalSqlite:"reverse/PAL.sqlite PalEffectEnableIs 0x101195A9 guards effect availability"],
    game="0x004120C0", pal="PalEffectEnableIs" => "0x101195A9");

// Category 20 - random.

/// category 20 index 0: random
///
/// Purpose: Return a uniformly distributed random integer in the inclusive
/// range [min, max].
///
/// VM arguments:
/// - pop[0]: min (Integer) — lower bound (inclusive).
/// - pop[1]: max (Integer) — upper bound (inclusive).
///
/// Return: Integer — random value in [min, max].
///
/// Side effects: none.
///
/// Evidence:
/// - Game.sqlite: reverse/Game.sqlite random wrapper
/// - RuntimeTrace: pal-vm dispatch_random_ext index 0 pops 2
///
/// Engine: Blocked — uses Rust rand to generate inclusive range result.
///
/// Decompiler: Blocked — renders as random(min, max).
static SIG_RANDOM: ExtSig = sig!(20, 0, "random", pop=2,
    params=["min":0=Integer, "max":1=Integer],
    return=Integer, effects=[],
    purpose="Return an inclusive random integer between the two popped bounds.",
    status=Blocked, decompiler=Blocked,
    evidence=[GameSqlite:"reverse/Game.sqlite random wrapper", RuntimeTrace:"pal-vm dispatch_random_ext index 0 pops 2"]);

// Category 21 - script thread state.

/// category 21 index 0: create_thread
///
/// Purpose: Start the native lightweight script-thread state at a target script
/// point.  The handler writes VM thread fields rather than creating an OS
/// thread.
///
/// VM arguments:
/// - pop[0]: point_id (PointId) — target script point id.
///
/// Return: status integer, 1.
///
/// Side effects: ChangesRunState.
///
/// Evidence: Game.sqlite VmExtcall_CreateThread 0x0042E900 pops one point id,
/// sets ctx_off_thread_active/running, resolves target pc when non-zero, stores
/// ctx_off_thread_id, and returns 1.
static SIG_CREATE_THREAD: ExtSig = sig!(21, 0, "create_thread", pop=1,
    params=["point_id":0=PointId=>"target script point id"],
    return=Integer, effects=[ChangesRunState],
    purpose="Start native script-thread scheduler state at a point id.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite VmExtcall_CreateThread 0x0042E900 pops point id and writes thread active/running/target/id fields"],
    game="0x0042E900");
/// category 21 index 1: exit_thread
///
/// Purpose: Clear native script-thread state.  No stack arguments are consumed.
///
/// Return: status integer, 1.
///
/// Side effects: ChangesRunState.
///
/// Evidence: Game.sqlite VmExtcall_ExitThread 0x0042E8C0 clears the thread
/// state OWORD and thread id.
static SIG_EXIT_THREAD: ExtSig = sig!(21, 1, "exit_thread", pop=0,
    params=[],
    return=Integer, effects=[ChangesRunState],
    purpose="Clear native script-thread scheduler state.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite VmExtcall_ExitThread 0x0042E8C0 clears thread state and returns 1"],
    game="0x0042E8C0");
/// category 21 index 2: suspend_thread
///
/// Purpose: Replace the native thread-running flag and return its previous
/// value.  Scripts use this around menu/title flow to restore scheduler state.
///
/// VM arguments:
/// - pop[0]: running (Flag) — new thread-running state.
///
/// Return: integer previous running flag.
///
/// Side effects: WritesVmMemory, ChangesRunState.
///
/// Evidence: Game.sqlite VmExtcall_SuspendThread 0x0042E860 pops one value,
/// writes the old ctx_off_thread_running to the return slot, then stores the
/// popped value as the new running state.
static SIG_SUSPEND_THREAD: ExtSig = sig!(21, 2, "suspend_thread", pop=1,
    params=["running":0=Flag=>"new native thread-running flag"],
    return=Integer, effects=[WritesVmMemory, ChangesRunState],
    purpose="Set native script-thread running flag and return the previous value.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite VmExtcall_SuspendThread 0x0042E860 pops one running flag, returns old running flag, stores new value"],
    game="0x0042E860");
/// category 21 index 3: get_thread
///
/// Purpose: Return the current native script-thread id/point id.
///
/// VM arguments: none.
///
/// Return: integer current thread id.
///
/// Side effects: WritesVmMemory.
///
/// Evidence: Game.sqlite VmExtcall_GetThread 0x0042E9A0 writes
/// ctx_off_thread_id to the extcall destination.
static SIG_GET_THREAD: ExtSig = sig!(21, 3, "get_thread", pop=0,
    params=[],
    return=Integer, effects=[WritesVmMemory],
    purpose="Return current native script-thread id.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite VmExtcall_GetThread 0x0042E9A0 returns ctx_off_thread_id"],
    game="0x0042E9A0");

// Category 22 - run/effect transition state.

/// category 22 index 0: run
///
/// Purpose: Execute a scene/effect transition.  Game.exe records the run
/// duration in VM timing fields; the PAL API call itself is non-waiting.
///
/// VM arguments (display order: effect_id, arg1, arg2):
/// - pop[0]: effect_id (Integer) — transition effect identifier.
/// - pop[1]: duration_ms (DurationMs) — transition/effect duration.
/// - pop[2]: effect_arg (Integer) — effect-specific argument.
///
/// Return: void.
///
/// Side effects: ChangesRunState, BlocksScript.
///
/// Evidence:
/// - Game.sqlite: sub_421FD0 pops effect_id/duration/effect_arg, validates
///   effect_id<=100, calls PalEffectEx(effect_id,duration,effect_arg,0),
///   updates run timing fields, and returns 1 (0 only on invalid/blocked native setup).
/// - PAL.sqlite: PalEffectEx 0x1011A1E8.
/// - RuntimeTrace: pal-vm dispatch_run_ext index 0 pops 3 and returns 1 after
///   setting the compatible transition pipeline.
///
/// Engine: Verified — stores the effect triple in the compatible run pipeline,
/// passes PAL wait=0, uses the recorded duration for the VM-side wait/yield, and
/// mirrors native queued run-stack mode.
///
/// Decompiler: Verified — renders as run(effect_id, arg1, arg2).
static SIG_RUN: ExtSig = sig!(22, 0, "run", pop=3,
    params=["effect_id":0=Integer, "duration_ms":1=DurationMs, "effect_arg":2=Integer],
    return=Void, effects=[ChangesRunState, BlocksScript], purpose="Run a scene/effect transition and wait for it.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_421FD0 pops effect_id/duration/effect_arg and calls PalEffectEx(effect_id,duration,effect_arg,0)", PalSqlite:"PalEffectEx 0x1011A1E8", RuntimeTrace:"pal-vm dispatch_run_ext index 0 pops 3 and updates run pipeline"],
    game="0x00421FD0", pal="PalEffectEx" => "0x1011A1E8");
/// category 22 index 1: run_no_wait
///
/// Purpose: Latch or flush a no-wait scene transition.  Does not pop any
/// transition parameters; the transition parameters are expected to have been
/// configured by a prior extcall.
///
/// VM arguments: none.
///
/// Return: void.
///
/// Side effects: ChangesRunState.
///
/// Evidence:
/// - Game.sqlite: sub_421F80 has no VM stack pop, toggles the native no-wait
///   transition latch, and either queues run-stack state or calls the run setup
///   helper.
/// - RuntimeTrace: pal-vm dispatch_run_ext index 1 pops 0 and latches the
///   compatible no-wait transition.
///
/// Engine: Verified — marks the compatible transition as no-wait without
/// consuming arguments.
///
/// Decompiler: Verified — renders as run_no_wait().
static SIG_RUN_NO_WAIT: ExtSig = sig!(22, 1, "run_no_wait", pop=0,
    params=[],
    return=Void, effects=[ChangesRunState], purpose="Latch/flush a no-wait scene transition; Game.exe category 22 index 1 does not pop transition parameters.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_421F80 has no VM pop and toggles run_no_wait transition state", RuntimeTrace:"pal-vm dispatch_run_ext index 1 pops 0 and sets no_wait_latch"],
    game="0x00421F80");
/// category 22 index 2: run_stack
///
/// Purpose: Toggle Game.exe run-stack transition mode.  When enabled, the
/// engine stacks effect layers rather than replacing them.
///
/// VM arguments:
/// - pop[0]: enabled (Flag) — 1 to enable stack mode, 0 to disable.
///
/// Return: void.
///
/// Side effects: ChangesRunState.
///
/// Evidence:
/// - Game.sqlite: sub_422160 pops enabled, toggles native run-stack mode, and
///   flushes a queued run transition through PalEffectEx when disabling.
/// - PAL.sqlite: PalEffectEx 0x1011A1E8.
/// - RuntimeTrace: pal-vm dispatch_run_ext index 2 pops one flag and flushes
///   the compatible pending run pipeline.
///
/// Engine: Verified — toggles compatible stack mode and flushes queued
/// transition state according to native sub_422160.
///
/// Decompiler: Verified — renders as run_stack(enabled).
static SIG_RUN_STACK: ExtSig = sig!(22, 2, "run_stack", pop=1,
    params=["enabled":0=Flag], return=Void, effects=[ChangesRunState], purpose="Toggle Game.exe run-stack transition mode.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_422160 pops enabled, toggles run-stack, and conditionally calls PalEffectEx for queued effects", PalSqlite:"PalEffectEx 0x1011A1E8", RuntimeTrace:"pal-vm dispatch_run_ext index 2 pops enabled and flushes pending transition"],
    game="0x00422160", pal="PalEffectEx" => "0x1011A1E8");

/// Remaining reachable semantic promotions from the latest IDB extcall table.
static SIG_TEXT_TASK_FREE: ExtSig = sig!(2, 26, "text_task_free", pop=0,
    params=[], return=Integer, effects=[ChangesTextState],
    purpose="Free native text reveal/task state.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite VmExtcall_TextTaskFree 0x0043E550 pops no args and frees text task"],
    game="0x0043E550");
static SIG_TEXT_SPRITE_LOCK: ExtSig = sig!(2, 27, "text_sprite_lock", pop=0,
    params=[], return=Integer, effects=[ChangesTextState, MutatesSprite],
    purpose="Lock native text sprite and clear alpha bytes.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite VmExtcall_TextSpriteLock 0x0043E4A0 pops no args and locks/edits text sprite pixels"],
    game="0x0043E4A0");
static SIG_TEXT_TASK_REDRAW_FLAG: ExtSig = sig!(2, 13, "text_task_redraw_flag", pop=0,
    params=[], return=Integer, effects=[ChangesTextState],
    purpose="Set the native ADV text redraw flag at text context +4192.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_43EE00 consumes no args and writes text_ctx+4192 = 1"],
    game="0x0043EE00");
static SIG_INPUT_MOUSE_TO_MEM: ExtSig = sig!(9, 12, "input_mouse_to_mem", pop=2,
    params=["dst_x":0=BufferPointer=>"Mem.dat destination for mouse x or -1", "dst_y":1=BufferPointer=>"Mem.dat destination for mouse y or -1"],
    return=Integer, effects=[WritesVmMemory],
    purpose="Write mouse x/y into Mem.dat destinations.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_438770 pops dst_x/dst_y and writes PalInputGetMouseX/Y to Mem.dat"],
    game="0x00438770");
static SIG_MEMORY_BANK_CLEAR: ExtSig = sig!(9, 14, "memory_bank_clear", pop=0,
    params=[], return=Integer, effects=[WritesVmMemory],
    purpose="Clear native VM scratch memory bank.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_438460 memset(ctx+711860,0,0x1000) and pops no args"],
    game="0x00438460");
static SIG_CANCEL_SCENE_SKIP: ExtSig = sig!(9, 52, "cancel_scene_skip", pop=0,
    params=[], return=Integer, effects=[ChangesRunState],
    purpose="Cancel native scene-skip latch and update save-point state.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_437150 consumes no args and clears ctx[201064] when scene skip is active"],
    game="0x00437150");
static SIG_SYSTEM_SCRATCH_CLEAR: ExtSig = sig!(9, 25, "system_scratch_clear", pop=0,
    params=[], return=Integer, effects=[WritesVmMemory],
    purpose="Clear Game.exe ctx+804244 system/list scratch latch.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_437BA0 consumes no args and writes ctx+804244 = 0"],
    game="0x00437BA0");
static SIG_SYSTEM_SCRATCH_GET: ExtSig = sig!(9, 26, "system_scratch_get", pop=0,
    params=[], return=Integer, effects=[WritesVmMemory],
    purpose="Return Game.exe ctx+804244 system/list scratch latch.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_437B80 consumes no args and writes ctx+804244 to the extcall destination"],
    game="0x00437B80");
static SIG_HISTORY_SCROLL: ExtSig = sig!(14, 3, "history_scroll", pop=0,
    params=[], return=Integer, effects=[WritesVmMemory],
    purpose="Return native history scroll/current offset.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite VmExtcall_HistScroll 0x0042BA80 returns ctx+88444"],
    game="0x0042BA80");
static SIG_HISTORY_UPDATE: ExtSig = sig!(14, 6, "history_update", pop=0,
    params=[], return=Integer, effects=[ChangesHistoryState],
    purpose="Update interactive history/backlog hover/voice state.",
    status=Blocked, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite VmExtcall_HistUpdate 0x0042B7C0 pops no args and updates history sprite hover/voice playback"],
    game="0x0042B7C0");
static SIG_STR_APPEND: ExtSig = sig!(18, 4, "strcatf_string", pop=2,
    params=["dst":0=BufferPointer=>"dynamic string destination", "src":1=TextStringFromTextDat=>"string to append"],
    return=Integer, effects=[WritesVmMemory],
    purpose="Append resolved string to dynamic string buffer.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_41A850 pops dst/src and appends resolved string to dynamic buffer"],
    game="0x0041A850");
static SIG_STR_COPY_LEN: ExtSig = sig!(18, 7, "strcpyf_string", pop=3,
    params=["dst":0=BufferPointer=>"dynamic string destination", "src":1=TextStringFromTextDat=>"source string", "len":2=Integer=>"copy length or -1 for full"],
    return=Integer, effects=[WritesVmMemory],
    purpose="Copy resolved string to dynamic string buffer with optional length.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_41A4B0 pops dst/src/len and strncpy()s into dynamic buffer"],
    game="0x0041A4B0");
static SIG_SYSTEM_TASK_VALUE: ExtSig = sig!(18, 15, "system_task_value", pop=0,
    params=[], return=Integer, effects=[WritesVmMemory],
    purpose="Return native task-data value when the system latch is active.",
    status=Blocked, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_417BD0 pops no args and returns PalTaskGetTaskData(0)+820 when dword_4BD0C0 is set"],
    game="0x00417BD0");
/// category 18 index 17: work_process_value
///
/// Purpose: Query/update the native background work-process value used by
/// startup disc/system checks.
///
/// VM arguments:
/// - pop[0]: arg_base_value (Integer) — value passed through the current
///   argument-base wrapper.
///
/// Return: integer — native helper result.
///
/// Side effects: WritesVmMemory, ChangesRunState.
///
/// Evidence:
/// - Game.sqlite: sub_417AF0 calls sub_44A6C0(&value), writes the result to the
///   extcall return slot, and sets ctx+804084 dirty.
/// - Disassembly: docs/dis.txt 0002CE0C pushes arg_base+64 before
///   ext_0012_0011.
///
/// Engine: Blocked — exact sub_44A6C0 internals are not fully modeled; runtime
/// preserves argument/result flow with the current portable work-process state.
///
/// Decompiler: Verified — renders work_process_value(arg_base_value), not raw
/// ext_0012_0011.
static SIG_WORK_PROCESS_VALUE: ExtSig = sig!(18, 17, "work_process_value", pop=1,
    params=["arg_base_value":0=Integer=>"argument-base value passed to native helper"],
    return=Integer, effects=[WritesVmMemory, ChangesRunState],
    purpose="Query/update native background work-process value.",
    status=Blocked, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_417AF0 calls sub_44A6C0(&value), returns value, and sets +804084", Disassembly:"docs/dis.txt 0002CE0C pushes arg_base+64 before ext_0012_0011"],
    game="0x00417AF0");
static SIG_STR_REPLACE: ExtSig = sig!(18, 18, "strreplace_string", pop=3,
    params=["dst":0=BufferPointer=>"dynamic string destination", "needle":1=TextStringFromTextDat=>"substring to replace", "replacement":2=TextStringFromTextDat=>"replacement string"],
    return=Integer, effects=[WritesVmMemory],
    purpose="Replace first substring occurrence in dynamic string buffer.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_41A330 pops dst/needle/replacement and memmove()s replacement into the dynamic string"],
    game="0x0041A330");
static SIG_ACCESS_CLEAR: ExtSig = sig!(18, 40, "access_clear", pop=1,
    params=["entry":0=Integer=>"access flag id or resource string"],
    return=Integer, effects=[WritesVmMemory],
    purpose="Clear native access/read flag entry.",
    status=Blocked, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_417E50 pops one id/string and clears the corresponding access flag"],
    game="0x00417E50");
/// Totsulover's zero-argument PAL clock query. Its wait helper at Script.src
/// 0xE0A40/0xE0ACC subtracts two results and compares the elapsed milliseconds
/// against the requested duration. The native handler address is still unknown.
static SIG_PAL_TIME_MS: ExtSig = sig!(18, 121, "pal_time_ms", pop=0,
    params=[], return=Integer, effects=[],
    purpose="Read the monotonically advancing PAL millisecond clock.",
    status=Partial, decompiler=Partial,
    evidence=[Disassembly:"Totsulover Script.src 0xE0A40 and 0xE0ACC elapsed-time loop", RuntimeTrace:"Totsulover title start callback point[13] stalls when this call returns zero"]);
/// Seven default sentinels and a resource id precede this Totsulover call.
/// Leaving them on the value stack makes the next `pop arg_base` restore a
/// string sentinel as a memory base, corrupting later button coordinates.
static SIG_EXT_18_90: ExtSig = sig!(18, 90, "ext_18_90", pop=8,
    params=["arg0":0=Unknown, "arg1":1=Unknown, "arg2":2=Unknown, "arg3":3=Unknown,
            "arg4":4=Unknown, "arg5":5=Unknown, "arg6":6=Unknown, "arg7":7=Unknown],
    return=Status, effects=[UnknownSideEffect],
    purpose="Preserve the eight-argument extension's stack contract.",
    status=StackDisciplineOnly, decompiler=Partial,
    evidence=[Disassembly:"Totsulover Script.src 0x1021E4-0x102224 pushes eight arguments", RuntimeTrace:"Unconsumed values cause argument_base=0x1000003F during confirmation button layout"]);
static SIG_ACTION_TIMELINE_17_REAL: ExtSig = sig!(17, 17, "action_timeline_17", pop=5,
    params=["line":0=Integer, "sprite":1=SpriteSlot, "arg2":2=Integer, "arg3":3=Integer, "duration":4=DurationMs],
    return=Integer, effects=[CreatesTask, MutatesSprite],
    purpose="Append native action timeline entry type 10.",
    status=Verified, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite VmExtcall_ActionTimeline17 0x0040A650 pops line plus four action args and stores type 10"],
    game="0x0040A650");
static SIG_EFFECT_STOP_SKIP: ExtSig = sig!(16, 0, "effect_stop_skip", pop=0,
    params=[], return=Integer, effects=[ChangesRunState, MutatesWindow],
    purpose="Stop native transition/effect channels during skip/menu teardown.",
    status=Blocked, decompiler=Verified,
    evidence=[GameSqlite:"reverse/Game.sqlite sub_412450 reads two native arg-stack fields (flags,duration) and stops selected effect channels; reachable script wrapper is zero-arg effect_stop_skip()"],
    game="0x00412450");

pub fn lookup_sig(category: u16, index: u16) -> Option<&'static ExtSig> {
    match (category, index) {
        (2, 2) => Some(&SIG_TEXT),
        (2, 3) => Some(&SIG_TEXT_HIDE),
        (2, 4) => Some(&SIG_TEXT_SHOW),
        (2, 5) => Some(&SIG_TEXT_SET_BTN),
        (2, 8) => Some(&SIG_TEXT_CLEAR),
        (2, 9) => Some(&SIG_TEXT_CLEAR_EX),
        (2, 10) => Some(&SIG_TEXT_GET_TIME),
        (2, 13) => Some(&SIG_TEXT_TASK_REDRAW_FLAG),
        (2, 14) => Some(&SIG_TEXT_SET_ICON_ANIMATION_TIME),
        (2, 15) => Some(&SIG_TEXT_W),
        (2, 16) => Some(&SIG_TEXT_A),
        (2, 17) => Some(&SIG_TEXT_WA),
        (2, 22) => Some(&SIG_TEXT_SET_BASE),
        (2, 25) => Some(&SIG_TEXT_TIME_CHECK_SET),
        (2, 26) => Some(&SIG_TEXT_TASK_FREE),
        (2, 27) => Some(&SIG_TEXT_SPRITE_LOCK),
        (2, 28) => Some(&SIG_TEXT_SET_COLOR),
        (3, 2) => Some(&SIG_SP_SET),
        (3, 3) => Some(&SIG_SP_SET_EX),
        (3, 4) => Some(&SIG_SP_SET_POS),
        (3, 5) => Some(&SIG_SP_CLS),
        (3, 6) => Some(&SIG_SP_SET_ALPHA),
        (3, 7) => Some(&SIG_SP_SET_PRIORITY_LANE),
        (3, 8) => Some(&SIG_SP_GET_FILENAME),
        (3, 9) => Some(&SIG_SP_SET_CENTER),
        (3, 11) => Some(&SIG_SP_CLS_EX),
        (3, 12) => Some(&SIG_SP_SET_FILTER),
        (3, 16) => Some(&SIG_SP_SET_RENDER_MODE),
        (3, 15) => Some(&SIG_SP_SET_RECT_POS),
        (3, 17) => Some(&SIG_SP_SET_SCALE),
        (3, 18) => Some(&SIG_SP_SET_ROTATE),
        (3, 19) => Some(&SIG_FACE_INIT),
        (3, 20) => Some(&SIG_FACE_SET),
        (3, 21) => Some(&SIG_SP_GET_COLOR),
        (3, 22) => Some(&SIG_SPTEXT),
        (3, 23) => Some(&SIG_FACE_CLS),
        (3, 24) => Some(&SIG_SP_SET_RECT),
        (3, 25) => Some(&SIG_SP_SET_POS_MOVE),
        (3, 26) => Some(&SIG_SP_GET_ALPHA),
        (3, 27) => Some(&SIG_SP_GET_ROTATE),
        (3, 28) => Some(&SIG_SP_GET_POS_TO_MEM),
        (3, 29) => Some(&SIG_SP_GET_WIDTH),
        (3, 30) => Some(&SIG_SP_GET_HEIGHT),
        (3, 31) => Some(&SIG_SP_SET_ANIM2),
        (3, 34) => Some(&SIG_SP_SET_ANIM_PARAM),
        (3, 35) => Some(&SIG_SP_GET_ANIM_PARAM),
        (3, 36) => Some(&SIG_SP_GET_SCALE),
        (3, 37) => Some(&SIG_SP_SET_COLOR_SPRITE),
        (3, 39) => Some(&SIG_SP_SET_SHAKE),
        (3, 41) => Some(&SIG_SP_SET_ANIM_41),
        (3, 44) => Some(&SIG_SP_SET_VIS_CLIP),
        (3, 46) => Some(&SIG_SP_SHOW),
        (3, 47) => Some(&SIG_SP_HIDE),
        (3, 48) => Some(&SIG_IS_SP),
        (3, 49) => Some(&SIG_SP_SET_CHILD),
        (3, 50) => Some(&SIG_SP_SET_TRANSITION_HOT),
        (3, 51) => Some(&SIG_SP_COPY_IMAGE_HOT),
        (3, 52) => Some(&SIG_SP_TRANSITION_HOT),
        (3, 53) => Some(&SIG_SP_SET_ASPECT_POSITION_TYPE),
        (3, 54) => Some(&SIG_SP_GET_BACKBUFFER),
        (3, 55) => Some(&SIG_SP_SET_MOTION),
        (3, 56) => Some(&SIG_SP_SET_MOTION_POS),
        (3, 57) => Some(&SIG_SP_SET_ANIM_57),
        (4, 0) => Some(&SIG_BGM_PLAY),
        (4, 1) => Some(&SIG_BGM_STOP),
        (4, 2) => Some(&SIG_BGM_SET_VOLUME),
        (4, 3) => Some(&SIG_BGM_GET_VOLUME),
        (4, 6) => Some(&SIG_BGM_SET_AUTO_VOLUME),
        (4, 9) => Some(&SIG_BGM_LOAD),
        (4, 11) => Some(&SIG_SET_MASTER_VOLUME),
        (4, 13) => Some(&SIG_MUTE_MASTER_VOLUME),
        (4, 14) => Some(&SIG_BGM_MUTE),
        (4, 15) => Some(&SIG_MUTE_BGM_AUTO_VOLUME),
        (5, 0) => Some(&SIG_SE_LOAD),
        (5, 1) => Some(&SIG_SE_PLAY),
        (5, 2) => Some(&SIG_SE_PLAY_EX),
        (5, 3) => Some(&SIG_SE_STOP),
        (5, 4) => Some(&SIG_SE_SET_VOLUME),
        (5, 5) => Some(&SIG_SE_GET_VOLUME),
        (5, 14) => Some(&SIG_SE_MUTE),
        (7, 0) => Some(&SIG_WAIT),
        (7, 1) => Some(&SIG_WAIT_CLICK),
        (7, 2) => Some(&SIG_WAIT_SYNC_BEGIN),
        (7, 3) => Some(&SIG_WAIT_SYNC_RELEASE),
        (7, 4) => Some(&SIG_WAIT_SYNC_END),
        (7, 5) => Some(&SIG_WAIT_SYNC_STEP),
        (7, 7) => Some(&SIG_WAIT_CLICK_NO_ANIM),
        (7, 8) => Some(&SIG_WAIT_SYNC_GET_TIME),
        (7, 9) => Some(&SIG_WAIT_TIME_PUSH),
        (7, 10) => Some(&SIG_WAIT_TIME_POP),
        (6, 0) => Some(&SIG_SELECT_INIT),
        (6, 2) => Some(&SIG_SELECT_SET),
        (6, 3) => Some(&SIG_SELECT_COMMIT),
        (6, 4) => Some(&SIG_SELECT_CLEAR),
        (8, 0) => Some(&SIG_BTN_INIT),
        (8, 1) => Some(&SIG_BTN_UNINIT),
        (8, 3) => Some(&SIG_BTN_SET),
        (8, 4) => Some(&SIG_BTN_HIDE),
        (8, 5) => Some(&SIG_BTN_SHOW),
        (8, 6) => Some(&SIG_BTN_SET_POS),
        (8, 8) => Some(&SIG_BTN_RELEASE),
        (8, 9) => Some(&SIG_BTN_SLIDER_GET),
        (8, 10) => Some(&SIG_BTN_SLIDER_SET),
        (8, 11) => Some(&SIG_BTN_SLIDER_BEGIN),
        (8, 12) => Some(&SIG_BTN_ON_CHECK),
        (8, 13) => Some(&SIG_BTN_SET_TOGGLE),
        (8, 14) => Some(&SIG_BTN_GET_POS),
        (8, 15) => Some(&SIG_BTN_ENABLE),
        (8, 16) => Some(&SIG_BTN_SET_ALPHA),
        (8, 17) => Some(&SIG_BTN_GET_PUSH),
        (8, 19) => Some(&SIG_BTN_LOCK),
        (8, 20) => Some(&SIG_BTN_UNLOCK),
        (8, 21) => Some(&SIG_BTN_SET_ANIM),
        (8, 22) => Some(&SIG_BTN_SET_HIT),
        (8, 23) => Some(&SIG_BTN_GET_ONMOUSE),
        (8, 29) => Some(&SIG_BTN_SET_STATE),
        (8, 37) => Some(&SIG_BTN_GET_ALPHA),
        (5, 8) => Some(&SIG_CHANNEL_ERROR_SET_SE_INFO),
        (9, 0) => Some(&SIG_SKIP_SET),
        (9, 1) => Some(&SIG_SKIP_IS),
        (9, 2) => Some(&SIG_AUTO_SET),
        (9, 3) => Some(&SIG_AUTO_IS),
        (9, 4) => Some(&SIG_AUTO_SET_SPEED),
        (9, 6) => Some(&SIG_WINDOW_CHANGE_MODE),
        (9, 7) => Some(&SIG_WINDOW_SET_MODE_CACHE),
        (9, 8) => Some(&SIG_EFFECT_ENABLE),
        (9, 9) => Some(&SIG_EFFECT_ENABLE_IS),
        (9, 10) => Some(&SIG_WINDOW_GET_MODE_CACHE),
        (9, 12) => Some(&SIG_INPUT_MOUSE_TO_MEM),
        (9, 14) => Some(&SIG_MEMORY_BANK_CLEAR),
        (9, 17) => Some(&SIG_SET_LANGUAGE),
        (9, 18) => Some(&SIG_INPUT_KEY_CANCEL),
        (9, 19) => Some(&SIG_SET_FONT_COLOR),
        (9, 20) => Some(&SIG_LOAD_FONT_EX),
        (9, 21) => Some(&SIG_MEMORY_STACK_PUSH),
        (9, 22) => Some(&SIG_MEMORY_STACK_POP),
        (9, 23) => Some(&SIG_LIST_STACK_PUSH_POINT),
        (9, 24) => Some(&SIG_LIST_STACK_POP_COUNT),
        (9, 25) => Some(&SIG_SYSTEM_SCRATCH_CLEAR),
        (9, 26) => Some(&SIG_SYSTEM_SCRATCH_GET),
        (9, 27) => Some(&SIG_SET_FONT_SIZE),
        (9, 28) => Some(&SIG_GET_FONT_SIZE),
        (9, 29) => Some(&SIG_GET_FONT_TYPE),
        (9, 30) => Some(&SIG_SET_FONT_EFFECT),
        (9, 31) => Some(&SIG_GET_FONT_EFFECT),
        (9, 52) => Some(&SIG_CANCEL_SCENE_SKIP),
        (9, 53) => Some(&SIG_LIST_STACK_GET_COUNT),
        (15, 4) => Some(&SIG_SYSTEM_WINDOW_OVERLAY_SET),
        (15, 5) => Some(&SIG_DEBUG_WINDOW_SET),
        (12, 0) => Some(&SIG_SYSTEM_BTN_SET),
        (12, 1) => Some(&SIG_SYSTEM_BTN_RELEASE),
        (12, 2) => Some(&SIG_SYSTEM_BTN_ENABLE),
        (11, 0) => Some(&SIG_MOVIE_PLAY),
        (11, 1) => Some(&SIG_MSP_SET_LOOP_SP_EP),
        (11, 2) => Some(&SIG_MSP_CLS),
        (11, 3) => Some(&SIG_MSP_WAIT),
        (11, 4) => Some(&SIG_MSP_LOCK),
        (11, 5) => Some(&SIG_MSP_UNLOCK),
        (13, 0) => Some(&SIG_VOICE_PLAY),
        (13, 1) => Some(&SIG_VOICE_STOP),
        (13, 2) => Some(&SIG_VOICE_SET_VOLUME),
        (13, 3) => Some(&SIG_VOICE_GET_VOLUME),
        (13, 7) => Some(&SIG_VOICE_PLAY_FADE),
        (13, 16) => Some(&SIG_VOICE_AUTOPAN_SIZE_OVER),
        (13, 18) => Some(&SIG_VOICE_WAIT),
        (10, 12) => Some(&SIG_SAVE_THUMBNAIL_MOSAIC_SET),
        (10, 13) => Some(&SIG_SAVE_TIME_DRAW),
        (14, 3) => Some(&SIG_HISTORY_SCROLL),
        (14, 6) => Some(&SIG_HISTORY_UPDATE),
        (16, 0) => Some(&SIG_EFFECT_STOP_SKIP),
        (16, 1) => Some(&SIG_WINDOW_EFFECT_1),
        (16, 2) => Some(&SIG_WINDOW_EFFECT_2),
        (17, 0) => Some(&SIG_ACTION_RUN_COUNT_OVER),
        (17, 1) => Some(&SIG_ACTION_SYNC_RUN_COUNT_OVER),
        (17, 3) => Some(&SIG_ACTION_CLEAR_COUNT_OVER),
        (17, 5) => Some(&SIG_ACTION_TIMELINE_5),
        (17, 6) => Some(&SIG_ACTION_TIMELINE_6),
        (17, 7) => Some(&SIG_ACTION_TIMELINE_7),
        (17, 8) => Some(&SIG_ACTION_TIMELINE_8),
        (17, 9) => Some(&SIG_ACTION_TIMELINE_9),
        (17, 10) => Some(&SIG_ACTION_TIMELINE_10),
        (17, 11) => Some(&SIG_ACTION_TIMELINE_11),
        (17, 14) => Some(&SIG_ACTION_TIMELINE_14),
        (17, 15) => Some(&SIG_ACTION_TIMELINE_15),
        (17, 17) => Some(&SIG_ACTION_TIMELINE_17_REAL),
        (17, 20) => Some(&SIG_ACTION_TIMELINE_20),
        (17, 21) => Some(&SIG_ACTION_PUSH),
        (17, 22) => Some(&SIG_ACTION_POP),
        (17, 23) => Some(&SIG_ACTION_SET_ACTIVE),
        (17, 29) => Some(&SIG_ACTION_TIMELINE_29),
        (17, 30) => Some(&SIG_ACTION_SET_CLEAR),
        (18, 1) => Some(&SIG_APP_EXEC),
        (18, 3) => Some(&SIG_STRING_NOT_EQUAL),
        (18, 4) => Some(&SIG_STR_APPEND),
        (18, 5) => Some(&SIG_STRGETCF),
        (18, 6) => Some(&SIG_ARG_GET),
        (18, 7) => Some(&SIG_STR_COPY_LEN),
        (18, 8) => Some(&SIG_FILE_EXIST),
        (18, 9) => Some(&SIG_WSPRINT),
        (18, 10) => Some(&SIG_CHECK_DISC),
        (18, 12) => Some(&SIG_STRING_LENGTH),
        (18, 13) => Some(&SIG_PROCESS_CHECKPOINT_SET),
        (18, 14) => Some(&SIG_UPDATE_ACCESS),
        (18, 15) => Some(&SIG_SYSTEM_TASK_VALUE),
        (18, 17) => Some(&SIG_WORK_PROCESS_VALUE),
        (18, 18) => Some(&SIG_STR_REPLACE),
        (18, 21) => Some(&SIG_STRING_NON_EMPTY),
        (18, 28) => Some(&SIG_ATTACH_WORK_PROCESS),
        (18, 29) => Some(&SIG_DETACH_WORK_PROCESS),
        (18, 30) => Some(&SIG_OPENFILE),
        (18, 31) => Some(&SIG_READ_FILE),
        (18, 32) => Some(&SIG_CLOSE_FILE_NOT_HANDLE),
        (18, 33) => Some(&SIG_SET_FILE_POINTER),
        (18, 34) => Some(&SIG_FILE_STRING),
        (18, 35) => Some(&SIG_SET_LAST_PROCESS),
        (18, 36) => Some(&SIG_SZ_BUF),
        (18, 37) => Some(&SIG_GETPRIVATEPROFILEINT),
        (18, 40) => Some(&SIG_ACCESS_CLEAR),
        (18, 90) => Some(&SIG_EXT_18_90),
        (18, 121) => Some(&SIG_PAL_TIME_MS),
        (20, 0) => Some(&SIG_RANDOM),
        (21, 0) => Some(&SIG_CREATE_THREAD),
        (21, 1) => Some(&SIG_EXIT_THREAD),
        (21, 2) => Some(&SIG_SUSPEND_THREAD),
        (21, 3) => Some(&SIG_GET_THREAD),
        (22, 0) => Some(&SIG_RUN),
        (22, 1) => Some(&SIG_RUN_NO_WAIT),
        (22, 2) => Some(&SIG_RUN_STACK),
        _ => extsig_auto::lookup_auto_sig(category, index),
    }
}

static ALL_SIGNATURES: &[&ExtSig] = &[
    &SIG_TEXT,
    &SIG_TEXT_HIDE,
    &SIG_TEXT_SHOW,
    &SIG_TEXT_SET_BTN,
    &SIG_TEXT_CLEAR,
    &SIG_TEXT_CLEAR_EX,
    &SIG_TEXT_GET_TIME,
    &SIG_TEXT_TASK_REDRAW_FLAG,
    &SIG_TEXT_SET_ICON_ANIMATION_TIME,
    &SIG_TEXT_W,
    &SIG_TEXT_A,
    &SIG_TEXT_WA,
    &SIG_TEXT_SET_BASE,
    &SIG_TEXT_TIME_CHECK_SET,
    &SIG_TEXT_TASK_FREE,
    &SIG_TEXT_SPRITE_LOCK,
    &SIG_TEXT_SET_COLOR,
    &SIG_SP_SET,
    &SIG_SP_SET_EX,
    &SIG_SP_SET_POS,
    &SIG_SP_CLS,
    &SIG_SP_SET_ALPHA,
    &SIG_SP_SET_PRIORITY_LANE,
    &SIG_SP_GET_FILENAME,
    &SIG_SP_SET_CENTER,
    &SIG_SP_CLS_EX,
    &SIG_SP_SET_FILTER,
    &SIG_SP_SET_RENDER_MODE,
    &SIG_SP_SET_RECT_POS,
    &SIG_SP_SET_SCALE,
    &SIG_SP_SET_ROTATE,
    &SIG_FACE_INIT,
    &SIG_FACE_SET,
    &SIG_SP_GET_COLOR,
    &SIG_SPTEXT,
    &SIG_FACE_CLS,
    &SIG_SP_SET_RECT,
    &SIG_SP_SET_POS_MOVE,
    &SIG_SP_GET_ALPHA,
    &SIG_SP_GET_ROTATE,
    &SIG_SP_GET_POS_TO_MEM,
    &SIG_SP_GET_WIDTH,
    &SIG_SP_GET_HEIGHT,
    &SIG_SP_SET_ANIM2,
    &SIG_SP_SET_ANIM_PARAM,
    &SIG_SP_GET_ANIM_PARAM,
    &SIG_SP_GET_SCALE,
    &SIG_SP_SET_COLOR_SPRITE,
    &SIG_SP_SET_SHAKE,
    &SIG_SP_SET_ANIM_41,
    &SIG_SP_SET_VIS_CLIP,
    &SIG_SP_SHOW,
    &SIG_SP_HIDE,
    &SIG_IS_SP,
    &SIG_SP_SET_CHILD,
    &SIG_SP_SET_TRANSITION_HOT,
    &SIG_SP_COPY_IMAGE_HOT,
    &SIG_SP_TRANSITION_HOT,
    &SIG_SP_SET_ASPECT_POSITION_TYPE,
    &SIG_SP_GET_BACKBUFFER,
    &SIG_SP_SET_MOTION,
    &SIG_SP_SET_MOTION_POS,
    &SIG_SP_SET_ANIM_57,
    &SIG_BGM_PLAY,
    &SIG_BGM_STOP,
    &SIG_BGM_SET_VOLUME,
    &SIG_BGM_GET_VOLUME,
    &SIG_BGM_SET_AUTO_VOLUME,
    &SIG_BGM_LOAD,
    &SIG_SET_MASTER_VOLUME,
    &SIG_MUTE_MASTER_VOLUME,
    &SIG_BGM_MUTE,
    &SIG_MUTE_BGM_AUTO_VOLUME,
    &SIG_SE_PLAY,
    &SIG_SE_PLAY_EX,
    &SIG_SE_LOAD,
    &SIG_SE_STOP,
    &SIG_SE_SET_VOLUME,
    &SIG_SE_GET_VOLUME,
    &SIG_SE_MUTE,
    &SIG_WAIT,
    &SIG_WAIT_CLICK,
    &SIG_WAIT_SYNC_BEGIN,
    &SIG_WAIT_SYNC_RELEASE,
    &SIG_WAIT_SYNC_END,
    &SIG_WAIT_SYNC_STEP,
    &SIG_WAIT_CLICK_NO_ANIM,
    &SIG_WAIT_SYNC_GET_TIME,
    &SIG_WAIT_TIME_PUSH,
    &SIG_WAIT_TIME_POP,
    &SIG_SELECT_INIT,
    &SIG_SELECT_SET,
    &SIG_SELECT_COMMIT,
    &SIG_SELECT_CLEAR,
    &SIG_BTN_INIT,
    &SIG_BTN_UNINIT,
    &SIG_BTN_SET,
    &SIG_BTN_HIDE,
    &SIG_BTN_SHOW,
    &SIG_BTN_SET_POS,
    &SIG_BTN_RELEASE,
    &SIG_BTN_SLIDER_GET,
    &SIG_BTN_SLIDER_SET,
    &SIG_BTN_SLIDER_BEGIN,
    &SIG_BTN_ON_CHECK,
    &SIG_BTN_SET_TOGGLE,
    &SIG_BTN_GET_POS,
    &SIG_BTN_SET_STATE,
    &SIG_BTN_ENABLE,
    &SIG_BTN_SET_ALPHA,
    &SIG_BTN_GET_PUSH,
    &SIG_BTN_LOCK,
    &SIG_BTN_UNLOCK,
    &SIG_BTN_SET_ANIM,
    &SIG_BTN_SET_HIT,
    &SIG_BTN_GET_ONMOUSE,
    &SIG_BTN_GET_ALPHA,
    &SIG_CHANNEL_ERROR_SET_SE_INFO,
    &SIG_SKIP_SET,
    &SIG_SKIP_IS,
    &SIG_AUTO_SET,
    &SIG_AUTO_IS,
    &SIG_AUTO_SET_SPEED,
    &SIG_WINDOW_CHANGE_MODE,
    &SIG_WINDOW_SET_MODE_CACHE,
    &SIG_EFFECT_ENABLE,
    &SIG_EFFECT_ENABLE_IS,
    &SIG_WINDOW_GET_MODE_CACHE,
    &SIG_INPUT_MOUSE_TO_MEM,
    &SIG_MEMORY_BANK_CLEAR,
    &SIG_SET_LANGUAGE,
    &SIG_INPUT_KEY_CANCEL,
    &SIG_SET_FONT_COLOR,
    &SIG_LOAD_FONT_EX,
    &SIG_MEMORY_STACK_PUSH,
    &SIG_MEMORY_STACK_POP,
    &SIG_LIST_STACK_PUSH_POINT,
    &SIG_LIST_STACK_POP_COUNT,
    &SIG_SYSTEM_SCRATCH_CLEAR,
    &SIG_SYSTEM_SCRATCH_GET,
    &SIG_SET_FONT_SIZE,
    &SIG_GET_FONT_SIZE,
    &SIG_GET_FONT_TYPE,
    &SIG_SET_FONT_EFFECT,
    &SIG_GET_FONT_EFFECT,
    &SIG_CANCEL_SCENE_SKIP,
    &SIG_LIST_STACK_GET_COUNT,
    &SIG_SYSTEM_WINDOW_OVERLAY_SET,
    &SIG_DEBUG_WINDOW_SET,
    &SIG_SYSTEM_BTN_SET,
    &SIG_SYSTEM_BTN_RELEASE,
    &SIG_SYSTEM_BTN_ENABLE,
    &SIG_MOVIE_PLAY,
    &SIG_MSP_SET_LOOP_SP_EP,
    &SIG_MSP_CLS,
    &SIG_MSP_WAIT,
    &SIG_MSP_LOCK,
    &SIG_MSP_UNLOCK,
    &SIG_VOICE_PLAY,
    &SIG_VOICE_STOP,
    &SIG_VOICE_SET_VOLUME,
    &SIG_VOICE_GET_VOLUME,
    &SIG_VOICE_PLAY_FADE,
    &SIG_VOICE_AUTOPAN_SIZE_OVER,
    &SIG_VOICE_WAIT,
    &SIG_SAVE_THUMBNAIL_MOSAIC_SET,
    &SIG_SAVE_TIME_DRAW,
    &SIG_HISTORY_SCROLL,
    &SIG_HISTORY_UPDATE,
    &SIG_EFFECT_STOP_SKIP,
    &SIG_WINDOW_EFFECT_1,
    &SIG_WINDOW_EFFECT_2,
    &SIG_ACTION_RUN_COUNT_OVER,
    &SIG_ACTION_SYNC_RUN_COUNT_OVER,
    &SIG_ACTION_CLEAR_COUNT_OVER,
    &SIG_ACTION_TIMELINE_5,
    &SIG_ACTION_TIMELINE_6,
    &SIG_ACTION_TIMELINE_7,
    &SIG_ACTION_TIMELINE_8,
    &SIG_ACTION_TIMELINE_9,
    &SIG_ACTION_TIMELINE_10,
    &SIG_ACTION_TIMELINE_11,
    &SIG_ACTION_TIMELINE_14,
    &SIG_ACTION_TIMELINE_15,
    &SIG_ACTION_TIMELINE_17_REAL,
    &SIG_ACTION_TIMELINE_20,
    &SIG_ACTION_PUSH,
    &SIG_ACTION_POP,
    &SIG_ACTION_SET_ACTIVE,
    &SIG_ACTION_TIMELINE_29,
    &SIG_ACTION_SET_CLEAR,
    &SIG_APP_EXEC,
    &SIG_STRING_NOT_EQUAL,
    &SIG_STR_APPEND,
    &SIG_STRGETCF,
    &SIG_ARG_GET,
    &SIG_STR_COPY_LEN,
    &SIG_FILE_EXIST,
    &SIG_WSPRINT,
    &SIG_CHECK_DISC,
    &SIG_STRING_LENGTH,
    &SIG_PROCESS_CHECKPOINT_SET,
    &SIG_UPDATE_ACCESS,
    &SIG_SYSTEM_TASK_VALUE,
    &SIG_WORK_PROCESS_VALUE,
    &SIG_STR_REPLACE,
    &SIG_STRING_NON_EMPTY,
    &SIG_ATTACH_WORK_PROCESS,
    &SIG_DETACH_WORK_PROCESS,
    &SIG_OPENFILE,
    &SIG_READ_FILE,
    &SIG_CLOSE_FILE_NOT_HANDLE,
    &SIG_SET_FILE_POINTER,
    &SIG_FILE_STRING,
    &SIG_SET_LAST_PROCESS,
    &SIG_SZ_BUF,
    &SIG_GETPRIVATEPROFILEINT,
    &SIG_ACCESS_CLEAR,
    &SIG_EXT_18_90,
    &SIG_PAL_TIME_MS,
    &SIG_RANDOM,
    &SIG_CREATE_THREAD,
    &SIG_EXIT_THREAD,
    &SIG_SUSPEND_THREAD,
    &SIG_GET_THREAD,
    &SIG_RUN,
    &SIG_RUN_NO_WAIT,
    &SIG_RUN_STACK,
];

pub fn all_signatures() -> &'static [&'static ExtSig] {
    ALL_SIGNATURES
}

pub fn auto_signatures() -> &'static [ExtSig] {
    extsig_auto::auto_signatures()
}

/// Best-effort arity observed from `docs/dis.txt`.
///
/// This is not a replacement for a real Game.exe handler pass. It is a runtime
/// safety net for extcalls that are still `Unknown`/`Stub`: the VM can at least
/// consume the same number of pushed arguments that the current script uses,
/// and the decompiler can keep displaying raw arguments instead of pretending
/// their semantics are known.
pub fn observed_pop_count(category: u16, index: u16) -> Option<usize> {
    match (category, index) {
        (2, 0) => Some(8),
        (2, 1) => Some(4),
        (2, 2) => Some(4),
        (2, 3) => Some(2),
        (2, 4) => Some(1),
        (2, 5) => Some(1),
        (2, 8) => Some(0),
        (2, 9) => Some(1),
        (2, 10) => Some(2),
        (2, 11) => Some(2),
        (2, 12) => Some(3),
        (2, 13) => Some(1),
        (2, 14) => Some(2),
        (2, 15) => Some(5),
        (2, 16) => Some(3),
        (2, 17) => Some(3),
        (2, 18) => Some(4),
        (2, 19) => Some(4),
        (2, 20) => Some(3),
        (2, 21) => Some(0),
        (2, 22) => Some(4),
        (2, 23) => Some(1),
        (2, 24) => Some(0),
        (2, 25) => Some(5),
        (2, 26) => Some(1),
        (2, 27) => Some(0),
        (2, 29) => Some(0),
        (2, 30) => Some(1),
        (2, 31) => Some(4),
        (2, 32) => Some(1),
        (3, 2) => Some(4),
        (3, 3) => Some(5),
        (3, 5) => Some(1),
        (3, 6) => Some(2),
        (3, 7) => Some(1),
        (3, 8) => Some(2),
        (3, 9) => Some(3),
        (3, 11) => Some(2),
        (3, 12) => Some(9),
        (3, 13) => Some(4),
        (3, 14) => Some(4),
        (3, 15) => Some(3),
        (3, 16) => Some(2),
        (3, 17) => Some(2),
        (3, 18) => Some(5),
        (3, 19) => Some(5),
        (3, 20) => Some(4),
        (3, 21) => Some(1),
        (3, 22) => Some(5),
        (3, 23) => Some(2),
        (3, 24) => Some(5),
        (3, 25) => Some(4),
        (3, 26) => Some(1),
        (3, 27) => Some(2),
        (3, 28) => Some(4),
        (3, 29) => Some(1),
        (3, 30) => Some(1),
        (3, 31) => Some(9),
        (3, 32) => Some(6),
        (3, 34) => Some(2),
        (3, 35) => Some(2),
        (3, 36) => Some(1),
        (3, 37) => Some(2),
        (3, 38) => Some(9),
        (3, 39) => Some(4),
        (3, 40) => Some(6),
        (3, 44) => Some(1),
        (3, 46) => Some(1),
        (3, 47) => Some(1),
        (3, 48) => Some(1),
        (3, 49) => Some(5),
        (3, 50) => Some(4),
        (3, 51) => Some(1),
        (3, 52) => Some(3),
        (3, 53) => Some(2),
        (3, 54) => Some(1),
        (3, 55) => Some(9),
        (3, 56) => Some(9),
        (3, 57) => Some(3),
        (4, 0) => Some(7),
        (4, 1) => Some(2),
        (4, 2) => Some(1),
        (4, 3) => Some(2),
        (4, 4) => Some(2),
        (4, 6) => Some(2),
        (4, 8) => Some(3),
        (4, 11) => Some(1),
        (4, 12) => Some(3),
        (4, 13) => Some(1),
        (4, 14) => Some(1),
        (4, 15) => Some(1),
        (5, 0) => Some(3),
        (5, 1) => Some(4),
        (5, 2) => Some(5),
        (5, 3) => Some(2),
        (5, 4) => Some(2),
        (5, 5) => Some(3),
        (5, 6) => Some(1),
        (5, 7) => Some(1),
        (5, 14) => Some(2),
        (6, 0) => Some(4),
        (6, 1) => Some(1),
        (6, 2) => Some(6),
        (6, 3) => Some(0),
        (6, 4) => Some(0),
        (6, 6) => Some(3),
        (7, 0) => Some(2),
        (7, 1) => Some(2),
        (7, 2) => Some(0),
        (7, 3) => Some(1),
        (7, 4) => Some(0),
        (7, 5) => Some(0),
        (7, 6) => Some(0),
        (7, 8) => Some(0),
        (7, 9) => Some(0),
        (7, 10) => Some(0),
        (8, 0) => Some(3),
        (8, 1) => Some(1),
        (8, 3) => Some(7),
        (8, 4) => Some(2),
        (8, 5) => Some(2),
        (8, 6) => Some(4),
        (8, 8) => Some(2),
        (8, 9) => Some(5),
        (8, 10) => Some(5),
        (8, 11) => Some(2),
        (8, 12) => Some(2),
        (8, 13) => Some(3),
        (8, 14) => Some(4),
        (8, 15) => Some(3),
        (8, 16) => Some(3),
        (8, 17) => Some(2),
        (8, 18) => Some(3),
        (8, 19) => Some(3),
        (8, 20) => Some(1),
        (8, 21) => Some(4),
        (8, 22) => Some(2),
        (8, 23) => Some(2),
        (8, 24) => Some(2),
        (8, 26) => Some(1),
        (9, 0) => Some(2),
        (9, 1) => Some(1),
        (9, 2) => Some(2),
        (9, 3) => Some(0),
        (9, 4) => Some(1),
        (9, 5) => Some(0),
        (9, 6) => Some(1),
        (9, 7) => Some(1),
        (9, 8) => Some(1),
        (9, 9) => Some(0),
        (9, 10) => Some(0),
        (9, 12) => Some(2),
        (9, 15) => Some(1),
        (9, 17) => Some(1),
        (9, 18) => Some(3),
        (9, 19) => Some(2),
        (9, 20) => Some(1),
        (9, 21) => Some(1),
        (9, 22) => Some(0),
        (9, 23) => Some(1),
        (9, 24) => Some(1),
        (9, 25) => Some(1),
        (9, 26) => Some(2),
        (9, 27) => Some(1),
        (9, 28) => Some(0),
        (9, 29) => Some(0),
        (9, 30) => Some(1),
        (9, 31) => Some(0),
        (9, 35) => Some(0),
        (9, 36) => Some(4),
        (9, 48) => Some(3),
        (9, 49) => Some(1),
        (9, 51) => Some(1),
        (9, 52) => Some(1),
        (9, 53) => Some(0),
        (10, 0) => Some(3),
        (10, 1) => Some(3),
        (10, 4) => Some(2),
        (10, 5) => Some(4),
        (10, 7) => Some(1),
        (10, 9) => Some(1),
        (10, 11) => Some(2),
        (10, 12) => Some(1),
        (10, 13) => Some(5),
        (10, 14) => Some(5),
        (10, 15) => Some(2),
        (10, 16) => Some(4),
        (10, 17) => Some(1),
        (10, 22) => Some(0),
        (10, 24) => Some(1),
        (10, 25) => Some(0),
        (10, 26) => Some(2),
        (10, 27) => Some(2),
        (10, 28) => Some(1),
        (10, 29) => Some(3),
        (10, 30) => Some(3),
        (10, 32) => Some(2),
        (10, 33) => Some(1),
        (10, 34) => Some(1),
        (10, 35) => Some(0),
        (10, 36) => Some(1),
        (11, 0) => Some(2),
        (11, 1) => Some(8),
        (11, 2) => Some(1),
        (11, 3) => Some(2),
        (11, 4) => Some(1),
        (11, 5) => Some(1),
        (12, 0) => Some(3),
        (12, 1) => Some(1),
        (12, 2) => Some(2),
        (13, 0) => Some(4),
        (13, 1) => Some(2),
        (13, 2) => Some(1),
        (13, 3) => Some(2),
        (13, 4) => Some(4),
        (13, 5) => Some(3),
        (13, 6) => Some(1),
        (13, 7) => Some(1),
        (13, 8) => Some(4),
        (13, 11) => Some(1),
        (13, 12) => Some(3),
        (13, 14) => Some(1),
        (13, 15) => Some(1),
        (13, 16) => Some(4),
        (13, 17) => Some(0),
        (13, 18) => Some(2),
        (13, 19) => Some(3),
        (13, 20) => Some(1),
        (13, 21) => Some(1),
        (13, 22) => Some(2),
        (13, 24) => Some(1),
        (14, 0) => Some(9),
        (14, 1) => Some(0),
        (14, 2) => Some(1),
        (14, 3) => Some(0),
        (14, 4) => Some(1),
        (14, 5) => Some(0),
        (14, 6) => Some(0),
        (14, 7) => Some(0),
        (14, 8) => Some(0),
        (14, 9) => Some(0),
        (14, 10) => Some(4),
        (14, 11) => Some(0),
        (14, 12) => Some(1),
        (15, 2) => Some(3),
        (15, 4) => Some(8),
        (15, 5) => Some(1),
        (16, 0) => Some(2),
        (16, 1) => Some(4),
        (16, 2) => Some(4),
        (17, 0) => Some(2),
        (17, 1) => Some(2),
        (17, 3) => Some(1),
        (17, 5) => Some(6),
        (17, 6) => Some(5),
        (17, 7) => Some(5),
        (17, 8) => Some(4),
        (17, 9) => Some(2),
        (17, 10) => Some(6),
        (17, 11) => Some(4),
        (17, 14) => Some(6),
        (17, 15) => Some(6),
        (17, 17) => Some(6),
        (17, 20) => Some(5),
        (17, 21) => Some(7),
        (17, 23) => Some(1),
        (17, 24) => Some(1),
        (17, 28) => Some(0),
        (17, 29) => Some(6),
        (17, 30) => Some(1),
        (18, 1) => Some(1),
        (18, 3) => Some(2),
        (18, 4) => Some(2),
        (18, 5) => Some(3),
        (18, 6) => Some(1),
        (18, 7) => Some(3),
        (18, 8) => Some(1),
        (18, 9) => Some(10),
        (18, 10) => Some(1),
        (18, 12) => Some(1),
        (18, 13) => Some(1),
        (18, 14) => Some(1),
        (18, 15) => Some(1),
        (18, 17) => Some(1),
        (18, 18) => Some(3),
        (18, 21) => Some(1),
        (18, 28) => Some(0),
        (18, 29) => Some(0),
        (18, 30) => Some(1),
        (18, 31) => Some(3),
        (18, 32) => Some(1),
        (18, 33) => Some(3),
        (18, 34) => Some(3),
        (18, 36) => Some(5),
        (18, 37) => Some(4),
        (18, 40) => Some(16),
        (20, 0) => Some(3),
        (21, 0) => Some(1),
        (21, 2) => Some(1),
        (22, 0) => Some(3),
        (22, 1) => Some(0),
        (23, 0) => Some(1),
        (23, 1) => Some(0),
        (23, 2) => Some(1),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn extended_button_and_clock_signatures_keep_totsulover_stack_balanced() {
        for (category, index, name, pop_count) in [
            (5, 8, "channel_error_set_se_info", 4),
            (8, 14, "btn_get_pos", 4),
            (8, 29, "btn_set_state", 4),
            (8, 37, "btn_get_alpha_0x", 2),
            (18, 90, "ext_18_90", 8),
            (18, 121, "pal_time_ms", 0),
        ] {
            let sig = lookup_sig(category, index).unwrap();
            assert_eq!(sig.name, name);
            assert_eq!(sig.pop_count, pop_count);
        }
    }

    #[test]
    fn testcase_visible_extcalls_use_observed_categories() {
        assert_eq!(lookup_sig(11, 0).map(|s| s.name), Some("movie_play"));
        assert_eq!(
            lookup_sig(11, 1).map(|s| s.name),
            Some("msp_set_loop_sp_ep")
        );
        assert_eq!(lookup_sig(11, 3).map(|s| s.name), Some("msp_wait"));
        assert_eq!(lookup_sig(3, 88), None);

        assert_eq!(lookup_sig(8, 3).map(|s| s.name), Some("btn_set"));
        assert_eq!(lookup_sig(8, 3).map(|s| s.pop_count), Some(7));
        assert_eq!(lookup_sig(8, 6).map(|s| s.name), Some("btn_set_pos"));
        assert_eq!(lookup_sig(8, 17).map(|s| s.name), Some("btn_get_push"));
        assert_eq!(lookup_sig(8, 19).map(|s| s.pop_count), Some(2));
        assert_eq!(lookup_sig(8, 20).map(|s| s.pop_count), Some(1));

        assert_eq!(lookup_sig(4, 1).map(|s| s.name), Some("bgm_stop"));
        assert_eq!(lookup_sig(5, 0).map(|s| s.name), Some("se_load"));
        assert_eq!(lookup_sig(13, 18).map(|s| s.name), Some("voice_wait"));
        assert_eq!(lookup_sig(18, 33).map(|s| s.name), Some("set_file_pointer"));
    }

    #[test]
    fn observed_pop_counts_cover_current_testcase_hotspots() {
        assert_eq!(observed_pop_count(2, 15), Some(5));
        assert_eq!(observed_pop_count(7, 1), Some(2));
        assert_eq!(observed_pop_count(8, 18), Some(3));
        assert_eq!(observed_pop_count(11, 1), Some(8));
        assert_eq!(observed_pop_count(23, 2), Some(1));
        assert_eq!(observed_pop_count(99, 99), None);
    }

    #[test]
    fn all_signatures_have_unique_category_index_keys() {
        let mut seen = BTreeSet::new();
        for sig in all_signatures() {
            assert!(
                seen.insert((sig.category, sig.index)),
                "duplicate ext signature {:04X}:{:04X} {}",
                sig.category,
                sig.index,
                sig.name
            );
            assert_eq!(lookup_sig(sig.category, sig.index), Some(*sig));
            assert_eq!(
                sig.pop_count,
                sig.params.len(),
                "{} pop_count mismatch",
                sig.name
            );
        }
    }
}
