use pal_vm::{
    DrawCommand, PalColor, PalPoint2, PalPoint3, PalRect, PalRenderMode, PalVec3, SceneTexture,
    SceneTextureId, SpriteDesc, SpriteSurfaceId, SpriteSystem,
};

fn make_system() -> SpriteSystem {
    let mut sprites = SpriteSystem::new();
    sprites.insert_texture(SceneTexture::rgba8(
        SceneTextureId(10),
        1,
        64,
        32,
        vec![255; 64 * 32 * 4],
    ));
    sprites.insert_texture(SceneTexture::rgba8(
        SceneTextureId(11),
        1,
        64,
        32,
        vec![255; 64 * 32 * 4],
    ));
    sprites
}

#[test]
fn rgba_sprite_create_and_replace_updates_draw_surface() {
    let mut sprites = SpriteSystem::new();
    let handle = sprites
        .create_rgba_sprite(
            2,
            1,
            vec![255, 0, 0, 255, 0, 255, 0, 255],
            PalVec3::new(10, 20, 30),
            30,
            "text:first",
        )
        .expect("rgba sprite should be created");
    let sprite = sprites.get(handle).unwrap();
    assert_eq!(sprite.cell_size.width, 2);
    assert_eq!(sprite.cell_size.height, 1);
    assert_eq!(sprite.source_name, "text:first");
    assert_eq!(sprites.commands().len(), 1);

    assert!(sprites.replace_sprite_surface(
        handle,
        1,
        2,
        vec![0, 0, 255, 255, 255, 255, 255, 128],
        "text:next"
    ));
    let sprite = sprites.get(handle).unwrap();
    assert_eq!(sprite.texture_size.width, 1);
    assert_eq!(sprite.texture_size.height, 2);
    assert_eq!(sprite.source_rect, PalRect::new(0, 0, 1, 2));
    assert_eq!(sprite.source_name, "text:next");
}

#[test]
fn image_overlay_composites_into_existing_sprite_surface() {
    let mut sprites = SpriteSystem::new();
    let handle = sprites
        .create_rgba_sprite(
            3,
            1,
            vec![0, 0, 128, 255].repeat(3),
            PalVec3::new(0, 0, 0),
            0,
            "popup",
        )
        .unwrap();
    let before = sprites
        .surface(sprites.get(handle).unwrap().surface)
        .unwrap()
        .generation;
    assert!(sprites.composite_rgba_to_sprite(
        handle,
        1,
        0,
        2,
        1,
        &[255, 0, 0, 255, 0, 255, 0, 0],
        0,
        0,
        2,
        1,
    ));
    let surface = sprites
        .surface(sprites.get(handle).unwrap().surface)
        .unwrap();
    assert!(surface.generation > before);
    assert_eq!(
        &*surface.pixels,
        &[0, 0, 128, 255, 255, 0, 0, 255, 0, 0, 128, 255]
    );
}

#[test]
fn release_removes_render_node_and_unreferenced_surface() {
    let mut sprites = make_system();
    let first = sprites.create(SpriteDesc {
        texture_id: SceneTextureId(10),
        texture_width: 64,
        texture_height: 32,
        visible: true,
        ..SpriteDesc::new(SceneTextureId(10), 64, 32)
    });
    let second = sprites.create(SpriteDesc {
        texture_id: SceneTextureId(10),
        texture_width: 64,
        texture_height: 32,
        visible: true,
        ..SpriteDesc::new(SceneTextureId(10), 64, 32)
    });

    assert_eq!(sprites.len(), 2);
    assert_eq!(sprites.render_node_count(), 2);
    assert!(sprites.surface(SpriteSurfaceId(10)).is_some());

    assert!(sprites.release(first));
    assert_eq!(sprites.len(), 1);
    assert_eq!(sprites.render_node_count(), 1);
    assert!(sprites.surface(SpriteSurfaceId(10)).is_some());

    assert!(sprites.release(second));
    assert_eq!(sprites.len(), 0);
    assert_eq!(sprites.render_node_count(), 0);
    assert!(sprites.surface(SpriteSurfaceId(10)).is_none());
}

