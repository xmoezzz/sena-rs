use std::collections::BTreeMap;
use std::io::Cursor;
use std::sync::Arc;

use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings};
use kira::sound::{EndPosition, PlaybackPosition, PlaybackState, Region};
use kira::{AudioManager, AudioManagerSettings, DefaultBackend, Frame, Tween};
use pal_asset::{LoadedAsset, ResourceManager};
use wmv_decoder::AsfWmaDecoder;

#[derive(Clone, Debug)]
pub struct AudioConfig {
    pub enabled: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

pub struct AudioSystem {
    enabled: bool,
    manager: Option<AudioManager<DefaultBackend>>,
    groups: BTreeMap<PalSoundGroup, Vec<AudioSlot>>,
    primary_volume: PalVolume,
    group_volumes: BTreeMap<PalSoundGroup, PalVolume>,
}

impl AudioSystem {
    pub fn new(config: AudioConfig) -> anyhow::Result<Self> {
        let manager = if config.enabled {
            match AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()) {
                Ok(manager) => Some(manager),
                Err(error) => {
                    log::warn!("audio backend initialization failed; audio disabled: {error:#}");
                    None
                }
            }
        } else {
            None
        };
        let mut groups = BTreeMap::new();
        let mut group_volumes = BTreeMap::new();
        for group in PalSoundGroup::ALL {
            groups.insert(
                group,
                (0..group.slot_count())
                    .map(|_| AudioSlot::default())
                    .collect(),
            );
            group_volumes.insert(group, PalVolume::MAX);
        }
        Ok(Self {
            enabled: manager.is_some(),
            manager,
            groups,
            primary_volume: PalVolume::MAX,
            group_volumes,
        })
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn load_static_from_resource(
        &mut self,
        resource_manager: &mut ResourceManager,
        name: &str,
        group: PalSoundGroup,
    ) -> anyhow::Result<AudioHandle> {
        let asset = resource_manager.open(name)?;
        self.load_static_asset(asset, group)
    }

    pub fn load_static_asset(
        &mut self,
        asset: LoadedAsset,
        group: PalSoundGroup,
    ) -> anyhow::Result<AudioHandle> {
        let data = decode_static_sound(&asset.bytes)?;
        self.load_static_data(asset.name, data, group)
    }

    pub fn load_static_data(
        &mut self,
        name: impl Into<String>,
        data: StaticSoundData,
        group: PalSoundGroup,
    ) -> anyhow::Result<AudioHandle> {
        let slot_index = self.find_free_slot(group)?;
        let slot = self.slot_mut(AudioHandle::new(group, slot_index))?;
        slot.name = Some(name.into());
        slot.data = Some(data);
        slot.handle = None;
        slot.looping = false;
        slot.volume = PalVolume::MAX;
        slot.start_ms = 0;
        slot.end_ms = 0;
        slot.loop_start_samples = 0;
        slot.loop_end_samples = 0;
        slot.pan = 0;
        slot.frequency = 0;
        Ok(AudioHandle::new(group, slot_index))
    }

    pub fn copy_sound(
        &mut self,
        source: AudioHandle,
        target_group: PalSoundGroup,
    ) -> anyhow::Result<AudioHandle> {
        let (data, name, loop_start_samples, loop_end_samples) = {
            let source_slot = self.slot(source)?;
            let data = source_slot
                .data
                .as_ref()
                .ok_or_else(|| {
                    anyhow::anyhow!("audio handle {:?} has no loaded sound data", source)
                })?
                .clone();
            (
                data,
                source_slot.name.clone(),
                source_slot.loop_start_samples,
                source_slot.loop_end_samples,
            )
        };
        let slot_index = self.find_free_slot(target_group)?;
        let slot = self.slot_mut(AudioHandle::new(target_group, slot_index))?;
        slot.name = name;
        slot.data = Some(data);
        slot.handle = None;
        slot.looping = false;
        slot.volume = PalVolume::MAX;
        slot.start_ms = 0;
        slot.end_ms = 0;
        slot.loop_start_samples = loop_start_samples;
        slot.loop_end_samples = loop_end_samples;
        slot.pan = 0;
        slot.frequency = 0;
        Ok(AudioHandle::new(target_group, slot_index))
    }

    pub fn play(&mut self, handle: AudioHandle, looping: bool) -> anyhow::Result<()> {
        if !self.enabled {
            return Ok(());
        }
        let data = {
            let slot = self.slot(handle)?;
            let data = slot
                .data
                .as_ref()
                .ok_or_else(|| {
                    anyhow::anyhow!("audio handle {:?} has no loaded sound data", handle)
                })?
                .clone();
            apply_loop_region(
                data,
                looping,
                slot.loop_start_samples,
                slot.loop_end_samples,
            )
        };
        let effective = self.effective_volume(handle)?;
        let decibels = effective.to_decibels() as f32;
        log::debug!(
            "[trace-audio] play handle={handle:?} looping={looping} effective_raw={} db={decibels:.2}",
            effective.raw()
        );
        let manager = self
            .manager
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("audio backend is disabled"))?;
        let mut static_handle = manager.play(data)?;
        static_handle.set_volume(decibels, Tween::default());
        let slot = self.slot_mut(handle)?;
        slot.handle = Some(static_handle);
        slot.looping = looping;
        Ok(())
    }

    pub fn stop(&mut self, handle: AudioHandle) -> anyhow::Result<()> {
        let slot = self.slot_mut(handle)?;
        if let Some(sound) = slot.handle.as_mut() {
            sound.stop(Tween::default());
        }
        slot.handle = None;
        Ok(())
    }

    pub fn pause(&mut self, handle: AudioHandle) -> anyhow::Result<()> {
        let slot = self.slot_mut(handle)?;
        if let Some(sound) = slot.handle.as_mut() {
            sound.pause(Tween::default());
        }
        Ok(())
    }

    pub fn resume(&mut self, handle: AudioHandle) -> anyhow::Result<()> {
        let slot = self.slot_mut(handle)?;
        if let Some(sound) = slot.handle.as_mut() {
            sound.resume(Tween::default());
        }
        Ok(())
    }

    pub fn release(&mut self, handle: AudioHandle) -> anyhow::Result<()> {
        self.stop(handle)?;
        let slot = self.slot_mut(handle)?;
        *slot = AudioSlot::default();
        Ok(())
    }

    pub fn release_group(&mut self, group: PalSoundGroup) -> anyhow::Result<()> {
        let len = self.groups.get(&group).map_or(0, Vec::len);
        for index in 0..len {
            self.release(AudioHandle::new(group, index))?;
        }
        Ok(())
    }

    pub fn set_primary_volume(&mut self, volume: PalVolume) -> anyhow::Result<()> {
        self.primary_volume = volume.clamped();
        log::debug!(
            "[trace-audio] set_primary_volume raw={}",
            self.primary_volume.raw()
        );
        self.apply_all_volumes()
    }

    pub fn primary_volume(&self) -> PalVolume {
        self.primary_volume
    }

    pub fn set_group_volume(
        &mut self,
        group: PalSoundGroup,
        volume: PalVolume,
    ) -> anyhow::Result<()> {
        self.group_volumes.insert(group, volume.clamped());
        log::debug!(
            "[trace-audio] set_group_volume group={group:?} raw={}",
            volume.clamped().raw()
        );
        let len = self.groups.get(&group).map_or(0, Vec::len);
        for index in 0..len {
            self.apply_slot_volume(AudioHandle::new(group, index))?;
        }
        Ok(())
    }

    pub fn group_volume(&self, group: PalSoundGroup) -> PalVolume {
        *self.group_volumes.get(&group).unwrap_or(&PalVolume::MAX)
    }

    pub fn set_channel_volume(
        &mut self,
        handle: AudioHandle,
        volume: PalVolume,
    ) -> anyhow::Result<()> {
        self.slot_mut(handle)?.volume = volume.clamped();
        log::debug!(
            "[trace-audio] set_channel_volume handle={handle:?} raw={}",
            volume.clamped().raw()
        );
        self.apply_slot_volume(handle)
    }

    pub fn channel_volume(&self, handle: AudioHandle) -> anyhow::Result<PalVolume> {
        Ok(self.slot(handle)?.volume)
    }

    pub fn set_start_end(
        &mut self,
        handle: AudioHandle,
        start_ms: i32,
        end_ms: i32,
    ) -> anyhow::Result<()> {
        let slot = self.slot_mut(handle)?;
        ensure_loaded(slot, handle)?;
        slot.start_ms = start_ms;
        slot.end_ms = end_ms;
        Ok(())
    }

    /// `BGM.CSV` loop columns are PCM sample indices, not milliseconds.
    pub fn set_loop_samples(
        &mut self,
        handle: AudioHandle,
        loop_start_samples: i64,
        loop_end_samples: i64,
    ) -> anyhow::Result<()> {
        let slot = self.slot_mut(handle)?;
        ensure_loaded(slot, handle)?;
        slot.loop_start_samples = loop_start_samples;
        slot.loop_end_samples = loop_end_samples;
        Ok(())
    }

    pub fn loop_samples(&self, handle: AudioHandle) -> anyhow::Result<(i64, i64)> {
        let slot = self.slot(handle)?;
        ensure_loaded(slot, handle)?;
        Ok((slot.loop_start_samples, slot.loop_end_samples))
    }