#[test]
fn sprite_rect_reset_uses_texture_size_but_rect_set_pos_uses_cell_size() {
    let mut sprites = make_system();
    let sprite = sprites.create(SpriteDesc {
        texture_id: SceneTextureId(10),
        texture_width: 64,
        texture_height: 32,
        cell_width: 16,
        cell_height: 16,
        visible: true,
        ..SpriteDesc::new(SceneTextureId(10), 64, 32)
    });

    assert_eq!(
        sprites.get(sprite).unwrap().source_rect,
        PalRect::new(0, 0, 16, 16)
    );
    assert!(sprites.rect_set_pos(sprite, 3, 1));
    assert_eq!(
        sprites.get(sprite).unwrap().source_rect,
        PalRect::new(48, 16, 64, 32)
    );
    assert!(sprites.set_rect(sprite, None));
    assert_eq!(
        sprites.get(sprite).unwrap().source_rect,
        PalRect::new(0, 0, 64, 32)
    );
}

#[test]
fn center_offset_setter_stores_negative_value_and_does_not_move_2d_draw_position() {
    let mut sprites = make_system();
    let sprite = sprites.create(SpriteDesc {
        texture_id: SceneTextureId(10),
        texture_width: 64,
        texture_height: 32,
        cell_width: 16,
        cell_height: 16,
        position: PalPoint3::new(10, 20, 0),
        offset: PalPoint2::new(3, 4),
        visible: true,
        ..SpriteDesc::new(SceneTextureId(10), 64, 32)
    });

    assert!(sprites.set_center_offset(sprite, 8, 9));
    let pal_sprite = sprites.get(sprite).unwrap();
    assert_eq!(pal_sprite.center_offset, PalPoint2::new(-8, -9));

    let commands = sprites.commands();
    let pal_vm::DrawCommand::Sprite(draw) = &commands[0] else {
        panic!("expected sprite draw")
    };
    assert_eq!(draw.dst.x, 13.0);
    assert_eq!(draw.dst.y, 24.0);
    assert_eq!(draw.center_offset, [-8.0, -9.0]);
}

#[test]
fn render_nodes_sort_by_effective_priority() {
    let mut sprites = make_system();
    let low = sprites.create(SpriteDesc {
        texture_id: SceneTextureId(10),
        texture_width: 64,
        texture_height: 32,
        visible: true,
        base_priority: 20,
        ..SpriteDesc::new(SceneTextureId(10), 64, 32)
    });
    let high = sprites.create(SpriteDesc {
        texture_id: SceneTextureId(11),
        texture_width: 64,
        texture_height: 32,
        visible: true,
        base_priority: 10,
        position: PalPoint3::new(0, 0, 5),
        ..SpriteDesc::new(SceneTextureId(11), 64, 32)
    });

    let commands = sprites.commands();
    let pal_vm::DrawCommand::Sprite(first) = &commands[0] else {
        panic!("expected sprite draw")
    };
    let pal_vm::DrawCommand::Sprite(second) = &commands[1] else {
        panic!("expected sprite draw")
    };
    assert_eq!(first.texture_id, SceneTextureId(11));
    assert_eq!(first.priority, 15);
    assert_eq!(second.texture_id, SceneTextureId(10));
    assert_eq!(second.priority, 20);

    assert!(sprites.set_pos(high, 0, 0, 30));
    let commands = sprites.commands();
    let pal_vm::DrawCommand::Sprite(first) = &commands[0] else {
        panic!("expected sprite draw")
    };
    assert_eq!(first.texture_id, SceneTextureId(10));
    assert_eq!(sprites.get(low).unwrap().effective_priority(), 20);
}