    pub fn start_end(&self, handle: AudioHandle) -> anyhow::Result<(i32, i32)> {
        let slot = self.slot(handle)?;
        ensure_loaded(slot, handle)?;
        Ok((slot.start_ms, slot.end_ms))
    }

    pub fn set_channel_pan(&mut self, handle: AudioHandle, pan: i32) -> anyhow::Result<()> {
        let slot = self.slot_mut(handle)?;
        ensure_loaded(slot, handle)?;
        slot.pan = pan;
        Ok(())
    }

    pub fn channel_pan(&self, handle: AudioHandle) -> anyhow::Result<i32> {
        let slot = self.slot(handle)?;
        ensure_loaded(slot, handle)?;
        Ok(slot.pan)
    }

    pub fn set_channel_frequency(
        &mut self,
        handle: AudioHandle,
        frequency: i32,
    ) -> anyhow::Result<()> {
        let slot = self.slot_mut(handle)?;
        ensure_loaded(slot, handle)?;
        slot.frequency = frequency;
        Ok(())
    }

    pub fn channel_frequency(&self, handle: AudioHandle) -> anyhow::Result<i32> {
        let slot = self.slot(handle)?;
        ensure_loaded(slot, handle)?;
        Ok(slot.frequency)
    }

    pub fn sound_param(&self, group: PalSoundGroup, index: usize, param: usize) -> i32 {
        let Some(slot) = self.groups.get(&group).and_then(|slots| slots.get(index)) else {
            return 0;
        };
        match param {
            0 => i32::from(slot.data.is_some()),
            1 => i32::from(slot.handle.is_some()),
            2 => i32::from(slot.looping),
            3 => slot.start_ms,
            4 => slot.end_ms,
            _ => 0,
        }
    }

    pub fn sound_status(&self, handle: AudioHandle) -> anyhow::Result<PalSoundStatus> {
        let slot = self.slot(handle)?;
        if slot.data.is_none() {
            return Ok(PalSoundStatus::Free);
        }
        if let Some(sound) = slot.handle.as_ref() {
            return Ok(match sound.state() {
                PlaybackState::Playing => PalSoundStatus::Playing,
                PlaybackState::Pausing
                | PlaybackState::Paused
                | PlaybackState::WaitingToResume
                | PlaybackState::Resuming => PalSoundStatus::Paused,
                PlaybackState::Stopping | PlaybackState::Stopped => PalSoundStatus::Stopped,
            });
        }
        Ok(PalSoundStatus::Loaded)
    }

    pub fn loaded_channel_count_for_handle(&self, handle: AudioHandle) -> usize {
        self.loaded_channel_count(handle.group)
    }

    pub fn group_channel_count(&self, group: PalSoundGroup) -> usize {
        group.slot_count()
    }

    pub fn now_channel_count(&self, group: PalSoundGroup) -> usize {
        self.groups
            .get(&group)
            .map(|slots| {
                slots
                    .iter()
                    .filter(|slot| {
                        slot.handle
                            .as_ref()
                            .is_some_and(|sound| matches!(sound.state(), PlaybackState::Playing))
                    })
                    .count()
            })
            .unwrap_or(0)
    }

    pub fn is_playing(&self, handle: AudioHandle) -> anyhow::Result<bool> {
        let slot = self.slot(handle)?;
        Ok(match slot.handle.as_ref() {
            Some(sound) => matches!(sound.state(), PlaybackState::Playing),
            None => false,
        })
    }

    pub fn loaded_channel_count(&self, group: PalSoundGroup) -> usize {
        self.groups
            .get(&group)
            .map(|slots| slots.iter().filter(|slot| slot.data.is_some()).count())
            .unwrap_or(0)
    }

    pub fn free_channel_count(&self, group: PalSoundGroup) -> usize {
        self.groups
            .get(&group)
            .map(|slots| slots.iter().filter(|slot| slot.data.is_none()).count())
            .unwrap_or(0)
    }

    pub fn update(&mut self) {
        for slots in self.groups.values_mut() {
            for slot in slots {
                if slot
                    .handle
                    .as_ref()
                    .is_some_and(|sound| matches!(sound.state(), PlaybackState::Stopped))
                {
                    slot.handle = None;
                }
            }
        }
    }

    fn find_free_slot(&self, group: PalSoundGroup) -> anyhow::Result<usize> {
        self.groups
            .get(&group)
            .and_then(|slots| slots.iter().position(|slot| slot.data.is_none()))
            .ok_or_else(|| anyhow::anyhow!("no free audio slot in group {:?}", group))
    }

    fn apply_all_volumes(&mut self) -> anyhow::Result<()> {
        for group in PalSoundGroup::ALL {
            let len = self.groups.get(&group).map_or(0, Vec::len);
            for index in 0..len {
                self.apply_slot_volume(AudioHandle::new(group, index))?;
            }
        }
        Ok(())
    }

    fn apply_slot_volume(&mut self, handle: AudioHandle) -> anyhow::Result<()> {
        let effective = self.effective_volume(handle)?;
        let decibels = effective.to_decibels() as f32;
        let slot = self.slot_mut(handle)?;
        let has_loaded_sound = slot.data.is_some() || slot.handle.is_some();
        if has_loaded_sound {
            log::debug!(
                "[trace-audio] apply_slot_volume handle={handle:?} effective_raw={} db={decibels:.2}",
                effective.raw()
            );
        }
        if let Some(sound) = slot.handle.as_mut() {
            sound.set_volume(decibels, Tween::default());
        }
        Ok(())
    }

    pub fn effective_volume(&self, handle: AudioHandle) -> anyhow::Result<PalVolume> {
        let slot = self.slot(handle)?;
        let group = self.group_volume(handle.group);
        Ok(PalVolume::from_raw(
            (self.primary_volume.raw() as i64 * group.raw() as i64 * slot.volume.raw() as i64
                / 10000
                / 10000) as i32,
        ))
    }

    fn slot(&self, handle: AudioHandle) -> anyhow::Result<&AudioSlot> {
        let slots = self
            .groups
            .get(&handle.group)
            .ok_or_else(|| anyhow::anyhow!("invalid audio group {:?}", handle.group))?;
        slots
            .get(handle.index)
            .ok_or_else(|| anyhow::anyhow!("audio handle {:?} points outside its group", handle))
    }

    fn slot_mut(&mut self, handle: AudioHandle) -> anyhow::Result<&mut AudioSlot> {
        let slots = self
            .groups
            .get_mut(&handle.group)
            .ok_or_else(|| anyhow::anyhow!("invalid audio group {:?}", handle.group))?;
        slots
            .get_mut(handle.index)
            .ok_or_else(|| anyhow::anyhow!("audio handle {:?} points outside its group", handle))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PalSoundStatus {
    Free,
    Loaded,
    Playing,
    Paused,
    Stopped,
}

impl PalSoundStatus {
    pub const fn raw(self) -> i32 {
        match self {
            Self::Free => 0,
            Self::Loaded => 1,
            Self::Playing => 2,
            Self::Paused => 3,
            Self::Stopped => 4,
        }
    }
}

#[derive(Default)]
struct AudioSlot {
    name: Option<String>,
    data: Option<StaticSoundData>,
    handle: Option<StaticSoundHandle>,
    looping: bool,
    volume: PalVolume,
    start_ms: i32,
    end_ms: i32,
    loop_start_samples: i64,
    loop_end_samples: i64,
    pan: i32,
    frequency: i32,
}

/// One `BGM.CSV` row. Loop columns are PCM sample indices in that file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BgmLoop {
    pub intro_samples: i64,
    pub loop_start_samples: i64,
    pub loop_end_samples: i64,
}

pub fn audio_lookup_key(name: &str) -> String {
    let leaf = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let stem = match leaf.rsplit_once('.') {
        Some((stem, ext))
            if ext.eq_ignore_ascii_case("ogg")
                || ext.eq_ignore_ascii_case("wav")
                || ext.eq_ignore_ascii_case("wma")
                || ext.eq_ignore_ascii_case("mix") =>
        {
            stem
        }
        _ => leaf,
    };
    stem.to_ascii_uppercase()
}

/// Parse Palette `BGM.CSV` (`name, intro, loop_start, loop_end`).
pub fn parse_bgm_csv(bytes: &[u8]) -> BTreeMap<String, BgmLoop> {
    let text = decode_bgm_csv_text(bytes);
    let mut table = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim().trim_start_matches('\u{feff}');
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }
        let parts = split_csv_row(line);
        if parts.len() < 4 {
            continue;
        }
        let name = parts[0].trim().trim_matches('"').trim();
        if name.is_empty() {
            continue;
        }
        let Ok(intro_samples) = parts[1].trim().parse::<i64>() else {
            continue;
        };
        let Ok(loop_start_samples) = parts[2].trim().parse::<i64>() else {
            continue;
        };
        let Ok(loop_end_samples) = parts[3].trim().parse::<i64>() else {
            continue;
        };
        table.insert(
            audio_lookup_key(name),
            BgmLoop {
                intro_samples,
                loop_start_samples,
                loop_end_samples,
            },
        );
    }
    table
}