#[test]
fn equal_priority_draws_newer_render_nodes_over_older_nodes() {
    let mut sprites = make_system();
    let foreground = sprites.create(SpriteDesc {
        texture_id: SceneTextureId(10),
        texture_width: 64,
        texture_height: 32,
        visible: true,
        base_priority: 0,
        ..SpriteDesc::new(SceneTextureId(10), 64, 32)
    });
    let fullscreen_background = sprites.create(SpriteDesc {
        texture_id: SceneTextureId(11),
        texture_width: 1280,
        texture_height: 720,
        visible: true,
        base_priority: 0,
        ..SpriteDesc::new(SceneTextureId(11), 1280, 720)
    });

    let commands = sprites.commands();
    let pal_vm::DrawCommand::Sprite(first) = &commands[0] else {
        panic!("expected sprite draw")
    };
    let pal_vm::DrawCommand::Sprite(second) = &commands[1] else {
        panic!("expected sprite draw")
    };
    assert_eq!(first.texture_id, SceneTextureId(10));
    assert_eq!(second.texture_id, SceneTextureId(11));
    assert_eq!(
        sprites
            .get(fullscreen_background)
            .unwrap()
            .effective_priority(),
        sprites.get(foreground).unwrap().effective_priority()
    );
}

#[test]
fn transition_or_lock_suppresses_normal_sprite_draw() {
    let mut sprites = make_system();
    let sprite = sprites.create(SpriteDesc {
        texture_id: SceneTextureId(10),
        texture_width: 64,
        texture_height: 32,
        visible: true,
        color: PalColor::from_argb(0x80FF_0000),
        render_mode: PalRenderMode::new(3),
        ..SpriteDesc::new(SceneTextureId(10), 64, 32)
    });
    assert_eq!(sprites.commands().len(), 1);

    sprites.get_mut(sprite).unwrap().transition_block = 1;
    assert!(sprites.commands().is_empty());
    sprites.get_mut(sprite).unwrap().transition_block = 0;

    assert!(sprites.lock(sprite).is_some());
    assert!(sprites.commands().is_empty());
    assert!(sprites.unlock(sprite));
    assert_eq!(sprites.commands().len(), 1);
}

#[test]
fn fully_transparent_argb_sprite_does_not_emit_draw_command() {
    let mut sprites = make_system();
    let sprite = sprites.create(SpriteDesc {
        texture_id: SceneTextureId(10),
        texture_width: 64,
        texture_height: 32,
        visible: true,
        color: PalColor::from_argb(0x00FF_FFFF),
        ..SpriteDesc::new(SceneTextureId(10), 64, 32)
    });

    assert!(sprites.commands().is_empty());
    sprites.get_mut(sprite).expect("sprite should exist").color = PalColor::from_argb(0x01FF_FFFF);
    assert_eq!(sprites.commands().len(), 1);
}

#[test]
fn sprite_scale_preserves_visible_source_aspect() {
    let mut sprites = make_system();
    let sprite = sprites.create(SpriteDesc {
        texture_id: SceneTextureId(10),
        texture_width: 64,
        texture_height: 32,
        visible: true,
        position: PalVec3::new(10, 20, 0),
        ..SpriteDesc::new(SceneTextureId(10), 64, 32)
    });
    assert!(sprites.set_center_offset(sprite, 32, 16));
    assert!(sprites.set_scale(sprite, 2.0));

    let DrawCommand::Sprite(draw) = &sprites.commands()[0] else {
        panic!("scaled sprite should draw");
    };
    // PAL.dll scales around the source rect center, so doubling a 64x32 sprite
    // shifts the submitted top-left position by half of the 64x32 growth.
    assert!((draw.dst.x - (-22.0)).abs() < 0.001);
    assert!((draw.dst.y - 4.0).abs() < 0.001);
    assert!((draw.dst.w - 128.0).abs() < 0.001);
    assert!((draw.dst.h - 64.0).abs() < 0.001);
    assert_eq!(draw.center_offset, [-32.0, -16.0]);
}

#[test]
fn offset_rect_and_change_priority_match_pal_sprite_external_semantics() {
    let mut sprites = make_system();
    let sprite = sprites.create(SpriteDesc {
        texture_id: SceneTextureId(10),
        texture_width: 64,
        texture_height: 32,
        visible: true,
        base_priority: 10,
        position: PalPoint3::new(0, 0, 7),
        ..SpriteDesc::new(SceneTextureId(10), 64, 32)
    });

    assert_eq!(sprites.get_width(sprite), Some(64));
    assert_eq!(sprites.get_height(sprite), Some(32));
    assert!(sprites.set_offset_rect(sprite, 8, 4));
    assert_eq!(sprites.get_width(sprite), Some(8));
    assert_eq!(sprites.get_height(sprite), Some(4));

    assert!(sprites.change_priority(sprite, 3));
    let pal_vm::DrawCommand::Sprite(draw) = &sprites.commands()[0] else {
        panic!("expected sprite draw")
    };
    assert_eq!(draw.priority, 10);
    assert_eq!(
        sprites
            .render_nodes()
            .find(|node| node.sprite == sprite)
            .unwrap()
            .priority,
        3
    );
}