/// A `.MIX` voice list is N null-padded 32-byte names and is not itself audio.
pub fn parse_mix_playlist(bytes: &[u8]) -> Option<Vec<String>> {
    if bytes.is_empty() || bytes.len() % 32 != 0 || is_encoded_audio(bytes) {
        return None;
    }
    let mut names = Vec::new();
    for chunk in bytes.chunks_exact(32) {
        let end = chunk.iter().position(|&byte| byte == 0)?;
        if end == 0 || chunk[end..].iter().any(|&byte| byte != 0) {
            return None;
        }
        let raw = &chunk[..end];
        if !raw
            .iter()
            .all(|byte| byte.is_ascii_graphic() || *byte >= 0x80)
        {
            return None;
        }
        names.push(decode_mix_name(raw));
    }
    if names.is_empty() {
        None
    } else {
        Some(names)
    }
}

pub fn decode_static_sound(bytes: &[u8]) -> anyhow::Result<StaticSoundData> {
    if is_asf_container(bytes) {
        return decode_wma_sound(bytes);
    }
    StaticSoundData::from_cursor(Cursor::new(bytes.to_vec()))
        .map_err(|err| anyhow::anyhow!("audio decode failed: {err}"))
}

/// Decode one voice, or sum every clip named by a `.MIX` playlist.
pub fn decode_game_audio(
    bytes: &[u8],
    open_member: &mut dyn FnMut(&str) -> anyhow::Result<Vec<u8>>,
) -> anyhow::Result<StaticSoundData> {
    decode_game_audio_depth(bytes, open_member, 2)
}

fn decode_game_audio_depth(
    bytes: &[u8],
    open_member: &mut dyn FnMut(&str) -> anyhow::Result<Vec<u8>>,
    depth: u8,
) -> anyhow::Result<StaticSoundData> {
    if depth > 0 {
        if let Some(names) = parse_mix_playlist(bytes) {
            let mut sounds = Vec::new();
            for name in names {
                match open_member(&name) {
                    Ok(member) => match decode_game_audio_depth(&member, open_member, depth - 1) {
                        Ok(sound) => sounds.push(sound),
                        Err(err) => {
                            log::warn!("[trace-audio] mix member {name:?} decode failed: {err}");
                        }
                    },
                    Err(err) => {
                        log::warn!("[trace-audio] mix member {name:?} open failed: {err}");
                    }
                }
            }
            if sounds.is_empty() {
                anyhow::bail!("mix playlist produced no playable voices");
            }
            return mix_static_sounds(&sounds);
        }
    }
    decode_static_sound(bytes)
}

pub fn mix_static_sounds(sounds: &[StaticSoundData]) -> anyhow::Result<StaticSoundData> {
    let first = sounds
        .first()
        .ok_or_else(|| anyhow::anyhow!("mix playlist is empty"))?;
    if sounds.len() == 1 {
        return Ok(first.clone());
    }
    let sample_rate = first.sample_rate.max(1);
    let mut mixed: Vec<Frame> = Vec::new();
    for sound in sounds {
        let frames = frames_at_rate(sound, sample_rate);
        if frames.len() > mixed.len() {
            mixed.resize(frames.len(), Frame::ZERO);
        }
        for (dst, src) in mixed.iter_mut().zip(frames) {
            *dst += src;
            dst.left = dst.left.clamp(-1.0, 1.0);
            dst.right = dst.right.clamp(-1.0, 1.0);
        }
    }
    Ok(StaticSoundData {
        sample_rate,
        frames: Arc::from(mixed),
        settings: StaticSoundSettings::new(),
        slice: None,
    })
}

pub fn apply_loop_region(
    data: StaticSoundData,
    looping: bool,
    loop_start_samples: i64,
    loop_end_samples: i64,
) -> StaticSoundData {
    let Some(region) = loop_region_for(looping, loop_start_samples, loop_end_samples) else {
        return data;
    };
    data.loop_region(region)
}

fn loop_region_for(
    looping: bool,
    loop_start_samples: i64,
    loop_end_samples: i64,
) -> Option<Region> {
    if !looping {
        return None;
    }
    if loop_end_samples > loop_start_samples && loop_start_samples >= 0 {
        let start = loop_start_samples as usize;
        let end = loop_end_samples as usize;
        Some(Region {
            start: PlaybackPosition::Samples(start),
            end: EndPosition::Custom(PlaybackPosition::Samples(end)),
        })
    } else {
        Some(Region {
            start: PlaybackPosition::Samples(0),
            end: EndPosition::EndOfAudio,
        })
    }
}

fn frames_at_rate(sound: &StaticSoundData, sample_rate: u32) -> Vec<Frame> {
    let source = sound.frames.as_ref();
    if source.is_empty() || sound.sample_rate == 0 || sound.sample_rate == sample_rate {
        return source.to_vec();
    }
    let dst_len = ((source.len() as u64 * u64::from(sample_rate)) / u64::from(sound.sample_rate))
        .max(1) as usize;
    let mut out = Vec::with_capacity(dst_len);
    for index in 0..dst_len {
        let src_pos = index as f64 * f64::from(sound.sample_rate) / f64::from(sample_rate);
        let src_index = src_pos.floor() as usize;
        let frac = (src_pos - src_index as f64) as f32;
        let left = source.get(src_index).copied().unwrap_or(Frame::ZERO);
        let right = source.get(src_index + 1).copied().unwrap_or(left);
        out.push(Frame::new(
            left.left + (right.left - left.left) * frac,
            left.right + (right.right - left.right) * frac,
        ));
    }
    out
}

fn is_asf_container(bytes: &[u8]) -> bool {
    bytes.len() >= 16 && bytes.starts_with(&[0x30, 0x26, 0xB2, 0x75])
}

fn is_encoded_audio(bytes: &[u8]) -> bool {
    is_asf_container(bytes)
        || bytes.starts_with(b"OggS")
        || bytes.starts_with(b"RIFF")
        || bytes.starts_with(b"ID3")
        || bytes.starts_with(&[0xFF, 0xFB])
}

fn decode_wma_sound(bytes: &[u8]) -> anyhow::Result<StaticSoundData> {
    let mut decoder = AsfWmaDecoder::open(Cursor::new(bytes.to_vec()))
        .map_err(|err| anyhow::anyhow!("wma open failed: {err}"))?;
    let sample_rate = decoder.sample_rate().max(1);
    let channels = usize::from(decoder.channels().max(1));
    let mut frames = Vec::new();
    loop {
        match decoder.next_frame() {
            Ok(Some(decoded)) => {
                append_interleaved_pcm(&mut frames, &decoded.frame.samples, channels)
            }
            Ok(None) => break,
            Err(wmv_decoder::DecoderError::EndOfStream) => break,
            Err(err) => return Err(anyhow::anyhow!("wma decode failed: {err}")),
        }
    }
    if frames.is_empty() {
        anyhow::bail!("wma decode produced no samples");
    }
    Ok(StaticSoundData {
        sample_rate,
        frames: Arc::from(frames),
        settings: StaticSoundSettings::new(),
        slice: None,
    })
}

fn append_interleaved_pcm(frames: &mut Vec<Frame>, samples: &[f32], channels: usize) {
    if channels <= 1 {
        frames.extend(samples.iter().copied().map(Frame::from_mono));
        return;
    }
    for chunk in samples.chunks(channels) {
        let left = chunk.first().copied().unwrap_or(0.0);
        let right = chunk.get(1).copied().unwrap_or(left);
        frames.push(Frame::new(left, right));
    }
}

fn decode_bgm_csv_text(bytes: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_owned();
    }
    let (text, _, _) = encoding_rs::SHIFT_JIS.decode(bytes);
    text.into_owned()
}