#[test]
fn sprite_surface_copy_mix_and_mask_mutate_pixels() {
    let mut sprites = SpriteSystem::new();
    sprites.insert_texture(SceneTexture::rgba8(
        SceneTextureId(20),
        1,
        2,
        2,
        vec![
            10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160,
        ],
    ));
    sprites.insert_texture(SceneTexture::rgba8(
        SceneTextureId(21),
        1,
        2,
        2,
        vec![
            200, 210, 220, 230, 200, 210, 220, 230, 200, 210, 220, 230, 200, 210, 220, 230,
        ],
    ));
    sprites.insert_texture(SceneTexture::rgba8(
        SceneTextureId(22),
        1,
        2,
        2,
        vec![
            128, 0, 0, 255, 128, 0, 0, 255, 128, 0, 0, 255, 128, 0, 0, 255,
        ],
    ));

    let src = sprites.create(SpriteDesc::new(SceneTextureId(20), 2, 2));
    let dst = sprites.create(SpriteDesc::new(SceneTextureId(21), 2, 2));
    let mask = sprites.create(SpriteDesc::new(SceneTextureId(22), 2, 2));

    assert!(sprites.copy_sprite_to_sprite_rgb(dst, 0, 0, src, 0, 0, 1, 1));
    let dst_surface = sprites.surface(SpriteSurfaceId(21)).unwrap();
    assert_eq!(&dst_surface.pixels[0..4], &[10, 20, 30, 40]);

    assert!(sprites.mix_sprite_to_sprite(dst, 1, 0, src, 1, 0, 1, 1));
    let dst_surface = sprites.surface(SpriteSurfaceId(21)).unwrap();
    assert_eq!(&dst_surface.pixels[4..8], &[152, 162, 172, 230]);

    assert!(sprites.mask_alpha(dst, 0, 0, 1, 1, mask, 0, 0));
    let dst_surface = sprites.surface(SpriteSurfaceId(21)).unwrap();
    assert_eq!(dst_surface.pixels[3], 20);
}

#[test]
fn set_buffer_and_paint_update_sprite_surface_generation() {
    let mut sprites = make_system();
    let sprite = sprites.create(SpriteDesc::new(SceneTextureId(10), 64, 32));
    let before = sprites.surface(SpriteSurfaceId(10)).unwrap().generation;

    assert!(sprites.set_buffer(sprite, 1, 1, 1, 1, &[1, 2, 3, 4]));
    let surface = sprites.surface(SpriteSurfaceId(10)).unwrap();
    let idx = ((64 + 1) * 4) as usize;
    assert_eq!(&surface.pixels[idx..idx + 4], &[1, 2, 3, 4]);
    assert!(surface.generation > before);

    assert!(sprites.paint(sprite, 0, 0, PalColor::from_argb(0x8040_2010)));
    let surface = sprites.surface(SpriteSurfaceId(10)).unwrap();
    assert_eq!(&surface.pixels[0..4], &[0x40, 0x20, 0x10, 0x80]);
}