fn decode_mix_name(bytes: &[u8]) -> String {
    if bytes.iter().all(|byte| byte.is_ascii()) {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    let (text, _, _) = encoding_rs::SHIFT_JIS.decode(bytes);
    text.into_owned()
}

fn split_csv_row(line: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut rest = line;
    while !rest.is_empty() {
        if rest.starts_with('"') {
            if let Some(end) = rest[1..].find('"') {
                parts.push(&rest[1..1 + end]);
                rest = rest[1 + end + 1..].trim_start_matches(',').trim_start();
                continue;
            }
        }
        if let Some(comma) = rest.find(',') {
            parts.push(rest[..comma].trim());
            rest = rest[comma + 1..].trim_start();
        } else {
            parts.push(rest.trim());
            break;
        }
    }
    parts
}

fn ensure_loaded(slot: &AudioSlot, handle: AudioHandle) -> anyhow::Result<()> {
    if slot.data.is_some() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "audio handle {:?} has no loaded sound data",
            handle
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AudioHandle {
    pub group: PalSoundGroup,
    pub index: usize,
}

impl AudioHandle {
    pub fn new(group: PalSoundGroup, index: usize) -> Self {
        Self { group, index }
    }

    pub fn raw(self) -> u32 {
        self.group.raw_prefix() | self.index as u32
    }

    pub fn from_raw(raw: u32) -> Option<Self> {
        let prefix = raw & 0xF000_0000;
        let index = (raw & 0x0FFF_FFFF) as usize;
        PalSoundGroup::from_raw_prefix(prefix).map(|group| Self { group, index })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PalSoundGroup(pub u8);

impl PalSoundGroup {
    pub const GROUP0: Self = Self(0);
    pub const GROUP1: Self = Self(1);
    pub const GROUP2: Self = Self(2);
    pub const GROUP3: Self = Self(3);
    pub const GROUP4: Self = Self(4);
    pub const GROUP5: Self = Self(5);
    pub const GROUP6: Self = Self(6);
    pub const ALL: [Self; 7] = [
        Self::GROUP0,
        Self::GROUP1,
        Self::GROUP2,
        Self::GROUP3,
        Self::GROUP4,
        Self::GROUP5,
        Self::GROUP6,
    ];

    pub fn from_original_kind(kind: i32) -> Self {
        match kind {
            0 => Self::GROUP0,
            2 => Self::GROUP2,
            3 => Self::GROUP3,
            4 => Self::GROUP4,
            5 => Self::GROUP5,
            6 => Self::GROUP6,
            _ => Self::GROUP1,
        }
    }

    pub fn slot_count(self) -> usize {
        match self.0 {
            0 => 2,
            1 => 16,
            2 => 8,
            3 => 2,
            4 => 16,
            5 => 64,
            6 => 16,
            _ => 0,
        }
    }

    pub fn raw_prefix(self) -> u32 {
        match self.0 {
            0 => 0x1000_0000,
            1 => 0x3000_0000,
            2 => 0x7000_0000,
            3 => 0x2000_0000,
            4 => 0x4000_0000,
            5 => 0x5000_0000,
            6 => 0x6000_0000,
            _ => 0,
        }
    }

    pub fn from_raw_prefix(prefix: u32) -> Option<Self> {
        match prefix {
            0x1000_0000 => Some(Self::GROUP0),
            0x3000_0000 => Some(Self::GROUP1),
            0x7000_0000 => Some(Self::GROUP2),
            0x2000_0000 => Some(Self::GROUP3),
            0x4000_0000 => Some(Self::GROUP4),
            0x5000_0000 => Some(Self::GROUP5),
            0x6000_0000 => Some(Self::GROUP6),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PalVolume(i32);

impl PalVolume {
    pub const MIN: Self = Self(0);
    pub const MAX: Self = Self(10000);

    pub fn from_raw(value: i32) -> Self {
        Self(value).clamped()
    }

    pub fn raw(self) -> i32 {
        self.0
    }

    pub fn clamped(self) -> Self {
        Self(self.0.clamp(Self::MIN.0, Self::MAX.0))
    }

    pub fn to_decibels(self) -> f64 {
        let raw = self.clamped().0;
        if raw == 0 {
            -80.0
        } else {
            20.0 * (raw as f64 / 10000.0).log10()
        }
    }
}

impl Default for PalVolume {
    fn default() -> Self {
        Self::MAX
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_loop_region, audio_lookup_key, decode_game_audio, decode_static_sound,
        mix_static_sounds, parse_bgm_csv, parse_mix_playlist,
    };
    use kira::sound::{EndPosition, PlaybackPosition};

    #[test]
    fn bgm_csv_rows_are_sample_loop_points() {
        let csv = b"//name,intro,loop_start,loop_end\r\n\"BGM01\",0,0,7931200\r\n\"bgm007.ogg\",0,562275,5325075\r\n\"BGM00\",0,0,0\r\n";
        let table = parse_bgm_csv(csv);
        assert_eq!(table["BGM01"].loop_end_samples, 7_931_200);
        assert_eq!(table["BGM007"].loop_start_samples, 562_275);
        assert_eq!(table["BGM007"].loop_end_samples, 5_325_075);
        assert_eq!(table["BGM00"].loop_end_samples, 0);
        assert_eq!(audio_lookup_key("bgm\\BGM01.OGG"), "BGM01");
    }

    #[test]
    fn parses_real_bgm_csv_when_fixture_is_set() {
        let Some(path) = std::env::var_os("SENA_BGM_CSV") else {
            return;
        };
        let bytes = std::fs::read(path).expect("read SENA_BGM_CSV");
        let table = parse_bgm_csv(&bytes);
        assert!(table.len() >= 10, "rows {}", table.len());
        let tracked = table
            .values()
            .filter(|row| row.loop_end_samples > row.loop_start_samples)
            .count();
        assert!(tracked >= 1, "expected at least one loop region");
    }

    #[test]
    fn mix_playlist_lists_voice_names_and_rejects_audio() {
        let mut bytes = vec![0u8; 64];
        bytes[..8].copy_from_slice(b"vo98_005");
        bytes[32..32 + 9].copy_from_slice(b"vo01_1257");
        assert_eq!(
            parse_mix_playlist(&bytes).as_deref(),
            Some(["vo98_005".to_owned(), "vo01_1257".to_owned()].as_slice())
        );
        assert!(parse_mix_playlist(b"OggSrest").is_none());
        assert!(parse_mix_playlist(&[0x30, 0x26, 0xB2, 0x75]).is_none());
    }

    #[test]
    fn mix_playlist_sums_member_clips_into_one_buffer() {
        let left = pcm16_wav(8_000, &[0, 1000, 0]);
        let right = pcm16_wav(8_000, &[0, 1000]);
        let mut playlist = vec![0u8; 64];
        playlist[..1].copy_from_slice(b"a");
        playlist[32..33].copy_from_slice(b"b");
        let mixed = decode_game_audio(&playlist, &mut |name| {
            Ok(match name {
                "a" => left.clone(),
                "b" => right.clone(),
                other => anyhow::bail!("missing {other}"),
            })
        })
        .expect("mix");
        assert_eq!(mixed.frames.len(), 3);
        assert!(mixed.frames[1].left > mixed.frames[0].left);
        assert!(mixed.frames[2].left.abs() < mixed.frames[1].left.abs());
    }

    #[test]
    fn loop_region_uses_sample_points_only_while_looping() {
        let sound = decode_static_sound(&pcm16_wav(8_000, &[0, 1000, 0, -1000])).expect("wav");
        let once = apply_loop_region(sound.clone(), false, 10, 30);
        assert!(once.settings.loop_region.is_none());
        let ranged = apply_loop_region(sound.clone(), true, 562_275, 5_325_075);
        let region = ranged.settings.loop_region.expect("region");
        assert_eq!(region.start, PlaybackPosition::Samples(562_275));
        assert_eq!(
            region.end,
            EndPosition::Custom(PlaybackPosition::Samples(5_325_075))
        );
        let whole = apply_loop_region(sound, true, 0, 0);
        let region = whole.settings.loop_region.expect("whole");
        assert_eq!(region.end, EndPosition::EndOfAudio);
    }

    #[test]
    fn truncated_asf_is_routed_to_the_wma_decoder() {
        let bytes = [0x30, 0x26, 0xB2, 0x75, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let err = decode_static_sound(&bytes).expect_err("truncated wma");
        let message = format!("{err:#}");
        assert!(
            message.contains("wma"),
            "unexpected decoder error: {message}"
        );
    }

    #[test]
    fn real_mix_decodes_when_game_root_is_set() {
        let Some(root) = std::env::var_os("SENA_GAME_ROOT") else {
            return;
        };
        let Some(mix_name) = std::env::var("SENA_MIX_NAME").ok() else {
            return;
        };
        let mut manager = pal_asset::ResourceManager::bootstrap(root, pal_asset::Nls::ShiftJis)
            .expect("bootstrap");
        let asset = manager.open(&mix_name).expect("mix");
        let names = parse_mix_playlist(&asset.bytes).expect("playlist");
        assert!(!names.is_empty());
        let mixed = decode_game_audio(&asset.bytes, &mut |member| {
            for ext in ["", ".OGG", ".ogg", ".WAV", ".wav", ".WMA", ".wma"] {
                if let Ok(opened) = manager.open(&format!("{member}{ext}")) {
                    return Ok(opened.bytes);
                }
            }
            anyhow::bail!("missing mix member {member}");
        })
        .expect("decode mix");
        assert!(mixed.frames.len() > 100, "frames {}", mixed.frames.len());
    }

    #[test]
    fn single_mix_member_keeps_its_samples() {
        let clip = pcm16_wav(8_000, &[0, 2000, -2000, 0]);
        let sounds = [decode_static_sound(&clip).expect("wav")];
        let mixed = mix_static_sounds(&sounds).expect("one");
        assert_eq!(mixed.frames.len(), sounds[0].frames.len());
    }

    fn pcm16_wav(rate: u32, samples: &[i16]) -> Vec<u8> {
        let data_len = (samples.len() * 2) as u32;
        let mut out = Vec::new();
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(36 + data_len).to_le_bytes());
        out.extend_from_slice(b"WAVEfmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&rate.to_le_bytes());
        out.extend_from_slice(&(rate * 2).to_le_bytes());
        out.extend_from_slice(&2u16.to_le_bytes());
        out.extend_from_slice(&16u16.to_le_bytes());
        out.extend_from_slice(b"data");
        out.extend_from_slice(&data_len.to_le_bytes());
        for sample in samples {
            out.extend_from_slice(&sample.to_le_bytes());
        }
        out
    }
}