#[test]
fn sprite_transition_handle_marks_and_cancels_participating_sprites() {
    let mut sprites = make_system();
    let from = sprites.create(SpriteDesc {
        texture_id: SceneTextureId(10),
        texture_width: 64,
        texture_height: 32,
        visible: true,
        ..SpriteDesc::new(SceneTextureId(10), 64, 32)
    });
    let to = sprites.create(SpriteDesc {
        texture_id: SceneTextureId(11),
        texture_width: 64,
        texture_height: 32,
        visible: true,
        ..SpriteDesc::new(SceneTextureId(11), 64, 32)
    });
    let transition = sprites.create_transition_handle();

    assert_eq!(sprites.transition_state(transition), 3);
    assert!(sprites.set_transition(transition, 1, Some(from), Some(to), 9, 500, 0));
    assert_eq!(sprites.transition_state(transition), 1);
    assert_eq!(sprites.get(from).unwrap().transition_block, 9);
    assert_eq!(sprites.get(to).unwrap().transition_block, 9);
    assert!(sprites.commands().is_empty());
    assert_eq!(sprites.transition_commands().len(), 2);

    sprites.advance_transitions(250);
    let commands = sprites.transition_commands();
    assert_eq!(commands.len(), 2);
    let DrawCommand::Sprite(to_draw) = &commands[1] else {
        panic!("transition target must be drawn as sprite");
    };
    assert!(to_draw.color[3] > 0.0 && to_draw.color[3] < 1.0);

    sprites.advance_transitions(250);
    assert_eq!(sprites.transition_state(transition), 3);
    assert_eq!(sprites.get(from).unwrap().transition_block, 0);
    assert_eq!(sprites.get(to).unwrap().transition_block, 0);
    assert_eq!(sprites.commands().len(), 2);
    assert!(!sprites.cancel_transition(transition));
    assert!(sprites.release_transition_handle(transition));
}

#[test]
fn screen_copy_sprite_is_hidden_after_transition_finishes() {
    let mut sprites = SpriteSystem::new();
    let backbuffer = sprites
        .create_rgba_sprite(
            4,
            4,
            vec![0; 4 * 4 * 4],
            PalVec3::default(),
            0,
            "screen-copy:backbuffer",
        )
        .expect("backbuffer sprite");
    assert!(sprites.view_ctrl(backbuffer, true));
    let transition = sprites.create_transition_handle();

    assert!(sprites.set_transition(transition, 1, None, Some(backbuffer), 1, 16, 0));
    assert_eq!(sprites.transition_commands().len(), 1);
    sprites.advance_transitions(16);

    assert_eq!(sprites.transition_state(transition), 3);
    assert!(!sprites.get(backbuffer).unwrap().visible);
    assert!(sprites.commands().is_empty());
}

#[test]
fn sprite_fx_effect_state_tracks_set_update_and_release() {
    let mut sprites = make_system();
    let sprite = sprites.create(SpriteDesc::new(SceneTextureId(10), 64, 32));

    assert_eq!(sprites.sprite_fx_effect_state(sprite), 0);
    assert!(sprites.set_sprite_fx_effect(sprite, 4, 1, 100));
    assert_eq!(sprites.sprite_fx_effect_state(sprite), 1);
    assert!(!sprites.set_sprite_fx_effect(sprite, 4, 1, 100));
    assert!(sprites.set_sprite_fx_effect(sprite, 4, 2, 200));
    assert!(sprites.release_sprite_fx_effect(sprite));
    assert_eq!(sprites.sprite_fx_effect_state(sprite), 0);
}

#[test]
fn named_ani_motion_lanes_share_one_motion_entry_and_clear_on_completion() {
    let mut sprites = make_system();
    let sprite = sprites.create(SpriteDesc {
        texture_id: SceneTextureId(10),
        texture_width: 64,
        texture_height: 32,
        visible: true,
        ..SpriteDesc::new(SceneTextureId(10), 64, 32)
    });

    assert!(sprites.tween_pos_by(sprite, 30.0, 12.0, 3.0, 100));
    assert!(sprites.tween_scale_to(sprite, 2.0, 100));
    assert!(sprites.tween_alpha_to(sprite, 64, 100));
    assert_eq!(sprites.motion_entry_count(), 1);

    sprites.advance_motion_entries(50);
    let mid = sprites.get(sprite).unwrap();
    assert_eq!(mid.position, PalVec3::from_f32(15.0, 6.0, 1.5));
    assert_eq!(mid.scale, 1.5);
    assert_eq!(mid.color.alpha(), 160);
    assert_eq!(sprites.motion_entry_count(), 1);

    sprites.advance_motion_entries(50);
    let done = sprites.get(sprite).unwrap();
    assert_eq!(done.position, PalVec3::from_f32(30.0, 12.0, 3.0));
    assert_eq!(done.scale, 2.0);
    assert_eq!(done.color.alpha(), 64);
    assert_eq!(sprites.motion_entry_count(), 0);
}
