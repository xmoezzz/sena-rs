#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimaryOpcode {
    pub opcode: u16,
    pub name: &'static str,
    pub argc: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtOpcode {
    pub category: u16,
    pub index: u16,
    pub name: Option<&'static str>,
}

pub fn primary_opcode(opcode: u16) -> Option<PrimaryOpcode> {
    match opcode {
        1 => Some(PrimaryOpcode {
            opcode: 1,
            name: "mov",
            argc: 2,
        }),
        2 => Some(PrimaryOpcode {
            opcode: 2,
            name: "add",
            argc: 2,
        }),
        3 => Some(PrimaryOpcode {
            opcode: 3,
            name: "sub",
            argc: 2,
        }),
        4 => Some(PrimaryOpcode {
            opcode: 4,
            name: "mul",
            argc: 2,
        }),
        5 => Some(PrimaryOpcode {
            opcode: 5,
            name: "div",
            argc: 2,
        }),
        6 => Some(PrimaryOpcode {
            opcode: 6,
            name: "bitand",
            argc: 2,
        }),
        7 => Some(PrimaryOpcode {
            opcode: 7,
            name: "bitor",
            argc: 2,
        }),
        8 => Some(PrimaryOpcode {
            opcode: 8,
            name: "bitxor",
            argc: 2,
        }),
        9 => Some(PrimaryOpcode {
            opcode: 9,
            name: "jmp_point",
            argc: 1,
        }),
        10 => Some(PrimaryOpcode {
            opcode: 10,
            name: "jf_point",
            argc: 2,
        }),
        11 => Some(PrimaryOpcode {
            opcode: 11,
            name: "gosub_point",
            argc: 1,
        }),
        12 => Some(PrimaryOpcode {
            opcode: 12,
            name: "eq",
            argc: 2,
        }),
        13 => Some(PrimaryOpcode {
            opcode: 13,
            name: "ne",
            argc: 2,
        }),
        14 => Some(PrimaryOpcode {
            opcode: 14,
            name: "le",
            argc: 2,
        }),
        15 => Some(PrimaryOpcode {
            opcode: 15,
            name: "ge",
            argc: 2,
        }),
        16 => Some(PrimaryOpcode {
            opcode: 16,
            name: "lt",
            argc: 2,
        }),
        17 => Some(PrimaryOpcode {
            opcode: 17,
            name: "gt",
            argc: 2,
        }),
        18 => Some(PrimaryOpcode {
            opcode: 18,
            name: "lor",
            argc: 2,
        }),
        19 => Some(PrimaryOpcode {
            opcode: 19,
            name: "land",
            argc: 2,
        }),
        20 => Some(PrimaryOpcode {
            opcode: 20,
            name: "lnot_slot",
            argc: 1,
        }),
        21 => Some(PrimaryOpcode {
            opcode: 21,
            name: "end",
            argc: 0,
        }),
        22 => Some(PrimaryOpcode {
            opcode: 22,
            name: "nop",
            argc: 0,
        }),
        23 => Some(PrimaryOpcode {
            opcode: 23,
            name: "extcall",
            argc: 2,
        }),
        24 => Some(PrimaryOpcode {
            opcode: 24,
            name: "ret",
            argc: 0,
        }),
        25 => Some(PrimaryOpcode {
            opcode: 25,
            name: "reset_adv",
            argc: 0,
        }),
        26 => Some(PrimaryOpcode {
            opcode: 26,
            name: "mod",
            argc: 2,
        }),
        27 => Some(PrimaryOpcode {
            opcode: 27,
            name: "shl",
            argc: 2,
        }),
        28 => Some(PrimaryOpcode {
            opcode: 28,
            name: "shr",
            argc: 2,
        }),
        29 => Some(PrimaryOpcode {
            opcode: 29,
            name: "neg_slot",
            argc: 1,
        }),
        30 => Some(PrimaryOpcode {
            opcode: 30,
            name: "pop",
            argc: 1,
        }),
        31 => Some(PrimaryOpcode {
            opcode: 31,
            name: "push",
            argc: 1,
        }),
        32 => Some(PrimaryOpcode {
            opcode: 32,
            name: "pack_args",
            argc: 1,
        }),
        33 => Some(PrimaryOpcode {
            opcode: 33,
            name: "drop_args",
            argc: 1,
        }),
        35 => Some(PrimaryOpcode {
            opcode: 35,
            name: "create_message",
            argc: 0,
        }),
        36 => Some(PrimaryOpcode {
            opcode: 36,
            name: "get_message",
            argc: 0,
        }),
        37 => Some(PrimaryOpcode {
            opcode: 37,
            name: "get_message_param",
            argc: 0,
        }),
        40 => Some(PrimaryOpcode {
            opcode: 40,
            name: "save",
            argc: 0,
        }),
        41 => Some(PrimaryOpcode {
            opcode: 41,
            name: "load",
            argc: 0,
        }),
        42 => Some(PrimaryOpcode {
            opcode: 42,
            name: "save_set_title",
            argc: 0,
        }),
        43 => Some(PrimaryOpcode {
            opcode: 43,
            name: "save_data",
            argc: 0,
        }),
        44 => Some(PrimaryOpcode {
            opcode: 44,
            name: "save_set_thumbnail_size",
            argc: 0,
        }),
        45 => Some(PrimaryOpcode {
            opcode: 45,
            name: "thumbnail_set",
            argc: 0,
        }),
        46 => Some(PrimaryOpcode {
            opcode: 46,
            name: "savetitledraw",
            argc: 0,
        }),
        47 => Some(PrimaryOpcode {
            opcode: 47,
            name: "save_set_font_size",
            argc: 0,
        }),
        48 => Some(PrimaryOpcode {
            opcode: 48,
            name: "getsaveday",
            argc: 0,
        }),
        49 => Some(PrimaryOpcode {
            opcode: 49,
            name: "is_save",
            argc: 0,
        }),
        50 => Some(PrimaryOpcode {
            opcode: 50,
            name: "getsaveusermemory",
            argc: 0,
        }),
        51 => Some(PrimaryOpcode {
            opcode: 51,
            name: "savepoint",
            argc: 0,
        }),
        52 => Some(PrimaryOpcode {
            opcode: 52,
            name: "save_thumbnail_mosaic_set",
            argc: 0,
        }),
        53 => Some(PrimaryOpcode {
            opcode: 53,
            name: "savetimedraw",
            argc: 0,
        }),
        54 => Some(PrimaryOpcode {
            opcode: 54,
            name: "savedaydraw",
            argc: 0,
        }),
        55 => Some(PrimaryOpcode {
            opcode: 55,
            name: "save_set_text_rect",
            argc: 0,
        }),
        56 => Some(PrimaryOpcode {
            opcode: 56,
            name: "savetextdraw",
            argc: 0,
        }),
        57 => Some(PrimaryOpcode {
            opcode: 57,
            name: "get_new_savefile",
            argc: 0,
        }),
        61 => Some(PrimaryOpcode {
            opcode: 61,
            name: "setsavetext",
            argc: 0,
        }),
        62 => Some(PrimaryOpcode {
            opcode: 62,
            name: "thumbnail_renew",
            argc: 0,
        }),
        63 => Some(PrimaryOpcode {
            opcode: 63,
            name: "save_set_font_type",
            argc: 0,
        }),
        64 => Some(PrimaryOpcode {
            opcode: 64,
            name: "set_load_after_process",
            argc: 0,
        }),
        65 => Some(PrimaryOpcode {
            opcode: 65,
            name: "savesystemdata",
            argc: 0,
        }),
        66 => Some(PrimaryOpcode {
            opcode: 66,
            name: "save_set_font_effect",
            argc: 0,
        }),
        67 => Some(PrimaryOpcode {
            opcode: 67,
            name: "save_set_font_color_0x_0x",
            argc: 0,
        }),
        68 => Some(PrimaryOpcode {
            opcode: 68,
            name: "delete_file",
            argc: 0,
        }),
        69 => Some(PrimaryOpcode {
            opcode: 69,
            name: "save_tmp_dat",
            argc: 0,
        }),
        70 => Some(PrimaryOpcode {
            opcode: 70,
            name: "copy_file",
            argc: 0,
        }),
        71 => Some(PrimaryOpcode {
            opcode: 71,
            name: "load_thumbnail",
            argc: 0,
        }),
        72 => Some(PrimaryOpcode {
            opcode: 72,
            name: "save_lock_not_open_savefileno",
            argc: 0,
        }),
        73 => Some(PrimaryOpcode {
            opcode: 73,
            name: "is_save_lock",
            argc: 0,
        }),
        74 => Some(PrimaryOpcode {
            opcode: 74,
            name: "is_prev_data",
            argc: 0,
        }),
        75 => Some(PrimaryOpcode {
            opcode: 75,
            name: "save_point_clear",
            argc: 0,
        }),
        76 => Some(PrimaryOpcode {
            opcode: 76,
            name: "save_point_lock",
            argc: 0,
        }),
        77 => Some(PrimaryOpcode {
            opcode: 77,
            name: "op_077",
            argc: 0,
        }),
        78 => Some(PrimaryOpcode {
            opcode: 78,
            name: "histload",
            argc: 0,
        }),
        80 => Some(PrimaryOpcode {
            opcode: 80,
            name: "se_load",
            argc: 0,
        }),
        81 => Some(PrimaryOpcode {
            opcode: 81,
            name: "se_play",
            argc: 0,
        }),
        82 => Some(PrimaryOpcode {
            opcode: 82,
            name: "se_play_ex_ch",
            argc: 0,
        }),
        83 => Some(PrimaryOpcode {
            opcode: 83,
            name: "se_stop",
            argc: 0,
        }),
        84 => Some(PrimaryOpcode {
            opcode: 84,
            name: "se_set_volume",
            argc: 0,
        }),
        85 => Some(PrimaryOpcode {
            opcode: 85,
            name: "se_get_volume",
            argc: 0,
        }),
        86 => Some(PrimaryOpcode {
            opcode: 86,
            name: "se_unload",
            argc: 0,
        }),
        87 => Some(PrimaryOpcode {
            opcode: 87,
            name: "se_wait",
            argc: 0,
        }),
        88 => Some(PrimaryOpcode {
            opcode: 88,
            name: "channel_error_set_se_info",
            argc: 0,
        }),
        89 => Some(PrimaryOpcode {
            opcode: 89,
            name: "get_se_ex_volume",
            argc: 0,
        }),
        90 => Some(PrimaryOpcode {
            opcode: 90,
            name: "channel_error_set_se_ex_volume",
            argc: 0,
        }),
        91 => Some(PrimaryOpcode {
            opcode: 91,
            name: "channel_error_se_enable",
            argc: 0,
        }),
        92 => Some(PrimaryOpcode {
            opcode: 92,
            name: "channel_error_is_se_enable",
            argc: 0,
        }),
        93 => Some(PrimaryOpcode {
            opcode: 93,
            name: "se_set_pan",
            argc: 0,
        }),
        94 => Some(PrimaryOpcode {
            opcode: 94,
            name: "se_mute",
            argc: 0,
        }),
        96 => Some(PrimaryOpcode {
            opcode: 96,
            name: "select_init",
            argc: 0,
        }),
        97 => Some(PrimaryOpcode {
            opcode: 97,
            name: "select",
            argc: 0,
        }),
        98 => Some(PrimaryOpcode {
            opcode: 98,
            name: "op_098",
            argc: 0,
        }),
        99 => Some(PrimaryOpcode {
            opcode: 99,
            name: "op_099",
            argc: 0,
        }),
        100 => Some(PrimaryOpcode {
            opcode: 100,
            name: "select_clear",
            argc: 0,
        }),
        101 => Some(PrimaryOpcode {
            opcode: 101,
            name: "select_set_offset",
            argc: 0,
        }),
        102 => Some(PrimaryOpcode {
            opcode: 102,
            name: "select_set_process",
            argc: 0,
        }),
        103 => Some(PrimaryOpcode {
            opcode: 103,
            name: "select_lock",
            argc: 0,
        }),
        104 => Some(PrimaryOpcode {
            opcode: 104,
            name: "get_select_on_key",
            argc: 0,
        }),
        105 => Some(PrimaryOpcode {
            opcode: 105,
            name: "get_select_pull_key",
            argc: 0,
        }),
        106 => Some(PrimaryOpcode {
            opcode: 106,
            name: "get_select_push_key",
            argc: 0,
        }),
        108 => Some(PrimaryOpcode {
            opcode: 108,
            name: "skip_set",
            argc: 0,
        }),
        109 => Some(PrimaryOpcode {
            opcode: 109,
            name: "skip_is",
            argc: 0,
        }),
        110 => Some(PrimaryOpcode {
            opcode: 110,
            name: "auto_set",
            argc: 0,
        }),
        111 => Some(PrimaryOpcode {
            opcode: 111,
            name: "auto_is",
            argc: 0,
        }),
        112 => Some(PrimaryOpcode {
            opcode: 112,
            name: "op_112",
            argc: 0,
        }),
        113 => Some(PrimaryOpcode {
            opcode: 113,
            name: "auto_get_time",
            argc: 0,
        }),
        114 => Some(PrimaryOpcode {
            opcode: 114,
            name: "op_114",
            argc: 0,
        }),
        115 => Some(PrimaryOpcode {
            opcode: 115,
            name: "op_115",
            argc: 0,
        }),
        116 => Some(PrimaryOpcode {
            opcode: 116,
            name: "op_116",
            argc: 0,
        }),
        117 => Some(PrimaryOpcode {
            opcode: 117,
            name: "op_117",
            argc: 0,
        }),
        118 => Some(PrimaryOpcode {
            opcode: 118,
            name: "op_118",
            argc: 0,
        }),
        119 => Some(PrimaryOpcode {
            opcode: 119,
            name: "op_119",
            argc: 0,
        }),
        120 => Some(PrimaryOpcode {
            opcode: 120,
            name: "op_120",
            argc: 0,
        }),
        121 => Some(PrimaryOpcode {
            opcode: 121,
            name: "op_121",
            argc: 0,
        }),
        122 => Some(PrimaryOpcode {
            opcode: 122,
            name: "op_122",
            argc: 0,
        }),
        123 => Some(PrimaryOpcode {
            opcode: 123,
            name: "load_font",
            argc: 0,
        }),
        124 => Some(PrimaryOpcode {
            opcode: 124,
            name: "unload_font",
            argc: 0,
        }),
        125 => Some(PrimaryOpcode {
            opcode: 125,
            name: "set_language",
            argc: 0,
        }),
        126 => Some(PrimaryOpcode {
            opcode: 126,
            name: "key_canncel",
            argc: 0,
        }),
        127 => Some(PrimaryOpcode {
            opcode: 127,
            name: "set_font_color",
            argc: 0,
        }),
        128 => Some(PrimaryOpcode {
            opcode: 128,
            name: "load_font_ex",
            argc: 0,
        }),
        129 => Some(PrimaryOpcode {
            opcode: 129,
            name: "op_129",
            argc: 0,
        }),
        130 => Some(PrimaryOpcode {
            opcode: 130,
            name: "op_130",
            argc: 0,
        }),
        131 => Some(PrimaryOpcode {
            opcode: 131,
            name: "op_131",
            argc: 0,
        }),
        132 => Some(PrimaryOpcode {
            opcode: 132,
            name: "op_132",
            argc: 0,
        }),
        133 => Some(PrimaryOpcode {
            opcode: 133,
            name: "op_133",
            argc: 0,
        }),
        134 => Some(PrimaryOpcode {
            opcode: 134,
            name: "op_134",
            argc: 0,
        }),
        135 => Some(PrimaryOpcode {
            opcode: 135,
            name: "set_font_size",
            argc: 0,
        }),
        136 => Some(PrimaryOpcode {
            opcode: 136,
            name: "get_font_size",
            argc: 0,
        }),
        137 => Some(PrimaryOpcode {
            opcode: 137,
            name: "get_font_type",
            argc: 0,
        }),
        138 => Some(PrimaryOpcode {
            opcode: 138,
            name: "set_font_effect",
            argc: 0,
        }),
        139 => Some(PrimaryOpcode {
            opcode: 139,
            name: "get_font_effect",
            argc: 0,
        }),
        140 => Some(PrimaryOpcode {
            opcode: 140,
            name: "get_pull_key",
            argc: 0,
        }),
        141 => Some(PrimaryOpcode {
            opcode: 141,
            name: "get_on_key",
            argc: 0,
        }),
        142 => Some(PrimaryOpcode {
            opcode: 142,
            name: "get_push_key",
            argc: 0,
        }),
        143 => Some(PrimaryOpcode {
            opcode: 143,
            name: "input_clear",
            argc: 0,
        }),
        144 => Some(PrimaryOpcode {
            opcode: 144,
            name: "change_window_size",
            argc: 0,
        }),
        145 => Some(PrimaryOpcode {
            opcode: 145,
            name: "change_aspect_mode",
            argc: 0,
        }),
        146 => Some(PrimaryOpcode {
            opcode: 146,
            name: "aspect_position_enable",
            argc: 0,
        }),
        147 => Some(PrimaryOpcode {
            opcode: 147,
            name: "op_147",
            argc: 0,
        }),
        148 => Some(PrimaryOpcode {
            opcode: 148,
            name: "get_aspect_mode",
            argc: 0,
        }),
        149 => Some(PrimaryOpcode {
            opcode: 149,
            name: "get_monitor_size",
            argc: 0,
        }),
        150 => Some(PrimaryOpcode {
            opcode: 150,
            name: "op_150",
            argc: 0,
        }),
        151 => Some(PrimaryOpcode {
            opcode: 151,
            name: "get_system_metrics",
            argc: 0,
        }),
        152 => Some(PrimaryOpcode {
            opcode: 152,
            name: "set_system_path",
            argc: 0,
        }),
        153 => Some(PrimaryOpcode {
            opcode: 153,
            name: "set_allmosaicthumbnail",
            argc: 0,
        }),
        154 => Some(PrimaryOpcode {
            opcode: 154,
            name: "enable_window_change",
            argc: 0,
        }),
        155 => Some(PrimaryOpcode {
            opcode: 155,
            name: "is_enable_window_change",
            argc: 0,
        }),
        156 => Some(PrimaryOpcode {
            opcode: 156,
            name: "set_cursor_null",
            argc: 0,
        }),
        157 => Some(PrimaryOpcode {
            opcode: 157,
            name: "set_hide_cursor_time",
            argc: 0,
        }),
        158 => Some(PrimaryOpcode {
            opcode: 158,
            name: "get_hide_cursor_time",
            argc: 0,
        }),
        159 => Some(PrimaryOpcode {
            opcode: 159,
            name: "scene_skip",
            argc: 0,
        }),
        160 => Some(PrimaryOpcode {
            opcode: 160,
            name: "op_160",
            argc: 0,
        }),
        161 => Some(PrimaryOpcode {
            opcode: 161,
            name: "op_161",
            argc: 0,
        }),
        162 => Some(PrimaryOpcode {
            opcode: 162,
            name: "get_async_key",
            argc: 0,
        }),
        163 => Some(PrimaryOpcode {
            opcode: 163,
            name: "get_font_color",
            argc: 0,
        }),
        164 => Some(PrimaryOpcode {
            opcode: 164,
            name: "op_164",
            argc: 0,
        }),
        165 => Some(PrimaryOpcode {
            opcode: 165,
            name: "history_skip",
            argc: 0,
        }),
        166 => Some(PrimaryOpcode {
            opcode: 166,
            name: "op_166",
            argc: 0,
        }),
        167 => Some(PrimaryOpcode {
            opcode: 167,
            name: "op_167",
            argc: 0,
        }),
        168 => Some(PrimaryOpcode {
            opcode: 168,
            name: "set_language",
            argc: 0,
        }),
        169 => Some(PrimaryOpcode {
            opcode: 169,
            name: "set_achievement",
            argc: 0,
        }),
        171 => Some(PrimaryOpcode {
            opcode: 171,
            name: "system_btn_set",
            argc: 0,
        }),
        172 => Some(PrimaryOpcode {
            opcode: 172,
            name: "system_btn_release",
            argc: 0,
        }),
        173 => Some(PrimaryOpcode {
            opcode: 173,
            name: "system_btn_enable",
            argc: 0,
        }),
        176 => Some(PrimaryOpcode {
            opcode: 176,
            name: "text_init",
            argc: 0,
        }),
        177 => Some(PrimaryOpcode {
            opcode: 177,
            name: "text_set_icon",
            argc: 0,
        }),
        178 => Some(PrimaryOpcode {
            opcode: 178,
            name: "text",
            argc: 0,
        }),
        179 => Some(PrimaryOpcode {
            opcode: 179,
            name: "text_hide",
            argc: 0,
        }),
        180 => Some(PrimaryOpcode {
            opcode: 180,
            name: "text_show",
            argc: 0,
        }),
        181 => Some(PrimaryOpcode {
            opcode: 181,
            name: "text_set_btn",
            argc: 0,
        }),
        182 => Some(PrimaryOpcode {
            opcode: 182,
            name: "text_uninit",
            argc: 0,
        }),
        183 => Some(PrimaryOpcode {
            opcode: 183,
            name: "text_set_rect_invalid_param",
            argc: 0,
        }),
        184 => Some(PrimaryOpcode {
            opcode: 184,
            name: "text_clear",
            argc: 0,
        }),
        185 => Some(PrimaryOpcode {
            opcode: 185,
            name: "op_185",
            argc: 0,
        }),
        186 => Some(PrimaryOpcode {
            opcode: 186,
            name: "text_get_time",
            argc: 0,
        }),
        187 => Some(PrimaryOpcode {
            opcode: 187,
            name: "text_window_set_alpha",
            argc: 0,
        }),
        188 => Some(PrimaryOpcode {
            opcode: 188,
            name: "text_voice_play",
            argc: 0,
        }),
        189 => Some(PrimaryOpcode {
            opcode: 189,
            name: "op_189",
            argc: 0,
        }),
        190 => Some(PrimaryOpcode {
            opcode: 190,
            name: "text_set_icon_animation_time",
            argc: 0,
        }),
        191 => Some(PrimaryOpcode {
            opcode: 191,
            name: "text_w",
            argc: 0,
        }),
        192 => Some(PrimaryOpcode {
            opcode: 192,
            name: "text_a",
            argc: 0,
        }),
        193 => Some(PrimaryOpcode {
            opcode: 193,
            name: "text_wa",
            argc: 0,
        }),
        194 => Some(PrimaryOpcode {
            opcode: 194,
            name: "text_n",
            argc: 0,
        }),
        195 => Some(PrimaryOpcode {
            opcode: 195,
            name: "text_cat",
            argc: 0,
        }),
        196 => Some(PrimaryOpcode {
            opcode: 196,
            name: "set_history",
            argc: 0,
        }),
        197 => Some(PrimaryOpcode {
            opcode: 197,
            name: "is_text_visible",
            argc: 0,
        }),
        198 => Some(PrimaryOpcode {
            opcode: 198,
            name: "text_set_base",
            argc: 0,
        }),
        199 => Some(PrimaryOpcode {
            opcode: 199,
            name: "enable_voice_cut",
            argc: 0,
        }),
        200 => Some(PrimaryOpcode {
            opcode: 200,
            name: "is_voice_cut",
            argc: 0,
        }),
        201 => Some(PrimaryOpcode {
            opcode: 201,
            name: "texttimecheckset",
            argc: 0,
        }),
        202 => Some(PrimaryOpcode {
            opcode: 202,
            name: "op_202",
            argc: 0,
        }),
        203 => Some(PrimaryOpcode {
            opcode: 203,
            name: "op_203",
            argc: 0,
        }),
        204 => Some(PrimaryOpcode {
            opcode: 204,
            name: "text_set_color",
            argc: 0,
        }),
        205 => Some(PrimaryOpcode {
            opcode: 205,
            name: "textredraw",
            argc: 0,
        }),
        206 => Some(PrimaryOpcode {
            opcode: 206,
            name: "set_text_mode",
            argc: 0,
        }),
        207 => Some(PrimaryOpcode {
            opcode: 207,
            name: "text_init_visualnovelmode",
            argc: 0,
        }),
        208 => Some(PrimaryOpcode {
            opcode: 208,
            name: "text_set_icon_mode",
            argc: 0,
        }),
        209 => Some(PrimaryOpcode {
            opcode: 209,
            name: "text_vn_br",
            argc: 0,
        }),
        210 => Some(PrimaryOpcode {
            opcode: 210,
            name: "op_210",
            argc: 0,
        }),
        211 => Some(PrimaryOpcode {
            opcode: 211,
            name: "op_211",
            argc: 0,
        }),
        212 => Some(PrimaryOpcode {
            opcode: 212,
            name: "op_212",
            argc: 0,
        }),
        213 => Some(PrimaryOpcode {
            opcode: 213,
            name: "op_213",
            argc: 0,
        }),
        214 => Some(PrimaryOpcode {
            opcode: 214,
            name: "tips_get_str",
            argc: 0,
        }),
        215 => Some(PrimaryOpcode {
            opcode: 215,
            name: "tips_get_param",
            argc: 0,
        }),
        216 => Some(PrimaryOpcode {
            opcode: 216,
            name: "tips_reset",
            argc: 0,
        }),
        217 => Some(PrimaryOpcode {
            opcode: 217,
            name: "tips_search",
            argc: 0,
        }),
        218 => Some(PrimaryOpcode {
            opcode: 218,
            name: "tips_set_color",
            argc: 0,
        }),
        219 => Some(PrimaryOpcode {
            opcode: 219,
            name: "tips_stop",
            argc: 0,
        }),
        220 => Some(PrimaryOpcode {
            opcode: 220,
            name: "tips_get_flag",
            argc: 0,
        }),
        221 => Some(PrimaryOpcode {
            opcode: 221,
            name: "tips_init",
            argc: 0,
        }),
        222 => Some(PrimaryOpcode {
            opcode: 222,
            name: "tips_pause",
            argc: 0,
        }),
        224 => Some(PrimaryOpcode {
            opcode: 224,
            name: "voice_play",
            argc: 0,
        }),
        225 => Some(PrimaryOpcode {
            opcode: 225,
            name: "voice_stop",
            argc: 0,
        }),
        226 => Some(PrimaryOpcode {
            opcode: 226,
            name: "voice_set_volume",
            argc: 0,
        }),
        227 => Some(PrimaryOpcode {
            opcode: 227,
            name: "voice_get_volume",
            argc: 0,
        }),
        228 => Some(PrimaryOpcode {
            opcode: 228,
            name: "set_voice_info",
            argc: 0,
        }),
        229 => Some(PrimaryOpcode {
            opcode: 229,
            name: "voice_enable",
            argc: 0,
        }),
        230 => Some(PrimaryOpcode {
            opcode: 230,
            name: "is_voice_enable",
            argc: 0,
        }),
        231 => Some(PrimaryOpcode {
            opcode: 231,
            name: "op_231",
            argc: 0,
        }),
        232 => Some(PrimaryOpcode {
            opcode: 232,
            name: "bgv_play",
            argc: 0,
        }),
        233 => Some(PrimaryOpcode {
            opcode: 233,
            name: "bgv_stop",
            argc: 0,
        }),
        234 => Some(PrimaryOpcode {
            opcode: 234,
            name: "bgv_enable",
            argc: 0,
        }),
        235 => Some(PrimaryOpcode {
            opcode: 235,
            name: "get_voice_ex_volume",
            argc: 0,
        }),
        236 => Some(PrimaryOpcode {
            opcode: 236,
            name: "set_voice_ex_volume",
            argc: 0,
        }),
        237 => Some(PrimaryOpcode {
            opcode: 237,
            name: "voice_check_enable",
            argc: 0,
        }),
        238 => Some(PrimaryOpcode {
            opcode: 238,
            name: "voice_autopan_initialize",
            argc: 0,
        }),
        239 => Some(PrimaryOpcode {
            opcode: 239,
            name: "voice_autopan_enable",
            argc: 0,
        }),
        240 => Some(PrimaryOpcode {
            opcode: 240,
            name: "set_voice_autopan_size_over",
            argc: 0,
        }),
        241 => Some(PrimaryOpcode {
            opcode: 241,
            name: "is_voice_autopan_enable",
            argc: 0,
        }),
        242 => Some(PrimaryOpcode {
            opcode: 242,
            name: "voice_wait",
            argc: 0,
        }),
        243 => Some(PrimaryOpcode {
            opcode: 243,
            name: "bgv_pause",
            argc: 0,
        }),
        244 => Some(PrimaryOpcode {
            opcode: 244,
            name: "bgv_mute",
            argc: 0,
        }),
        245 => Some(PrimaryOpcode {
            opcode: 245,
            name: "set_bgv_volume",
            argc: 0,
        }),
        246 => Some(PrimaryOpcode {
            opcode: 246,
            name: "get_bgv_volume",
            argc: 0,
        }),
        247 => Some(PrimaryOpcode {
            opcode: 247,
            name: "set_bgv_auto_volume",
            argc: 0,
        }),
        248 => Some(PrimaryOpcode {
            opcode: 248,
            name: "voice_mute",
            argc: 0,
        }),
        249 => Some(PrimaryOpcode {
            opcode: 249,
            name: "voice_call",
            argc: 0,
        }),
        250 => Some(PrimaryOpcode {
            opcode: 250,
            name: "voice_call_clear",
            argc: 0,
        }),
        252 => Some(PrimaryOpcode {
            opcode: 252,
            name: "wait",
            argc: 0,
        }),
        253 => Some(PrimaryOpcode {
            opcode: 253,
            name: "wait_click",
            argc: 0,
        }),
        254 => Some(PrimaryOpcode {
            opcode: 254,
            name: "wait_sync_begin",
            argc: 0,
        }),
        255 => Some(PrimaryOpcode {
            opcode: 255,
            name: "wait_sync_release",
            argc: 0,
        }),
        256 => Some(PrimaryOpcode {
            opcode: 256,
            name: "wait_sync_end",
            argc: 0,
        }),
        257 => Some(PrimaryOpcode {
            opcode: 257,
            name: "op_257",
            argc: 0,
        }),
        258 => Some(PrimaryOpcode {
            opcode: 258,
            name: "wait_clear",
            argc: 0,
        }),
        259 => Some(PrimaryOpcode {
            opcode: 259,
            name: "wait_click_no_anim",
            argc: 0,
        }),
        260 => Some(PrimaryOpcode {
            opcode: 260,
            name: "wait_sync_get_time",
            argc: 0,
        }),
        261 => Some(PrimaryOpcode {
            opcode: 261,
            name: "wait_time_push",
            argc: 0,
        }),
        262 => Some(PrimaryOpcode {
            opcode: 262,
            name: "wait_time_pop",
            argc: 0,
        }),
        _ => None,
    }
}

pub fn ext_opcode(category: u16, index: u16) -> Option<ExtOpcode> {
    match (category, index) {
        (2, 0) => Some(ExtOpcode {
            category: 2,
            index: 0,
            name: Some("text_init"),
        }),
        (2, 1) => Some(ExtOpcode {
            category: 2,
            index: 1,
            name: Some("text_set_icon"),
        }),
        (2, 2) => Some(ExtOpcode {
            category: 2,
            index: 2,
            name: Some("text"),
        }),
        (2, 3) => Some(ExtOpcode {
            category: 2,
            index: 3,
            name: Some("text_hide"),
        }),
        (2, 4) => Some(ExtOpcode {
            category: 2,
            index: 4,
            name: Some("text_show"),
        }),
        (2, 5) => Some(ExtOpcode {
            category: 2,
            index: 5,
            name: Some("text_set_btn"),
        }),
        (2, 6) => Some(ExtOpcode {
            category: 2,
            index: 6,
            name: Some("text_uninit"),
        }),
        (2, 7) => Some(ExtOpcode {
            category: 2,
            index: 7,
            name: Some("text_set_rect_invalid_param"),
        }),
        (2, 8) => Some(ExtOpcode {
            category: 2,
            index: 8,
            name: Some("text_clear"),
        }),
        (2, 9) => Some(ExtOpcode {
            category: 2,
            index: 9,
            name: None,
        }),
        (2, 10) => Some(ExtOpcode {
            category: 2,
            index: 10,
            name: Some("text_get_time"),
        }),
        (2, 11) => Some(ExtOpcode {
            category: 2,
            index: 11,
            name: Some("text_window_set_alpha"),
        }),
        (2, 12) => Some(ExtOpcode {
            category: 2,
            index: 12,
            name: Some("text_voice_play"),
        }),
        (2, 13) => Some(ExtOpcode {
            category: 2,
            index: 13,
            name: None,
        }),
        (2, 14) => Some(ExtOpcode {
            category: 2,
            index: 14,
            name: Some("text_set_icon_animation_time"),
        }),
        (2, 15) => Some(ExtOpcode {
            category: 2,
            index: 15,
            name: Some("text_w"),
        }),
        (2, 16) => Some(ExtOpcode {
            category: 2,
            index: 16,
            name: Some("text_a"),
        }),
        (2, 17) => Some(ExtOpcode {
            category: 2,
            index: 17,
            name: Some("text_wa"),
        }),
        (2, 18) => Some(ExtOpcode {
            category: 2,
            index: 18,
            name: Some("text_n"),
        }),
        (2, 19) => Some(ExtOpcode {
            category: 2,
            index: 19,
            name: Some("text_cat"),
        }),
        (2, 20) => Some(ExtOpcode {
            category: 2,
            index: 20,
            name: Some("set_history"),
        }),
        (2, 21) => Some(ExtOpcode {
            category: 2,
            index: 21,
            name: Some("is_text_visible"),
        }),
        (2, 22) => Some(ExtOpcode {
            category: 2,
            index: 22,
            name: Some("text_set_base"),
        }),
        (2, 23) => Some(ExtOpcode {
            category: 2,
            index: 23,
            name: Some("enable_voice_cut"),
        }),
        (2, 24) => Some(ExtOpcode {
            category: 2,
            index: 24,
            name: Some("is_voice_cut"),
        }),
        (2, 25) => Some(ExtOpcode {
            category: 2,
            index: 25,
            name: Some("texttimecheckset"),
        }),
        (2, 26) => Some(ExtOpcode {
            category: 2,
            index: 26,
            name: None,
        }),
        (2, 27) => Some(ExtOpcode {
            category: 2,
            index: 27,
            name: None,
        }),
        (2, 28) => Some(ExtOpcode {
            category: 2,
            index: 28,
            name: Some("text_set_color"),
        }),
        (2, 29) => Some(ExtOpcode {
            category: 2,
            index: 29,
            name: Some("textredraw"),
        }),
        (2, 30) => Some(ExtOpcode {
            category: 2,
            index: 30,
            name: Some("set_text_mode"),
        }),
        (2, 31) => Some(ExtOpcode {
            category: 2,
            index: 31,
            name: Some("text_init_visualnovelmode"),
        }),
        (2, 32) => Some(ExtOpcode {
            category: 2,
            index: 32,
            name: Some("text_set_icon_mode"),
        }),
        (2, 33) => Some(ExtOpcode {
            category: 2,
            index: 33,
            name: Some("text_vn_br"),
        }),
        (2, 34) => Some(ExtOpcode {
            category: 2,
            index: 34,
            name: None,
        }),
        (2, 35) => Some(ExtOpcode {
            category: 2,
            index: 35,
            name: None,
        }),
        (2, 36) => Some(ExtOpcode {
            category: 2,
            index: 36,
            name: None,
        }),
        (2, 37) => Some(ExtOpcode {
            category: 2,
            index: 37,
            name: None,
        }),
        (2, 38) => Some(ExtOpcode {
            category: 2,
            index: 38,
            name: Some("tips_get_str"),
        }),
        (2, 39) => Some(ExtOpcode {
            category: 2,
            index: 39,
            name: Some("tips_get_param"),
        }),
        (2, 40) => Some(ExtOpcode {
            category: 2,
            index: 40,
            name: Some("tips_reset"),
        }),
        (2, 41) => Some(ExtOpcode {
            category: 2,
            index: 41,
            name: Some("tips_search"),
        }),
        (2, 42) => Some(ExtOpcode {
            category: 2,
            index: 42,
            name: Some("tips_set_color"),
        }),
        (2, 43) => Some(ExtOpcode {
            category: 2,
            index: 43,
            name: Some("tips_stop"),
        }),
        (2, 44) => Some(ExtOpcode {
            category: 2,
            index: 44,
            name: Some("tips_get_flag"),
        }),
        (2, 45) => Some(ExtOpcode {
            category: 2,
            index: 45,
            name: Some("tips_init"),
        }),
        (2, 46) => Some(ExtOpcode {
            category: 2,
            index: 46,
            name: Some("tips_pause"),
        }),
        (2, 48) => Some(ExtOpcode {
            category: 2,
            index: 48,
            name: Some("voice_play"),
        }),
        (2, 49) => Some(ExtOpcode {
            category: 2,
            index: 49,
            name: Some("voice_stop"),
        }),
        (2, 50) => Some(ExtOpcode {
            category: 2,
            index: 50,
            name: Some("voice_set_volume"),
        }),
        (2, 51) => Some(ExtOpcode {
            category: 2,
            index: 51,
            name: Some("voice_get_volume"),
        }),
        (2, 52) => Some(ExtOpcode {
            category: 2,
            index: 52,
            name: Some("set_voice_info"),
        }),
        (2, 53) => Some(ExtOpcode {
            category: 2,
            index: 53,
            name: Some("voice_enable"),
        }),
        (2, 54) => Some(ExtOpcode {
            category: 2,
            index: 54,
            name: Some("is_voice_enable"),
        }),
        (2, 55) => Some(ExtOpcode {
            category: 2,
            index: 55,
            name: None,
        }),
        (2, 56) => Some(ExtOpcode {
            category: 2,
            index: 56,
            name: Some("bgv_play"),
        }),
        (2, 57) => Some(ExtOpcode {
            category: 2,
            index: 57,
            name: Some("bgv_stop"),
        }),
        (2, 58) => Some(ExtOpcode {
            category: 2,
            index: 58,
            name: Some("bgv_enable"),
        }),
        (2, 59) => Some(ExtOpcode {
            category: 2,
            index: 59,
            name: Some("get_voice_ex_volume"),
        }),
        (2, 60) => Some(ExtOpcode {
            category: 2,
            index: 60,
            name: Some("set_voice_ex_volume"),
        }),
        (2, 61) => Some(ExtOpcode {
            category: 2,
            index: 61,
            name: Some("voice_check_enable"),
        }),
        (2, 62) => Some(ExtOpcode {
            category: 2,
            index: 62,
            name: Some("voice_autopan_initialize"),
        }),
        (2, 63) => Some(ExtOpcode {
            category: 2,
            index: 63,
            name: Some("voice_autopan_enable"),
        }),
        (2, 64) => Some(ExtOpcode {
            category: 2,
            index: 64,
            name: Some("set_voice_autopan_size_over"),
        }),
        (2, 65) => Some(ExtOpcode {
            category: 2,
            index: 65,
            name: Some("is_voice_autopan_enable"),
        }),
        (2, 66) => Some(ExtOpcode {
            category: 2,
            index: 66,
            name: Some("voice_wait"),
        }),
        (2, 67) => Some(ExtOpcode {
            category: 2,
            index: 67,
            name: Some("bgv_pause"),
        }),
        (2, 68) => Some(ExtOpcode {
            category: 2,
            index: 68,
            name: Some("bgv_mute"),
        }),
        (2, 69) => Some(ExtOpcode {
            category: 2,
            index: 69,
            name: Some("set_bgv_volume"),
        }),
        (2, 70) => Some(ExtOpcode {
            category: 2,
            index: 70,
            name: Some("get_bgv_volume"),
        }),
        (2, 71) => Some(ExtOpcode {
            category: 2,
            index: 71,
            name: Some("set_bgv_auto_volume"),
        }),
        (2, 72) => Some(ExtOpcode {
            category: 2,
            index: 72,
            name: Some("voice_mute"),
        }),
        (2, 73) => Some(ExtOpcode {
            category: 2,
            index: 73,
            name: Some("voice_call"),
        }),
        (2, 74) => Some(ExtOpcode {
            category: 2,
            index: 74,
            name: Some("voice_call_clear"),
        }),
        (2, 76) => Some(ExtOpcode {
            category: 2,
            index: 76,
            name: Some("wait"),
        }),
        (2, 77) => Some(ExtOpcode {
            category: 2,
            index: 77,
            name: Some("wait_click"),
        }),
        (2, 78) => Some(ExtOpcode {
            category: 2,
            index: 78,
            name: Some("wait_sync_begin"),
        }),
        (2, 79) => Some(ExtOpcode {
            category: 2,
            index: 79,
            name: Some("wait_sync_release"),
        }),
        (2, 80) => Some(ExtOpcode {
            category: 2,
            index: 80,
            name: Some("wait_sync_end"),
        }),
        (2, 81) => Some(ExtOpcode {
            category: 2,
            index: 81,
            name: None,
        }),
        (2, 82) => Some(ExtOpcode {
            category: 2,
            index: 82,
            name: Some("wait_clear"),
        }),
        (2, 83) => Some(ExtOpcode {
            category: 2,
            index: 83,
            name: Some("wait_click_no_anim"),
        }),
        (2, 84) => Some(ExtOpcode {
            category: 2,
            index: 84,
            name: Some("wait_sync_get_time"),
        }),
        (2, 85) => Some(ExtOpcode {
            category: 2,
            index: 85,
            name: Some("wait_time_push"),
        }),
        (2, 86) => Some(ExtOpcode {
            category: 2,
            index: 86,
            name: Some("wait_time_pop"),
        }),
        (2, 150) => Some(ExtOpcode {
            category: 2,
            index: 150,
            name: None,
        }),
        (2, 153) => Some(ExtOpcode {
            category: 2,
            index: 153,
            name: None,
        }),
        (2, 156) => Some(ExtOpcode {
            category: 2,
            index: 156,
            name: None,
        }),
        (2, 167) => Some(ExtOpcode {
            category: 2,
            index: 167,
            name: None,
        }),
        (2, 218) => Some(ExtOpcode {
            category: 2,
            index: 218,
            name: None,
        }),
        (2, 234) => Some(ExtOpcode {
            category: 2,
            index: 234,
            name: None,
        }),
        (2, 241) => Some(ExtOpcode {
            category: 2,
            index: 241,
            name: None,
        }),
        (2, 248) => Some(ExtOpcode {
            category: 2,
            index: 248,
            name: None,
        }),
        (2, 265) => Some(ExtOpcode {
            category: 2,
            index: 265,
            name: None,
        }),
        (2, 271) => Some(ExtOpcode {
            category: 2,
            index: 271,
            name: None,
        }),
        (2, 283) => Some(ExtOpcode {
            category: 2,
            index: 283,
            name: None,
        }),
        (3, 2) => Some(ExtOpcode {
            category: 3,
            index: 2,
            name: Some("sp_set"),
        }),
        (3, 3) => Some(ExtOpcode {
            category: 3,
            index: 3,
            name: Some("sp_set_ex"),
        }),
        (3, 4) => Some(ExtOpcode {
            category: 3,
            index: 4,
            name: Some("sp_set_pos"),
        }),
        (3, 5) => Some(ExtOpcode {
            category: 3,
            index: 5,
            name: Some("sp_cls"),
        }),
        (3, 6) => Some(ExtOpcode {
            category: 3,
            index: 6,
            name: Some("sp_set_alpha"),
        }),
        (3, 7) => Some(ExtOpcode {
            category: 3,
            index: 7,
            name: Some("set_priority"),
        }),
        (3, 8) => Some(ExtOpcode {
            category: 3,
            index: 8,
            name: None,
        }),
        (3, 9) => Some(ExtOpcode {
            category: 3,
            index: 9,
            name: Some("sp_set_center"),
        }),
        (3, 11) => Some(ExtOpcode {
            category: 3,
            index: 11,
            name: Some("sp_cls_ex"),
        }),
        (3, 12) => Some(ExtOpcode {
            category: 3,
            index: 12,
            name: Some("set_filter"),
        }),
        (3, 13) => Some(ExtOpcode {
            category: 3,
            index: 13,
            name: Some("sp_cls_transition"),
        }),
        (3, 14) => Some(ExtOpcode {
            category: 3,
            index: 14,
            name: Some("sp_move_ex"),
        }),
        (3, 15) => Some(ExtOpcode {
            category: 3,
            index: 15,
            name: Some("sp_set_rect_pos"),
        }),
        (3, 16) => Some(ExtOpcode {
            category: 3,
            index: 16,
            name: None,
        }),
        (3, 17) => Some(ExtOpcode {
            category: 3,
            index: 17,
            name: Some("sp_set_scale"),
        }),
        (3, 18) => Some(ExtOpcode {
            category: 3,
            index: 18,
            name: Some("sp_set_rotate"),
        }),
        (3, 19) => Some(ExtOpcode {
            category: 3,
            index: 19,
            name: Some("face_init"),
        }),
        (3, 20) => Some(ExtOpcode {
            category: 3,
            index: 20,
            name: Some("face_set"),
        }),
        (3, 21) => Some(ExtOpcode {
            category: 3,
            index: 21,
            name: Some("not_image_sp_get_color"),
        }),
        (3, 22) => Some(ExtOpcode {
            category: 3,
            index: 22,
            name: Some("sptext"),
        }),
        (3, 23) => Some(ExtOpcode {
            category: 3,
            index: 23,
            name: Some("face_cls"),
        }),
        (3, 24) => Some(ExtOpcode {
            category: 3,
            index: 24,
            name: Some("sp_set_rect"),
        }),
        (3, 25) => Some(ExtOpcode {
            category: 3,
            index: 25,
            name: Some("sp_set_pos_move"),
        }),
        (3, 26) => Some(ExtOpcode {
            category: 3,
            index: 26,
            name: Some("not_image_sp_get_alpha"),
        }),
        (3, 27) => Some(ExtOpcode {
            category: 3,
            index: 27,
            name: Some("not_image_sp_get_rotate"),
        }),
        (3, 28) => Some(ExtOpcode {
            category: 3,
            index: 28,
            name: None,
        }),
        (3, 29) => Some(ExtOpcode {
            category: 3,
            index: 29,
            name: None,
        }),
        (3, 30) => Some(ExtOpcode {
            category: 3,
            index: 30,
            name: None,
        }),
        (3, 31) => Some(ExtOpcode {
            category: 3,
            index: 31,
            name: None,
        }),
        (3, 32) => Some(ExtOpcode {
            category: 3,
            index: 32,
            name: Some("sp_create"),
        }),
        (3, 33) => Some(ExtOpcode {
            category: 3,
            index: 33,
            name: Some("sp_anime_clear"),
        }),
        (3, 34) => Some(ExtOpcode {
            category: 3,
            index: 34,
            name: None,
        }),
        (3, 35) => Some(ExtOpcode {
            category: 3,
            index: 35,
            name: None,
        }),
        (3, 36) => Some(ExtOpcode {
            category: 3,
            index: 36,
            name: Some("not_image_sp_get_scale"),
        }),
        (3, 37) => Some(ExtOpcode {
            category: 3,
            index: 37,
            name: Some("sp_set_color_0x"),
        }),
        (3, 38) => Some(ExtOpcode {
            category: 3,
            index: 38,
            name: Some("sp_bitblt"),
        }),
        (3, 39) => Some(ExtOpcode {
            category: 3,
            index: 39,
            name: Some("sp_set_shake"),
        }),
        (3, 40) => Some(ExtOpcode {
            category: 3,
            index: 40,
            name: Some("sp_paint"),
        }),
        (3, 41) => Some(ExtOpcode {
            category: 3,
            index: 41,
            name: None,
        }),
        (3, 42) => Some(ExtOpcode {
            category: 3,
            index: 42,
            name: Some("sp_load_wait_time"),
        }),
        (3, 43) => Some(ExtOpcode {
            category: 3,
            index: 43,
            name: Some("sp_draw"),
        }),
        (3, 44) => Some(ExtOpcode {
            category: 3,
            index: 44,
            name: Some("sp_wait_draw"),
        }),
        (3, 45) => Some(ExtOpcode {
            category: 3,
            index: 45,
            name: Some("sp_unlock"),
        }),
        (3, 46) => Some(ExtOpcode {
            category: 3,
            index: 46,
            name: Some("sp_show"),
        }),
        (3, 47) => Some(ExtOpcode {
            category: 3,
            index: 47,
            name: Some("sp_hide"),
        }),
        (3, 48) => Some(ExtOpcode {
            category: 3,
            index: 48,
            name: None,
        }),
        (3, 49) => Some(ExtOpcode {
            category: 3,
            index: 49,
            name: Some("sp_set_child"),
        }),
        (3, 50) => Some(ExtOpcode {
            category: 3,
            index: 50,
            name: Some("sp_set_transition"),
        }),
        (3, 51) => Some(ExtOpcode {
            category: 3,
            index: 51,
            name: Some("sp_copy_image"),
        }),
        (3, 52) => Some(ExtOpcode {
            category: 3,
            index: 52,
            name: Some("sp_transition"),
        }),
        (3, 53) => Some(ExtOpcode {
            category: 3,
            index: 53,
            name: Some("set_aspect_position_type"),
        }),
        (3, 54) => Some(ExtOpcode {
            category: 3,
            index: 54,
            name: Some("get_backbuffer"),
        }),
        (3, 55) => Some(ExtOpcode {
            category: 3,
            index: 55,
            name: Some("sp_set_mask"),
        }),
        (3, 56) => Some(ExtOpcode {
            category: 3,
            index: 56,
            name: None,
        }),
        (3, 57) => Some(ExtOpcode {
            category: 3,
            index: 57,
            name: Some("spsetanime"),
        }),
        (3, 58) => Some(ExtOpcode {
            category: 3,
            index: 58,
            name: Some("drawtext"),
        }),
        (3, 59) => Some(ExtOpcode {
            category: 3,
            index: 59,
            name: None,
        }),
        (3, 60) => Some(ExtOpcode {
            category: 3,
            index: 60,
            name: None,
        }),
        (3, 62) => Some(ExtOpcode {
            category: 3,
            index: 62,
            name: Some("history_init_0x_0x"),
        }),
        (3, 63) => Some(ExtOpcode {
            category: 3,
            index: 63,
            name: Some("historybegin_lpbyte_ptagdata_sztext"),
        }),
        (3, 64) => Some(ExtOpcode {
            category: 3,
            index: 64,
            name: Some("history_end"),
        }),
        (3, 65) => Some(ExtOpcode {
            category: 3,
            index: 65,
            name: None,
        }),
        (3, 66) => Some(ExtOpcode {
            category: 3,
            index: 66,
            name: None,
        }),
        (3, 67) => Some(ExtOpcode {
            category: 3,
            index: 67,
            name: Some("history_get_height"),
        }),
        (3, 68) => Some(ExtOpcode {
            category: 3,
            index: 68,
            name: None,
        }),
        (3, 69) => Some(ExtOpcode {
            category: 3,
            index: 69,
            name: None,
        }),
        (3, 70) => Some(ExtOpcode {
            category: 3,
            index: 70,
            name: None,
        }),
        (3, 71) => Some(ExtOpcode {
            category: 3,
            index: 71,
            name: None,
        }),
        (3, 72) => Some(ExtOpcode {
            category: 3,
            index: 72,
            name: Some("history_set_rect"),
        }),
        (3, 73) => Some(ExtOpcode {
            category: 3,
            index: 73,
            name: Some("history_clear"),
        }),
        (3, 74) => Some(ExtOpcode {
            category: 3,
            index: 74,
            name: Some("history_set"),
        }),
        (3, 75) => Some(ExtOpcode {
            category: 3,
            index: 75,
            name: None,
        }),
        (3, 76) => Some(ExtOpcode {
            category: 3,
            index: 76,
            name: None,
        }),
        (3, 77) => Some(ExtOpcode {
            category: 3,
            index: 77,
            name: None,
        }),
        (3, 78) => Some(ExtOpcode {
            category: 3,
            index: 78,
            name: None,
        }),
        (3, 79) => Some(ExtOpcode {
            category: 3,
            index: 79,
            name: Some("history_set_face_call"),
        }),
        (3, 80) => Some(ExtOpcode {
            category: 3,
            index: 80,
            name: Some("history_set_face_sound"),
        }),
        (3, 81) => Some(ExtOpcode {
            category: 3,
            index: 81,
            name: Some("history_set_face_sound_release"),
        }),
        (3, 82) => Some(ExtOpcode {
            category: 3,
            index: 82,
            name: Some("history_get_text"),
        }),
        (3, 83) => Some(ExtOpcode {
            category: 3,
            index: 83,
            name: None,
        }),
        (3, 84) => Some(ExtOpcode {
            category: 3,
            index: 84,
            name: None,
        }),
        (3, 85) => Some(ExtOpcode {
            category: 3,
            index: 85,
            name: None,
        }),
        (3, 86) => Some(ExtOpcode {
            category: 3,
            index: 86,
            name: None,
        }),
        (3, 88) => Some(ExtOpcode {
            category: 3,
            index: 88,
            name: Some("movie_play"),
        }),
        (3, 89) => Some(ExtOpcode {
            category: 3,
            index: 89,
            name: Some("msp_set_loop_sp_ep"),
        }),
        (3, 90) => Some(ExtOpcode {
            category: 3,
            index: 90,
            name: Some("msp_cls"),
        }),
        (3, 91) => Some(ExtOpcode {
            category: 3,
            index: 91,
            name: Some("msp_wait"),
        }),
        (3, 92) => Some(ExtOpcode {
            category: 3,
            index: 92,
            name: Some("msp_lock"),
        }),
        (3, 93) => Some(ExtOpcode {
            category: 3,
            index: 93,
            name: Some("msp_unlock"),
        }),
        (3, 94) => Some(ExtOpcode {
            category: 3,
            index: 94,
            name: Some("msp_play"),
        }),
        (3, 95) => Some(ExtOpcode {
            category: 3,
            index: 95,
            name: Some("msp_stop"),
        }),
        (3, 97) => Some(ExtOpcode {
            category: 3,
            index: 97,
            name: Some("create_thread"),
        }),
        (3, 98) => Some(ExtOpcode {
            category: 3,
            index: 98,
            name: Some("exit_thread"),
        }),
        (3, 99) => Some(ExtOpcode {
            category: 3,
            index: 99,
            name: None,
        }),
        (3, 100) => Some(ExtOpcode {
            category: 3,
            index: 100,
            name: Some("get_thread"),
        }),
        (3, 103) => Some(ExtOpcode {
            category: 3,
            index: 103,
            name: Some("mov"),
        }),
        (3, 104) => Some(ExtOpcode {
            category: 3,
            index: 104,
            name: Some("add"),
        }),
        (3, 105) => Some(ExtOpcode {
            category: 3,
            index: 105,
            name: Some("sub"),
        }),
        (3, 106) => Some(ExtOpcode {
            category: 3,
            index: 106,
            name: Some("mul"),
        }),
        (3, 107) => Some(ExtOpcode {
            category: 3,
            index: 107,
            name: Some("div"),
        }),
        (3, 108) => Some(ExtOpcode {
            category: 3,
            index: 108,
            name: Some("bitand"),
        }),
        (3, 109) => Some(ExtOpcode {
            category: 3,
            index: 109,
            name: Some("bitor"),
        }),
        (3, 110) => Some(ExtOpcode {
            category: 3,
            index: 110,
            name: Some("bitxor"),
        }),
        (3, 111) => Some(ExtOpcode {
            category: 3,
            index: 111,
            name: Some("jmp_point"),
        }),
        (3, 112) => Some(ExtOpcode {
            category: 3,
            index: 112,
            name: Some("jf_point"),
        }),
        (3, 113) => Some(ExtOpcode {
            category: 3,
            index: 113,
            name: Some("gosub_point"),
        }),
        (3, 114) => Some(ExtOpcode {
            category: 3,
            index: 114,
            name: Some("eq"),
        }),
        (3, 115) => Some(ExtOpcode {
            category: 3,
            index: 115,
            name: Some("ne"),
        }),
        (3, 116) => Some(ExtOpcode {
            category: 3,
            index: 116,
            name: Some("le"),
        }),
        (3, 117) => Some(ExtOpcode {
            category: 3,
            index: 117,
            name: Some("ge"),
        }),
        (3, 118) => Some(ExtOpcode {
            category: 3,
            index: 118,
            name: Some("lt"),
        }),
        (3, 119) => Some(ExtOpcode {
            category: 3,
            index: 119,
            name: Some("gt"),
        }),
        (3, 120) => Some(ExtOpcode {
            category: 3,
            index: 120,
            name: Some("lor"),
        }),
        (3, 121) => Some(ExtOpcode {
            category: 3,
            index: 121,
            name: Some("land"),
        }),
        (3, 122) => Some(ExtOpcode {
            category: 3,
            index: 122,
            name: Some("lnot_slot"),
        }),
        (3, 123) => Some(ExtOpcode {
            category: 3,
            index: 123,
            name: Some("end"),
        }),
        (3, 124) => Some(ExtOpcode {
            category: 3,
            index: 124,
            name: Some("nop"),
        }),
        (3, 125) => Some(ExtOpcode {
            category: 3,
            index: 125,
            name: Some("extcall"),
        }),
        (3, 126) => Some(ExtOpcode {
            category: 3,
            index: 126,
            name: Some("ret"),
        }),
        (3, 127) => Some(ExtOpcode {
            category: 3,
            index: 127,
            name: Some("reset_adv"),
        }),
        (3, 128) => Some(ExtOpcode {
            category: 3,
            index: 128,
            name: Some("mod"),
        }),
        (3, 129) => Some(ExtOpcode {
            category: 3,
            index: 129,
            name: Some("shl"),
        }),
        (3, 130) => Some(ExtOpcode {
            category: 3,
            index: 130,
            name: Some("shr"),
        }),
        (3, 131) => Some(ExtOpcode {
            category: 3,
            index: 131,
            name: Some("neg_slot"),
        }),
        (3, 132) => Some(ExtOpcode {
            category: 3,
            index: 132,
            name: Some("pop"),
        }),
        (3, 133) => Some(ExtOpcode {
            category: 3,
            index: 133,
            name: Some("push"),
        }),
        (3, 134) => Some(ExtOpcode {
            category: 3,
            index: 134,
            name: Some("pack_args"),
        }),
        (3, 135) => Some(ExtOpcode {
            category: 3,
            index: 135,
            name: Some("drop_args"),
        }),
        (3, 137) => Some(ExtOpcode {
            category: 3,
            index: 137,
            name: Some("create_message"),
        }),
        (3, 138) => Some(ExtOpcode {
            category: 3,
            index: 138,
            name: Some("get_message"),
        }),
        (3, 139) => Some(ExtOpcode {
            category: 3,
            index: 139,
            name: Some("get_message_param"),
        }),
        (3, 142) => Some(ExtOpcode {
            category: 3,
            index: 142,
            name: Some("save"),
        }),
        (3, 143) => Some(ExtOpcode {
            category: 3,
            index: 143,
            name: Some("load"),
        }),
        (3, 144) => Some(ExtOpcode {
            category: 3,
            index: 144,
            name: Some("save_set_title"),
        }),
        (3, 145) => Some(ExtOpcode {
            category: 3,
            index: 145,
            name: Some("save_data"),
        }),
        (3, 146) => Some(ExtOpcode {
            category: 3,
            index: 146,
            name: Some("save_set_thumbnail_size"),
        }),
        (3, 147) => Some(ExtOpcode {
            category: 3,
            index: 147,
            name: Some("thumbnail_set"),
        }),
        (3, 148) => Some(ExtOpcode {
            category: 3,
            index: 148,
            name: Some("savetitledraw"),
        }),
        (3, 149) => Some(ExtOpcode {
            category: 3,
            index: 149,
            name: Some("save_set_font_size"),
        }),
        (3, 150) => Some(ExtOpcode {
            category: 3,
            index: 150,
            name: Some("getsaveday"),
        }),
        (3, 151) => Some(ExtOpcode {
            category: 3,
            index: 151,
            name: Some("is_save"),
        }),
        (3, 152) => Some(ExtOpcode {
            category: 3,
            index: 152,
            name: Some("getsaveusermemory"),
        }),
        (3, 153) => Some(ExtOpcode {
            category: 3,
            index: 153,
            name: Some("savepoint"),
        }),
        (3, 154) => Some(ExtOpcode {
            category: 3,
            index: 154,
            name: Some("save_thumbnail_mosaic_set"),
        }),
        (3, 155) => Some(ExtOpcode {
            category: 3,
            index: 155,
            name: Some("savetimedraw"),
        }),
        (3, 156) => Some(ExtOpcode {
            category: 3,
            index: 156,
            name: Some("savedaydraw"),
        }),
        (3, 157) => Some(ExtOpcode {
            category: 3,
            index: 157,
            name: Some("save_set_text_rect"),
        }),
        (3, 158) => Some(ExtOpcode {
            category: 3,
            index: 158,
            name: Some("savetextdraw"),
        }),
        (3, 159) => Some(ExtOpcode {
            category: 3,
            index: 159,
            name: Some("get_new_savefile"),
        }),
        (3, 163) => Some(ExtOpcode {
            category: 3,
            index: 163,
            name: Some("setsavetext"),
        }),
        (3, 164) => Some(ExtOpcode {
            category: 3,
            index: 164,
            name: Some("thumbnail_renew"),
        }),
        (3, 165) => Some(ExtOpcode {
            category: 3,
            index: 165,
            name: Some("save_set_font_type"),
        }),
        (3, 166) => Some(ExtOpcode {
            category: 3,
            index: 166,
            name: Some("set_load_after_process"),
        }),
        (3, 167) => Some(ExtOpcode {
            category: 3,
            index: 167,
            name: Some("savesystemdata"),
        }),
        (3, 168) => Some(ExtOpcode {
            category: 3,
            index: 168,
            name: Some("save_set_font_effect"),
        }),
        (3, 169) => Some(ExtOpcode {
            category: 3,
            index: 169,
            name: Some("save_set_font_color_0x_0x"),
        }),
        (3, 170) => Some(ExtOpcode {
            category: 3,
            index: 170,
            name: Some("delete_file"),
        }),
        (3, 171) => Some(ExtOpcode {
            category: 3,
            index: 171,
            name: Some("save_tmp_dat"),
        }),
        (3, 172) => Some(ExtOpcode {
            category: 3,
            index: 172,
            name: Some("copy_file"),
        }),
        (3, 173) => Some(ExtOpcode {
            category: 3,
            index: 173,
            name: Some("load_thumbnail"),
        }),
        (3, 174) => Some(ExtOpcode {
            category: 3,
            index: 174,
            name: Some("save_lock_not_open_savefileno"),
        }),
        (3, 175) => Some(ExtOpcode {
            category: 3,
            index: 175,
            name: Some("is_save_lock"),
        }),
        (3, 176) => Some(ExtOpcode {
            category: 3,
            index: 176,
            name: Some("is_prev_data"),
        }),
        (3, 177) => Some(ExtOpcode {
            category: 3,
            index: 177,
            name: Some("save_point_clear"),
        }),
        (3, 178) => Some(ExtOpcode {
            category: 3,
            index: 178,
            name: Some("save_point_lock"),
        }),
        (3, 179) => Some(ExtOpcode {
            category: 3,
            index: 179,
            name: None,
        }),
        (3, 180) => Some(ExtOpcode {
            category: 3,
            index: 180,
            name: Some("histload"),
        }),
        (3, 182) => Some(ExtOpcode {
            category: 3,
            index: 182,
            name: Some("se_load"),
        }),
        (3, 183) => Some(ExtOpcode {
            category: 3,
            index: 183,
            name: Some("se_play"),
        }),
        (3, 184) => Some(ExtOpcode {
            category: 3,
            index: 184,
            name: Some("se_play_ex_ch"),
        }),
        (3, 185) => Some(ExtOpcode {
            category: 3,
            index: 185,
            name: Some("se_stop"),
        }),
        (3, 186) => Some(ExtOpcode {
            category: 3,
            index: 186,
            name: Some("se_set_volume"),
        }),
        (3, 187) => Some(ExtOpcode {
            category: 3,
            index: 187,
            name: Some("se_get_volume"),
        }),
        (3, 188) => Some(ExtOpcode {
            category: 3,
            index: 188,
            name: Some("se_unload"),
        }),
        (3, 189) => Some(ExtOpcode {
            category: 3,
            index: 189,
            name: Some("se_wait"),
        }),
        (3, 190) => Some(ExtOpcode {
            category: 3,
            index: 190,
            name: Some("channel_error_set_se_info"),
        }),
        (3, 191) => Some(ExtOpcode {
            category: 3,
            index: 191,
            name: Some("get_se_ex_volume"),
        }),
        (3, 192) => Some(ExtOpcode {
            category: 3,
            index: 192,
            name: Some("channel_error_set_se_ex_volume"),
        }),
        (3, 193) => Some(ExtOpcode {
            category: 3,
            index: 193,
            name: Some("channel_error_se_enable"),
        }),
        (3, 194) => Some(ExtOpcode {
            category: 3,
            index: 194,
            name: Some("channel_error_is_se_enable"),
        }),
        (3, 195) => Some(ExtOpcode {
            category: 3,
            index: 195,
            name: Some("se_set_pan"),
        }),
        (3, 196) => Some(ExtOpcode {
            category: 3,
            index: 196,
            name: Some("se_mute"),
        }),
        (3, 198) => Some(ExtOpcode {
            category: 3,
            index: 198,
            name: Some("select_init"),
        }),
        (3, 199) => Some(ExtOpcode {
            category: 3,
            index: 199,
            name: Some("select"),
        }),
        (3, 200) => Some(ExtOpcode {
            category: 3,
            index: 200,
            name: None,
        }),
        (3, 201) => Some(ExtOpcode {
            category: 3,
            index: 201,
            name: None,
        }),
        (3, 202) => Some(ExtOpcode {
            category: 3,
            index: 202,
            name: Some("select_clear"),
        }),
        (3, 203) => Some(ExtOpcode {
            category: 3,
            index: 203,
            name: Some("select_set_offset"),
        }),
        (3, 204) => Some(ExtOpcode {
            category: 3,
            index: 204,
            name: Some("select_set_process"),
        }),
        (3, 205) => Some(ExtOpcode {
            category: 3,
            index: 205,
            name: Some("select_lock"),
        }),
        (3, 206) => Some(ExtOpcode {
            category: 3,
            index: 206,
            name: Some("get_select_on_key"),
        }),
        (3, 207) => Some(ExtOpcode {
            category: 3,
            index: 207,
            name: Some("get_select_pull_key"),
        }),
        (3, 208) => Some(ExtOpcode {
            category: 3,
            index: 208,
            name: Some("get_select_push_key"),
        }),
        (3, 210) => Some(ExtOpcode {
            category: 3,
            index: 210,
            name: Some("skip_set"),
        }),
        (3, 211) => Some(ExtOpcode {
            category: 3,
            index: 211,
            name: Some("skip_is"),
        }),
        (3, 212) => Some(ExtOpcode {
            category: 3,
            index: 212,
            name: Some("auto_set"),
        }),
        (3, 213) => Some(ExtOpcode {
            category: 3,
            index: 213,
            name: Some("auto_is"),
        }),
        (3, 214) => Some(ExtOpcode {
            category: 3,
            index: 214,
            name: None,
        }),
        (3, 215) => Some(ExtOpcode {
            category: 3,
            index: 215,
            name: Some("auto_get_time"),
        }),
        (3, 216) => Some(ExtOpcode {
            category: 3,
            index: 216,
            name: None,
        }),
        (3, 217) => Some(ExtOpcode {
            category: 3,
            index: 217,
            name: None,
        }),
        (3, 218) => Some(ExtOpcode {
            category: 3,
            index: 218,
            name: None,
        }),
        (3, 219) => Some(ExtOpcode {
            category: 3,
            index: 219,
            name: None,
        }),
        (3, 220) => Some(ExtOpcode {
            category: 3,
            index: 220,
            name: None,
        }),
        (3, 221) => Some(ExtOpcode {
            category: 3,
            index: 221,
            name: None,
        }),
        (3, 222) => Some(ExtOpcode {
            category: 3,
            index: 222,
            name: None,
        }),
        (3, 223) => Some(ExtOpcode {
            category: 3,
            index: 223,
            name: None,
        }),
        (3, 224) => Some(ExtOpcode {
            category: 3,
            index: 224,
            name: None,
        }),
        (3, 225) => Some(ExtOpcode {
            category: 3,
            index: 225,
            name: Some("load_font"),
        }),
        (3, 226) => Some(ExtOpcode {
            category: 3,
            index: 226,
            name: Some("unload_font"),
        }),
        (3, 227) => Some(ExtOpcode {
            category: 3,
            index: 227,
            name: Some("set_language"),
        }),
        (3, 228) => Some(ExtOpcode {
            category: 3,
            index: 228,
            name: Some("key_canncel"),
        }),
        (3, 229) => Some(ExtOpcode {
            category: 3,
            index: 229,
            name: Some("set_font_color"),
        }),
        (3, 230) => Some(ExtOpcode {
            category: 3,
            index: 230,
            name: Some("load_font_ex"),
        }),
        (3, 231) => Some(ExtOpcode {
            category: 3,
            index: 231,
            name: None,
        }),
        (3, 232) => Some(ExtOpcode {
            category: 3,
            index: 232,
            name: None,
        }),
        (3, 233) => Some(ExtOpcode {
            category: 3,
            index: 233,
            name: None,
        }),
        (3, 234) => Some(ExtOpcode {
            category: 3,
            index: 234,
            name: None,
        }),
        (3, 235) => Some(ExtOpcode {
            category: 3,
            index: 235,
            name: None,
        }),
        (3, 236) => Some(ExtOpcode {
            category: 3,
            index: 236,
            name: None,
        }),
        (3, 237) => Some(ExtOpcode {
            category: 3,
            index: 237,
            name: Some("set_font_size"),
        }),
        (3, 238) => Some(ExtOpcode {
            category: 3,
            index: 238,
            name: Some("get_font_size"),
        }),
        (3, 239) => Some(ExtOpcode {
            category: 3,
            index: 239,
            name: Some("get_font_type"),
        }),
        (3, 240) => Some(ExtOpcode {
            category: 3,
            index: 240,
            name: Some("set_font_effect"),
        }),
        (3, 241) => Some(ExtOpcode {
            category: 3,
            index: 241,
            name: Some("get_font_effect"),
        }),
        (3, 242) => Some(ExtOpcode {
            category: 3,
            index: 242,
            name: Some("get_pull_key"),
        }),
        (3, 243) => Some(ExtOpcode {
            category: 3,
            index: 243,
            name: Some("get_on_key"),
        }),
        (3, 244) => Some(ExtOpcode {
            category: 3,
            index: 244,
            name: Some("get_push_key"),
        }),
        (3, 245) => Some(ExtOpcode {
            category: 3,
            index: 245,
            name: Some("input_clear"),
        }),
        (3, 246) => Some(ExtOpcode {
            category: 3,
            index: 246,
            name: Some("change_window_size"),
        }),
        (3, 247) => Some(ExtOpcode {
            category: 3,
            index: 247,
            name: Some("change_aspect_mode"),
        }),
        (3, 248) => Some(ExtOpcode {
            category: 3,
            index: 248,
            name: Some("aspect_position_enable"),
        }),
        (3, 249) => Some(ExtOpcode {
            category: 3,
            index: 249,
            name: None,
        }),
        (3, 250) => Some(ExtOpcode {
            category: 3,
            index: 250,
            name: Some("get_aspect_mode"),
        }),
        (3, 251) => Some(ExtOpcode {
            category: 3,
            index: 251,
            name: Some("get_monitor_size"),
        }),
        (3, 252) => Some(ExtOpcode {
            category: 3,
            index: 252,
            name: None,
        }),
        (3, 253) => Some(ExtOpcode {
            category: 3,
            index: 253,
            name: Some("get_system_metrics"),
        }),
        (3, 254) => Some(ExtOpcode {
            category: 3,
            index: 254,
            name: Some("set_system_path"),
        }),
        (3, 255) => Some(ExtOpcode {
            category: 3,
            index: 255,
            name: Some("set_allmosaicthumbnail"),
        }),
        (3, 256) => Some(ExtOpcode {
            category: 3,
            index: 256,
            name: Some("enable_window_change"),
        }),
        (3, 257) => Some(ExtOpcode {
            category: 3,
            index: 257,
            name: Some("is_enable_window_change"),
        }),
        (3, 258) => Some(ExtOpcode {
            category: 3,
            index: 258,
            name: Some("set_cursor_null"),
        }),
        (3, 259) => Some(ExtOpcode {
            category: 3,
            index: 259,
            name: Some("set_hide_cursor_time"),
        }),
        (3, 260) => Some(ExtOpcode {
            category: 3,
            index: 260,
            name: Some("get_hide_cursor_time"),
        }),
        (3, 261) => Some(ExtOpcode {
            category: 3,
            index: 261,
            name: Some("scene_skip"),
        }),
        (3, 262) => Some(ExtOpcode {
            category: 3,
            index: 262,
            name: None,
        }),
        (3, 263) => Some(ExtOpcode {
            category: 3,
            index: 263,
            name: None,
        }),
        (3, 264) => Some(ExtOpcode {
            category: 3,
            index: 264,
            name: Some("get_async_key"),
        }),
        (3, 265) => Some(ExtOpcode {
            category: 3,
            index: 265,
            name: Some("get_font_color"),
        }),
        (3, 266) => Some(ExtOpcode {
            category: 3,
            index: 266,
            name: None,
        }),
        (3, 267) => Some(ExtOpcode {
            category: 3,
            index: 267,
            name: Some("history_skip"),
        }),
        (3, 268) => Some(ExtOpcode {
            category: 3,
            index: 268,
            name: None,
        }),
        (3, 269) => Some(ExtOpcode {
            category: 3,
            index: 269,
            name: None,
        }),
        (3, 270) => Some(ExtOpcode {
            category: 3,
            index: 270,
            name: Some("set_language"),
        }),
        (3, 271) => Some(ExtOpcode {
            category: 3,
            index: 271,
            name: Some("set_achievement"),
        }),
        (3, 273) => Some(ExtOpcode {
            category: 3,
            index: 273,
            name: Some("system_btn_set"),
        }),
        (3, 274) => Some(ExtOpcode {
            category: 3,
            index: 274,
            name: Some("system_btn_release"),
        }),
        (3, 275) => Some(ExtOpcode {
            category: 3,
            index: 275,
            name: Some("system_btn_enable"),
        }),
        (3, 278) => Some(ExtOpcode {
            category: 3,
            index: 278,
            name: Some("text_init"),
        }),
        (3, 279) => Some(ExtOpcode {
            category: 3,
            index: 279,
            name: Some("text_set_icon"),
        }),
        (3, 280) => Some(ExtOpcode {
            category: 3,
            index: 280,
            name: Some("text"),
        }),
        (3, 281) => Some(ExtOpcode {
            category: 3,
            index: 281,
            name: Some("text_hide"),
        }),
        (3, 282) => Some(ExtOpcode {
            category: 3,
            index: 282,
            name: Some("text_show"),
        }),
        (3, 283) => Some(ExtOpcode {
            category: 3,
            index: 283,
            name: Some("text_set_btn"),
        }),
        (3, 284) => Some(ExtOpcode {
            category: 3,
            index: 284,
            name: Some("text_uninit"),
        }),
        (3, 285) => Some(ExtOpcode {
            category: 3,
            index: 285,
            name: Some("text_set_rect_invalid_param"),
        }),
        (3, 286) => Some(ExtOpcode {
            category: 3,
            index: 286,
            name: Some("text_clear"),
        }),
        (3, 287) => Some(ExtOpcode {
            category: 3,
            index: 287,
            name: None,
        }),
        (3, 288) => Some(ExtOpcode {
            category: 3,
            index: 288,
            name: Some("text_get_time"),
        }),
        (3, 289) => Some(ExtOpcode {
            category: 3,
            index: 289,
            name: Some("text_window_set_alpha"),
        }),
        (3, 290) => Some(ExtOpcode {
            category: 3,
            index: 290,
            name: Some("text_voice_play"),
        }),
        (3, 291) => Some(ExtOpcode {
            category: 3,
            index: 291,
            name: None,
        }),
        (3, 292) => Some(ExtOpcode {
            category: 3,
            index: 292,
            name: Some("text_set_icon_animation_time"),
        }),
        (3, 293) => Some(ExtOpcode {
            category: 3,
            index: 293,
            name: Some("text_w"),
        }),
        (3, 294) => Some(ExtOpcode {
            category: 3,
            index: 294,
            name: Some("text_a"),
        }),
        (3, 295) => Some(ExtOpcode {
            category: 3,
            index: 295,
            name: Some("text_wa"),
        }),
        (3, 296) => Some(ExtOpcode {
            category: 3,
            index: 296,
            name: Some("text_n"),
        }),
        (3, 297) => Some(ExtOpcode {
            category: 3,
            index: 297,
            name: Some("text_cat"),
        }),
        (3, 298) => Some(ExtOpcode {
            category: 3,
            index: 298,
            name: Some("set_history"),
        }),
        (3, 299) => Some(ExtOpcode {
            category: 3,
            index: 299,
            name: Some("is_text_visible"),
        }),
        (4, 0) => Some(ExtOpcode {
            category: 4,
            index: 0,
            name: Some("bgm_play"),
        }),
        (4, 1) => Some(ExtOpcode {
            category: 4,
            index: 1,
            name: Some("bgm_stop"),
        }),
        (4, 2) => Some(ExtOpcode {
            category: 4,
            index: 2,
            name: Some("bgm_set_volume"),
        }),
        (4, 3) => Some(ExtOpcode {
            category: 4,
            index: 3,
            name: Some("bgm_get_volume"),
        }),
        (4, 4) => Some(ExtOpcode {
            category: 4,
            index: 4,
            name: Some("bgm_get_auto_volume"),
        }),
        (4, 5) => Some(ExtOpcode {
            category: 4,
            index: 5,
            name: Some("bgm_set_volume_users"),
        }),
        (4, 6) => Some(ExtOpcode {
            category: 4,
            index: 6,
            name: Some("bgm_set_auto_volume"),
        }),
        (4, 7) => Some(ExtOpcode {
            category: 4,
            index: 7,
            name: Some("bgm_pause"),
        }),
        (4, 8) => Some(ExtOpcode {
            category: 4,
            index: 8,
            name: Some("get_bgm_filename"),
        }),
        (4, 9) => Some(ExtOpcode {
            category: 4,
            index: 9,
            name: Some("bgm_load"),
        }),
        (4, 10) => Some(ExtOpcode {
            category: 4,
            index: 10,
            name: Some("bgm_play2"),
        }),
        (4, 11) => Some(ExtOpcode {
            category: 4,
            index: 11,
            name: Some("set_master_volume"),
        }),
        (4, 12) => Some(ExtOpcode {
            category: 4,
            index: 12,
            name: Some("get_master_volume"),
        }),
        (4, 13) => Some(ExtOpcode {
            category: 4,
            index: 13,
            name: Some("mute_master_volume"),
        }),
        (4, 14) => Some(ExtOpcode {
            category: 4,
            index: 14,
            name: Some("bgm_mute"),
        }),
        (4, 15) => Some(ExtOpcode {
            category: 4,
            index: 15,
            name: Some("mute_bgm_auto_volume"),
        }),
        (4, 16) => Some(ExtOpcode {
            category: 4,
            index: 16,
            name: Some("1_get_bgm_status"),
        }),
        (4, 17) => Some(ExtOpcode {
            category: 4,
            index: 17,
            name: Some("1_get_bgm_pos"),
        }),
        (4, 18) => Some(ExtOpcode {
            category: 4,
            index: 18,
            name: Some("get_bgm_ch"),
        }),
        (4, 20) => Some(ExtOpcode {
            category: 4,
            index: 20,
            name: Some("btn_init"),
        }),
        (4, 21) => Some(ExtOpcode {
            category: 4,
            index: 21,
            name: Some("btn_uninit"),
        }),
        (4, 23) => Some(ExtOpcode {
            category: 4,
            index: 23,
            name: Some("btn_set"),
        }),
        (4, 24) => Some(ExtOpcode {
            category: 4,
            index: 24,
            name: Some("btn_hide"),
        }),
        (4, 25) => Some(ExtOpcode {
            category: 4,
            index: 25,
            name: Some("btn_show"),
        }),
        (4, 26) => Some(ExtOpcode {
            category: 4,
            index: 26,
            name: Some("btn_set_pos"),
        }),
        (4, 27) => Some(ExtOpcode {
            category: 4,
            index: 27,
            name: Some("btn_set_rect"),
        }),
        (4, 28) => Some(ExtOpcode {
            category: 4,
            index: 28,
            name: Some("btn_release"),
        }),
        (4, 29) => Some(ExtOpcode {
            category: 4,
            index: 29,
            name: None,
        }),
        (4, 30) => Some(ExtOpcode {
            category: 4,
            index: 30,
            name: None,
        }),
        (4, 31) => Some(ExtOpcode {
            category: 4,
            index: 31,
            name: None,
        }),
        (4, 32) => Some(ExtOpcode {
            category: 4,
            index: 32,
            name: None,
        }),
        (4, 33) => Some(ExtOpcode {
            category: 4,
            index: 33,
            name: Some("btn_set_toggle"),
        }),
        (4, 34) => Some(ExtOpcode {
            category: 4,
            index: 34,
            name: None,
        }),
        (4, 35) => Some(ExtOpcode {
            category: 4,
            index: 35,
            name: Some("btn_enable"),
        }),
        (4, 36) => Some(ExtOpcode {
            category: 4,
            index: 36,
            name: Some("btn_set_alpha_0x"),
        }),
        (4, 37) => Some(ExtOpcode {
            category: 4,
            index: 37,
            name: Some("btn_get_push"),
        }),
        (4, 38) => Some(ExtOpcode {
            category: 4,
            index: 38,
            name: Some("error_btn_expansion"),
        }),
        (4, 39) => Some(ExtOpcode {
            category: 4,
            index: 39,
            name: Some("btn_lock"),
        }),
        (4, 40) => Some(ExtOpcode {
            category: 4,
            index: 40,
            name: Some("btn_unlock"),
        }),
        (4, 41) => Some(ExtOpcode {
            category: 4,
            index: 41,
            name: Some("btn_set_anim"),
        }),
        (4, 42) => Some(ExtOpcode {
            category: 4,
            index: 42,
            name: Some("btn_set_hit"),
        }),
        (4, 43) => Some(ExtOpcode {
            category: 4,
            index: 43,
            name: Some("btn_get_onmouse"),
        }),
        (4, 44) => Some(ExtOpcode {
            category: 4,
            index: 44,
            name: Some("btn_anim_clear"),
        }),
        (4, 45) => Some(ExtOpcode {
            category: 4,
            index: 45,
            name: Some("btn_get_offmouse"),
        }),
        (4, 46) => Some(ExtOpcode {
            category: 4,
            index: 46,
            name: Some("btn_onmouse_clear"),
        }),
        (4, 47) => Some(ExtOpcode {
            category: 4,
            index: 47,
            name: Some("btn_blt"),
        }),
        (4, 48) => Some(ExtOpcode {
            category: 4,
            index: 48,
            name: Some("btn_link"),
        }),
        (4, 49) => Some(ExtOpcode {
            category: 4,
            index: 49,
            name: Some("btn_set_state"),
        }),
        (4, 50) => Some(ExtOpcode {
            category: 4,
            index: 50,
            name: Some("btn_get_link"),
        }),
        (4, 51) => Some(ExtOpcode {
            category: 4,
            index: 51,
            name: Some("btn_set_tips"),
        }),
        (4, 52) => Some(ExtOpcode {
            category: 4,
            index: 52,
            name: Some("btn_get_tips"),
        }),
        (4, 53) => Some(ExtOpcode {
            category: 4,
            index: 53,
            name: Some("btn_anime_is_true"),
        }),
        (4, 54) => Some(ExtOpcode {
            category: 4,
            index: 54,
            name: Some("btn_anime_get_status"),
        }),
        (4, 55) => Some(ExtOpcode {
            category: 4,
            index: 55,
            name: Some("btn_anime_finish"),
        }),
        (4, 56) => Some(ExtOpcode {
            category: 4,
            index: 56,
            name: Some("btn_mode"),
        }),
        (4, 57) => Some(ExtOpcode {
            category: 4,
            index: 57,
            name: Some("btn_get_alpha_0x"),
        }),
        (4, 58) => Some(ExtOpcode {
            category: 4,
            index: 58,
            name: Some("btn_on_check_0x"),
        }),
        (4, 60) => Some(ExtOpcode {
            category: 4,
            index: 60,
            name: None,
        }),
        (4, 61) => Some(ExtOpcode {
            category: 4,
            index: 61,
            name: None,
        }),
        (4, 62) => Some(ExtOpcode {
            category: 4,
            index: 62,
            name: Some("set_window_text"),
        }),
        (4, 63) => Some(ExtOpcode {
            category: 4,
            index: 63,
            name: None,
        }),
        (4, 64) => Some(ExtOpcode {
            category: 4,
            index: 64,
            name: None,
        }),
        (4, 65) => Some(ExtOpcode {
            category: 4,
            index: 65,
            name: None,
        }),
        (4, 66) => Some(ExtOpcode {
            category: 4,
            index: 66,
            name: None,
        }),
        (4, 67) => Some(ExtOpcode {
            category: 4,
            index: 67,
            name: None,
        }),
        (4, 68) => Some(ExtOpcode {
            category: 4,
            index: 68,
            name: None,
        }),
        (4, 69) => Some(ExtOpcode {
            category: 4,
            index: 69,
            name: None,
        }),
        (4, 70) => Some(ExtOpcode {
            category: 4,
            index: 70,
            name: Some("debug_break"),
        }),
        (4, 73) => Some(ExtOpcode {
            category: 4,
            index: 73,
            name: Some("app_exec"),
        }),
        (4, 74) => Some(ExtOpcode {
            category: 4,
            index: 74,
            name: Some("is_playername"),
        }),
        (4, 75) => Some(ExtOpcode {
            category: 4,
            index: 75,
            name: None,
        }),
        (4, 76) => Some(ExtOpcode {
            category: 4,
            index: 76,
            name: None,
        }),
        (4, 77) => Some(ExtOpcode {
            category: 4,
            index: 77,
            name: None,
        }),
        (4, 78) => Some(ExtOpcode {
            category: 4,
            index: 78,
            name: None,
        }),
        (4, 79) => Some(ExtOpcode {
            category: 4,
            index: 79,
            name: None,
        }),
        (4, 80) => Some(ExtOpcode {
            category: 4,
            index: 80,
            name: Some("file_exist"),
        }),
        (4, 81) => Some(ExtOpcode {
            category: 4,
            index: 81,
            name: Some("wsprint"),
        }),
        (4, 82) => Some(ExtOpcode {
            category: 4,
            index: 82,
            name: Some("check_disc"),
        }),
        (4, 83) => Some(ExtOpcode {
            category: 4,
            index: 83,
            name: None,
        }),
        (4, 84) => Some(ExtOpcode {
            category: 4,
            index: 84,
            name: None,
        }),
        (4, 85) => Some(ExtOpcode {
            category: 4,
            index: 85,
            name: None,
        }),
        (4, 86) => Some(ExtOpcode {
            category: 4,
            index: 86,
            name: Some("update_access"),
        }),
        (4, 87) => Some(ExtOpcode {
            category: 4,
            index: 87,
            name: None,
        }),
        (4, 88) => Some(ExtOpcode {
            category: 4,
            index: 88,
            name: None,
        }),
        (4, 89) => Some(ExtOpcode {
            category: 4,
            index: 89,
            name: None,
        }),
        (4, 90) => Some(ExtOpcode {
            category: 4,
            index: 90,
            name: None,
        }),
        (4, 91) => Some(ExtOpcode {
            category: 4,
            index: 91,
            name: None,
        }),
        (4, 92) => Some(ExtOpcode {
            category: 4,
            index: 92,
            name: None,
        }),
        (4, 93) => Some(ExtOpcode {
            category: 4,
            index: 93,
            name: None,
        }),
        (4, 94) => Some(ExtOpcode {
            category: 4,
            index: 94,
            name: Some("player_name_set_begin"),
        }),
        (4, 95) => Some(ExtOpcode {
            category: 4,
            index: 95,
            name: Some("player_name_set_end"),
        }),
        (4, 96) => Some(ExtOpcode {
            category: 4,
            index: 96,
            name: Some("player_name_set_check"),
        }),
        (4, 97) => Some(ExtOpcode {
            category: 4,
            index: 97,
            name: None,
        }),
        (4, 98) => Some(ExtOpcode {
            category: 4,
            index: 98,
            name: Some("player_name_reset"),
        }),
        (4, 99) => Some(ExtOpcode {
            category: 4,
            index: 99,
            name: Some("player_name_set_direct"),
        }),
        (4, 100) => Some(ExtOpcode {
            category: 4,
            index: 100,
            name: None,
        }),
        (4, 101) => Some(ExtOpcode {
            category: 4,
            index: 101,
            name: None,
        }),
        (4, 102) => Some(ExtOpcode {
            category: 4,
            index: 102,
            name: Some("openfile"),
        }),
        (4, 103) => Some(ExtOpcode {
            category: 4,
            index: 103,
            name: Some("read_file"),
        }),
        (4, 104) => Some(ExtOpcode {
            category: 4,
            index: 104,
            name: Some("close_file_not_handle"),
        }),
        (4, 105) => Some(ExtOpcode {
            category: 4,
            index: 105,
            name: Some("set_file_pointer"),
        }),
        (4, 106) => Some(ExtOpcode {
            category: 4,
            index: 106,
            name: Some("file_string"),
        }),
        (4, 107) => Some(ExtOpcode {
            category: 4,
            index: 107,
            name: Some("set_last_process"),
        }),
        (4, 108) => Some(ExtOpcode {
            category: 4,
            index: 108,
            name: Some("sz_buf"),
        }),
        (4, 109) => Some(ExtOpcode {
            category: 4,
            index: 109,
            name: Some("getprivateprofileint"),
        }),
        (4, 110) => Some(ExtOpcode {
            category: 4,
            index: 110,
            name: None,
        }),
        (4, 111) => Some(ExtOpcode {
            category: 4,
            index: 111,
            name: None,
        }),
        (4, 112) => Some(ExtOpcode {
            category: 4,
            index: 112,
            name: None,
        }),
        (4, 113) => Some(ExtOpcode {
            category: 4,
            index: 113,
            name: Some("is_tweet"),
        }),
        (4, 114) => Some(ExtOpcode {
            category: 4,
            index: 114,
            name: None,
        }),
        (4, 115) => Some(ExtOpcode {
            category: 4,
            index: 115,
            name: None,
        }),
        (4, 116) => Some(ExtOpcode {
            category: 4,
            index: 116,
            name: None,
        }),
        (4, 117) => Some(ExtOpcode {
            category: 4,
            index: 117,
            name: None,
        }),
        (4, 118) => Some(ExtOpcode {
            category: 4,
            index: 118,
            name: None,
        }),
        (4, 119) => Some(ExtOpcode {
            category: 4,
            index: 119,
            name: None,
        }),
        (4, 120) => Some(ExtOpcode {
            category: 4,
            index: 120,
            name: None,
        }),
        (4, 121) => Some(ExtOpcode {
            category: 4,
            index: 121,
            name: None,
        }),
        (4, 122) => Some(ExtOpcode {
            category: 4,
            index: 122,
            name: None,
        }),
        (4, 123) => Some(ExtOpcode {
            category: 4,
            index: 123,
            name: None,
        }),
        (4, 124) => Some(ExtOpcode {
            category: 4,
            index: 124,
            name: None,
        }),
        (4, 125) => Some(ExtOpcode {
            category: 4,
            index: 125,
            name: None,
        }),
        (4, 126) => Some(ExtOpcode {
            category: 4,
            index: 126,
            name: None,
        }),
        (4, 127) => Some(ExtOpcode {
            category: 4,
            index: 127,
            name: None,
        }),
        (4, 128) => Some(ExtOpcode {
            category: 4,
            index: 128,
            name: None,
        }),
        (4, 129) => Some(ExtOpcode {
            category: 4,
            index: 129,
            name: None,
        }),
        (4, 130) => Some(ExtOpcode {
            category: 4,
            index: 130,
            name: None,
        }),
        (4, 131) => Some(ExtOpcode {
            category: 4,
            index: 131,
            name: None,
        }),
        (4, 132) => Some(ExtOpcode {
            category: 4,
            index: 132,
            name: None,
        }),
        (4, 133) => Some(ExtOpcode {
            category: 4,
            index: 133,
            name: None,
        }),
        (4, 134) => Some(ExtOpcode {
            category: 4,
            index: 134,
            name: None,
        }),
        (4, 135) => Some(ExtOpcode {
            category: 4,
            index: 135,
            name: None,
        }),
        (4, 136) => Some(ExtOpcode {
            category: 4,
            index: 136,
            name: None,
        }),
        (4, 137) => Some(ExtOpcode {
            category: 4,
            index: 137,
            name: None,
        }),
        (4, 138) => Some(ExtOpcode {
            category: 4,
            index: 138,
            name: None,
        }),
        (4, 139) => Some(ExtOpcode {
            category: 4,
            index: 139,
            name: None,
        }),
        (4, 140) => Some(ExtOpcode {
            category: 4,
            index: 140,
            name: None,
        }),
        (4, 141) => Some(ExtOpcode {
            category: 4,
            index: 141,
            name: None,
        }),
        (4, 142) => Some(ExtOpcode {
            category: 4,
            index: 142,
            name: None,
        }),
        (4, 143) => Some(ExtOpcode {
            category: 4,
            index: 143,
            name: None,
        }),
        (4, 144) => Some(ExtOpcode {
            category: 4,
            index: 144,
            name: None,
        }),
        (4, 145) => Some(ExtOpcode {
            category: 4,
            index: 145,
            name: None,
        }),
        (4, 146) => Some(ExtOpcode {
            category: 4,
            index: 146,
            name: Some("result_tweet"),
        }),
        (4, 147) => Some(ExtOpcode {
            category: 4,
            index: 147,
            name: Some("get_tweet_key"),
        }),
        (4, 148) => Some(ExtOpcode {
            category: 4,
            index: 148,
            name: Some("set_tweet_key"),
        }),
        (4, 149) => Some(ExtOpcode {
            category: 4,
            index: 149,
            name: None,
        }),
        (4, 150) => Some(ExtOpcode {
            category: 4,
            index: 150,
            name: Some("tweet_authorize"),
        }),
        (4, 151) => Some(ExtOpcode {
            category: 4,
            index: 151,
            name: None,
        }),
        (4, 152) => Some(ExtOpcode {
            category: 4,
            index: 152,
            name: None,
        }),
        (4, 153) => Some(ExtOpcode {
            category: 4,
            index: 153,
            name: None,
        }),
        (4, 154) => Some(ExtOpcode {
            category: 4,
            index: 154,
            name: Some("tips_csv_read_error"),
        }),
        (4, 155) => Some(ExtOpcode {
            category: 4,
            index: 155,
            name: Some("tips_csv_get_error"),
        }),
        (4, 156) => Some(ExtOpcode {
            category: 4,
            index: 156,
            name: Some("tips_csv_search_not_found"),
        }),
        (4, 157) => Some(ExtOpcode {
            category: 4,
            index: 157,
            name: None,
        }),
        (4, 158) => Some(ExtOpcode {
            category: 4,
            index: 158,
            name: None,
        }),
        (4, 159) => Some(ExtOpcode {
            category: 4,
            index: 159,
            name: Some("tips_acc_save"),
        }),
        (4, 160) => Some(ExtOpcode {
            category: 4,
            index: 160,
            name: Some("is_network"),
        }),
        (4, 161) => Some(ExtOpcode {
            category: 4,
            index: 161,
            name: Some("is_touch"),
        }),
        (4, 162) => Some(ExtOpcode {
            category: 4,
            index: 162,
            name: None,
        }),
        (4, 163) => Some(ExtOpcode {
            category: 4,
            index: 163,
            name: None,
        }),
        (4, 164) => Some(ExtOpcode {
            category: 4,
            index: 164,
            name: None,
        }),
        (4, 165) => Some(ExtOpcode {
            category: 4,
            index: 165,
            name: None,
        }),
        (4, 166) => Some(ExtOpcode {
            category: 4,
            index: 166,
            name: None,
        }),
        (4, 167) => Some(ExtOpcode {
            category: 4,
            index: 167,
            name: None,
        }),
        (4, 168) => Some(ExtOpcode {
            category: 4,
            index: 168,
            name: None,
        }),
        (4, 169) => Some(ExtOpcode {
            category: 4,
            index: 169,
            name: None,
        }),
        (4, 170) => Some(ExtOpcode {
            category: 4,
            index: 170,
            name: None,
        }),
        (4, 171) => Some(ExtOpcode {
            category: 4,
            index: 171,
            name: None,
        }),
        (4, 172) => Some(ExtOpcode {
            category: 4,
            index: 172,
            name: None,
        }),
        (4, 173) => Some(ExtOpcode {
            category: 4,
            index: 173,
            name: None,
        }),
        (4, 174) => Some(ExtOpcode {
            category: 4,
            index: 174,
            name: None,
        }),
        (4, 175) => Some(ExtOpcode {
            category: 4,
            index: 175,
            name: None,
        }),
        (4, 176) => Some(ExtOpcode {
            category: 4,
            index: 176,
            name: None,
        }),
        (4, 177) => Some(ExtOpcode {
            category: 4,
            index: 177,
            name: None,
        }),
        (4, 178) => Some(ExtOpcode {
            category: 4,
            index: 178,
            name: None,
        }),
        (4, 179) => Some(ExtOpcode {
            category: 4,
            index: 179,
            name: None,
        }),
        (4, 180) => Some(ExtOpcode {
            category: 4,
            index: 180,
            name: None,
        }),
        (4, 181) => Some(ExtOpcode {
            category: 4,
            index: 181,
            name: None,
        }),
        (4, 182) => Some(ExtOpcode {
            category: 4,
            index: 182,
            name: None,
        }),
        (4, 183) => Some(ExtOpcode {
            category: 4,
            index: 183,
            name: None,
        }),
        (4, 184) => Some(ExtOpcode {
            category: 4,
            index: 184,
            name: None,
        }),
        (4, 186) => Some(ExtOpcode {
            category: 4,
            index: 186,
            name: None,
        }),
        (4, 187) => Some(ExtOpcode {
            category: 4,
            index: 187,
            name: Some("run_no_wait"),
        }),
        (4, 188) => Some(ExtOpcode {
            category: 4,
            index: 188,
            name: Some("run_stack"),
        }),
        (4, 190) => Some(ExtOpcode {
            category: 4,
            index: 190,
            name: Some("fx_effect_cls"),
        }),
        (4, 191) => Some(ExtOpcode {
            category: 4,
            index: 191,
            name: Some("fx_raster_stop"),
        }),
        (4, 192) => Some(ExtOpcode {
            category: 4,
            index: 192,
            name: Some("fx_effect_wait"),
        }),
        (4, 193) => Some(ExtOpcode {
            category: 4,
            index: 193,
            name: None,
        }),
        (4, 195) => Some(ExtOpcode {
            category: 4,
            index: 195,
            name: Some("random"),
        }),
        (4, 196) => Some(ExtOpcode {
            category: 4,
            index: 196,
            name: Some("abs"),
        }),
        (4, 197) => Some(ExtOpcode {
            category: 4,
            index: 197,
            name: Some("sin"),
        }),
        (4, 198) => Some(ExtOpcode {
            category: 4,
            index: 198,
            name: Some("cos"),
        }),
        (4, 199) => Some(ExtOpcode {
            category: 4,
            index: 199,
            name: Some("tan"),
        }),
        (4, 200) => Some(ExtOpcode {
            category: 4,
            index: 200,
            name: Some("atan"),
        }),
        (4, 201) => Some(ExtOpcode {
            category: 4,
            index: 201,
            name: Some("log"),
        }),
        (4, 202) => Some(ExtOpcode {
            category: 4,
            index: 202,
            name: Some("log10"),
        }),
        (4, 203) => Some(ExtOpcode {
            category: 4,
            index: 203,
            name: None,
        }),
        (4, 204) => Some(ExtOpcode {
            category: 4,
            index: 204,
            name: Some("sqrt"),
        }),
        (4, 205) => Some(ExtOpcode {
            category: 4,
            index: 205,
            name: None,
        }),
        (4, 206) => Some(ExtOpcode {
            category: 4,
            index: 206,
            name: None,
        }),
        (4, 210) => Some(ExtOpcode {
            category: 4,
            index: 210,
            name: Some("sp_set"),
        }),
        (4, 211) => Some(ExtOpcode {
            category: 4,
            index: 211,
            name: Some("sp_set_ex"),
        }),
        (4, 212) => Some(ExtOpcode {
            category: 4,
            index: 212,
            name: Some("sp_set_pos"),
        }),
        (4, 213) => Some(ExtOpcode {
            category: 4,
            index: 213,
            name: Some("sp_cls"),
        }),
        (4, 214) => Some(ExtOpcode {
            category: 4,
            index: 214,
            name: Some("sp_set_alpha"),
        }),
        (4, 215) => Some(ExtOpcode {
            category: 4,
            index: 215,
            name: Some("set_priority"),
        }),
        (4, 216) => Some(ExtOpcode {
            category: 4,
            index: 216,
            name: None,
        }),
        (4, 217) => Some(ExtOpcode {
            category: 4,
            index: 217,
            name: Some("sp_set_center"),
        }),
        (4, 219) => Some(ExtOpcode {
            category: 4,
            index: 219,
            name: Some("sp_cls_ex"),
        }),
        (4, 220) => Some(ExtOpcode {
            category: 4,
            index: 220,
            name: Some("set_filter"),
        }),
        (4, 221) => Some(ExtOpcode {
            category: 4,
            index: 221,
            name: Some("sp_cls_transition"),
        }),
        (4, 222) => Some(ExtOpcode {
            category: 4,
            index: 222,
            name: Some("sp_set_pos_ex"),
        }),
        (4, 223) => Some(ExtOpcode {
            category: 4,
            index: 223,
            name: Some("sp_set_rect_pos"),
        }),
        (4, 224) => Some(ExtOpcode {
            category: 4,
            index: 224,
            name: None,
        }),
        (4, 225) => Some(ExtOpcode {
            category: 4,
            index: 225,
            name: Some("sp_set_scale"),
        }),
        (4, 226) => Some(ExtOpcode {
            category: 4,
            index: 226,
            name: Some("sp_set_rotate"),
        }),
        (4, 227) => Some(ExtOpcode {
            category: 4,
            index: 227,
            name: Some("face_init"),
        }),
        (4, 228) => Some(ExtOpcode {
            category: 4,
            index: 228,
            name: Some("face_set"),
        }),
        (4, 229) => Some(ExtOpcode {
            category: 4,
            index: 229,
            name: Some("not_image_sp_get_color"),
        }),
        (4, 230) => Some(ExtOpcode {
            category: 4,
            index: 230,
            name: Some("sptext"),
        }),
        (4, 231) => Some(ExtOpcode {
            category: 4,
            index: 231,
            name: Some("face_cls"),
        }),
        (4, 232) => Some(ExtOpcode {
            category: 4,
            index: 232,
            name: Some("sp_set_rect"),
        }),
        (4, 233) => Some(ExtOpcode {
            category: 4,
            index: 233,
            name: Some("sp_set_pos_move"),
        }),
        (4, 234) => Some(ExtOpcode {
            category: 4,
            index: 234,
            name: Some("not_image_sp_get_alpha"),
        }),
        (4, 235) => Some(ExtOpcode {
            category: 4,
            index: 235,
            name: Some("not_image_sp_get_rotate"),
        }),
        (4, 236) => Some(ExtOpcode {
            category: 4,
            index: 236,
            name: None,
        }),
        (4, 237) => Some(ExtOpcode {
            category: 4,
            index: 237,
            name: None,
        }),
        (4, 238) => Some(ExtOpcode {
            category: 4,
            index: 238,
            name: None,
        }),
        (4, 239) => Some(ExtOpcode {
            category: 4,
            index: 239,
            name: None,
        }),
        (4, 240) => Some(ExtOpcode {
            category: 4,
            index: 240,
            name: Some("sp_create"),
        }),
        (4, 241) => Some(ExtOpcode {
            category: 4,
            index: 241,
            name: Some("sp_anime_clear"),
        }),
        (4, 242) => Some(ExtOpcode {
            category: 4,
            index: 242,
            name: None,
        }),
        (4, 243) => Some(ExtOpcode {
            category: 4,
            index: 243,
            name: None,
        }),
        (4, 244) => Some(ExtOpcode {
            category: 4,
            index: 244,
            name: Some("not_image_sp_get_scale"),
        }),
        (4, 245) => Some(ExtOpcode {
            category: 4,
            index: 245,
            name: Some("sp_set_color_0x"),
        }),
        (4, 246) => Some(ExtOpcode {
            category: 4,
            index: 246,
            name: Some("sp_bitblt"),
        }),
        (4, 247) => Some(ExtOpcode {
            category: 4,
            index: 247,
            name: Some("sp_set_shake"),
        }),
        (4, 248) => Some(ExtOpcode {
            category: 4,
            index: 248,
            name: Some("sp_paint"),
        }),
        (4, 249) => Some(ExtOpcode {
            category: 4,
            index: 249,
            name: None,
        }),
        (4, 250) => Some(ExtOpcode {
            category: 4,
            index: 250,
            name: Some("sp_load_wait_time"),
        }),
        (4, 251) => Some(ExtOpcode {
            category: 4,
            index: 251,
            name: Some("sp_draw"),
        }),
        (4, 252) => Some(ExtOpcode {
            category: 4,
            index: 252,
            name: None,
        }),
        (4, 253) => Some(ExtOpcode {
            category: 4,
            index: 253,
            name: Some("sp_unlock"),
        }),
        (4, 254) => Some(ExtOpcode {
            category: 4,
            index: 254,
            name: Some("sp_show"),
        }),
        (4, 255) => Some(ExtOpcode {
            category: 4,
            index: 255,
            name: Some("sp_hide"),
        }),
        (4, 256) => Some(ExtOpcode {
            category: 4,
            index: 256,
            name: None,
        }),
        (4, 257) => Some(ExtOpcode {
            category: 4,
            index: 257,
            name: Some("sp_set_child"),
        }),
        (4, 258) => Some(ExtOpcode {
            category: 4,
            index: 258,
            name: Some("sp_set_transition"),
        }),
        (4, 259) => Some(ExtOpcode {
            category: 4,
            index: 259,
            name: Some("sp_copy_image"),
        }),
        (4, 260) => Some(ExtOpcode {
            category: 4,
            index: 260,
            name: Some("sp_transition"),
        }),
        (4, 261) => Some(ExtOpcode {
            category: 4,
            index: 261,
            name: Some("set_aspect_position_type"),
        }),
        (4, 262) => Some(ExtOpcode {
            category: 4,
            index: 262,
            name: Some("get_backbuffer"),
        }),
        (4, 263) => Some(ExtOpcode {
            category: 4,
            index: 263,
            name: Some("sp_set_mask"),
        }),
        (4, 264) => Some(ExtOpcode {
            category: 4,
            index: 264,
            name: None,
        }),
        (4, 265) => Some(ExtOpcode {
            category: 4,
            index: 265,
            name: Some("spsetanime"),
        }),
        (4, 266) => Some(ExtOpcode {
            category: 4,
            index: 266,
            name: Some("drawtext"),
        }),
        (4, 267) => Some(ExtOpcode {
            category: 4,
            index: 267,
            name: None,
        }),
        (4, 268) => Some(ExtOpcode {
            category: 4,
            index: 268,
            name: None,
        }),
        (4, 270) => Some(ExtOpcode {
            category: 4,
            index: 270,
            name: Some("history_init_0x_0x"),
        }),
        (4, 271) => Some(ExtOpcode {
            category: 4,
            index: 271,
            name: Some("historybegin_lpbyte_ptagdata_sztext"),
        }),
        (4, 272) => Some(ExtOpcode {
            category: 4,
            index: 272,
            name: Some("history_end"),
        }),
        (4, 273) => Some(ExtOpcode {
            category: 4,
            index: 273,
            name: None,
        }),
        (4, 274) => Some(ExtOpcode {
            category: 4,
            index: 274,
            name: None,
        }),
        (4, 275) => Some(ExtOpcode {
            category: 4,
            index: 275,
            name: Some("history_get_height"),
        }),
        (4, 276) => Some(ExtOpcode {
            category: 4,
            index: 276,
            name: None,
        }),
        (4, 277) => Some(ExtOpcode {
            category: 4,
            index: 277,
            name: None,
        }),
        (4, 278) => Some(ExtOpcode {
            category: 4,
            index: 278,
            name: None,
        }),
        (4, 279) => Some(ExtOpcode {
            category: 4,
            index: 279,
            name: None,
        }),
        (4, 280) => Some(ExtOpcode {
            category: 4,
            index: 280,
            name: Some("history_set_rect"),
        }),
        (4, 281) => Some(ExtOpcode {
            category: 4,
            index: 281,
            name: Some("history_clear"),
        }),
        (4, 282) => Some(ExtOpcode {
            category: 4,
            index: 282,
            name: Some("history_set"),
        }),
        (4, 283) => Some(ExtOpcode {
            category: 4,
            index: 283,
            name: None,
        }),
        (4, 284) => Some(ExtOpcode {
            category: 4,
            index: 284,
            name: None,
        }),
        (4, 285) => Some(ExtOpcode {
            category: 4,
            index: 285,
            name: None,
        }),
        (4, 286) => Some(ExtOpcode {
            category: 4,
            index: 286,
            name: None,
        }),
        (4, 287) => Some(ExtOpcode {
            category: 4,
            index: 287,
            name: Some("history_set_face_call"),
        }),
        (4, 288) => Some(ExtOpcode {
            category: 4,
            index: 288,
            name: Some("history_set_face_sound"),
        }),
        (4, 289) => Some(ExtOpcode {
            category: 4,
            index: 289,
            name: Some("history_set_face_sound_release"),
        }),
        (4, 290) => Some(ExtOpcode {
            category: 4,
            index: 290,
            name: Some("history_get_text"),
        }),
        (4, 291) => Some(ExtOpcode {
            category: 4,
            index: 291,
            name: None,
        }),
        (4, 292) => Some(ExtOpcode {
            category: 4,
            index: 292,
            name: None,
        }),
        (4, 293) => Some(ExtOpcode {
            category: 4,
            index: 293,
            name: None,
        }),
        (4, 294) => Some(ExtOpcode {
            category: 4,
            index: 294,
            name: None,
        }),
        (4, 296) => Some(ExtOpcode {
            category: 4,
            index: 296,
            name: Some("movie_play"),
        }),
        (4, 297) => Some(ExtOpcode {
            category: 4,
            index: 297,
            name: Some("msp_set_loop_sp_ep"),
        }),
        (4, 298) => Some(ExtOpcode {
            category: 4,
            index: 298,
            name: Some("msp_cls"),
        }),
        (4, 299) => Some(ExtOpcode {
            category: 4,
            index: 299,
            name: Some("msp_wait"),
        }),
        (5, 0) => Some(ExtOpcode {
            category: 5,
            index: 0,
            name: Some("se_load"),
        }),
        (5, 1) => Some(ExtOpcode {
            category: 5,
            index: 1,
            name: Some("se_play"),
        }),
        (5, 2) => Some(ExtOpcode {
            category: 5,
            index: 2,
            name: Some("se_play_ex_ch"),
        }),
        (5, 3) => Some(ExtOpcode {
            category: 5,
            index: 3,
            name: Some("se_stop"),
        }),
        (5, 4) => Some(ExtOpcode {
            category: 5,
            index: 4,
            name: Some("se_set_volume"),
        }),
        (5, 5) => Some(ExtOpcode {
            category: 5,
            index: 5,
            name: Some("se_get_volume"),
        }),
        (5, 6) => Some(ExtOpcode {
            category: 5,
            index: 6,
            name: Some("se_unload"),
        }),
        (5, 7) => Some(ExtOpcode {
            category: 5,
            index: 7,
            name: Some("se_wait"),
        }),
        (5, 8) => Some(ExtOpcode {
            category: 5,
            index: 8,
            name: Some("channel_error_set_se_info"),
        }),
        (5, 9) => Some(ExtOpcode {
            category: 5,
            index: 9,
            name: Some("get_se_ex_volume"),
        }),
        (5, 10) => Some(ExtOpcode {
            category: 5,
            index: 10,
            name: Some("channel_error_set_se_ex_volume"),
        }),
        (5, 11) => Some(ExtOpcode {
            category: 5,
            index: 11,
            name: Some("channel_error_se_enable"),
        }),
        (5, 12) => Some(ExtOpcode {
            category: 5,
            index: 12,
            name: Some("channel_error_is_se_enable"),
        }),
        (5, 13) => Some(ExtOpcode {
            category: 5,
            index: 13,
            name: Some("se_set_pan"),
        }),
        (5, 14) => Some(ExtOpcode {
            category: 5,
            index: 14,
            name: Some("se_mute"),
        }),
        (5, 16) => Some(ExtOpcode {
            category: 5,
            index: 16,
            name: Some("select_init"),
        }),
        (5, 17) => Some(ExtOpcode {
            category: 5,
            index: 17,
            name: Some("select"),
        }),
        (5, 18) => Some(ExtOpcode {
            category: 5,
            index: 18,
            name: None,
        }),
        (5, 19) => Some(ExtOpcode {
            category: 5,
            index: 19,
            name: None,
        }),
        (5, 20) => Some(ExtOpcode {
            category: 5,
            index: 20,
            name: Some("select_clear"),
        }),
        (5, 21) => Some(ExtOpcode {
            category: 5,
            index: 21,
            name: Some("select_set_offset"),
        }),
        (5, 22) => Some(ExtOpcode {
            category: 5,
            index: 22,
            name: Some("select_set_process"),
        }),
        (5, 23) => Some(ExtOpcode {
            category: 5,
            index: 23,
            name: Some("select_lock"),
        }),
        (5, 24) => Some(ExtOpcode {
            category: 5,
            index: 24,
            name: Some("get_select_on_key"),
        }),
        (5, 25) => Some(ExtOpcode {
            category: 5,
            index: 25,
            name: Some("get_select_pull_key"),
        }),
        (5, 26) => Some(ExtOpcode {
            category: 5,
            index: 26,
            name: Some("get_select_push_key"),
        }),
        (5, 28) => Some(ExtOpcode {
            category: 5,
            index: 28,
            name: Some("skip_set"),
        }),
        (5, 29) => Some(ExtOpcode {
            category: 5,
            index: 29,
            name: Some("skip_is"),
        }),
        (5, 30) => Some(ExtOpcode {
            category: 5,
            index: 30,
            name: Some("auto_set"),
        }),
        (5, 31) => Some(ExtOpcode {
            category: 5,
            index: 31,
            name: Some("auto_is"),
        }),
        (5, 32) => Some(ExtOpcode {
            category: 5,
            index: 32,
            name: None,
        }),
        (5, 33) => Some(ExtOpcode {
            category: 5,
            index: 33,
            name: Some("auto_get_time"),
        }),
        (5, 34) => Some(ExtOpcode {
            category: 5,
            index: 34,
            name: None,
        }),
        (5, 35) => Some(ExtOpcode {
            category: 5,
            index: 35,
            name: None,
        }),
        (5, 36) => Some(ExtOpcode {
            category: 5,
            index: 36,
            name: None,
        }),
        (5, 37) => Some(ExtOpcode {
            category: 5,
            index: 37,
            name: None,
        }),
        (5, 38) => Some(ExtOpcode {
            category: 5,
            index: 38,
            name: None,
        }),
        (5, 39) => Some(ExtOpcode {
            category: 5,
            index: 39,
            name: None,
        }),
        (5, 40) => Some(ExtOpcode {
            category: 5,
            index: 40,
            name: None,
        }),
        (5, 41) => Some(ExtOpcode {
            category: 5,
            index: 41,
            name: None,
        }),
        (5, 42) => Some(ExtOpcode {
            category: 5,
            index: 42,
            name: None,
        }),
        (5, 43) => Some(ExtOpcode {
            category: 5,
            index: 43,
            name: Some("load_font"),
        }),
        (5, 44) => Some(ExtOpcode {
            category: 5,
            index: 44,
            name: Some("unload_font"),
        }),
        (5, 45) => Some(ExtOpcode {
            category: 5,
            index: 45,
            name: Some("set_language"),
        }),
        (5, 46) => Some(ExtOpcode {
            category: 5,
            index: 46,
            name: Some("key_canncel"),
        }),
        (5, 47) => Some(ExtOpcode {
            category: 5,
            index: 47,
            name: Some("set_font_color"),
        }),
        (5, 48) => Some(ExtOpcode {
            category: 5,
            index: 48,
            name: Some("load_font_ex"),
        }),
        (5, 49) => Some(ExtOpcode {
            category: 5,
            index: 49,
            name: None,
        }),
        (5, 50) => Some(ExtOpcode {
            category: 5,
            index: 50,
            name: None,
        }),
        (5, 51) => Some(ExtOpcode {
            category: 5,
            index: 51,
            name: None,
        }),
        (5, 52) => Some(ExtOpcode {
            category: 5,
            index: 52,
            name: None,
        }),
        (5, 53) => Some(ExtOpcode {
            category: 5,
            index: 53,
            name: None,
        }),
        (5, 54) => Some(ExtOpcode {
            category: 5,
            index: 54,
            name: None,
        }),
        (5, 55) => Some(ExtOpcode {
            category: 5,
            index: 55,
            name: Some("set_font_size"),
        }),
        (5, 56) => Some(ExtOpcode {
            category: 5,
            index: 56,
            name: Some("get_font_size"),
        }),
        (5, 57) => Some(ExtOpcode {
            category: 5,
            index: 57,
            name: Some("get_font_type"),
        }),
        (5, 58) => Some(ExtOpcode {
            category: 5,
            index: 58,
            name: Some("set_font_effect"),
        }),
        (5, 59) => Some(ExtOpcode {
            category: 5,
            index: 59,
            name: Some("get_font_effect"),
        }),
        (5, 60) => Some(ExtOpcode {
            category: 5,
            index: 60,
            name: Some("get_pull_key"),
        }),
        (5, 61) => Some(ExtOpcode {
            category: 5,
            index: 61,
            name: Some("get_on_key"),
        }),
        (5, 62) => Some(ExtOpcode {
            category: 5,
            index: 62,
            name: Some("get_push_key"),
        }),
        (5, 63) => Some(ExtOpcode {
            category: 5,
            index: 63,
            name: Some("input_clear"),
        }),
        (5, 64) => Some(ExtOpcode {
            category: 5,
            index: 64,
            name: Some("change_window_size"),
        }),
        (5, 65) => Some(ExtOpcode {
            category: 5,
            index: 65,
            name: Some("change_aspect_mode"),
        }),
        (5, 66) => Some(ExtOpcode {
            category: 5,
            index: 66,
            name: Some("aspect_position_enable"),
        }),
        (5, 67) => Some(ExtOpcode {
            category: 5,
            index: 67,
            name: None,
        }),
        (5, 68) => Some(ExtOpcode {
            category: 5,
            index: 68,
            name: Some("get_aspect_mode"),
        }),
        (5, 69) => Some(ExtOpcode {
            category: 5,
            index: 69,
            name: Some("get_monitor_size"),
        }),
        (5, 70) => Some(ExtOpcode {
            category: 5,
            index: 70,
            name: None,
        }),
        (5, 71) => Some(ExtOpcode {
            category: 5,
            index: 71,
            name: Some("get_system_metrics"),
        }),
        (5, 72) => Some(ExtOpcode {
            category: 5,
            index: 72,
            name: Some("set_system_path"),
        }),
        (5, 73) => Some(ExtOpcode {
            category: 5,
            index: 73,
            name: Some("set_allmosaicthumbnail"),
        }),
        (5, 74) => Some(ExtOpcode {
            category: 5,
            index: 74,
            name: Some("enable_window_change"),
        }),
        (5, 75) => Some(ExtOpcode {
            category: 5,
            index: 75,
            name: Some("is_enable_window_change"),
        }),
        (5, 76) => Some(ExtOpcode {
            category: 5,
            index: 76,
            name: Some("set_cursor_null"),
        }),
        (5, 77) => Some(ExtOpcode {
            category: 5,
            index: 77,
            name: Some("set_hide_cursor_time"),
        }),
        (5, 78) => Some(ExtOpcode {
            category: 5,
            index: 78,
            name: Some("get_hide_cursor_time"),
        }),
        (5, 79) => Some(ExtOpcode {
            category: 5,
            index: 79,
            name: Some("scene_skip"),
        }),
        (5, 80) => Some(ExtOpcode {
            category: 5,
            index: 80,
            name: None,
        }),
        (5, 81) => Some(ExtOpcode {
            category: 5,
            index: 81,
            name: None,
        }),
        (5, 82) => Some(ExtOpcode {
            category: 5,
            index: 82,
            name: Some("get_async_key"),
        }),
        (5, 83) => Some(ExtOpcode {
            category: 5,
            index: 83,
            name: Some("get_font_color"),
        }),
        (5, 84) => Some(ExtOpcode {
            category: 5,
            index: 84,
            name: None,
        }),
        (5, 85) => Some(ExtOpcode {
            category: 5,
            index: 85,
            name: Some("history_skip"),
        }),
        (5, 86) => Some(ExtOpcode {
            category: 5,
            index: 86,
            name: None,
        }),
        (5, 87) => Some(ExtOpcode {
            category: 5,
            index: 87,
            name: None,
        }),
        (5, 88) => Some(ExtOpcode {
            category: 5,
            index: 88,
            name: Some("set_language"),
        }),
        (5, 89) => Some(ExtOpcode {
            category: 5,
            index: 89,
            name: Some("set_achievement"),
        }),
        (5, 91) => Some(ExtOpcode {
            category: 5,
            index: 91,
            name: Some("system_btn_set"),
        }),
        (5, 92) => Some(ExtOpcode {
            category: 5,
            index: 92,
            name: Some("system_btn_release"),
        }),
        (5, 93) => Some(ExtOpcode {
            category: 5,
            index: 93,
            name: Some("system_btn_enable"),
        }),
        (5, 96) => Some(ExtOpcode {
            category: 5,
            index: 96,
            name: Some("text_init"),
        }),
        (5, 97) => Some(ExtOpcode {
            category: 5,
            index: 97,
            name: Some("text_set_icon"),
        }),
        (5, 98) => Some(ExtOpcode {
            category: 5,
            index: 98,
            name: Some("text"),
        }),
        (5, 99) => Some(ExtOpcode {
            category: 5,
            index: 99,
            name: Some("text_hide"),
        }),
        (5, 100) => Some(ExtOpcode {
            category: 5,
            index: 100,
            name: Some("text_show"),
        }),
        (5, 101) => Some(ExtOpcode {
            category: 5,
            index: 101,
            name: Some("text_set_btn"),
        }),
        (5, 102) => Some(ExtOpcode {
            category: 5,
            index: 102,
            name: Some("text_uninit"),
        }),
        (5, 103) => Some(ExtOpcode {
            category: 5,
            index: 103,
            name: Some("text_set_rect_invalid_param"),
        }),
        (5, 104) => Some(ExtOpcode {
            category: 5,
            index: 104,
            name: Some("text_clear"),
        }),
        (5, 105) => Some(ExtOpcode {
            category: 5,
            index: 105,
            name: None,
        }),
        (5, 106) => Some(ExtOpcode {
            category: 5,
            index: 106,
            name: Some("text_get_time"),
        }),
        (5, 107) => Some(ExtOpcode {
            category: 5,
            index: 107,
            name: Some("text_window_set_alpha"),
        }),
        (5, 108) => Some(ExtOpcode {
            category: 5,
            index: 108,
            name: Some("text_voice_play"),
        }),
        (5, 109) => Some(ExtOpcode {
            category: 5,
            index: 109,
            name: None,
        }),
        (5, 110) => Some(ExtOpcode {
            category: 5,
            index: 110,
            name: Some("text_set_icon_animation_time"),
        }),
        (5, 111) => Some(ExtOpcode {
            category: 5,
            index: 111,
            name: Some("text_w"),
        }),
        (5, 112) => Some(ExtOpcode {
            category: 5,
            index: 112,
            name: Some("text_a"),
        }),
        (5, 113) => Some(ExtOpcode {
            category: 5,
            index: 113,
            name: Some("text_wa"),
        }),
        (5, 114) => Some(ExtOpcode {
            category: 5,
            index: 114,
            name: Some("text_n"),
        }),
        (5, 115) => Some(ExtOpcode {
            category: 5,
            index: 115,
            name: Some("text_cat"),
        }),
        (5, 116) => Some(ExtOpcode {
            category: 5,
            index: 116,
            name: Some("set_history"),
        }),
        (5, 117) => Some(ExtOpcode {
            category: 5,
            index: 117,
            name: Some("is_text_visible"),
        }),
        (5, 118) => Some(ExtOpcode {
            category: 5,
            index: 118,
            name: Some("text_set_base"),
        }),
        (5, 119) => Some(ExtOpcode {
            category: 5,
            index: 119,
            name: Some("enable_voice_cut"),
        }),
        (5, 120) => Some(ExtOpcode {
            category: 5,
            index: 120,
            name: Some("is_voice_cut"),
        }),
        (5, 121) => Some(ExtOpcode {
            category: 5,
            index: 121,
            name: Some("texttimecheckset"),
        }),
        (5, 122) => Some(ExtOpcode {
            category: 5,
            index: 122,
            name: None,
        }),
        (5, 123) => Some(ExtOpcode {
            category: 5,
            index: 123,
            name: None,
        }),
        (5, 124) => Some(ExtOpcode {
            category: 5,
            index: 124,
            name: Some("text_set_color"),
        }),
        (5, 125) => Some(ExtOpcode {
            category: 5,
            index: 125,
            name: Some("textredraw"),
        }),
        (5, 126) => Some(ExtOpcode {
            category: 5,
            index: 126,
            name: Some("set_text_mode"),
        }),
        (5, 127) => Some(ExtOpcode {
            category: 5,
            index: 127,
            name: Some("text_init_visualnovelmode"),
        }),
        (5, 128) => Some(ExtOpcode {
            category: 5,
            index: 128,
            name: Some("text_set_icon_mode"),
        }),
        (5, 129) => Some(ExtOpcode {
            category: 5,
            index: 129,
            name: Some("text_vn_br"),
        }),
        (5, 130) => Some(ExtOpcode {
            category: 5,
            index: 130,
            name: None,
        }),
        (5, 131) => Some(ExtOpcode {
            category: 5,
            index: 131,
            name: None,
        }),
        (5, 132) => Some(ExtOpcode {
            category: 5,
            index: 132,
            name: None,
        }),
        (5, 133) => Some(ExtOpcode {
            category: 5,
            index: 133,
            name: None,
        }),
        (5, 134) => Some(ExtOpcode {
            category: 5,
            index: 134,
            name: Some("tips_get_str"),
        }),
        (5, 135) => Some(ExtOpcode {
            category: 5,
            index: 135,
            name: Some("tips_get_param"),
        }),
        (5, 136) => Some(ExtOpcode {
            category: 5,
            index: 136,
            name: Some("tips_reset"),
        }),
        (5, 137) => Some(ExtOpcode {
            category: 5,
            index: 137,
            name: Some("tips_search"),
        }),
        (5, 138) => Some(ExtOpcode {
            category: 5,
            index: 138,
            name: Some("tips_set_color"),
        }),
        (5, 139) => Some(ExtOpcode {
            category: 5,
            index: 139,
            name: Some("tips_stop"),
        }),
        (5, 140) => Some(ExtOpcode {
            category: 5,
            index: 140,
            name: Some("tips_get_flag"),
        }),
        (5, 141) => Some(ExtOpcode {
            category: 5,
            index: 141,
            name: Some("tips_init"),
        }),
        (5, 142) => Some(ExtOpcode {
            category: 5,
            index: 142,
            name: Some("tips_pause"),
        }),
        (5, 144) => Some(ExtOpcode {
            category: 5,
            index: 144,
            name: Some("voice_play"),
        }),
        (5, 145) => Some(ExtOpcode {
            category: 5,
            index: 145,
            name: Some("voice_stop"),
        }),
        (5, 146) => Some(ExtOpcode {
            category: 5,
            index: 146,
            name: Some("voice_set_volume"),
        }),
        (5, 147) => Some(ExtOpcode {
            category: 5,
            index: 147,
            name: Some("voice_get_volume"),
        }),
        (5, 148) => Some(ExtOpcode {
            category: 5,
            index: 148,
            name: Some("set_voice_info"),
        }),
        (5, 149) => Some(ExtOpcode {
            category: 5,
            index: 149,
            name: Some("voice_enable"),
        }),
        (5, 150) => Some(ExtOpcode {
            category: 5,
            index: 150,
            name: Some("is_voice_enable"),
        }),
        (5, 151) => Some(ExtOpcode {
            category: 5,
            index: 151,
            name: None,
        }),
        (5, 152) => Some(ExtOpcode {
            category: 5,
            index: 152,
            name: Some("bgv_play"),
        }),
        (5, 153) => Some(ExtOpcode {
            category: 5,
            index: 153,
            name: Some("bgv_stop"),
        }),
        (5, 154) => Some(ExtOpcode {
            category: 5,
            index: 154,
            name: Some("bgv_enable"),
        }),
        (5, 155) => Some(ExtOpcode {
            category: 5,
            index: 155,
            name: Some("get_voice_ex_volume"),
        }),
        (5, 156) => Some(ExtOpcode {
            category: 5,
            index: 156,
            name: Some("set_voice_ex_volume"),
        }),
        (5, 157) => Some(ExtOpcode {
            category: 5,
            index: 157,
            name: Some("voice_check_enable"),
        }),
        (5, 158) => Some(ExtOpcode {
            category: 5,
            index: 158,
            name: Some("voice_autopan_initialize"),
        }),
        (5, 159) => Some(ExtOpcode {
            category: 5,
            index: 159,
            name: Some("voice_autopan_enable"),
        }),
        (5, 160) => Some(ExtOpcode {
            category: 5,
            index: 160,
            name: Some("set_voice_autopan_size_over"),
        }),
        (5, 161) => Some(ExtOpcode {
            category: 5,
            index: 161,
            name: Some("is_voice_autopan_enable"),
        }),
        (5, 162) => Some(ExtOpcode {
            category: 5,
            index: 162,
            name: Some("voice_wait"),
        }),
        (5, 163) => Some(ExtOpcode {
            category: 5,
            index: 163,
            name: Some("bgv_pause"),
        }),
        (5, 164) => Some(ExtOpcode {
            category: 5,
            index: 164,
            name: Some("bgv_mute"),
        }),
        (5, 165) => Some(ExtOpcode {
            category: 5,
            index: 165,
            name: Some("set_bgv_volume"),
        }),
        (5, 166) => Some(ExtOpcode {
            category: 5,
            index: 166,
            name: Some("get_bgv_volume"),
        }),
        (5, 167) => Some(ExtOpcode {
            category: 5,
            index: 167,
            name: Some("set_bgv_auto_volume"),
        }),
        (5, 168) => Some(ExtOpcode {
            category: 5,
            index: 168,
            name: Some("voice_mute"),
        }),
        (5, 169) => Some(ExtOpcode {
            category: 5,
            index: 169,
            name: Some("voice_call"),
        }),
        (5, 170) => Some(ExtOpcode {
            category: 5,
            index: 170,
            name: Some("voice_call_clear"),
        }),
        (5, 172) => Some(ExtOpcode {
            category: 5,
            index: 172,
            name: Some("wait"),
        }),
        (5, 173) => Some(ExtOpcode {
            category: 5,
            index: 173,
            name: Some("wait_click"),
        }),
        (5, 174) => Some(ExtOpcode {
            category: 5,
            index: 174,
            name: Some("wait_sync_begin"),
        }),
        (5, 175) => Some(ExtOpcode {
            category: 5,
            index: 175,
            name: Some("wait_sync_release"),
        }),
        (5, 176) => Some(ExtOpcode {
            category: 5,
            index: 176,
            name: Some("wait_sync_end"),
        }),
        (5, 177) => Some(ExtOpcode {
            category: 5,
            index: 177,
            name: None,
        }),
        (5, 178) => Some(ExtOpcode {
            category: 5,
            index: 178,
            name: Some("wait_clear"),
        }),
        (5, 179) => Some(ExtOpcode {
            category: 5,
            index: 179,
            name: Some("wait_click_no_anim"),
        }),
        (5, 180) => Some(ExtOpcode {
            category: 5,
            index: 180,
            name: Some("wait_sync_get_time"),
        }),
        (5, 181) => Some(ExtOpcode {
            category: 5,
            index: 181,
            name: Some("wait_time_push"),
        }),
        (5, 182) => Some(ExtOpcode {
            category: 5,
            index: 182,
            name: Some("wait_time_pop"),
        }),
        (5, 246) => Some(ExtOpcode {
            category: 5,
            index: 246,
            name: None,
        }),
        (5, 249) => Some(ExtOpcode {
            category: 5,
            index: 249,
            name: None,
        }),
        (5, 252) => Some(ExtOpcode {
            category: 5,
            index: 252,
            name: None,
        }),
        (5, 263) => Some(ExtOpcode {
            category: 5,
            index: 263,
            name: None,
        }),
        (6, 0) => Some(ExtOpcode {
            category: 6,
            index: 0,
            name: Some("select_init"),
        }),
        (6, 1) => Some(ExtOpcode {
            category: 6,
            index: 1,
            name: Some("select"),
        }),
        (6, 2) => Some(ExtOpcode {
            category: 6,
            index: 2,
            name: None,
        }),
        (6, 3) => Some(ExtOpcode {
            category: 6,
            index: 3,
            name: None,
        }),
        (6, 4) => Some(ExtOpcode {
            category: 6,
            index: 4,
            name: Some("select_clear"),
        }),
        (6, 5) => Some(ExtOpcode {
            category: 6,
            index: 5,
            name: Some("select_set_offset"),
        }),
        (6, 6) => Some(ExtOpcode {
            category: 6,
            index: 6,
            name: Some("select_set_process"),
        }),
        (6, 7) => Some(ExtOpcode {
            category: 6,
            index: 7,
            name: Some("select_lock"),
        }),
        (6, 8) => Some(ExtOpcode {
            category: 6,
            index: 8,
            name: Some("get_select_on_key"),
        }),
        (6, 9) => Some(ExtOpcode {
            category: 6,
            index: 9,
            name: Some("get_select_pull_key"),
        }),
        (6, 10) => Some(ExtOpcode {
            category: 6,
            index: 10,
            name: Some("get_select_push_key"),
        }),
        (6, 12) => Some(ExtOpcode {
            category: 6,
            index: 12,
            name: Some("skip_set"),
        }),
        (6, 13) => Some(ExtOpcode {
            category: 6,
            index: 13,
            name: Some("skip_is"),
        }),
        (6, 14) => Some(ExtOpcode {
            category: 6,
            index: 14,
            name: Some("auto_set"),
        }),
        (6, 15) => Some(ExtOpcode {
            category: 6,
            index: 15,
            name: Some("auto_is"),
        }),
        (6, 16) => Some(ExtOpcode {
            category: 6,
            index: 16,
            name: None,
        }),
        (6, 17) => Some(ExtOpcode {
            category: 6,
            index: 17,
            name: Some("auto_get_time"),
        }),
        (6, 18) => Some(ExtOpcode {
            category: 6,
            index: 18,
            name: None,
        }),
        (6, 19) => Some(ExtOpcode {
            category: 6,
            index: 19,
            name: None,
        }),
        (6, 20) => Some(ExtOpcode {
            category: 6,
            index: 20,
            name: None,
        }),
        (6, 21) => Some(ExtOpcode {
            category: 6,
            index: 21,
            name: None,
        }),
        (6, 22) => Some(ExtOpcode {
            category: 6,
            index: 22,
            name: None,
        }),
        (6, 23) => Some(ExtOpcode {
            category: 6,
            index: 23,
            name: None,
        }),
        (6, 24) => Some(ExtOpcode {
            category: 6,
            index: 24,
            name: None,
        }),
        (6, 25) => Some(ExtOpcode {
            category: 6,
            index: 25,
            name: None,
        }),
        (6, 26) => Some(ExtOpcode {
            category: 6,
            index: 26,
            name: None,
        }),
        (6, 27) => Some(ExtOpcode {
            category: 6,
            index: 27,
            name: Some("load_font"),
        }),
        (6, 28) => Some(ExtOpcode {
            category: 6,
            index: 28,
            name: Some("unload_font"),
        }),
        (6, 29) => Some(ExtOpcode {
            category: 6,
            index: 29,
            name: Some("set_language"),
        }),
        (6, 30) => Some(ExtOpcode {
            category: 6,
            index: 30,
            name: Some("key_canncel"),
        }),
        (6, 31) => Some(ExtOpcode {
            category: 6,
            index: 31,
            name: Some("set_font_color"),
        }),
        (6, 32) => Some(ExtOpcode {
            category: 6,
            index: 32,
            name: Some("load_font_ex"),
        }),
        (6, 33) => Some(ExtOpcode {
            category: 6,
            index: 33,
            name: None,
        }),
        (6, 34) => Some(ExtOpcode {
            category: 6,
            index: 34,
            name: None,
        }),
        (6, 35) => Some(ExtOpcode {
            category: 6,
            index: 35,
            name: None,
        }),
        (6, 36) => Some(ExtOpcode {
            category: 6,
            index: 36,
            name: None,
        }),
        (6, 37) => Some(ExtOpcode {
            category: 6,
            index: 37,
            name: None,
        }),
        (6, 38) => Some(ExtOpcode {
            category: 6,
            index: 38,
            name: None,
        }),
        (6, 39) => Some(ExtOpcode {
            category: 6,
            index: 39,
            name: Some("set_font_size"),
        }),
        (6, 40) => Some(ExtOpcode {
            category: 6,
            index: 40,
            name: Some("get_font_size"),
        }),
        (6, 41) => Some(ExtOpcode {
            category: 6,
            index: 41,
            name: Some("get_font_type"),
        }),
        (6, 42) => Some(ExtOpcode {
            category: 6,
            index: 42,
            name: Some("set_font_effect"),
        }),
        (6, 43) => Some(ExtOpcode {
            category: 6,
            index: 43,
            name: Some("get_font_effect"),
        }),
        (6, 44) => Some(ExtOpcode {
            category: 6,
            index: 44,
            name: Some("get_pull_key"),
        }),
        (6, 45) => Some(ExtOpcode {
            category: 6,
            index: 45,
            name: Some("get_on_key"),
        }),
        (6, 46) => Some(ExtOpcode {
            category: 6,
            index: 46,
            name: Some("get_push_key"),
        }),
        (6, 47) => Some(ExtOpcode {
            category: 6,
            index: 47,
            name: Some("input_clear"),
        }),
        (6, 48) => Some(ExtOpcode {
            category: 6,
            index: 48,
            name: Some("change_window_size"),
        }),
        (6, 49) => Some(ExtOpcode {
            category: 6,
            index: 49,
            name: Some("change_aspect_mode"),
        }),
        (6, 50) => Some(ExtOpcode {
            category: 6,
            index: 50,
            name: Some("aspect_position_enable"),
        }),
        (6, 51) => Some(ExtOpcode {
            category: 6,
            index: 51,
            name: None,
        }),
        (6, 52) => Some(ExtOpcode {
            category: 6,
            index: 52,
            name: Some("get_aspect_mode"),
        }),
        (6, 53) => Some(ExtOpcode {
            category: 6,
            index: 53,
            name: Some("get_monitor_size"),
        }),
        (6, 54) => Some(ExtOpcode {
            category: 6,
            index: 54,
            name: None,
        }),
        (6, 55) => Some(ExtOpcode {
            category: 6,
            index: 55,
            name: Some("get_system_metrics"),
        }),
        (6, 56) => Some(ExtOpcode {
            category: 6,
            index: 56,
            name: Some("set_system_path"),
        }),
        (6, 57) => Some(ExtOpcode {
            category: 6,
            index: 57,
            name: Some("set_allmosaicthumbnail"),
        }),
        (6, 58) => Some(ExtOpcode {
            category: 6,
            index: 58,
            name: Some("enable_window_change"),
        }),
        (6, 59) => Some(ExtOpcode {
            category: 6,
            index: 59,
            name: Some("is_enable_window_change"),
        }),
        (6, 60) => Some(ExtOpcode {
            category: 6,
            index: 60,
            name: Some("set_cursor_null"),
        }),
        (6, 61) => Some(ExtOpcode {
            category: 6,
            index: 61,
            name: Some("set_hide_cursor_time"),
        }),
        (6, 62) => Some(ExtOpcode {
            category: 6,
            index: 62,
            name: Some("get_hide_cursor_time"),
        }),
        (6, 63) => Some(ExtOpcode {
            category: 6,
            index: 63,
            name: Some("scene_skip"),
        }),
        (6, 64) => Some(ExtOpcode {
            category: 6,
            index: 64,
            name: None,
        }),
        (6, 65) => Some(ExtOpcode {
            category: 6,
            index: 65,
            name: None,
        }),
        (6, 66) => Some(ExtOpcode {
            category: 6,
            index: 66,
            name: Some("get_async_key"),
        }),
        (6, 67) => Some(ExtOpcode {
            category: 6,
            index: 67,
            name: Some("get_font_color"),
        }),
        (6, 68) => Some(ExtOpcode {
            category: 6,
            index: 68,
            name: None,
        }),
        (6, 69) => Some(ExtOpcode {
            category: 6,
            index: 69,
            name: Some("history_skip"),
        }),
        (6, 70) => Some(ExtOpcode {
            category: 6,
            index: 70,
            name: None,
        }),
        (6, 71) => Some(ExtOpcode {
            category: 6,
            index: 71,
            name: None,
        }),
        (6, 72) => Some(ExtOpcode {
            category: 6,
            index: 72,
            name: Some("set_language"),
        }),
        (6, 73) => Some(ExtOpcode {
            category: 6,
            index: 73,
            name: Some("set_achievement"),
        }),
        (6, 75) => Some(ExtOpcode {
            category: 6,
            index: 75,
            name: Some("system_btn_set"),
        }),
        (6, 76) => Some(ExtOpcode {
            category: 6,
            index: 76,
            name: Some("system_btn_release"),
        }),
        (6, 77) => Some(ExtOpcode {
            category: 6,
            index: 77,
            name: Some("system_btn_enable"),
        }),
        (6, 80) => Some(ExtOpcode {
            category: 6,
            index: 80,
            name: Some("text_init"),
        }),
        (6, 81) => Some(ExtOpcode {
            category: 6,
            index: 81,
            name: Some("text_set_icon"),
        }),
        (6, 82) => Some(ExtOpcode {
            category: 6,
            index: 82,
            name: Some("text"),
        }),
        (6, 83) => Some(ExtOpcode {
            category: 6,
            index: 83,
            name: Some("text_hide"),
        }),
        (6, 84) => Some(ExtOpcode {
            category: 6,
            index: 84,
            name: Some("text_show"),
        }),
        (6, 85) => Some(ExtOpcode {
            category: 6,
            index: 85,
            name: Some("text_set_btn"),
        }),
        (6, 86) => Some(ExtOpcode {
            category: 6,
            index: 86,
            name: Some("text_uninit"),
        }),
        (6, 87) => Some(ExtOpcode {
            category: 6,
            index: 87,
            name: Some("text_set_rect_invalid_param"),
        }),
        (6, 88) => Some(ExtOpcode {
            category: 6,
            index: 88,
            name: Some("text_clear"),
        }),
        (6, 89) => Some(ExtOpcode {
            category: 6,
            index: 89,
            name: None,
        }),
        (6, 90) => Some(ExtOpcode {
            category: 6,
            index: 90,
            name: Some("text_get_time"),
        }),
        (6, 91) => Some(ExtOpcode {
            category: 6,
            index: 91,
            name: Some("text_window_set_alpha"),
        }),
        (6, 92) => Some(ExtOpcode {
            category: 6,
            index: 92,
            name: Some("text_voice_play"),
        }),
        (6, 93) => Some(ExtOpcode {
            category: 6,
            index: 93,
            name: None,
        }),
        (6, 94) => Some(ExtOpcode {
            category: 6,
            index: 94,
            name: Some("text_set_icon_animation_time"),
        }),
        (6, 95) => Some(ExtOpcode {
            category: 6,
            index: 95,
            name: Some("text_w"),
        }),
        (6, 96) => Some(ExtOpcode {
            category: 6,
            index: 96,
            name: Some("text_a"),
        }),
        (6, 97) => Some(ExtOpcode {
            category: 6,
            index: 97,
            name: Some("text_wa"),
        }),
        (6, 98) => Some(ExtOpcode {
            category: 6,
            index: 98,
            name: Some("text_n"),
        }),
        (6, 99) => Some(ExtOpcode {
            category: 6,
            index: 99,
            name: Some("text_cat"),
        }),
        (6, 100) => Some(ExtOpcode {
            category: 6,
            index: 100,
            name: Some("set_history"),
        }),
        (6, 101) => Some(ExtOpcode {
            category: 6,
            index: 101,
            name: Some("is_text_visible"),
        }),
        (6, 102) => Some(ExtOpcode {
            category: 6,
            index: 102,
            name: Some("text_set_base"),
        }),
        (6, 103) => Some(ExtOpcode {
            category: 6,
            index: 103,
            name: Some("enable_voice_cut"),
        }),
        (6, 104) => Some(ExtOpcode {
            category: 6,
            index: 104,
            name: Some("is_voice_cut"),
        }),
        (6, 105) => Some(ExtOpcode {
            category: 6,
            index: 105,
            name: Some("texttimecheckset"),
        }),
        (6, 106) => Some(ExtOpcode {
            category: 6,
            index: 106,
            name: None,
        }),
        (6, 107) => Some(ExtOpcode {
            category: 6,
            index: 107,
            name: None,
        }),
        (6, 108) => Some(ExtOpcode {
            category: 6,
            index: 108,
            name: Some("text_set_color"),
        }),
        (6, 109) => Some(ExtOpcode {
            category: 6,
            index: 109,
            name: Some("textredraw"),
        }),
        (6, 110) => Some(ExtOpcode {
            category: 6,
            index: 110,
            name: Some("set_text_mode"),
        }),
        (6, 111) => Some(ExtOpcode {
            category: 6,
            index: 111,
            name: Some("text_init_visualnovelmode"),
        }),
        (6, 112) => Some(ExtOpcode {
            category: 6,
            index: 112,
            name: Some("text_set_icon_mode"),
        }),
        (6, 113) => Some(ExtOpcode {
            category: 6,
            index: 113,
            name: Some("text_vn_br"),
        }),
        (6, 114) => Some(ExtOpcode {
            category: 6,
            index: 114,
            name: None,
        }),
        (6, 115) => Some(ExtOpcode {
            category: 6,
            index: 115,
            name: None,
        }),
        (6, 116) => Some(ExtOpcode {
            category: 6,
            index: 116,
            name: None,
        }),
        (6, 117) => Some(ExtOpcode {
            category: 6,
            index: 117,
            name: None,
        }),
        (6, 118) => Some(ExtOpcode {
            category: 6,
            index: 118,
            name: Some("tips_get_str"),
        }),
        (6, 119) => Some(ExtOpcode {
            category: 6,
            index: 119,
            name: Some("tips_get_param"),
        }),
        (6, 120) => Some(ExtOpcode {
            category: 6,
            index: 120,
            name: Some("tips_reset"),
        }),
        (6, 121) => Some(ExtOpcode {
            category: 6,
            index: 121,
            name: Some("tips_search"),
        }),
        (6, 122) => Some(ExtOpcode {
            category: 6,
            index: 122,
            name: Some("tips_set_color"),
        }),
        (6, 123) => Some(ExtOpcode {
            category: 6,
            index: 123,
            name: Some("tips_stop"),
        }),
        (6, 124) => Some(ExtOpcode {
            category: 6,
            index: 124,
            name: Some("tips_get_flag"),
        }),
        (6, 125) => Some(ExtOpcode {
            category: 6,
            index: 125,
            name: Some("tips_init"),
        }),
        (6, 126) => Some(ExtOpcode {
            category: 6,
            index: 126,
            name: Some("tips_pause"),
        }),
        (6, 128) => Some(ExtOpcode {
            category: 6,
            index: 128,
            name: Some("voice_play"),
        }),
        (6, 129) => Some(ExtOpcode {
            category: 6,
            index: 129,
            name: Some("voice_stop"),
        }),
        (6, 130) => Some(ExtOpcode {
            category: 6,
            index: 130,
            name: Some("voice_set_volume"),
        }),
        (6, 131) => Some(ExtOpcode {
            category: 6,
            index: 131,
            name: Some("voice_get_volume"),
        }),
        (6, 132) => Some(ExtOpcode {
            category: 6,
            index: 132,
            name: Some("set_voice_info"),
        }),
        (6, 133) => Some(ExtOpcode {
            category: 6,
            index: 133,
            name: Some("voice_enable"),
        }),
        (6, 134) => Some(ExtOpcode {
            category: 6,
            index: 134,
            name: Some("is_voice_enable"),
        }),
        (6, 135) => Some(ExtOpcode {
            category: 6,
            index: 135,
            name: None,
        }),
        (6, 136) => Some(ExtOpcode {
            category: 6,
            index: 136,
            name: Some("bgv_play"),
        }),
        (6, 137) => Some(ExtOpcode {
            category: 6,
            index: 137,
            name: Some("bgv_stop"),
        }),
        (6, 138) => Some(ExtOpcode {
            category: 6,
            index: 138,
            name: Some("bgv_enable"),
        }),
        (6, 139) => Some(ExtOpcode {
            category: 6,
            index: 139,
            name: Some("get_voice_ex_volume"),
        }),
        (6, 140) => Some(ExtOpcode {
            category: 6,
            index: 140,
            name: Some("set_voice_ex_volume"),
        }),
        (6, 141) => Some(ExtOpcode {
            category: 6,
            index: 141,
            name: Some("voice_check_enable"),
        }),
        (6, 142) => Some(ExtOpcode {
            category: 6,
            index: 142,
            name: Some("voice_autopan_initialize"),
        }),
        (6, 143) => Some(ExtOpcode {
            category: 6,
            index: 143,
            name: Some("voice_autopan_enable"),
        }),
        (6, 144) => Some(ExtOpcode {
            category: 6,
            index: 144,
            name: Some("set_voice_autopan_size_over"),
        }),
        (6, 145) => Some(ExtOpcode {
            category: 6,
            index: 145,
            name: Some("is_voice_autopan_enable"),
        }),
        (6, 146) => Some(ExtOpcode {
            category: 6,
            index: 146,
            name: Some("voice_wait"),
        }),
        (6, 147) => Some(ExtOpcode {
            category: 6,
            index: 147,
            name: Some("bgv_pause"),
        }),
        (6, 148) => Some(ExtOpcode {
            category: 6,
            index: 148,
            name: Some("bgv_mute"),
        }),
        (6, 149) => Some(ExtOpcode {
            category: 6,
            index: 149,
            name: Some("set_bgv_volume"),
        }),
        (6, 150) => Some(ExtOpcode {
            category: 6,
            index: 150,
            name: Some("get_bgv_volume"),
        }),
        (6, 151) => Some(ExtOpcode {
            category: 6,
            index: 151,
            name: Some("set_bgv_auto_volume"),
        }),
        (6, 152) => Some(ExtOpcode {
            category: 6,
            index: 152,
            name: Some("voice_mute"),
        }),
        (6, 153) => Some(ExtOpcode {
            category: 6,
            index: 153,
            name: Some("voice_call"),
        }),
        (6, 154) => Some(ExtOpcode {
            category: 6,
            index: 154,
            name: Some("voice_call_clear"),
        }),
        (6, 156) => Some(ExtOpcode {
            category: 6,
            index: 156,
            name: Some("wait"),
        }),
        (6, 157) => Some(ExtOpcode {
            category: 6,
            index: 157,
            name: Some("wait_click"),
        }),
        (6, 158) => Some(ExtOpcode {
            category: 6,
            index: 158,
            name: Some("wait_sync_begin"),
        }),
        (6, 159) => Some(ExtOpcode {
            category: 6,
            index: 159,
            name: Some("wait_sync_release"),
        }),
        (6, 160) => Some(ExtOpcode {
            category: 6,
            index: 160,
            name: Some("wait_sync_end"),
        }),
        (6, 161) => Some(ExtOpcode {
            category: 6,
            index: 161,
            name: None,
        }),
        (6, 162) => Some(ExtOpcode {
            category: 6,
            index: 162,
            name: Some("wait_clear"),
        }),
        (6, 163) => Some(ExtOpcode {
            category: 6,
            index: 163,
            name: Some("wait_click_no_anim"),
        }),
        (6, 164) => Some(ExtOpcode {
            category: 6,
            index: 164,
            name: Some("wait_sync_get_time"),
        }),
        (6, 165) => Some(ExtOpcode {
            category: 6,
            index: 165,
            name: Some("wait_time_push"),
        }),
        (6, 166) => Some(ExtOpcode {
            category: 6,
            index: 166,
            name: Some("wait_time_pop"),
        }),
        (6, 230) => Some(ExtOpcode {
            category: 6,
            index: 230,
            name: None,
        }),
        (6, 233) => Some(ExtOpcode {
            category: 6,
            index: 233,
            name: None,
        }),
        (6, 236) => Some(ExtOpcode {
            category: 6,
            index: 236,
            name: None,
        }),
        (6, 247) => Some(ExtOpcode {
            category: 6,
            index: 247,
            name: None,
        }),
        (6, 298) => Some(ExtOpcode {
            category: 6,
            index: 298,
            name: None,
        }),
        (7, 0) => Some(ExtOpcode {
            category: 7,
            index: 0,
            name: Some("wait"),
        }),
        (7, 1) => Some(ExtOpcode {
            category: 7,
            index: 1,
            name: Some("wait_click"),
        }),
        (7, 2) => Some(ExtOpcode {
            category: 7,
            index: 2,
            name: Some("wait_sync_begin"),
        }),
        (7, 3) => Some(ExtOpcode {
            category: 7,
            index: 3,
            name: Some("wait_sync_release"),
        }),
        (7, 4) => Some(ExtOpcode {
            category: 7,
            index: 4,
            name: Some("wait_sync_end"),
        }),
        (7, 5) => Some(ExtOpcode {
            category: 7,
            index: 5,
            name: None,
        }),
        (7, 6) => Some(ExtOpcode {
            category: 7,
            index: 6,
            name: Some("wait_clear"),
        }),
        (7, 7) => Some(ExtOpcode {
            category: 7,
            index: 7,
            name: Some("wait_click_no_anim"),
        }),
        (7, 8) => Some(ExtOpcode {
            category: 7,
            index: 8,
            name: Some("wait_sync_get_time"),
        }),
        (7, 9) => Some(ExtOpcode {
            category: 7,
            index: 9,
            name: Some("wait_time_push"),
        }),
        (7, 10) => Some(ExtOpcode {
            category: 7,
            index: 10,
            name: Some("wait_time_pop"),
        }),
        (7, 74) => Some(ExtOpcode {
            category: 7,
            index: 74,
            name: None,
        }),
        (7, 77) => Some(ExtOpcode {
            category: 7,
            index: 77,
            name: None,
        }),
        (7, 80) => Some(ExtOpcode {
            category: 7,
            index: 80,
            name: None,
        }),
        (7, 91) => Some(ExtOpcode {
            category: 7,
            index: 91,
            name: None,
        }),
        (7, 142) => Some(ExtOpcode {
            category: 7,
            index: 142,
            name: None,
        }),
        (7, 158) => Some(ExtOpcode {
            category: 7,
            index: 158,
            name: None,
        }),
        (7, 165) => Some(ExtOpcode {
            category: 7,
            index: 165,
            name: None,
        }),
        (7, 172) => Some(ExtOpcode {
            category: 7,
            index: 172,
            name: None,
        }),
        (7, 189) => Some(ExtOpcode {
            category: 7,
            index: 189,
            name: None,
        }),
        (7, 195) => Some(ExtOpcode {
            category: 7,
            index: 195,
            name: None,
        }),
        (7, 207) => Some(ExtOpcode {
            category: 7,
            index: 207,
            name: None,
        }),
        (7, 277) => Some(ExtOpcode {
            category: 7,
            index: 277,
            name: None,
        }),
        (7, 284) => Some(ExtOpcode {
            category: 7,
            index: 284,
            name: None,
        }),
        (7, 293) => Some(ExtOpcode {
            category: 7,
            index: 293,
            name: None,
        }),
        (8, 0) => Some(ExtOpcode {
            category: 8,
            index: 0,
            name: Some("btn_init"),
        }),
        (8, 1) => Some(ExtOpcode {
            category: 8,
            index: 1,
            name: Some("btn_uninit"),
        }),
        (8, 3) => Some(ExtOpcode {
            category: 8,
            index: 3,
            name: Some("btn_set"),
        }),
        (8, 4) => Some(ExtOpcode {
            category: 8,
            index: 4,
            name: Some("btn_hide"),
        }),
        (8, 5) => Some(ExtOpcode {
            category: 8,
            index: 5,
            name: Some("btn_show"),
        }),
        (8, 6) => Some(ExtOpcode {
            category: 8,
            index: 6,
            name: Some("btn_set_pos"),
        }),
        (8, 7) => Some(ExtOpcode {
            category: 8,
            index: 7,
            name: Some("btn_set_rect"),
        }),
        (8, 8) => Some(ExtOpcode {
            category: 8,
            index: 8,
            name: Some("btn_release"),
        }),
        (8, 9) => Some(ExtOpcode {
            category: 8,
            index: 9,
            name: None,
        }),
        (8, 10) => Some(ExtOpcode {
            category: 8,
            index: 10,
            name: None,
        }),
        (8, 11) => Some(ExtOpcode {
            category: 8,
            index: 11,
            name: None,
        }),
        (8, 12) => Some(ExtOpcode {
            category: 8,
            index: 12,
            name: None,
        }),
        (8, 13) => Some(ExtOpcode {
            category: 8,
            index: 13,
            name: Some("btn_set_toggle"),
        }),
        (8, 14) => Some(ExtOpcode {
            category: 8,
            index: 14,
            name: Some("btn_get_pos"),
        }),
        (8, 15) => Some(ExtOpcode {
            category: 8,
            index: 15,
            name: Some("btn_enable"),
        }),
        (8, 16) => Some(ExtOpcode {
            category: 8,
            index: 16,
            name: Some("btn_set_alpha_0x"),
        }),
        (8, 17) => Some(ExtOpcode {
            category: 8,
            index: 17,
            name: Some("btn_get_push"),
        }),
        (8, 18) => Some(ExtOpcode {
            category: 8,
            index: 18,
            name: Some("error_btn_expansion"),
        }),
        (8, 19) => Some(ExtOpcode {
            category: 8,
            index: 19,
            name: Some("btn_lock"),
        }),
        (8, 20) => Some(ExtOpcode {
            category: 8,
            index: 20,
            name: Some("btn_unlock"),
        }),
        (8, 21) => Some(ExtOpcode {
            category: 8,
            index: 21,
            name: Some("btn_set_anim"),
        }),
        (8, 22) => Some(ExtOpcode {
            category: 8,
            index: 22,
            name: Some("btn_set_hit"),
        }),
        (8, 23) => Some(ExtOpcode {
            category: 8,
            index: 23,
            name: Some("btn_get_onmouse"),
        }),
        (8, 24) => Some(ExtOpcode {
            category: 8,
            index: 24,
            name: Some("btn_anim_clear"),
        }),
        (8, 25) => Some(ExtOpcode {
            category: 8,
            index: 25,
            name: Some("btn_get_offmouse"),
        }),
        (8, 26) => Some(ExtOpcode {
            category: 8,
            index: 26,
            name: Some("btn_onmouse_clear"),
        }),
        (8, 27) => Some(ExtOpcode {
            category: 8,
            index: 27,
            name: Some("btn_blt"),
        }),
        (8, 28) => Some(ExtOpcode {
            category: 8,
            index: 28,
            name: Some("btn_link"),
        }),
        (8, 29) => Some(ExtOpcode {
            category: 8,
            index: 29,
            name: Some("btn_set_state"),
        }),
        (8, 30) => Some(ExtOpcode {
            category: 8,
            index: 30,
            name: Some("btn_get_link"),
        }),
        (8, 31) => Some(ExtOpcode {
            category: 8,
            index: 31,
            name: Some("btn_set_tips"),
        }),
        (8, 32) => Some(ExtOpcode {
            category: 8,
            index: 32,
            name: Some("btn_get_tips"),
        }),
        (8, 33) => Some(ExtOpcode {
            category: 8,
            index: 33,
            name: Some("btn_anime_is_true"),
        }),
        (8, 34) => Some(ExtOpcode {
            category: 8,
            index: 34,
            name: Some("btn_anime_get_status"),
        }),
        (8, 35) => Some(ExtOpcode {
            category: 8,
            index: 35,
            name: Some("btn_anime_finish"),
        }),
        (8, 36) => Some(ExtOpcode {
            category: 8,
            index: 36,
            name: Some("btn_mode"),
        }),
        (8, 37) => Some(ExtOpcode {
            category: 8,
            index: 37,
            name: Some("btn_get_alpha_0x"),
        }),
        (8, 38) => Some(ExtOpcode {
            category: 8,
            index: 38,
            name: Some("btn_on_check_0x"),
        }),
        (8, 40) => Some(ExtOpcode {
            category: 8,
            index: 40,
            name: None,
        }),
        (8, 41) => Some(ExtOpcode {
            category: 8,
            index: 41,
            name: None,
        }),
        (8, 42) => Some(ExtOpcode {
            category: 8,
            index: 42,
            name: Some("set_window_text"),
        }),
        (8, 43) => Some(ExtOpcode {
            category: 8,
            index: 43,
            name: None,
        }),
        (8, 44) => Some(ExtOpcode {
            category: 8,
            index: 44,
            name: None,
        }),
        (8, 45) => Some(ExtOpcode {
            category: 8,
            index: 45,
            name: None,
        }),
        (8, 46) => Some(ExtOpcode {
            category: 8,
            index: 46,
            name: None,
        }),
        (8, 47) => Some(ExtOpcode {
            category: 8,
            index: 47,
            name: None,
        }),
        (8, 48) => Some(ExtOpcode {
            category: 8,
            index: 48,
            name: None,
        }),
        (8, 49) => Some(ExtOpcode {
            category: 8,
            index: 49,
            name: None,
        }),
        (8, 50) => Some(ExtOpcode {
            category: 8,
            index: 50,
            name: Some("debug_break"),
        }),
        (8, 53) => Some(ExtOpcode {
            category: 8,
            index: 53,
            name: Some("app_exec"),
        }),
        (8, 54) => Some(ExtOpcode {
            category: 8,
            index: 54,
            name: Some("is_playername"),
        }),
        (8, 55) => Some(ExtOpcode {
            category: 8,
            index: 55,
            name: None,
        }),
        (8, 56) => Some(ExtOpcode {
            category: 8,
            index: 56,
            name: None,
        }),
        (8, 57) => Some(ExtOpcode {
            category: 8,
            index: 57,
            name: None,
        }),
        (8, 58) => Some(ExtOpcode {
            category: 8,
            index: 58,
            name: None,
        }),
        (8, 59) => Some(ExtOpcode {
            category: 8,
            index: 59,
            name: None,
        }),
        (8, 60) => Some(ExtOpcode {
            category: 8,
            index: 60,
            name: Some("file_exist"),
        }),
        (8, 61) => Some(ExtOpcode {
            category: 8,
            index: 61,
            name: Some("wsprint"),
        }),
        (8, 62) => Some(ExtOpcode {
            category: 8,
            index: 62,
            name: Some("check_disc"),
        }),
        (8, 63) => Some(ExtOpcode {
            category: 8,
            index: 63,
            name: None,
        }),
        (8, 64) => Some(ExtOpcode {
            category: 8,
            index: 64,
            name: None,
        }),
        (8, 65) => Some(ExtOpcode {
            category: 8,
            index: 65,
            name: None,
        }),
        (8, 66) => Some(ExtOpcode {
            category: 8,
            index: 66,
            name: Some("update_access"),
        }),
        (8, 67) => Some(ExtOpcode {
            category: 8,
            index: 67,
            name: None,
        }),
        (8, 68) => Some(ExtOpcode {
            category: 8,
            index: 68,
            name: None,
        }),
        (8, 69) => Some(ExtOpcode {
            category: 8,
            index: 69,
            name: None,
        }),
        (8, 70) => Some(ExtOpcode {
            category: 8,
            index: 70,
            name: None,
        }),
        (8, 71) => Some(ExtOpcode {
            category: 8,
            index: 71,
            name: None,
        }),
        (8, 72) => Some(ExtOpcode {
            category: 8,
            index: 72,
            name: None,
        }),
        (8, 73) => Some(ExtOpcode {
            category: 8,
            index: 73,
            name: None,
        }),
        (8, 74) => Some(ExtOpcode {
            category: 8,
            index: 74,
            name: Some("player_name_set_begin"),
        }),
        (8, 75) => Some(ExtOpcode {
            category: 8,
            index: 75,
            name: Some("player_name_set_end"),
        }),
        (8, 76) => Some(ExtOpcode {
            category: 8,
            index: 76,
            name: Some("player_name_set_check"),
        }),
        (8, 77) => Some(ExtOpcode {
            category: 8,
            index: 77,
            name: None,
        }),
        (8, 78) => Some(ExtOpcode {
            category: 8,
            index: 78,
            name: Some("player_name_reset"),
        }),
        (8, 79) => Some(ExtOpcode {
            category: 8,
            index: 79,
            name: Some("player_name_set_direct"),
        }),
        (8, 80) => Some(ExtOpcode {
            category: 8,
            index: 80,
            name: None,
        }),
        (8, 81) => Some(ExtOpcode {
            category: 8,
            index: 81,
            name: None,
        }),
        (8, 82) => Some(ExtOpcode {
            category: 8,
            index: 82,
            name: Some("openfile"),
        }),
        (8, 83) => Some(ExtOpcode {
            category: 8,
            index: 83,
            name: Some("read_file"),
        }),
        (8, 84) => Some(ExtOpcode {
            category: 8,
            index: 84,
            name: Some("close_file_not_handle"),
        }),
        (8, 85) => Some(ExtOpcode {
            category: 8,
            index: 85,
            name: Some("set_file_pointer"),
        }),
        (8, 86) => Some(ExtOpcode {
            category: 8,
            index: 86,
            name: Some("file_string"),
        }),
        (8, 87) => Some(ExtOpcode {
            category: 8,
            index: 87,
            name: Some("set_last_process"),
        }),
        (8, 88) => Some(ExtOpcode {
            category: 8,
            index: 88,
            name: Some("sz_buf"),
        }),
        (8, 89) => Some(ExtOpcode {
            category: 8,
            index: 89,
            name: Some("getprivateprofileint"),
        }),
        (8, 90) => Some(ExtOpcode {
            category: 8,
            index: 90,
            name: None,
        }),
        (8, 91) => Some(ExtOpcode {
            category: 8,
            index: 91,
            name: None,
        }),
        (8, 92) => Some(ExtOpcode {
            category: 8,
            index: 92,
            name: None,
        }),
        (8, 93) => Some(ExtOpcode {
            category: 8,
            index: 93,
            name: Some("is_tweet"),
        }),
        (8, 94) => Some(ExtOpcode {
            category: 8,
            index: 94,
            name: None,
        }),
        (8, 95) => Some(ExtOpcode {
            category: 8,
            index: 95,
            name: None,
        }),
        (8, 96) => Some(ExtOpcode {
            category: 8,
            index: 96,
            name: None,
        }),
        (8, 97) => Some(ExtOpcode {
            category: 8,
            index: 97,
            name: None,
        }),
        (8, 98) => Some(ExtOpcode {
            category: 8,
            index: 98,
            name: None,
        }),
        (8, 99) => Some(ExtOpcode {
            category: 8,
            index: 99,
            name: None,
        }),
        (8, 100) => Some(ExtOpcode {
            category: 8,
            index: 100,
            name: None,
        }),
        (8, 101) => Some(ExtOpcode {
            category: 8,
            index: 101,
            name: None,
        }),
        (8, 102) => Some(ExtOpcode {
            category: 8,
            index: 102,
            name: None,
        }),
        (8, 103) => Some(ExtOpcode {
            category: 8,
            index: 103,
            name: None,
        }),
        (8, 104) => Some(ExtOpcode {
            category: 8,
            index: 104,
            name: None,
        }),
        (8, 105) => Some(ExtOpcode {
            category: 8,
            index: 105,
            name: None,
        }),
        (8, 106) => Some(ExtOpcode {
            category: 8,
            index: 106,
            name: None,
        }),
        (8, 107) => Some(ExtOpcode {
            category: 8,
            index: 107,
            name: None,
        }),
        (8, 108) => Some(ExtOpcode {
            category: 8,
            index: 108,
            name: None,
        }),
        (8, 109) => Some(ExtOpcode {
            category: 8,
            index: 109,
            name: None,
        }),
        (8, 110) => Some(ExtOpcode {
            category: 8,
            index: 110,
            name: None,
        }),
        (8, 111) => Some(ExtOpcode {
            category: 8,
            index: 111,
            name: None,
        }),
        (8, 112) => Some(ExtOpcode {
            category: 8,
            index: 112,
            name: None,
        }),
        (8, 113) => Some(ExtOpcode {
            category: 8,
            index: 113,
            name: None,
        }),
        (8, 114) => Some(ExtOpcode {
            category: 8,
            index: 114,
            name: None,
        }),
        (8, 115) => Some(ExtOpcode {
            category: 8,
            index: 115,
            name: None,
        }),
        (8, 116) => Some(ExtOpcode {
            category: 8,
            index: 116,
            name: None,
        }),
        (8, 117) => Some(ExtOpcode {
            category: 8,
            index: 117,
            name: None,
        }),
        (8, 118) => Some(ExtOpcode {
            category: 8,
            index: 118,
            name: None,
        }),
        (8, 119) => Some(ExtOpcode {
            category: 8,
            index: 119,
            name: None,
        }),
        (8, 120) => Some(ExtOpcode {
            category: 8,
            index: 120,
            name: None,
        }),
        (8, 121) => Some(ExtOpcode {
            category: 8,
            index: 121,
            name: None,
        }),
        (8, 122) => Some(ExtOpcode {
            category: 8,
            index: 122,
            name: None,
        }),
        (8, 123) => Some(ExtOpcode {
            category: 8,
            index: 123,
            name: None,
        }),
        (8, 124) => Some(ExtOpcode {
            category: 8,
            index: 124,
            name: None,
        }),
        (8, 125) => Some(ExtOpcode {
            category: 8,
            index: 125,
            name: None,
        }),
        (8, 126) => Some(ExtOpcode {
            category: 8,
            index: 126,
            name: Some("result_tweet"),
        }),
        (8, 127) => Some(ExtOpcode {
            category: 8,
            index: 127,
            name: Some("get_tweet_key"),
        }),
        (8, 128) => Some(ExtOpcode {
            category: 8,
            index: 128,
            name: Some("set_tweet_key"),
        }),
        (8, 129) => Some(ExtOpcode {
            category: 8,
            index: 129,
            name: None,
        }),
        (8, 130) => Some(ExtOpcode {
            category: 8,
            index: 130,
            name: Some("tweet_authorize"),
        }),
        (8, 131) => Some(ExtOpcode {
            category: 8,
            index: 131,
            name: None,
        }),
        (8, 132) => Some(ExtOpcode {
            category: 8,
            index: 132,
            name: None,
        }),
        (8, 133) => Some(ExtOpcode {
            category: 8,
            index: 133,
            name: None,
        }),
        (8, 134) => Some(ExtOpcode {
            category: 8,
            index: 134,
            name: Some("tips_csv_read_error"),
        }),
        (8, 135) => Some(ExtOpcode {
            category: 8,
            index: 135,
            name: Some("tips_csv_get_error"),
        }),
        (8, 136) => Some(ExtOpcode {
            category: 8,
            index: 136,
            name: Some("tips_csv_search_not_found"),
        }),
        (8, 137) => Some(ExtOpcode {
            category: 8,
            index: 137,
            name: None,
        }),
        (8, 138) => Some(ExtOpcode {
            category: 8,
            index: 138,
            name: None,
        }),
        (8, 139) => Some(ExtOpcode {
            category: 8,
            index: 139,
            name: Some("tips_acc_save"),
        }),
        (8, 140) => Some(ExtOpcode {
            category: 8,
            index: 140,
            name: Some("is_network"),
        }),
        (8, 141) => Some(ExtOpcode {
            category: 8,
            index: 141,
            name: Some("is_touch"),
        }),
        (8, 142) => Some(ExtOpcode {
            category: 8,
            index: 142,
            name: None,
        }),
        (8, 143) => Some(ExtOpcode {
            category: 8,
            index: 143,
            name: None,
        }),
        (8, 144) => Some(ExtOpcode {
            category: 8,
            index: 144,
            name: None,
        }),
        (8, 145) => Some(ExtOpcode {
            category: 8,
            index: 145,
            name: None,
        }),
        (8, 146) => Some(ExtOpcode {
            category: 8,
            index: 146,
            name: None,
        }),
        (8, 147) => Some(ExtOpcode {
            category: 8,
            index: 147,
            name: None,
        }),
        (8, 148) => Some(ExtOpcode {
            category: 8,
            index: 148,
            name: None,
        }),
        (8, 149) => Some(ExtOpcode {
            category: 8,
            index: 149,
            name: None,
        }),
        (8, 150) => Some(ExtOpcode {
            category: 8,
            index: 150,
            name: None,
        }),
        (8, 151) => Some(ExtOpcode {
            category: 8,
            index: 151,
            name: None,
        }),
        (8, 152) => Some(ExtOpcode {
            category: 8,
            index: 152,
            name: None,
        }),
        (8, 153) => Some(ExtOpcode {
            category: 8,
            index: 153,
            name: None,
        }),
        (8, 154) => Some(ExtOpcode {
            category: 8,
            index: 154,
            name: None,
        }),
        (8, 155) => Some(ExtOpcode {
            category: 8,
            index: 155,
            name: None,
        }),
        (8, 156) => Some(ExtOpcode {
            category: 8,
            index: 156,
            name: None,
        }),
        (8, 157) => Some(ExtOpcode {
            category: 8,
            index: 157,
            name: None,
        }),
        (8, 158) => Some(ExtOpcode {
            category: 8,
            index: 158,
            name: None,
        }),
        (8, 159) => Some(ExtOpcode {
            category: 8,
            index: 159,
            name: None,
        }),
        (8, 160) => Some(ExtOpcode {
            category: 8,
            index: 160,
            name: None,
        }),
        (8, 161) => Some(ExtOpcode {
            category: 8,
            index: 161,
            name: None,
        }),
        (8, 162) => Some(ExtOpcode {
            category: 8,
            index: 162,
            name: None,
        }),
        (8, 163) => Some(ExtOpcode {
            category: 8,
            index: 163,
            name: None,
        }),
        (8, 164) => Some(ExtOpcode {
            category: 8,
            index: 164,
            name: None,
        }),
        (8, 166) => Some(ExtOpcode {
            category: 8,
            index: 166,
            name: None,
        }),
        (8, 167) => Some(ExtOpcode {
            category: 8,
            index: 167,
            name: Some("run_no_wait"),
        }),
        (8, 168) => Some(ExtOpcode {
            category: 8,
            index: 168,
            name: Some("run_stack"),
        }),
        (8, 170) => Some(ExtOpcode {
            category: 8,
            index: 170,
            name: Some("fx_effect_cls"),
        }),
        (8, 171) => Some(ExtOpcode {
            category: 8,
            index: 171,
            name: Some("fx_raster_stop"),
        }),
        (8, 172) => Some(ExtOpcode {
            category: 8,
            index: 172,
            name: Some("fx_effect_wait"),
        }),
        (8, 173) => Some(ExtOpcode {
            category: 8,
            index: 173,
            name: None,
        }),
        (8, 175) => Some(ExtOpcode {
            category: 8,
            index: 175,
            name: Some("random"),
        }),
        (8, 176) => Some(ExtOpcode {
            category: 8,
            index: 176,
            name: Some("abs"),
        }),
        (8, 177) => Some(ExtOpcode {
            category: 8,
            index: 177,
            name: Some("sin"),
        }),
        (8, 178) => Some(ExtOpcode {
            category: 8,
            index: 178,
            name: Some("cos"),
        }),
        (8, 179) => Some(ExtOpcode {
            category: 8,
            index: 179,
            name: Some("tan"),
        }),
        (8, 180) => Some(ExtOpcode {
            category: 8,
            index: 180,
            name: Some("atan"),
        }),
        (8, 181) => Some(ExtOpcode {
            category: 8,
            index: 181,
            name: Some("log"),
        }),
        (8, 182) => Some(ExtOpcode {
            category: 8,
            index: 182,
            name: Some("log10"),
        }),
        (8, 183) => Some(ExtOpcode {
            category: 8,
            index: 183,
            name: None,
        }),
        (8, 184) => Some(ExtOpcode {
            category: 8,
            index: 184,
            name: Some("sqrt"),
        }),
        (8, 185) => Some(ExtOpcode {
            category: 8,
            index: 185,
            name: None,
        }),
        (8, 186) => Some(ExtOpcode {
            category: 8,
            index: 186,
            name: None,
        }),
        (8, 190) => Some(ExtOpcode {
            category: 8,
            index: 190,
            name: Some("sp_set"),
        }),
        (8, 191) => Some(ExtOpcode {
            category: 8,
            index: 191,
            name: Some("sp_set_ex"),
        }),
        (8, 192) => Some(ExtOpcode {
            category: 8,
            index: 192,
            name: Some("sp_set_pos"),
        }),
        (8, 193) => Some(ExtOpcode {
            category: 8,
            index: 193,
            name: Some("sp_cls"),
        }),
        (8, 194) => Some(ExtOpcode {
            category: 8,
            index: 194,
            name: Some("sp_set_alpha"),
        }),
        (8, 195) => Some(ExtOpcode {
            category: 8,
            index: 195,
            name: Some("set_priority"),
        }),
        (8, 196) => Some(ExtOpcode {
            category: 8,
            index: 196,
            name: None,
        }),
        (8, 197) => Some(ExtOpcode {
            category: 8,
            index: 197,
            name: Some("sp_set_center"),
        }),
        (8, 199) => Some(ExtOpcode {
            category: 8,
            index: 199,
            name: Some("sp_cls_ex"),
        }),
        (8, 200) => Some(ExtOpcode {
            category: 8,
            index: 200,
            name: Some("set_filter"),
        }),
        (8, 201) => Some(ExtOpcode {
            category: 8,
            index: 201,
            name: Some("sp_cls_transition"),
        }),
        (8, 202) => Some(ExtOpcode {
            category: 8,
            index: 202,
            name: Some("sp_set_pos_ex"),
        }),
        (8, 203) => Some(ExtOpcode {
            category: 8,
            index: 203,
            name: Some("sp_set_rect_pos"),
        }),
        (8, 204) => Some(ExtOpcode {
            category: 8,
            index: 204,
            name: None,
        }),
        (8, 205) => Some(ExtOpcode {
            category: 8,
            index: 205,
            name: Some("sp_set_scale"),
        }),
        (8, 206) => Some(ExtOpcode {
            category: 8,
            index: 206,
            name: Some("sp_set_rotate"),
        }),
        (8, 207) => Some(ExtOpcode {
            category: 8,
            index: 207,
            name: Some("face_init"),
        }),
        (8, 208) => Some(ExtOpcode {
            category: 8,
            index: 208,
            name: Some("face_set"),
        }),
        (8, 209) => Some(ExtOpcode {
            category: 8,
            index: 209,
            name: Some("not_image_sp_get_color"),
        }),
        (8, 210) => Some(ExtOpcode {
            category: 8,
            index: 210,
            name: Some("sptext"),
        }),
        (8, 211) => Some(ExtOpcode {
            category: 8,
            index: 211,
            name: Some("face_cls"),
        }),
        (8, 212) => Some(ExtOpcode {
            category: 8,
            index: 212,
            name: Some("sp_set_rect"),
        }),
        (8, 213) => Some(ExtOpcode {
            category: 8,
            index: 213,
            name: Some("sp_set_pos_move"),
        }),
        (8, 214) => Some(ExtOpcode {
            category: 8,
            index: 214,
            name: Some("not_image_sp_get_alpha"),
        }),
        (8, 215) => Some(ExtOpcode {
            category: 8,
            index: 215,
            name: Some("not_image_sp_get_rotate"),
        }),
        (8, 216) => Some(ExtOpcode {
            category: 8,
            index: 216,
            name: None,
        }),
        (8, 217) => Some(ExtOpcode {
            category: 8,
            index: 217,
            name: None,
        }),
        (8, 218) => Some(ExtOpcode {
            category: 8,
            index: 218,
            name: None,
        }),
        (8, 219) => Some(ExtOpcode {
            category: 8,
            index: 219,
            name: None,
        }),
        (8, 220) => Some(ExtOpcode {
            category: 8,
            index: 220,
            name: Some("sp_create"),
        }),
        (8, 221) => Some(ExtOpcode {
            category: 8,
            index: 221,
            name: Some("sp_anime_clear"),
        }),
        (8, 222) => Some(ExtOpcode {
            category: 8,
            index: 222,
            name: None,
        }),
        (8, 223) => Some(ExtOpcode {
            category: 8,
            index: 223,
            name: None,
        }),
        (8, 224) => Some(ExtOpcode {
            category: 8,
            index: 224,
            name: Some("not_image_sp_get_scale"),
        }),
        (8, 225) => Some(ExtOpcode {
            category: 8,
            index: 225,
            name: Some("sp_set_color_0x"),
        }),
        (8, 226) => Some(ExtOpcode {
            category: 8,
            index: 226,
            name: Some("sp_bitblt"),
        }),
        (8, 227) => Some(ExtOpcode {
            category: 8,
            index: 227,
            name: Some("sp_set_shake"),
        }),
        (8, 228) => Some(ExtOpcode {
            category: 8,
            index: 228,
            name: Some("sp_paint"),
        }),
        (8, 229) => Some(ExtOpcode {
            category: 8,
            index: 229,
            name: None,
        }),
        (8, 230) => Some(ExtOpcode {
            category: 8,
            index: 230,
            name: Some("sp_load_wait_time"),
        }),
        (8, 231) => Some(ExtOpcode {
            category: 8,
            index: 231,
            name: Some("sp_draw"),
        }),
        (8, 232) => Some(ExtOpcode {
            category: 8,
            index: 232,
            name: None,
        }),
        (8, 233) => Some(ExtOpcode {
            category: 8,
            index: 233,
            name: Some("sp_unlock"),
        }),
        (8, 234) => Some(ExtOpcode {
            category: 8,
            index: 234,
            name: Some("sp_show"),
        }),
        (8, 235) => Some(ExtOpcode {
            category: 8,
            index: 235,
            name: Some("sp_hide"),
        }),
        (8, 236) => Some(ExtOpcode {
            category: 8,
            index: 236,
            name: None,
        }),
        (8, 237) => Some(ExtOpcode {
            category: 8,
            index: 237,
            name: Some("sp_set_child"),
        }),
        (8, 238) => Some(ExtOpcode {
            category: 8,
            index: 238,
            name: Some("sp_set_transition"),
        }),
        (8, 239) => Some(ExtOpcode {
            category: 8,
            index: 239,
            name: Some("sp_copy_image"),
        }),
        (8, 240) => Some(ExtOpcode {
            category: 8,
            index: 240,
            name: Some("sp_transition"),
        }),
        (8, 241) => Some(ExtOpcode {
            category: 8,
            index: 241,
            name: Some("set_aspect_position_type"),
        }),
        (8, 242) => Some(ExtOpcode {
            category: 8,
            index: 242,
            name: Some("get_backbuffer"),
        }),
        (8, 243) => Some(ExtOpcode {
            category: 8,
            index: 243,
            name: Some("sp_set_mask"),
        }),
        (8, 244) => Some(ExtOpcode {
            category: 8,
            index: 244,
            name: None,
        }),
        (8, 245) => Some(ExtOpcode {
            category: 8,
            index: 245,
            name: Some("spsetanime"),
        }),
        (8, 246) => Some(ExtOpcode {
            category: 8,
            index: 246,
            name: Some("drawtext"),
        }),
        (8, 247) => Some(ExtOpcode {
            category: 8,
            index: 247,
            name: None,
        }),
        (8, 248) => Some(ExtOpcode {
            category: 8,
            index: 248,
            name: None,
        }),
        (8, 250) => Some(ExtOpcode {
            category: 8,
            index: 250,
            name: Some("history_init_0x_0x"),
        }),
        (8, 251) => Some(ExtOpcode {
            category: 8,
            index: 251,
            name: Some("historybegin_lpbyte_ptagdata_sztext"),
        }),
        (8, 252) => Some(ExtOpcode {
            category: 8,
            index: 252,
            name: Some("history_end"),
        }),
        (8, 253) => Some(ExtOpcode {
            category: 8,
            index: 253,
            name: None,
        }),
        (8, 254) => Some(ExtOpcode {
            category: 8,
            index: 254,
            name: None,
        }),
        (8, 255) => Some(ExtOpcode {
            category: 8,
            index: 255,
            name: Some("history_get_height"),
        }),
        (8, 256) => Some(ExtOpcode {
            category: 8,
            index: 256,
            name: None,
        }),
        (8, 257) => Some(ExtOpcode {
            category: 8,
            index: 257,
            name: None,
        }),
        (8, 258) => Some(ExtOpcode {
            category: 8,
            index: 258,
            name: None,
        }),
        (8, 259) => Some(ExtOpcode {
            category: 8,
            index: 259,
            name: None,
        }),
        (8, 260) => Some(ExtOpcode {
            category: 8,
            index: 260,
            name: Some("history_set_rect"),
        }),
        (8, 261) => Some(ExtOpcode {
            category: 8,
            index: 261,
            name: Some("history_clear"),
        }),
        (8, 262) => Some(ExtOpcode {
            category: 8,
            index: 262,
            name: Some("history_set"),
        }),
        (8, 263) => Some(ExtOpcode {
            category: 8,
            index: 263,
            name: None,
        }),
        (8, 264) => Some(ExtOpcode {
            category: 8,
            index: 264,
            name: None,
        }),
        (8, 265) => Some(ExtOpcode {
            category: 8,
            index: 265,
            name: None,
        }),
        (8, 266) => Some(ExtOpcode {
            category: 8,
            index: 266,
            name: None,
        }),
        (8, 267) => Some(ExtOpcode {
            category: 8,
            index: 267,
            name: Some("history_set_face_call"),
        }),
        (8, 268) => Some(ExtOpcode {
            category: 8,
            index: 268,
            name: Some("history_set_face_sound"),
        }),
        (8, 269) => Some(ExtOpcode {
            category: 8,
            index: 269,
            name: Some("history_set_face_sound_release"),
        }),
        (8, 270) => Some(ExtOpcode {
            category: 8,
            index: 270,
            name: Some("history_get_text"),
        }),
        (8, 271) => Some(ExtOpcode {
            category: 8,
            index: 271,
            name: None,
        }),
        (8, 272) => Some(ExtOpcode {
            category: 8,
            index: 272,
            name: None,
        }),
        (8, 273) => Some(ExtOpcode {
            category: 8,
            index: 273,
            name: None,
        }),
        (8, 274) => Some(ExtOpcode {
            category: 8,
            index: 274,
            name: None,
        }),
        (8, 276) => Some(ExtOpcode {
            category: 8,
            index: 276,
            name: Some("movie_play"),
        }),
        (8, 277) => Some(ExtOpcode {
            category: 8,
            index: 277,
            name: Some("msp_set_loop_sp_ep"),
        }),
        (8, 278) => Some(ExtOpcode {
            category: 8,
            index: 278,
            name: Some("msp_cls"),
        }),
        (8, 279) => Some(ExtOpcode {
            category: 8,
            index: 279,
            name: Some("msp_wait"),
        }),
        (8, 280) => Some(ExtOpcode {
            category: 8,
            index: 280,
            name: Some("msp_lock"),
        }),
        (8, 281) => Some(ExtOpcode {
            category: 8,
            index: 281,
            name: Some("msp_unlock"),
        }),
        (8, 282) => Some(ExtOpcode {
            category: 8,
            index: 282,
            name: Some("msp_play"),
        }),
        (8, 283) => Some(ExtOpcode {
            category: 8,
            index: 283,
            name: Some("msp_stop"),
        }),
        (8, 285) => Some(ExtOpcode {
            category: 8,
            index: 285,
            name: Some("create_thread"),
        }),
        (8, 286) => Some(ExtOpcode {
            category: 8,
            index: 286,
            name: Some("exit_thread"),
        }),
        (8, 287) => Some(ExtOpcode {
            category: 8,
            index: 287,
            name: None,
        }),
        (8, 288) => Some(ExtOpcode {
            category: 8,
            index: 288,
            name: Some("get_thread"),
        }),
        (8, 291) => Some(ExtOpcode {
            category: 8,
            index: 291,
            name: Some("mov"),
        }),
        (8, 292) => Some(ExtOpcode {
            category: 8,
            index: 292,
            name: Some("add"),
        }),
        (8, 293) => Some(ExtOpcode {
            category: 8,
            index: 293,
            name: Some("sub"),
        }),
        (8, 294) => Some(ExtOpcode {
            category: 8,
            index: 294,
            name: Some("mul"),
        }),
        (8, 295) => Some(ExtOpcode {
            category: 8,
            index: 295,
            name: Some("div"),
        }),
        (8, 296) => Some(ExtOpcode {
            category: 8,
            index: 296,
            name: Some("bitand"),
        }),
        (8, 297) => Some(ExtOpcode {
            category: 8,
            index: 297,
            name: Some("bitor"),
        }),
        (8, 298) => Some(ExtOpcode {
            category: 8,
            index: 298,
            name: Some("bitxor"),
        }),
        (8, 299) => Some(ExtOpcode {
            category: 8,
            index: 299,
            name: Some("jmp_point"),
        }),
        (9, 0) => Some(ExtOpcode {
            category: 9,
            index: 0,
            name: Some("skip_set"),
        }),
        (9, 1) => Some(ExtOpcode {
            category: 9,
            index: 1,
            name: Some("skip_is"),
        }),
        (9, 2) => Some(ExtOpcode {
            category: 9,
            index: 2,
            name: Some("auto_set"),
        }),
        (9, 3) => Some(ExtOpcode {
            category: 9,
            index: 3,
            name: Some("auto_is"),
        }),
        (9, 4) => Some(ExtOpcode {
            category: 9,
            index: 4,
            name: None,
        }),
        (9, 5) => Some(ExtOpcode {
            category: 9,
            index: 5,
            name: Some("auto_get_time"),
        }),
        (9, 6) => Some(ExtOpcode {
            category: 9,
            index: 6,
            name: None,
        }),
        (9, 7) => Some(ExtOpcode {
            category: 9,
            index: 7,
            name: None,
        }),
        (9, 8) => Some(ExtOpcode {
            category: 9,
            index: 8,
            name: None,
        }),
        (9, 9) => Some(ExtOpcode {
            category: 9,
            index: 9,
            name: None,
        }),
        (9, 10) => Some(ExtOpcode {
            category: 9,
            index: 10,
            name: None,
        }),
        (9, 11) => Some(ExtOpcode {
            category: 9,
            index: 11,
            name: None,
        }),
        (9, 12) => Some(ExtOpcode {
            category: 9,
            index: 12,
            name: None,
        }),
        (9, 13) => Some(ExtOpcode {
            category: 9,
            index: 13,
            name: None,
        }),
        (9, 14) => Some(ExtOpcode {
            category: 9,
            index: 14,
            name: None,
        }),
        (9, 15) => Some(ExtOpcode {
            category: 9,
            index: 15,
            name: Some("load_font"),
        }),
        (9, 16) => Some(ExtOpcode {
            category: 9,
            index: 16,
            name: Some("unload_font"),
        }),
        (9, 17) => Some(ExtOpcode {
            category: 9,
            index: 17,
            name: Some("set_language"),
        }),
        (9, 18) => Some(ExtOpcode {
            category: 9,
            index: 18,
            name: Some("key_canncel"),
        }),
        (9, 19) => Some(ExtOpcode {
            category: 9,
            index: 19,
            name: Some("set_font_color"),
        }),
        (9, 20) => Some(ExtOpcode {
            category: 9,
            index: 20,
            name: Some("load_font_ex"),
        }),
        (9, 21) => Some(ExtOpcode {
            category: 9,
            index: 21,
            name: None,
        }),
        (9, 22) => Some(ExtOpcode {
            category: 9,
            index: 22,
            name: None,
        }),
        (9, 23) => Some(ExtOpcode {
            category: 9,
            index: 23,
            name: None,
        }),
        (9, 24) => Some(ExtOpcode {
            category: 9,
            index: 24,
            name: None,
        }),
        (9, 25) => Some(ExtOpcode {
            category: 9,
            index: 25,
            name: None,
        }),
        (9, 26) => Some(ExtOpcode {
            category: 9,
            index: 26,
            name: None,
        }),
        (9, 27) => Some(ExtOpcode {
            category: 9,
            index: 27,
            name: Some("set_font_size"),
        }),
        (9, 28) => Some(ExtOpcode {
            category: 9,
            index: 28,
            name: Some("get_font_size"),
        }),
        (9, 29) => Some(ExtOpcode {
            category: 9,
            index: 29,
            name: Some("get_font_type"),
        }),
        (9, 30) => Some(ExtOpcode {
            category: 9,
            index: 30,
            name: Some("set_font_effect"),
        }),
        (9, 31) => Some(ExtOpcode {
            category: 9,
            index: 31,
            name: Some("get_font_effect"),
        }),
        (9, 32) => Some(ExtOpcode {
            category: 9,
            index: 32,
            name: Some("get_pull_key"),
        }),
        (9, 33) => Some(ExtOpcode {
            category: 9,
            index: 33,
            name: Some("get_on_key"),
        }),
        (9, 34) => Some(ExtOpcode {
            category: 9,
            index: 34,
            name: Some("get_push_key"),
        }),
        (9, 35) => Some(ExtOpcode {
            category: 9,
            index: 35,
            name: Some("input_clear"),
        }),
        (9, 36) => Some(ExtOpcode {
            category: 9,
            index: 36,
            name: Some("change_window_size"),
        }),
        (9, 37) => Some(ExtOpcode {
            category: 9,
            index: 37,
            name: Some("change_aspect_mode"),
        }),
        (9, 38) => Some(ExtOpcode {
            category: 9,
            index: 38,
            name: Some("aspect_position_enable"),
        }),
        (9, 39) => Some(ExtOpcode {
            category: 9,
            index: 39,
            name: None,
        }),
        (9, 40) => Some(ExtOpcode {
            category: 9,
            index: 40,
            name: Some("get_aspect_mode"),
        }),
        (9, 41) => Some(ExtOpcode {
            category: 9,
            index: 41,
            name: Some("get_monitor_size"),
        }),
        (9, 42) => Some(ExtOpcode {
            category: 9,
            index: 42,
            name: None,
        }),
        (9, 43) => Some(ExtOpcode {
            category: 9,
            index: 43,
            name: Some("get_system_metrics"),
        }),
        (9, 44) => Some(ExtOpcode {
            category: 9,
            index: 44,
            name: Some("set_system_path"),
        }),
        (9, 45) => Some(ExtOpcode {
            category: 9,
            index: 45,
            name: Some("set_allmosaicthumbnail"),
        }),
        (9, 46) => Some(ExtOpcode {
            category: 9,
            index: 46,
            name: Some("enable_window_change"),
        }),
        (9, 47) => Some(ExtOpcode {
            category: 9,
            index: 47,
            name: Some("is_enable_window_change"),
        }),
        (9, 48) => Some(ExtOpcode {
            category: 9,
            index: 48,
            name: Some("set_cursor_null"),
        }),
        (9, 49) => Some(ExtOpcode {
            category: 9,
            index: 49,
            name: Some("set_hide_cursor_time"),
        }),
        (9, 50) => Some(ExtOpcode {
            category: 9,
            index: 50,
            name: Some("get_hide_cursor_time"),
        }),
        (9, 51) => Some(ExtOpcode {
            category: 9,
            index: 51,
            name: Some("scene_skip"),
        }),
        (9, 52) => Some(ExtOpcode {
            category: 9,
            index: 52,
            name: None,
        }),
        (9, 53) => Some(ExtOpcode {
            category: 9,
            index: 53,
            name: None,
        }),
        (9, 54) => Some(ExtOpcode {
            category: 9,
            index: 54,
            name: Some("get_async_key"),
        }),
        (9, 55) => Some(ExtOpcode {
            category: 9,
            index: 55,
            name: Some("get_font_color"),
        }),
        (9, 56) => Some(ExtOpcode {
            category: 9,
            index: 56,
            name: None,
        }),
        (9, 57) => Some(ExtOpcode {
            category: 9,
            index: 57,
            name: Some("history_skip"),
        }),
        (9, 58) => Some(ExtOpcode {
            category: 9,
            index: 58,
            name: None,
        }),
        (9, 59) => Some(ExtOpcode {
            category: 9,
            index: 59,
            name: None,
        }),
        (9, 60) => Some(ExtOpcode {
            category: 9,
            index: 60,
            name: Some("set_language"),
        }),
        (9, 61) => Some(ExtOpcode {
            category: 9,
            index: 61,
            name: Some("set_achievement"),
        }),
        (9, 63) => Some(ExtOpcode {
            category: 9,
            index: 63,
            name: Some("system_btn_set"),
        }),
        (9, 64) => Some(ExtOpcode {
            category: 9,
            index: 64,
            name: Some("system_btn_release"),
        }),
        (9, 65) => Some(ExtOpcode {
            category: 9,
            index: 65,
            name: Some("system_btn_enable"),
        }),
        (9, 68) => Some(ExtOpcode {
            category: 9,
            index: 68,
            name: Some("text_init"),
        }),
        (9, 69) => Some(ExtOpcode {
            category: 9,
            index: 69,
            name: Some("text_set_icon"),
        }),
        (9, 70) => Some(ExtOpcode {
            category: 9,
            index: 70,
            name: Some("text"),
        }),
        (9, 71) => Some(ExtOpcode {
            category: 9,
            index: 71,
            name: Some("text_hide"),
        }),
        (9, 72) => Some(ExtOpcode {
            category: 9,
            index: 72,
            name: Some("text_show"),
        }),
        (9, 73) => Some(ExtOpcode {
            category: 9,
            index: 73,
            name: Some("text_set_btn"),
        }),
        (9, 74) => Some(ExtOpcode {
            category: 9,
            index: 74,
            name: Some("text_uninit"),
        }),
        (9, 75) => Some(ExtOpcode {
            category: 9,
            index: 75,
            name: Some("text_set_rect_invalid_param"),
        }),
        (9, 76) => Some(ExtOpcode {
            category: 9,
            index: 76,
            name: Some("text_clear"),
        }),
        (9, 77) => Some(ExtOpcode {
            category: 9,
            index: 77,
            name: None,
        }),
        (9, 78) => Some(ExtOpcode {
            category: 9,
            index: 78,
            name: Some("text_get_time"),
        }),
        (9, 79) => Some(ExtOpcode {
            category: 9,
            index: 79,
            name: Some("text_window_set_alpha"),
        }),
        (9, 80) => Some(ExtOpcode {
            category: 9,
            index: 80,
            name: Some("text_voice_play"),
        }),
        (9, 81) => Some(ExtOpcode {
            category: 9,
            index: 81,
            name: None,
        }),
        (9, 82) => Some(ExtOpcode {
            category: 9,
            index: 82,
            name: Some("text_set_icon_animation_time"),
        }),
        (9, 83) => Some(ExtOpcode {
            category: 9,
            index: 83,
            name: Some("text_w"),
        }),
        (9, 84) => Some(ExtOpcode {
            category: 9,
            index: 84,
            name: Some("text_a"),
        }),
        (9, 85) => Some(ExtOpcode {
            category: 9,
            index: 85,
            name: Some("text_wa"),
        }),
        (9, 86) => Some(ExtOpcode {
            category: 9,
            index: 86,
            name: Some("text_n"),
        }),
        (9, 87) => Some(ExtOpcode {
            category: 9,
            index: 87,
            name: Some("text_cat"),
        }),
        (9, 88) => Some(ExtOpcode {
            category: 9,
            index: 88,
            name: Some("set_history"),
        }),
        (9, 89) => Some(ExtOpcode {
            category: 9,
            index: 89,
            name: Some("is_text_visible"),
        }),
        (9, 90) => Some(ExtOpcode {
            category: 9,
            index: 90,
            name: Some("text_set_base"),
        }),
        (9, 91) => Some(ExtOpcode {
            category: 9,
            index: 91,
            name: Some("enable_voice_cut"),
        }),
        (9, 92) => Some(ExtOpcode {
            category: 9,
            index: 92,
            name: Some("is_voice_cut"),
        }),
        (9, 93) => Some(ExtOpcode {
            category: 9,
            index: 93,
            name: Some("texttimecheckset"),
        }),
        (9, 94) => Some(ExtOpcode {
            category: 9,
            index: 94,
            name: None,
        }),
        (9, 95) => Some(ExtOpcode {
            category: 9,
            index: 95,
            name: None,
        }),
        (9, 96) => Some(ExtOpcode {
            category: 9,
            index: 96,
            name: Some("text_set_color"),
        }),
        (9, 97) => Some(ExtOpcode {
            category: 9,
            index: 97,
            name: Some("textredraw"),
        }),
        (9, 98) => Some(ExtOpcode {
            category: 9,
            index: 98,
            name: Some("set_text_mode"),
        }),
        (9, 99) => Some(ExtOpcode {
            category: 9,
            index: 99,
            name: Some("text_init_visualnovelmode"),
        }),
        (9, 100) => Some(ExtOpcode {
            category: 9,
            index: 100,
            name: Some("text_set_icon_mode"),
        }),
        (9, 101) => Some(ExtOpcode {
            category: 9,
            index: 101,
            name: Some("text_vn_br"),
        }),
        (9, 102) => Some(ExtOpcode {
            category: 9,
            index: 102,
            name: None,
        }),
        (9, 103) => Some(ExtOpcode {
            category: 9,
            index: 103,
            name: None,
        }),
        (9, 104) => Some(ExtOpcode {
            category: 9,
            index: 104,
            name: None,
        }),
        (9, 105) => Some(ExtOpcode {
            category: 9,
            index: 105,
            name: None,
        }),
        (9, 106) => Some(ExtOpcode {
            category: 9,
            index: 106,
            name: Some("tips_get_str"),
        }),
        (9, 107) => Some(ExtOpcode {
            category: 9,
            index: 107,
            name: Some("tips_get_param"),
        }),
        (9, 108) => Some(ExtOpcode {
            category: 9,
            index: 108,
            name: Some("tips_reset"),
        }),
        (9, 109) => Some(ExtOpcode {
            category: 9,
            index: 109,
            name: Some("tips_search"),
        }),
        (9, 110) => Some(ExtOpcode {
            category: 9,
            index: 110,
            name: Some("tips_set_color"),
        }),
        (9, 111) => Some(ExtOpcode {
            category: 9,
            index: 111,
            name: Some("tips_stop"),
        }),
        (9, 112) => Some(ExtOpcode {
            category: 9,
            index: 112,
            name: Some("tips_get_flag"),
        }),
        (9, 113) => Some(ExtOpcode {
            category: 9,
            index: 113,
            name: Some("tips_init"),
        }),
        (9, 114) => Some(ExtOpcode {
            category: 9,
            index: 114,
            name: Some("tips_pause"),
        }),
        (9, 116) => Some(ExtOpcode {
            category: 9,
            index: 116,
            name: Some("voice_play"),
        }),
        (9, 117) => Some(ExtOpcode {
            category: 9,
            index: 117,
            name: Some("voice_stop"),
        }),
        (9, 118) => Some(ExtOpcode {
            category: 9,
            index: 118,
            name: Some("voice_set_volume"),
        }),
        (9, 119) => Some(ExtOpcode {
            category: 9,
            index: 119,
            name: Some("voice_get_volume"),
        }),
        (9, 120) => Some(ExtOpcode {
            category: 9,
            index: 120,
            name: Some("set_voice_info"),
        }),
        (9, 121) => Some(ExtOpcode {
            category: 9,
            index: 121,
            name: Some("voice_enable"),
        }),
        (9, 122) => Some(ExtOpcode {
            category: 9,
            index: 122,
            name: Some("is_voice_enable"),
        }),
        (9, 123) => Some(ExtOpcode {
            category: 9,
            index: 123,
            name: None,
        }),
        (9, 124) => Some(ExtOpcode {
            category: 9,
            index: 124,
            name: Some("bgv_play"),
        }),
        (9, 125) => Some(ExtOpcode {
            category: 9,
            index: 125,
            name: Some("bgv_stop"),
        }),
        (9, 126) => Some(ExtOpcode {
            category: 9,
            index: 126,
            name: Some("bgv_enable"),
        }),
        (9, 127) => Some(ExtOpcode {
            category: 9,
            index: 127,
            name: Some("get_voice_ex_volume"),
        }),
        (9, 128) => Some(ExtOpcode {
            category: 9,
            index: 128,
            name: Some("set_voice_ex_volume"),
        }),
        (9, 129) => Some(ExtOpcode {
            category: 9,
            index: 129,
            name: Some("voice_check_enable"),
        }),
        (9, 130) => Some(ExtOpcode {
            category: 9,
            index: 130,
            name: Some("voice_autopan_initialize"),
        }),
        (9, 131) => Some(ExtOpcode {
            category: 9,
            index: 131,
            name: Some("voice_autopan_enable"),
        }),
        (9, 132) => Some(ExtOpcode {
            category: 9,
            index: 132,
            name: Some("set_voice_autopan_size_over"),
        }),
        (9, 133) => Some(ExtOpcode {
            category: 9,
            index: 133,
            name: Some("is_voice_autopan_enable"),
        }),
        (9, 134) => Some(ExtOpcode {
            category: 9,
            index: 134,
            name: Some("voice_wait"),
        }),
        (9, 135) => Some(ExtOpcode {
            category: 9,
            index: 135,
            name: Some("bgv_pause"),
        }),
        (9, 136) => Some(ExtOpcode {
            category: 9,
            index: 136,
            name: Some("bgv_mute"),
        }),
        (9, 137) => Some(ExtOpcode {
            category: 9,
            index: 137,
            name: Some("set_bgv_volume"),
        }),
        (9, 138) => Some(ExtOpcode {
            category: 9,
            index: 138,
            name: Some("get_bgv_volume"),
        }),
        (9, 139) => Some(ExtOpcode {
            category: 9,
            index: 139,
            name: Some("set_bgv_auto_volume"),
        }),
        (9, 140) => Some(ExtOpcode {
            category: 9,
            index: 140,
            name: Some("voice_mute"),
        }),
        (9, 141) => Some(ExtOpcode {
            category: 9,
            index: 141,
            name: Some("voice_call"),
        }),
        (9, 142) => Some(ExtOpcode {
            category: 9,
            index: 142,
            name: Some("voice_call_clear"),
        }),
        (9, 144) => Some(ExtOpcode {
            category: 9,
            index: 144,
            name: Some("wait"),
        }),
        (9, 145) => Some(ExtOpcode {
            category: 9,
            index: 145,
            name: Some("wait_click"),
        }),
        (9, 146) => Some(ExtOpcode {
            category: 9,
            index: 146,
            name: Some("wait_sync_begin"),
        }),
        (9, 147) => Some(ExtOpcode {
            category: 9,
            index: 147,
            name: Some("wait_sync_release"),
        }),
        (9, 148) => Some(ExtOpcode {
            category: 9,
            index: 148,
            name: Some("wait_sync_end"),
        }),
        (9, 149) => Some(ExtOpcode {
            category: 9,
            index: 149,
            name: None,
        }),
        (9, 150) => Some(ExtOpcode {
            category: 9,
            index: 150,
            name: Some("wait_clear"),
        }),
        (9, 151) => Some(ExtOpcode {
            category: 9,
            index: 151,
            name: Some("wait_click_no_anim"),
        }),
        (9, 152) => Some(ExtOpcode {
            category: 9,
            index: 152,
            name: Some("wait_sync_get_time"),
        }),
        (9, 153) => Some(ExtOpcode {
            category: 9,
            index: 153,
            name: Some("wait_time_push"),
        }),
        (9, 154) => Some(ExtOpcode {
            category: 9,
            index: 154,
            name: Some("wait_time_pop"),
        }),
        (9, 218) => Some(ExtOpcode {
            category: 9,
            index: 218,
            name: None,
        }),
        (9, 221) => Some(ExtOpcode {
            category: 9,
            index: 221,
            name: None,
        }),
        (9, 224) => Some(ExtOpcode {
            category: 9,
            index: 224,
            name: None,
        }),
        (9, 235) => Some(ExtOpcode {
            category: 9,
            index: 235,
            name: None,
        }),
        (9, 286) => Some(ExtOpcode {
            category: 9,
            index: 286,
            name: None,
        }),
        (10, 0) => Some(ExtOpcode {
            category: 10,
            index: 0,
            name: Some("save"),
        }),
        (10, 1) => Some(ExtOpcode {
            category: 10,
            index: 1,
            name: Some("load"),
        }),
        (10, 2) => Some(ExtOpcode {
            category: 10,
            index: 2,
            name: Some("save_set_title"),
        }),
        (10, 3) => Some(ExtOpcode {
            category: 10,
            index: 3,
            name: Some("save_data"),
        }),
        (10, 4) => Some(ExtOpcode {
            category: 10,
            index: 4,
            name: Some("save_set_thumbnail_size"),
        }),
        (10, 5) => Some(ExtOpcode {
            category: 10,
            index: 5,
            name: Some("thumbnail_set"),
        }),
        (10, 6) => Some(ExtOpcode {
            category: 10,
            index: 6,
            name: Some("savetitledraw"),
        }),
        (10, 7) => Some(ExtOpcode {
            category: 10,
            index: 7,
            name: Some("save_set_font_size"),
        }),
        (10, 8) => Some(ExtOpcode {
            category: 10,
            index: 8,
            name: Some("getsaveday"),
        }),
        (10, 9) => Some(ExtOpcode {
            category: 10,
            index: 9,
            name: Some("is_save"),
        }),
        (10, 10) => Some(ExtOpcode {
            category: 10,
            index: 10,
            name: Some("getsaveusermemory"),
        }),
        (10, 11) => Some(ExtOpcode {
            category: 10,
            index: 11,
            name: Some("savepoint"),
        }),
        (10, 12) => Some(ExtOpcode {
            category: 10,
            index: 12,
            name: Some("save_thumbnail_mosaic_set"),
        }),
        (10, 13) => Some(ExtOpcode {
            category: 10,
            index: 13,
            name: Some("savetimedraw"),
        }),
        (10, 14) => Some(ExtOpcode {
            category: 10,
            index: 14,
            name: Some("savedaydraw"),
        }),
        (10, 15) => Some(ExtOpcode {
            category: 10,
            index: 15,
            name: Some("save_set_text_rect"),
        }),
        (10, 16) => Some(ExtOpcode {
            category: 10,
            index: 16,
            name: Some("savetextdraw"),
        }),
        (10, 17) => Some(ExtOpcode {
            category: 10,
            index: 17,
            name: Some("get_new_savefile"),
        }),
        (10, 21) => Some(ExtOpcode {
            category: 10,
            index: 21,
            name: Some("setsavetext"),
        }),
        (10, 22) => Some(ExtOpcode {
            category: 10,
            index: 22,
            name: Some("thumbnail_renew"),
        }),
        (10, 23) => Some(ExtOpcode {
            category: 10,
            index: 23,
            name: Some("save_set_font_type"),
        }),
        (10, 24) => Some(ExtOpcode {
            category: 10,
            index: 24,
            name: Some("set_load_after_process"),
        }),
        (10, 25) => Some(ExtOpcode {
            category: 10,
            index: 25,
            name: Some("savesystemdata"),
        }),
        (10, 26) => Some(ExtOpcode {
            category: 10,
            index: 26,
            name: Some("save_set_font_effect"),
        }),
        (10, 27) => Some(ExtOpcode {
            category: 10,
            index: 27,
            name: Some("save_set_font_color_0x_0x"),
        }),
        (10, 28) => Some(ExtOpcode {
            category: 10,
            index: 28,
            name: Some("delete_file"),
        }),
        (10, 29) => Some(ExtOpcode {
            category: 10,
            index: 29,
            name: Some("save_tmp_dat"),
        }),
        (10, 30) => Some(ExtOpcode {
            category: 10,
            index: 30,
            name: Some("copy_file"),
        }),
        (10, 31) => Some(ExtOpcode {
            category: 10,
            index: 31,
            name: Some("load_thumbnail"),
        }),
        (10, 32) => Some(ExtOpcode {
            category: 10,
            index: 32,
            name: Some("save_lock_not_open_savefileno"),
        }),
        (10, 33) => Some(ExtOpcode {
            category: 10,
            index: 33,
            name: Some("is_save_lock"),
        }),
        (10, 34) => Some(ExtOpcode {
            category: 10,
            index: 34,
            name: Some("is_prev_data"),
        }),
        (10, 35) => Some(ExtOpcode {
            category: 10,
            index: 35,
            name: Some("save_point_clear"),
        }),
        (10, 36) => Some(ExtOpcode {
            category: 10,
            index: 36,
            name: Some("save_point_lock"),
        }),
        (10, 37) => Some(ExtOpcode {
            category: 10,
            index: 37,
            name: None,
        }),
        (10, 38) => Some(ExtOpcode {
            category: 10,
            index: 38,
            name: Some("histload"),
        }),
        (10, 40) => Some(ExtOpcode {
            category: 10,
            index: 40,
            name: Some("se_load"),
        }),
        (10, 41) => Some(ExtOpcode {
            category: 10,
            index: 41,
            name: Some("se_play"),
        }),
        (10, 42) => Some(ExtOpcode {
            category: 10,
            index: 42,
            name: Some("se_play_ex_ch"),
        }),
        (10, 43) => Some(ExtOpcode {
            category: 10,
            index: 43,
            name: Some("se_stop"),
        }),
        (10, 44) => Some(ExtOpcode {
            category: 10,
            index: 44,
            name: Some("se_set_volume"),
        }),
        (10, 45) => Some(ExtOpcode {
            category: 10,
            index: 45,
            name: Some("se_get_volume"),
        }),
        (10, 46) => Some(ExtOpcode {
            category: 10,
            index: 46,
            name: Some("se_unload"),
        }),
        (10, 47) => Some(ExtOpcode {
            category: 10,
            index: 47,
            name: Some("se_wait"),
        }),
        (10, 48) => Some(ExtOpcode {
            category: 10,
            index: 48,
            name: Some("channel_error_set_se_info"),
        }),
        (10, 49) => Some(ExtOpcode {
            category: 10,
            index: 49,
            name: Some("get_se_ex_volume"),
        }),
        (10, 50) => Some(ExtOpcode {
            category: 10,
            index: 50,
            name: Some("channel_error_set_se_ex_volume"),
        }),
        (10, 51) => Some(ExtOpcode {
            category: 10,
            index: 51,
            name: Some("channel_error_se_enable"),
        }),
        (10, 52) => Some(ExtOpcode {
            category: 10,
            index: 52,
            name: Some("channel_error_is_se_enable"),
        }),
        (10, 53) => Some(ExtOpcode {
            category: 10,
            index: 53,
            name: Some("se_set_pan"),
        }),
        (10, 54) => Some(ExtOpcode {
            category: 10,
            index: 54,
            name: Some("se_mute"),
        }),
        (10, 56) => Some(ExtOpcode {
            category: 10,
            index: 56,
            name: Some("select_init"),
        }),
        (10, 57) => Some(ExtOpcode {
            category: 10,
            index: 57,
            name: Some("select"),
        }),
        (10, 58) => Some(ExtOpcode {
            category: 10,
            index: 58,
            name: None,
        }),
        (10, 59) => Some(ExtOpcode {
            category: 10,
            index: 59,
            name: None,
        }),
        (10, 60) => Some(ExtOpcode {
            category: 10,
            index: 60,
            name: Some("select_clear"),
        }),
        (10, 61) => Some(ExtOpcode {
            category: 10,
            index: 61,
            name: Some("select_set_offset"),
        }),
        (10, 62) => Some(ExtOpcode {
            category: 10,
            index: 62,
            name: Some("select_set_process"),
        }),
        (10, 63) => Some(ExtOpcode {
            category: 10,
            index: 63,
            name: Some("select_lock"),
        }),
        (10, 64) => Some(ExtOpcode {
            category: 10,
            index: 64,
            name: Some("get_select_on_key"),
        }),
        (10, 65) => Some(ExtOpcode {
            category: 10,
            index: 65,
            name: Some("get_select_pull_key"),
        }),
        (10, 66) => Some(ExtOpcode {
            category: 10,
            index: 66,
            name: Some("get_select_push_key"),
        }),
        (10, 68) => Some(ExtOpcode {
            category: 10,
            index: 68,
            name: Some("skip_set"),
        }),
        (10, 69) => Some(ExtOpcode {
            category: 10,
            index: 69,
            name: Some("skip_is"),
        }),
        (10, 70) => Some(ExtOpcode {
            category: 10,
            index: 70,
            name: Some("auto_set"),
        }),
        (10, 71) => Some(ExtOpcode {
            category: 10,
            index: 71,
            name: Some("auto_is"),
        }),
        (10, 72) => Some(ExtOpcode {
            category: 10,
            index: 72,
            name: None,
        }),
        (10, 73) => Some(ExtOpcode {
            category: 10,
            index: 73,
            name: Some("auto_get_time"),
        }),
        (10, 74) => Some(ExtOpcode {
            category: 10,
            index: 74,
            name: None,
        }),
        (10, 75) => Some(ExtOpcode {
            category: 10,
            index: 75,
            name: None,
        }),
        (10, 76) => Some(ExtOpcode {
            category: 10,
            index: 76,
            name: None,
        }),
        (10, 77) => Some(ExtOpcode {
            category: 10,
            index: 77,
            name: None,
        }),
        (10, 78) => Some(ExtOpcode {
            category: 10,
            index: 78,
            name: None,
        }),
        (10, 79) => Some(ExtOpcode {
            category: 10,
            index: 79,
            name: None,
        }),
        (10, 80) => Some(ExtOpcode {
            category: 10,
            index: 80,
            name: None,
        }),
        (10, 81) => Some(ExtOpcode {
            category: 10,
            index: 81,
            name: None,
        }),
        (10, 82) => Some(ExtOpcode {
            category: 10,
            index: 82,
            name: None,
        }),
        (10, 83) => Some(ExtOpcode {
            category: 10,
            index: 83,
            name: Some("load_font"),
        }),
        (10, 84) => Some(ExtOpcode {
            category: 10,
            index: 84,
            name: Some("unload_font"),
        }),
        (10, 85) => Some(ExtOpcode {
            category: 10,
            index: 85,
            name: Some("set_language"),
        }),
        (10, 86) => Some(ExtOpcode {
            category: 10,
            index: 86,
            name: Some("key_canncel"),
        }),
        (10, 87) => Some(ExtOpcode {
            category: 10,
            index: 87,
            name: Some("set_font_color"),
        }),
        (10, 88) => Some(ExtOpcode {
            category: 10,
            index: 88,
            name: Some("load_font_ex"),
        }),
        (10, 89) => Some(ExtOpcode {
            category: 10,
            index: 89,
            name: None,
        }),
        (10, 90) => Some(ExtOpcode {
            category: 10,
            index: 90,
            name: None,
        }),
        (10, 91) => Some(ExtOpcode {
            category: 10,
            index: 91,
            name: None,
        }),
        (10, 92) => Some(ExtOpcode {
            category: 10,
            index: 92,
            name: None,
        }),
        (10, 93) => Some(ExtOpcode {
            category: 10,
            index: 93,
            name: None,
        }),
        (10, 94) => Some(ExtOpcode {
            category: 10,
            index: 94,
            name: None,
        }),
        (10, 95) => Some(ExtOpcode {
            category: 10,
            index: 95,
            name: Some("set_font_size"),
        }),
        (10, 96) => Some(ExtOpcode {
            category: 10,
            index: 96,
            name: Some("get_font_size"),
        }),
        (10, 97) => Some(ExtOpcode {
            category: 10,
            index: 97,
            name: Some("get_font_type"),
        }),
        (10, 98) => Some(ExtOpcode {
            category: 10,
            index: 98,
            name: Some("set_font_effect"),
        }),
        (10, 99) => Some(ExtOpcode {
            category: 10,
            index: 99,
            name: Some("get_font_effect"),
        }),
        (10, 100) => Some(ExtOpcode {
            category: 10,
            index: 100,
            name: Some("get_pull_key"),
        }),
        (10, 101) => Some(ExtOpcode {
            category: 10,
            index: 101,
            name: Some("get_on_key"),
        }),
        (10, 102) => Some(ExtOpcode {
            category: 10,
            index: 102,
            name: Some("get_push_key"),
        }),
        (10, 103) => Some(ExtOpcode {
            category: 10,
            index: 103,
            name: Some("input_clear"),
        }),
        (10, 104) => Some(ExtOpcode {
            category: 10,
            index: 104,
            name: Some("change_window_size"),
        }),
        (10, 105) => Some(ExtOpcode {
            category: 10,
            index: 105,
            name: Some("change_aspect_mode"),
        }),
        (10, 106) => Some(ExtOpcode {
            category: 10,
            index: 106,
            name: Some("aspect_position_enable"),
        }),
        (10, 107) => Some(ExtOpcode {
            category: 10,
            index: 107,
            name: None,
        }),
        (10, 108) => Some(ExtOpcode {
            category: 10,
            index: 108,
            name: Some("get_aspect_mode"),
        }),
        (10, 109) => Some(ExtOpcode {
            category: 10,
            index: 109,
            name: Some("get_monitor_size"),
        }),
        (10, 110) => Some(ExtOpcode {
            category: 10,
            index: 110,
            name: None,
        }),
        (10, 111) => Some(ExtOpcode {
            category: 10,
            index: 111,
            name: Some("get_system_metrics"),
        }),
        (10, 112) => Some(ExtOpcode {
            category: 10,
            index: 112,
            name: Some("set_system_path"),
        }),
        (10, 113) => Some(ExtOpcode {
            category: 10,
            index: 113,
            name: Some("set_allmosaicthumbnail"),
        }),
        (10, 114) => Some(ExtOpcode {
            category: 10,
            index: 114,
            name: Some("enable_window_change"),
        }),
        (10, 115) => Some(ExtOpcode {
            category: 10,
            index: 115,
            name: Some("is_enable_window_change"),
        }),
        (10, 116) => Some(ExtOpcode {
            category: 10,
            index: 116,
            name: Some("set_cursor_null"),
        }),
        (10, 117) => Some(ExtOpcode {
            category: 10,
            index: 117,
            name: Some("set_hide_cursor_time"),
        }),
        (10, 118) => Some(ExtOpcode {
            category: 10,
            index: 118,
            name: Some("get_hide_cursor_time"),
        }),
        (10, 119) => Some(ExtOpcode {
            category: 10,
            index: 119,
            name: Some("scene_skip"),
        }),
        (10, 120) => Some(ExtOpcode {
            category: 10,
            index: 120,
            name: None,
        }),
        (10, 121) => Some(ExtOpcode {
            category: 10,
            index: 121,
            name: None,
        }),
        (10, 122) => Some(ExtOpcode {
            category: 10,
            index: 122,
            name: Some("get_async_key"),
        }),
        (10, 123) => Some(ExtOpcode {
            category: 10,
            index: 123,
            name: Some("get_font_color"),
        }),
        (10, 124) => Some(ExtOpcode {
            category: 10,
            index: 124,
            name: None,
        }),
        (10, 125) => Some(ExtOpcode {
            category: 10,
            index: 125,
            name: Some("history_skip"),
        }),
        (10, 126) => Some(ExtOpcode {
            category: 10,
            index: 126,
            name: None,
        }),
        (10, 127) => Some(ExtOpcode {
            category: 10,
            index: 127,
            name: None,
        }),
        (10, 128) => Some(ExtOpcode {
            category: 10,
            index: 128,
            name: Some("set_language"),
        }),
        (10, 129) => Some(ExtOpcode {
            category: 10,
            index: 129,
            name: Some("set_achievement"),
        }),
        (10, 131) => Some(ExtOpcode {
            category: 10,
            index: 131,
            name: Some("system_btn_set"),
        }),
        (10, 132) => Some(ExtOpcode {
            category: 10,
            index: 132,
            name: Some("system_btn_release"),
        }),
        (10, 133) => Some(ExtOpcode {
            category: 10,
            index: 133,
            name: Some("system_btn_enable"),
        }),
        (10, 136) => Some(ExtOpcode {
            category: 10,
            index: 136,
            name: Some("text_init"),
        }),
        (10, 137) => Some(ExtOpcode {
            category: 10,
            index: 137,
            name: Some("text_set_icon"),
        }),
        (10, 138) => Some(ExtOpcode {
            category: 10,
            index: 138,
            name: Some("text"),
        }),
        (10, 139) => Some(ExtOpcode {
            category: 10,
            index: 139,
            name: Some("text_hide"),
        }),
        (10, 140) => Some(ExtOpcode {
            category: 10,
            index: 140,
            name: Some("text_show"),
        }),
        (10, 141) => Some(ExtOpcode {
            category: 10,
            index: 141,
            name: Some("text_set_btn"),
        }),
        (10, 142) => Some(ExtOpcode {
            category: 10,
            index: 142,
            name: Some("text_uninit"),
        }),
        (10, 143) => Some(ExtOpcode {
            category: 10,
            index: 143,
            name: Some("text_set_rect_invalid_param"),
        }),
        (10, 144) => Some(ExtOpcode {
            category: 10,
            index: 144,
            name: Some("text_clear"),
        }),
        (10, 145) => Some(ExtOpcode {
            category: 10,
            index: 145,
            name: None,
        }),
        (10, 146) => Some(ExtOpcode {
            category: 10,
            index: 146,
            name: Some("text_get_time"),
        }),
        (10, 147) => Some(ExtOpcode {
            category: 10,
            index: 147,
            name: Some("text_window_set_alpha"),
        }),
        (10, 148) => Some(ExtOpcode {
            category: 10,
            index: 148,
            name: Some("text_voice_play"),
        }),
        (10, 149) => Some(ExtOpcode {
            category: 10,
            index: 149,
            name: None,
        }),
        (10, 150) => Some(ExtOpcode {
            category: 10,
            index: 150,
            name: Some("text_set_icon_animation_time"),
        }),
        (10, 151) => Some(ExtOpcode {
            category: 10,
            index: 151,
            name: Some("text_w"),
        }),
        (10, 152) => Some(ExtOpcode {
            category: 10,
            index: 152,
            name: Some("text_a"),
        }),
        (10, 153) => Some(ExtOpcode {
            category: 10,
            index: 153,
            name: Some("text_wa"),
        }),
        (10, 154) => Some(ExtOpcode {
            category: 10,
            index: 154,
            name: Some("text_n"),
        }),
        (10, 155) => Some(ExtOpcode {
            category: 10,
            index: 155,
            name: Some("text_cat"),
        }),
        (10, 156) => Some(ExtOpcode {
            category: 10,
            index: 156,
            name: Some("set_history"),
        }),
        (10, 157) => Some(ExtOpcode {
            category: 10,
            index: 157,
            name: Some("is_text_visible"),
        }),
        (10, 158) => Some(ExtOpcode {
            category: 10,
            index: 158,
            name: Some("text_set_base"),
        }),
        (10, 159) => Some(ExtOpcode {
            category: 10,
            index: 159,
            name: Some("enable_voice_cut"),
        }),
        (10, 160) => Some(ExtOpcode {
            category: 10,
            index: 160,
            name: Some("is_voice_cut"),
        }),
        (10, 161) => Some(ExtOpcode {
            category: 10,
            index: 161,
            name: Some("texttimecheckset"),
        }),
        (10, 162) => Some(ExtOpcode {
            category: 10,
            index: 162,
            name: None,
        }),
        (10, 163) => Some(ExtOpcode {
            category: 10,
            index: 163,
            name: None,
        }),
        (10, 164) => Some(ExtOpcode {
            category: 10,
            index: 164,
            name: Some("text_set_color"),
        }),
        (10, 165) => Some(ExtOpcode {
            category: 10,
            index: 165,
            name: Some("textredraw"),
        }),
        (10, 166) => Some(ExtOpcode {
            category: 10,
            index: 166,
            name: Some("set_text_mode"),
        }),
        (10, 167) => Some(ExtOpcode {
            category: 10,
            index: 167,
            name: Some("text_init_visualnovelmode"),
        }),
        (10, 168) => Some(ExtOpcode {
            category: 10,
            index: 168,
            name: Some("text_set_icon_mode"),
        }),
        (10, 169) => Some(ExtOpcode {
            category: 10,
            index: 169,
            name: Some("text_vn_br"),
        }),
        (10, 170) => Some(ExtOpcode {
            category: 10,
            index: 170,
            name: None,
        }),
        (10, 171) => Some(ExtOpcode {
            category: 10,
            index: 171,
            name: None,
        }),
        (10, 172) => Some(ExtOpcode {
            category: 10,
            index: 172,
            name: None,
        }),
        (10, 173) => Some(ExtOpcode {
            category: 10,
            index: 173,
            name: None,
        }),
        (10, 174) => Some(ExtOpcode {
            category: 10,
            index: 174,
            name: Some("tips_get_str"),
        }),
        (10, 175) => Some(ExtOpcode {
            category: 10,
            index: 175,
            name: Some("tips_get_param"),
        }),
        (10, 176) => Some(ExtOpcode {
            category: 10,
            index: 176,
            name: Some("tips_reset"),
        }),
        (10, 177) => Some(ExtOpcode {
            category: 10,
            index: 177,
            name: Some("tips_search"),
        }),
        (10, 178) => Some(ExtOpcode {
            category: 10,
            index: 178,
            name: Some("tips_set_color"),
        }),
        (10, 179) => Some(ExtOpcode {
            category: 10,
            index: 179,
            name: Some("tips_stop"),
        }),
        (10, 180) => Some(ExtOpcode {
            category: 10,
            index: 180,
            name: Some("tips_get_flag"),
        }),
        (10, 181) => Some(ExtOpcode {
            category: 10,
            index: 181,
            name: Some("tips_init"),
        }),
        (10, 182) => Some(ExtOpcode {
            category: 10,
            index: 182,
            name: Some("tips_pause"),
        }),
        (10, 184) => Some(ExtOpcode {
            category: 10,
            index: 184,
            name: Some("voice_play"),
        }),
        (10, 185) => Some(ExtOpcode {
            category: 10,
            index: 185,
            name: Some("voice_stop"),
        }),
        (10, 186) => Some(ExtOpcode {
            category: 10,
            index: 186,
            name: Some("voice_set_volume"),
        }),
        (10, 187) => Some(ExtOpcode {
            category: 10,
            index: 187,
            name: Some("voice_get_volume"),
        }),
        (10, 188) => Some(ExtOpcode {
            category: 10,
            index: 188,
            name: Some("set_voice_info"),
        }),
        (10, 189) => Some(ExtOpcode {
            category: 10,
            index: 189,
            name: Some("voice_enable"),
        }),
        (10, 190) => Some(ExtOpcode {
            category: 10,
            index: 190,
            name: Some("is_voice_enable"),
        }),
        (10, 191) => Some(ExtOpcode {
            category: 10,
            index: 191,
            name: None,
        }),
        (10, 192) => Some(ExtOpcode {
            category: 10,
            index: 192,
            name: Some("bgv_play"),
        }),
        (10, 193) => Some(ExtOpcode {
            category: 10,
            index: 193,
            name: Some("bgv_stop"),
        }),
        (10, 194) => Some(ExtOpcode {
            category: 10,
            index: 194,
            name: Some("bgv_enable"),
        }),
        (10, 195) => Some(ExtOpcode {
            category: 10,
            index: 195,
            name: Some("get_voice_ex_volume"),
        }),
        (10, 196) => Some(ExtOpcode {
            category: 10,
            index: 196,
            name: Some("set_voice_ex_volume"),
        }),
        (10, 197) => Some(ExtOpcode {
            category: 10,
            index: 197,
            name: Some("voice_check_enable"),
        }),
        (10, 198) => Some(ExtOpcode {
            category: 10,
            index: 198,
            name: Some("voice_autopan_initialize"),
        }),
        (10, 199) => Some(ExtOpcode {
            category: 10,
            index: 199,
            name: Some("voice_autopan_enable"),
        }),
        (10, 200) => Some(ExtOpcode {
            category: 10,
            index: 200,
            name: Some("set_voice_autopan_size_over"),
        }),
        (10, 201) => Some(ExtOpcode {
            category: 10,
            index: 201,
            name: Some("is_voice_autopan_enable"),
        }),
        (10, 202) => Some(ExtOpcode {
            category: 10,
            index: 202,
            name: Some("voice_wait"),
        }),
        (10, 203) => Some(ExtOpcode {
            category: 10,
            index: 203,
            name: Some("bgv_pause"),
        }),
        (10, 204) => Some(ExtOpcode {
            category: 10,
            index: 204,
            name: Some("bgv_mute"),
        }),
        (10, 205) => Some(ExtOpcode {
            category: 10,
            index: 205,
            name: Some("set_bgv_volume"),
        }),
        (10, 206) => Some(ExtOpcode {
            category: 10,
            index: 206,
            name: Some("get_bgv_volume"),
        }),
        (10, 207) => Some(ExtOpcode {
            category: 10,
            index: 207,
            name: Some("set_bgv_auto_volume"),
        }),
        (10, 208) => Some(ExtOpcode {
            category: 10,
            index: 208,
            name: Some("voice_mute"),
        }),
        (10, 209) => Some(ExtOpcode {
            category: 10,
            index: 209,
            name: Some("voice_call"),
        }),
        (10, 210) => Some(ExtOpcode {
            category: 10,
            index: 210,
            name: Some("voice_call_clear"),
        }),
        (10, 212) => Some(ExtOpcode {
            category: 10,
            index: 212,
            name: Some("wait"),
        }),
        (10, 213) => Some(ExtOpcode {
            category: 10,
            index: 213,
            name: Some("wait_click"),
        }),
        (10, 214) => Some(ExtOpcode {
            category: 10,
            index: 214,
            name: Some("wait_sync_begin"),
        }),
        (10, 215) => Some(ExtOpcode {
            category: 10,
            index: 215,
            name: Some("wait_sync_release"),
        }),
        (10, 216) => Some(ExtOpcode {
            category: 10,
            index: 216,
            name: Some("wait_sync_end"),
        }),
        (10, 217) => Some(ExtOpcode {
            category: 10,
            index: 217,
            name: None,
        }),
        (10, 218) => Some(ExtOpcode {
            category: 10,
            index: 218,
            name: Some("wait_clear"),
        }),
        (10, 219) => Some(ExtOpcode {
            category: 10,
            index: 219,
            name: Some("wait_click_no_anim"),
        }),
        (10, 220) => Some(ExtOpcode {
            category: 10,
            index: 220,
            name: Some("wait_sync_get_time"),
        }),
        (10, 221) => Some(ExtOpcode {
            category: 10,
            index: 221,
            name: Some("wait_time_push"),
        }),
        (10, 222) => Some(ExtOpcode {
            category: 10,
            index: 222,
            name: Some("wait_time_pop"),
        }),
        (10, 286) => Some(ExtOpcode {
            category: 10,
            index: 286,
            name: None,
        }),
        (10, 289) => Some(ExtOpcode {
            category: 10,
            index: 289,
            name: None,
        }),
        (10, 292) => Some(ExtOpcode {
            category: 10,
            index: 292,
            name: None,
        }),
        (11, 0) => Some(ExtOpcode {
            category: 11,
            index: 0,
            name: Some("movie_play"),
        }),
        (11, 1) => Some(ExtOpcode {
            category: 11,
            index: 1,
            name: Some("msp_set_loop_sp_ep"),
        }),
        (11, 2) => Some(ExtOpcode {
            category: 11,
            index: 2,
            name: Some("msp_cls"),
        }),
        (11, 3) => Some(ExtOpcode {
            category: 11,
            index: 3,
            name: Some("msp_wait"),
        }),
        (11, 4) => Some(ExtOpcode {
            category: 11,
            index: 4,
            name: Some("msp_lock"),
        }),
        (11, 5) => Some(ExtOpcode {
            category: 11,
            index: 5,
            name: Some("msp_unlock"),
        }),
        (11, 6) => Some(ExtOpcode {
            category: 11,
            index: 6,
            name: Some("msp_play"),
        }),
        (11, 7) => Some(ExtOpcode {
            category: 11,
            index: 7,
            name: Some("msp_stop"),
        }),
        (11, 9) => Some(ExtOpcode {
            category: 11,
            index: 9,
            name: Some("create_thread"),
        }),
        (11, 10) => Some(ExtOpcode {
            category: 11,
            index: 10,
            name: Some("exit_thread"),
        }),
        (11, 11) => Some(ExtOpcode {
            category: 11,
            index: 11,
            name: None,
        }),
        (11, 12) => Some(ExtOpcode {
            category: 11,
            index: 12,
            name: Some("get_thread"),
        }),
        (11, 15) => Some(ExtOpcode {
            category: 11,
            index: 15,
            name: Some("mov"),
        }),
        (11, 16) => Some(ExtOpcode {
            category: 11,
            index: 16,
            name: Some("add"),
        }),
        (11, 17) => Some(ExtOpcode {
            category: 11,
            index: 17,
            name: Some("sub"),
        }),
        (11, 18) => Some(ExtOpcode {
            category: 11,
            index: 18,
            name: Some("mul"),
        }),
        (11, 19) => Some(ExtOpcode {
            category: 11,
            index: 19,
            name: Some("div"),
        }),
        (11, 20) => Some(ExtOpcode {
            category: 11,
            index: 20,
            name: Some("bitand"),
        }),
        (11, 21) => Some(ExtOpcode {
            category: 11,
            index: 21,
            name: Some("bitor"),
        }),
        (11, 22) => Some(ExtOpcode {
            category: 11,
            index: 22,
            name: Some("bitxor"),
        }),
        (11, 23) => Some(ExtOpcode {
            category: 11,
            index: 23,
            name: Some("jmp_point"),
        }),
        (11, 24) => Some(ExtOpcode {
            category: 11,
            index: 24,
            name: Some("jf_point"),
        }),
        (11, 25) => Some(ExtOpcode {
            category: 11,
            index: 25,
            name: Some("gosub_point"),
        }),
        (11, 26) => Some(ExtOpcode {
            category: 11,
            index: 26,
            name: Some("eq"),
        }),
        (11, 27) => Some(ExtOpcode {
            category: 11,
            index: 27,
            name: Some("ne"),
        }),
        (11, 28) => Some(ExtOpcode {
            category: 11,
            index: 28,
            name: Some("le"),
        }),
        (11, 29) => Some(ExtOpcode {
            category: 11,
            index: 29,
            name: Some("ge"),
        }),
        (11, 30) => Some(ExtOpcode {
            category: 11,
            index: 30,
            name: Some("lt"),
        }),
        (11, 31) => Some(ExtOpcode {
            category: 11,
            index: 31,
            name: Some("gt"),
        }),
        (11, 32) => Some(ExtOpcode {
            category: 11,
            index: 32,
            name: Some("lor"),
        }),
        (11, 33) => Some(ExtOpcode {
            category: 11,
            index: 33,
            name: Some("land"),
        }),
        (11, 34) => Some(ExtOpcode {
            category: 11,
            index: 34,
            name: Some("lnot_slot"),
        }),
        (11, 35) => Some(ExtOpcode {
            category: 11,
            index: 35,
            name: Some("end"),
        }),
        (11, 36) => Some(ExtOpcode {
            category: 11,
            index: 36,
            name: Some("nop"),
        }),
        (11, 37) => Some(ExtOpcode {
            category: 11,
            index: 37,
            name: Some("extcall"),
        }),
        (11, 38) => Some(ExtOpcode {
            category: 11,
            index: 38,
            name: Some("ret"),
        }),
        (11, 39) => Some(ExtOpcode {
            category: 11,
            index: 39,
            name: Some("reset_adv"),
        }),
        (11, 40) => Some(ExtOpcode {
            category: 11,
            index: 40,
            name: Some("mod"),
        }),
        (11, 41) => Some(ExtOpcode {
            category: 11,
            index: 41,
            name: Some("shl"),
        }),
        (11, 42) => Some(ExtOpcode {
            category: 11,
            index: 42,
            name: Some("shr"),
        }),
        (11, 43) => Some(ExtOpcode {
            category: 11,
            index: 43,
            name: Some("neg_slot"),
        }),
        (11, 44) => Some(ExtOpcode {
            category: 11,
            index: 44,
            name: Some("pop"),
        }),
        (11, 45) => Some(ExtOpcode {
            category: 11,
            index: 45,
            name: Some("push"),
        }),
        (11, 46) => Some(ExtOpcode {
            category: 11,
            index: 46,
            name: Some("pack_args"),
        }),
        (11, 47) => Some(ExtOpcode {
            category: 11,
            index: 47,
            name: Some("drop_args"),
        }),
        (11, 49) => Some(ExtOpcode {
            category: 11,
            index: 49,
            name: Some("create_message"),
        }),
        (11, 50) => Some(ExtOpcode {
            category: 11,
            index: 50,
            name: Some("get_message"),
        }),
        (11, 51) => Some(ExtOpcode {
            category: 11,
            index: 51,
            name: Some("get_message_param"),
        }),
        (11, 54) => Some(ExtOpcode {
            category: 11,
            index: 54,
            name: Some("save"),
        }),
        (11, 55) => Some(ExtOpcode {
            category: 11,
            index: 55,
            name: Some("load"),
        }),
        (11, 56) => Some(ExtOpcode {
            category: 11,
            index: 56,
            name: Some("save_set_title"),
        }),
        (11, 57) => Some(ExtOpcode {
            category: 11,
            index: 57,
            name: Some("save_data"),
        }),
        (11, 58) => Some(ExtOpcode {
            category: 11,
            index: 58,
            name: Some("save_set_thumbnail_size"),
        }),
        (11, 59) => Some(ExtOpcode {
            category: 11,
            index: 59,
            name: Some("thumbnail_set"),
        }),
        (11, 60) => Some(ExtOpcode {
            category: 11,
            index: 60,
            name: Some("savetitledraw"),
        }),
        (11, 61) => Some(ExtOpcode {
            category: 11,
            index: 61,
            name: Some("save_set_font_size"),
        }),
        (11, 62) => Some(ExtOpcode {
            category: 11,
            index: 62,
            name: Some("getsaveday"),
        }),
        (11, 63) => Some(ExtOpcode {
            category: 11,
            index: 63,
            name: Some("is_save"),
        }),
        (11, 64) => Some(ExtOpcode {
            category: 11,
            index: 64,
            name: Some("getsaveusermemory"),
        }),
        (11, 65) => Some(ExtOpcode {
            category: 11,
            index: 65,
            name: Some("savepoint"),
        }),
        (11, 66) => Some(ExtOpcode {
            category: 11,
            index: 66,
            name: Some("save_thumbnail_mosaic_set"),
        }),
        (11, 67) => Some(ExtOpcode {
            category: 11,
            index: 67,
            name: Some("savetimedraw"),
        }),
        (11, 68) => Some(ExtOpcode {
            category: 11,
            index: 68,
            name: Some("savedaydraw"),
        }),
        (11, 69) => Some(ExtOpcode {
            category: 11,
            index: 69,
            name: Some("save_set_text_rect"),
        }),
        (11, 70) => Some(ExtOpcode {
            category: 11,
            index: 70,
            name: Some("savetextdraw"),
        }),
        (11, 71) => Some(ExtOpcode {
            category: 11,
            index: 71,
            name: Some("get_new_savefile"),
        }),
        (11, 75) => Some(ExtOpcode {
            category: 11,
            index: 75,
            name: Some("setsavetext"),
        }),
        (11, 76) => Some(ExtOpcode {
            category: 11,
            index: 76,
            name: Some("thumbnail_renew"),
        }),
        (11, 77) => Some(ExtOpcode {
            category: 11,
            index: 77,
            name: Some("save_set_font_type"),
        }),
        (11, 78) => Some(ExtOpcode {
            category: 11,
            index: 78,
            name: Some("set_load_after_process"),
        }),
        (11, 79) => Some(ExtOpcode {
            category: 11,
            index: 79,
            name: Some("savesystemdata"),
        }),
        (11, 80) => Some(ExtOpcode {
            category: 11,
            index: 80,
            name: Some("save_set_font_effect"),
        }),
        (11, 81) => Some(ExtOpcode {
            category: 11,
            index: 81,
            name: Some("save_set_font_color_0x_0x"),
        }),
        (11, 82) => Some(ExtOpcode {
            category: 11,
            index: 82,
            name: Some("delete_file"),
        }),
        (11, 83) => Some(ExtOpcode {
            category: 11,
            index: 83,
            name: Some("save_tmp_dat"),
        }),
        (11, 84) => Some(ExtOpcode {
            category: 11,
            index: 84,
            name: Some("copy_file"),
        }),
        (11, 85) => Some(ExtOpcode {
            category: 11,
            index: 85,
            name: Some("load_thumbnail"),
        }),
        (11, 86) => Some(ExtOpcode {
            category: 11,
            index: 86,
            name: Some("save_lock_not_open_savefileno"),
        }),
        (11, 87) => Some(ExtOpcode {
            category: 11,
            index: 87,
            name: Some("is_save_lock"),
        }),
        (11, 88) => Some(ExtOpcode {
            category: 11,
            index: 88,
            name: Some("is_prev_data"),
        }),
        (11, 89) => Some(ExtOpcode {
            category: 11,
            index: 89,
            name: Some("save_point_clear"),
        }),
        (11, 90) => Some(ExtOpcode {
            category: 11,
            index: 90,
            name: Some("save_point_lock"),
        }),
        (11, 91) => Some(ExtOpcode {
            category: 11,
            index: 91,
            name: None,
        }),
        (11, 92) => Some(ExtOpcode {
            category: 11,
            index: 92,
            name: Some("histload"),
        }),
        (11, 94) => Some(ExtOpcode {
            category: 11,
            index: 94,
            name: Some("se_load"),
        }),
        (11, 95) => Some(ExtOpcode {
            category: 11,
            index: 95,
            name: Some("se_play"),
        }),
        (11, 96) => Some(ExtOpcode {
            category: 11,
            index: 96,
            name: Some("se_play_ex_ch"),
        }),
        (11, 97) => Some(ExtOpcode {
            category: 11,
            index: 97,
            name: Some("se_stop"),
        }),
        (11, 98) => Some(ExtOpcode {
            category: 11,
            index: 98,
            name: Some("se_set_volume"),
        }),
        (11, 99) => Some(ExtOpcode {
            category: 11,
            index: 99,
            name: Some("se_get_volume"),
        }),
        (11, 100) => Some(ExtOpcode {
            category: 11,
            index: 100,
            name: Some("se_unload"),
        }),
        (11, 101) => Some(ExtOpcode {
            category: 11,
            index: 101,
            name: Some("se_wait"),
        }),
        (11, 102) => Some(ExtOpcode {
            category: 11,
            index: 102,
            name: Some("channel_error_set_se_info"),
        }),
        (11, 103) => Some(ExtOpcode {
            category: 11,
            index: 103,
            name: Some("get_se_ex_volume"),
        }),
        (11, 104) => Some(ExtOpcode {
            category: 11,
            index: 104,
            name: Some("channel_error_set_se_ex_volume"),
        }),
        (11, 105) => Some(ExtOpcode {
            category: 11,
            index: 105,
            name: Some("channel_error_se_enable"),
        }),
        (11, 106) => Some(ExtOpcode {
            category: 11,
            index: 106,
            name: Some("channel_error_is_se_enable"),
        }),
        (11, 107) => Some(ExtOpcode {
            category: 11,
            index: 107,
            name: Some("se_set_pan"),
        }),
        (11, 108) => Some(ExtOpcode {
            category: 11,
            index: 108,
            name: Some("se_mute"),
        }),
        (11, 110) => Some(ExtOpcode {
            category: 11,
            index: 110,
            name: Some("select_init"),
        }),
        (11, 111) => Some(ExtOpcode {
            category: 11,
            index: 111,
            name: Some("select"),
        }),
        (11, 112) => Some(ExtOpcode {
            category: 11,
            index: 112,
            name: None,
        }),
        (11, 113) => Some(ExtOpcode {
            category: 11,
            index: 113,
            name: None,
        }),
        (11, 114) => Some(ExtOpcode {
            category: 11,
            index: 114,
            name: Some("select_clear"),
        }),
        (11, 115) => Some(ExtOpcode {
            category: 11,
            index: 115,
            name: Some("select_set_offset"),
        }),
        (11, 116) => Some(ExtOpcode {
            category: 11,
            index: 116,
            name: Some("select_set_process"),
        }),
        (11, 117) => Some(ExtOpcode {
            category: 11,
            index: 117,
            name: Some("select_lock"),
        }),
        (11, 118) => Some(ExtOpcode {
            category: 11,
            index: 118,
            name: Some("get_select_on_key"),
        }),
        (11, 119) => Some(ExtOpcode {
            category: 11,
            index: 119,
            name: Some("get_select_pull_key"),
        }),
        (11, 120) => Some(ExtOpcode {
            category: 11,
            index: 120,
            name: Some("get_select_push_key"),
        }),
        (11, 122) => Some(ExtOpcode {
            category: 11,
            index: 122,
            name: Some("skip_set"),
        }),
        (11, 123) => Some(ExtOpcode {
            category: 11,
            index: 123,
            name: Some("skip_is"),
        }),
        (11, 124) => Some(ExtOpcode {
            category: 11,
            index: 124,
            name: Some("auto_set"),
        }),
        (11, 125) => Some(ExtOpcode {
            category: 11,
            index: 125,
            name: Some("auto_is"),
        }),
        (11, 126) => Some(ExtOpcode {
            category: 11,
            index: 126,
            name: None,
        }),
        (11, 127) => Some(ExtOpcode {
            category: 11,
            index: 127,
            name: Some("auto_get_time"),
        }),
        (11, 128) => Some(ExtOpcode {
            category: 11,
            index: 128,
            name: None,
        }),
        (11, 129) => Some(ExtOpcode {
            category: 11,
            index: 129,
            name: None,
        }),
        (11, 130) => Some(ExtOpcode {
            category: 11,
            index: 130,
            name: None,
        }),
        (11, 131) => Some(ExtOpcode {
            category: 11,
            index: 131,
            name: None,
        }),
        (11, 132) => Some(ExtOpcode {
            category: 11,
            index: 132,
            name: None,
        }),
        (11, 133) => Some(ExtOpcode {
            category: 11,
            index: 133,
            name: None,
        }),
        (11, 134) => Some(ExtOpcode {
            category: 11,
            index: 134,
            name: None,
        }),
        (11, 135) => Some(ExtOpcode {
            category: 11,
            index: 135,
            name: None,
        }),
        (11, 136) => Some(ExtOpcode {
            category: 11,
            index: 136,
            name: None,
        }),
        (11, 137) => Some(ExtOpcode {
            category: 11,
            index: 137,
            name: Some("load_font"),
        }),
        (11, 138) => Some(ExtOpcode {
            category: 11,
            index: 138,
            name: Some("unload_font"),
        }),
        (11, 139) => Some(ExtOpcode {
            category: 11,
            index: 139,
            name: Some("set_language"),
        }),
        (11, 140) => Some(ExtOpcode {
            category: 11,
            index: 140,
            name: Some("key_canncel"),
        }),
        (11, 141) => Some(ExtOpcode {
            category: 11,
            index: 141,
            name: Some("set_font_color"),
        }),
        (11, 142) => Some(ExtOpcode {
            category: 11,
            index: 142,
            name: Some("load_font_ex"),
        }),
        (11, 143) => Some(ExtOpcode {
            category: 11,
            index: 143,
            name: None,
        }),
        (11, 144) => Some(ExtOpcode {
            category: 11,
            index: 144,
            name: None,
        }),
        (11, 145) => Some(ExtOpcode {
            category: 11,
            index: 145,
            name: None,
        }),
        (11, 146) => Some(ExtOpcode {
            category: 11,
            index: 146,
            name: None,
        }),
        (11, 147) => Some(ExtOpcode {
            category: 11,
            index: 147,
            name: None,
        }),
        (11, 148) => Some(ExtOpcode {
            category: 11,
            index: 148,
            name: None,
        }),
        (11, 149) => Some(ExtOpcode {
            category: 11,
            index: 149,
            name: Some("set_font_size"),
        }),
        (11, 150) => Some(ExtOpcode {
            category: 11,
            index: 150,
            name: Some("get_font_size"),
        }),
        (11, 151) => Some(ExtOpcode {
            category: 11,
            index: 151,
            name: Some("get_font_type"),
        }),
        (11, 152) => Some(ExtOpcode {
            category: 11,
            index: 152,
            name: Some("set_font_effect"),
        }),
        (11, 153) => Some(ExtOpcode {
            category: 11,
            index: 153,
            name: Some("get_font_effect"),
        }),
        (11, 154) => Some(ExtOpcode {
            category: 11,
            index: 154,
            name: Some("get_pull_key"),
        }),
        (11, 155) => Some(ExtOpcode {
            category: 11,
            index: 155,
            name: Some("get_on_key"),
        }),
        (11, 156) => Some(ExtOpcode {
            category: 11,
            index: 156,
            name: Some("get_push_key"),
        }),
        (11, 157) => Some(ExtOpcode {
            category: 11,
            index: 157,
            name: Some("input_clear"),
        }),
        (11, 158) => Some(ExtOpcode {
            category: 11,
            index: 158,
            name: Some("change_window_size"),
        }),
        (11, 159) => Some(ExtOpcode {
            category: 11,
            index: 159,
            name: Some("change_aspect_mode"),
        }),
        (11, 160) => Some(ExtOpcode {
            category: 11,
            index: 160,
            name: Some("aspect_position_enable"),
        }),
        (11, 161) => Some(ExtOpcode {
            category: 11,
            index: 161,
            name: None,
        }),
        (11, 162) => Some(ExtOpcode {
            category: 11,
            index: 162,
            name: Some("get_aspect_mode"),
        }),
        (11, 163) => Some(ExtOpcode {
            category: 11,
            index: 163,
            name: Some("get_monitor_size"),
        }),
        (11, 164) => Some(ExtOpcode {
            category: 11,
            index: 164,
            name: None,
        }),
        (11, 165) => Some(ExtOpcode {
            category: 11,
            index: 165,
            name: Some("get_system_metrics"),
        }),
        (11, 166) => Some(ExtOpcode {
            category: 11,
            index: 166,
            name: Some("set_system_path"),
        }),
        (11, 167) => Some(ExtOpcode {
            category: 11,
            index: 167,
            name: Some("set_allmosaicthumbnail"),
        }),
        (11, 168) => Some(ExtOpcode {
            category: 11,
            index: 168,
            name: Some("enable_window_change"),
        }),
        (11, 169) => Some(ExtOpcode {
            category: 11,
            index: 169,
            name: Some("is_enable_window_change"),
        }),
        (11, 170) => Some(ExtOpcode {
            category: 11,
            index: 170,
            name: Some("set_cursor_null"),
        }),
        (11, 171) => Some(ExtOpcode {
            category: 11,
            index: 171,
            name: Some("set_hide_cursor_time"),
        }),
        (11, 172) => Some(ExtOpcode {
            category: 11,
            index: 172,
            name: Some("get_hide_cursor_time"),
        }),
        (11, 173) => Some(ExtOpcode {
            category: 11,
            index: 173,
            name: Some("scene_skip"),
        }),
        (11, 174) => Some(ExtOpcode {
            category: 11,
            index: 174,
            name: None,
        }),
        (11, 175) => Some(ExtOpcode {
            category: 11,
            index: 175,
            name: None,
        }),
        (11, 176) => Some(ExtOpcode {
            category: 11,
            index: 176,
            name: Some("get_async_key"),
        }),
        (11, 177) => Some(ExtOpcode {
            category: 11,
            index: 177,
            name: Some("get_font_color"),
        }),
        (11, 178) => Some(ExtOpcode {
            category: 11,
            index: 178,
            name: None,
        }),
        (11, 179) => Some(ExtOpcode {
            category: 11,
            index: 179,
            name: Some("history_skip"),
        }),
        (11, 180) => Some(ExtOpcode {
            category: 11,
            index: 180,
            name: None,
        }),
        (11, 181) => Some(ExtOpcode {
            category: 11,
            index: 181,
            name: None,
        }),
        (11, 182) => Some(ExtOpcode {
            category: 11,
            index: 182,
            name: Some("set_language"),
        }),
        (11, 183) => Some(ExtOpcode {
            category: 11,
            index: 183,
            name: Some("set_achievement"),
        }),
        (11, 185) => Some(ExtOpcode {
            category: 11,
            index: 185,
            name: Some("system_btn_set"),
        }),
        (11, 186) => Some(ExtOpcode {
            category: 11,
            index: 186,
            name: Some("system_btn_release"),
        }),
        (11, 187) => Some(ExtOpcode {
            category: 11,
            index: 187,
            name: Some("system_btn_enable"),
        }),
        (11, 190) => Some(ExtOpcode {
            category: 11,
            index: 190,
            name: Some("text_init"),
        }),
        (11, 191) => Some(ExtOpcode {
            category: 11,
            index: 191,
            name: Some("text_set_icon"),
        }),
        (11, 192) => Some(ExtOpcode {
            category: 11,
            index: 192,
            name: Some("text"),
        }),
        (11, 193) => Some(ExtOpcode {
            category: 11,
            index: 193,
            name: Some("text_hide"),
        }),
        (11, 194) => Some(ExtOpcode {
            category: 11,
            index: 194,
            name: Some("text_show"),
        }),
        (11, 195) => Some(ExtOpcode {
            category: 11,
            index: 195,
            name: Some("text_set_btn"),
        }),
        (11, 196) => Some(ExtOpcode {
            category: 11,
            index: 196,
            name: Some("text_uninit"),
        }),
        (11, 197) => Some(ExtOpcode {
            category: 11,
            index: 197,
            name: Some("text_set_rect_invalid_param"),
        }),
        (11, 198) => Some(ExtOpcode {
            category: 11,
            index: 198,
            name: Some("text_clear"),
        }),
        (11, 199) => Some(ExtOpcode {
            category: 11,
            index: 199,
            name: None,
        }),
        (11, 200) => Some(ExtOpcode {
            category: 11,
            index: 200,
            name: Some("text_get_time"),
        }),
        (11, 201) => Some(ExtOpcode {
            category: 11,
            index: 201,
            name: Some("text_window_set_alpha"),
        }),
        (11, 202) => Some(ExtOpcode {
            category: 11,
            index: 202,
            name: Some("text_voice_play"),
        }),
        (11, 203) => Some(ExtOpcode {
            category: 11,
            index: 203,
            name: None,
        }),
        (11, 204) => Some(ExtOpcode {
            category: 11,
            index: 204,
            name: Some("text_set_icon_animation_time"),
        }),
        (11, 205) => Some(ExtOpcode {
            category: 11,
            index: 205,
            name: Some("text_w"),
        }),
        (11, 206) => Some(ExtOpcode {
            category: 11,
            index: 206,
            name: Some("text_a"),
        }),
        (11, 207) => Some(ExtOpcode {
            category: 11,
            index: 207,
            name: Some("text_wa"),
        }),
        (11, 208) => Some(ExtOpcode {
            category: 11,
            index: 208,
            name: Some("text_n"),
        }),
        (11, 209) => Some(ExtOpcode {
            category: 11,
            index: 209,
            name: Some("text_cat"),
        }),
        (11, 210) => Some(ExtOpcode {
            category: 11,
            index: 210,
            name: Some("set_history"),
        }),
        (11, 211) => Some(ExtOpcode {
            category: 11,
            index: 211,
            name: Some("is_text_visible"),
        }),
        (11, 212) => Some(ExtOpcode {
            category: 11,
            index: 212,
            name: Some("text_set_base"),
        }),
        (11, 213) => Some(ExtOpcode {
            category: 11,
            index: 213,
            name: Some("enable_voice_cut"),
        }),
        (11, 214) => Some(ExtOpcode {
            category: 11,
            index: 214,
            name: Some("is_voice_cut"),
        }),
        (11, 215) => Some(ExtOpcode {
            category: 11,
            index: 215,
            name: Some("texttimecheckset"),
        }),
        (11, 216) => Some(ExtOpcode {
            category: 11,
            index: 216,
            name: None,
        }),
        (11, 217) => Some(ExtOpcode {
            category: 11,
            index: 217,
            name: None,
        }),
        (11, 218) => Some(ExtOpcode {
            category: 11,
            index: 218,
            name: Some("text_set_color"),
        }),
        (11, 219) => Some(ExtOpcode {
            category: 11,
            index: 219,
            name: Some("textredraw"),
        }),
        (11, 220) => Some(ExtOpcode {
            category: 11,
            index: 220,
            name: Some("set_text_mode"),
        }),
        (11, 221) => Some(ExtOpcode {
            category: 11,
            index: 221,
            name: Some("text_init_visualnovelmode"),
        }),
        (11, 222) => Some(ExtOpcode {
            category: 11,
            index: 222,
            name: Some("text_set_icon_mode"),
        }),
        (11, 223) => Some(ExtOpcode {
            category: 11,
            index: 223,
            name: Some("text_vn_br"),
        }),
        (11, 224) => Some(ExtOpcode {
            category: 11,
            index: 224,
            name: None,
        }),
        (11, 225) => Some(ExtOpcode {
            category: 11,
            index: 225,
            name: None,
        }),
        (11, 226) => Some(ExtOpcode {
            category: 11,
            index: 226,
            name: None,
        }),
        (11, 227) => Some(ExtOpcode {
            category: 11,
            index: 227,
            name: None,
        }),
        (11, 228) => Some(ExtOpcode {
            category: 11,
            index: 228,
            name: Some("tips_get_str"),
        }),
        (11, 229) => Some(ExtOpcode {
            category: 11,
            index: 229,
            name: Some("tips_get_param"),
        }),
        (11, 230) => Some(ExtOpcode {
            category: 11,
            index: 230,
            name: Some("tips_reset"),
        }),
        (11, 231) => Some(ExtOpcode {
            category: 11,
            index: 231,
            name: Some("tips_search"),
        }),
        (11, 232) => Some(ExtOpcode {
            category: 11,
            index: 232,
            name: Some("tips_set_color"),
        }),
        (11, 233) => Some(ExtOpcode {
            category: 11,
            index: 233,
            name: Some("tips_stop"),
        }),
        (11, 234) => Some(ExtOpcode {
            category: 11,
            index: 234,
            name: Some("tips_get_flag"),
        }),
        (11, 235) => Some(ExtOpcode {
            category: 11,
            index: 235,
            name: Some("tips_init"),
        }),
        (11, 236) => Some(ExtOpcode {
            category: 11,
            index: 236,
            name: Some("tips_pause"),
        }),
        (11, 238) => Some(ExtOpcode {
            category: 11,
            index: 238,
            name: Some("voice_play"),
        }),
        (11, 239) => Some(ExtOpcode {
            category: 11,
            index: 239,
            name: Some("voice_stop"),
        }),
        (11, 240) => Some(ExtOpcode {
            category: 11,
            index: 240,
            name: Some("voice_set_volume"),
        }),
        (11, 241) => Some(ExtOpcode {
            category: 11,
            index: 241,
            name: Some("voice_get_volume"),
        }),
        (11, 242) => Some(ExtOpcode {
            category: 11,
            index: 242,
            name: Some("set_voice_info"),
        }),
        (11, 243) => Some(ExtOpcode {
            category: 11,
            index: 243,
            name: Some("voice_enable"),
        }),
        (11, 244) => Some(ExtOpcode {
            category: 11,
            index: 244,
            name: Some("is_voice_enable"),
        }),
        (11, 245) => Some(ExtOpcode {
            category: 11,
            index: 245,
            name: None,
        }),
        (11, 246) => Some(ExtOpcode {
            category: 11,
            index: 246,
            name: Some("bgv_play"),
        }),
        (11, 247) => Some(ExtOpcode {
            category: 11,
            index: 247,
            name: Some("bgv_stop"),
        }),
        (11, 248) => Some(ExtOpcode {
            category: 11,
            index: 248,
            name: Some("bgv_enable"),
        }),
        (11, 249) => Some(ExtOpcode {
            category: 11,
            index: 249,
            name: Some("get_voice_ex_volume"),
        }),
        (11, 250) => Some(ExtOpcode {
            category: 11,
            index: 250,
            name: Some("set_voice_ex_volume"),
        }),
        (11, 251) => Some(ExtOpcode {
            category: 11,
            index: 251,
            name: Some("voice_check_enable"),
        }),
        (11, 252) => Some(ExtOpcode {
            category: 11,
            index: 252,
            name: Some("voice_autopan_initialize"),
        }),
        (11, 253) => Some(ExtOpcode {
            category: 11,
            index: 253,
            name: Some("voice_autopan_enable"),
        }),
        (11, 254) => Some(ExtOpcode {
            category: 11,
            index: 254,
            name: Some("set_voice_autopan_size_over"),
        }),
        (11, 255) => Some(ExtOpcode {
            category: 11,
            index: 255,
            name: Some("is_voice_autopan_enable"),
        }),
        (11, 256) => Some(ExtOpcode {
            category: 11,
            index: 256,
            name: Some("voice_wait"),
        }),
        (11, 257) => Some(ExtOpcode {
            category: 11,
            index: 257,
            name: Some("bgv_pause"),
        }),
        (11, 258) => Some(ExtOpcode {
            category: 11,
            index: 258,
            name: Some("bgv_mute"),
        }),
        (11, 259) => Some(ExtOpcode {
            category: 11,
            index: 259,
            name: Some("set_bgv_volume"),
        }),
        (11, 260) => Some(ExtOpcode {
            category: 11,
            index: 260,
            name: Some("get_bgv_volume"),
        }),
        (11, 261) => Some(ExtOpcode {
            category: 11,
            index: 261,
            name: Some("set_bgv_auto_volume"),
        }),
        (11, 262) => Some(ExtOpcode {
            category: 11,
            index: 262,
            name: Some("voice_mute"),
        }),
        (11, 263) => Some(ExtOpcode {
            category: 11,
            index: 263,
            name: Some("voice_call"),
        }),
        (11, 264) => Some(ExtOpcode {
            category: 11,
            index: 264,
            name: Some("voice_call_clear"),
        }),
        (11, 266) => Some(ExtOpcode {
            category: 11,
            index: 266,
            name: Some("wait"),
        }),
        (11, 267) => Some(ExtOpcode {
            category: 11,
            index: 267,
            name: Some("wait_click"),
        }),
        (11, 268) => Some(ExtOpcode {
            category: 11,
            index: 268,
            name: Some("wait_sync_begin"),
        }),
        (11, 269) => Some(ExtOpcode {
            category: 11,
            index: 269,
            name: Some("wait_sync_release"),
        }),
        (11, 270) => Some(ExtOpcode {
            category: 11,
            index: 270,
            name: Some("wait_sync_end"),
        }),
        (11, 271) => Some(ExtOpcode {
            category: 11,
            index: 271,
            name: None,
        }),
        (11, 272) => Some(ExtOpcode {
            category: 11,
            index: 272,
            name: Some("wait_clear"),
        }),
        (11, 273) => Some(ExtOpcode {
            category: 11,
            index: 273,
            name: Some("wait_click_no_anim"),
        }),
        (11, 274) => Some(ExtOpcode {
            category: 11,
            index: 274,
            name: Some("wait_sync_get_time"),
        }),
        (11, 275) => Some(ExtOpcode {
            category: 11,
            index: 275,
            name: Some("wait_time_push"),
        }),
        (11, 276) => Some(ExtOpcode {
            category: 11,
            index: 276,
            name: Some("wait_time_pop"),
        }),
        (12, 0) => Some(ExtOpcode {
            category: 12,
            index: 0,
            name: Some("system_btn_set"),
        }),
        (12, 1) => Some(ExtOpcode {
            category: 12,
            index: 1,
            name: Some("system_btn_release"),
        }),
        (12, 2) => Some(ExtOpcode {
            category: 12,
            index: 2,
            name: Some("system_btn_enable"),
        }),
        (12, 5) => Some(ExtOpcode {
            category: 12,
            index: 5,
            name: Some("text_init"),
        }),
        (12, 6) => Some(ExtOpcode {
            category: 12,
            index: 6,
            name: Some("text_set_icon"),
        }),
        (12, 7) => Some(ExtOpcode {
            category: 12,
            index: 7,
            name: Some("text"),
        }),
        (12, 8) => Some(ExtOpcode {
            category: 12,
            index: 8,
            name: Some("text_hide"),
        }),
        (12, 9) => Some(ExtOpcode {
            category: 12,
            index: 9,
            name: Some("text_show"),
        }),
        (12, 10) => Some(ExtOpcode {
            category: 12,
            index: 10,
            name: Some("text_set_btn"),
        }),
        (12, 11) => Some(ExtOpcode {
            category: 12,
            index: 11,
            name: Some("text_uninit"),
        }),
        (12, 12) => Some(ExtOpcode {
            category: 12,
            index: 12,
            name: Some("text_set_rect_invalid_param"),
        }),
        (12, 13) => Some(ExtOpcode {
            category: 12,
            index: 13,
            name: Some("text_clear"),
        }),
        (12, 14) => Some(ExtOpcode {
            category: 12,
            index: 14,
            name: None,
        }),
        (12, 15) => Some(ExtOpcode {
            category: 12,
            index: 15,
            name: Some("text_get_time"),
        }),
        (12, 16) => Some(ExtOpcode {
            category: 12,
            index: 16,
            name: Some("text_window_set_alpha"),
        }),
        (12, 17) => Some(ExtOpcode {
            category: 12,
            index: 17,
            name: Some("text_voice_play"),
        }),
        (12, 18) => Some(ExtOpcode {
            category: 12,
            index: 18,
            name: None,
        }),
        (12, 19) => Some(ExtOpcode {
            category: 12,
            index: 19,
            name: Some("text_set_icon_animation_time"),
        }),
        (12, 20) => Some(ExtOpcode {
            category: 12,
            index: 20,
            name: Some("text_w"),
        }),
        (12, 21) => Some(ExtOpcode {
            category: 12,
            index: 21,
            name: Some("text_a"),
        }),
        (12, 22) => Some(ExtOpcode {
            category: 12,
            index: 22,
            name: Some("text_wa"),
        }),
        (12, 23) => Some(ExtOpcode {
            category: 12,
            index: 23,
            name: Some("text_n"),
        }),
        (12, 24) => Some(ExtOpcode {
            category: 12,
            index: 24,
            name: Some("text_cat"),
        }),
        (12, 25) => Some(ExtOpcode {
            category: 12,
            index: 25,
            name: Some("set_history"),
        }),
        (12, 26) => Some(ExtOpcode {
            category: 12,
            index: 26,
            name: Some("is_text_visible"),
        }),
        (12, 27) => Some(ExtOpcode {
            category: 12,
            index: 27,
            name: Some("text_set_base"),
        }),
        (12, 28) => Some(ExtOpcode {
            category: 12,
            index: 28,
            name: Some("enable_voice_cut"),
        }),
        (12, 29) => Some(ExtOpcode {
            category: 12,
            index: 29,
            name: Some("is_voice_cut"),
        }),
        (12, 30) => Some(ExtOpcode {
            category: 12,
            index: 30,
            name: Some("texttimecheckset"),
        }),
        (12, 31) => Some(ExtOpcode {
            category: 12,
            index: 31,
            name: None,
        }),
        (12, 32) => Some(ExtOpcode {
            category: 12,
            index: 32,
            name: None,
        }),
        (12, 33) => Some(ExtOpcode {
            category: 12,
            index: 33,
            name: Some("text_set_color"),
        }),
        (12, 34) => Some(ExtOpcode {
            category: 12,
            index: 34,
            name: Some("textredraw"),
        }),
        (12, 35) => Some(ExtOpcode {
            category: 12,
            index: 35,
            name: Some("set_text_mode"),
        }),
        (12, 36) => Some(ExtOpcode {
            category: 12,
            index: 36,
            name: Some("text_init_visualnovelmode"),
        }),
        (12, 37) => Some(ExtOpcode {
            category: 12,
            index: 37,
            name: Some("text_set_icon_mode"),
        }),
        (12, 38) => Some(ExtOpcode {
            category: 12,
            index: 38,
            name: Some("text_vn_br"),
        }),
        (12, 39) => Some(ExtOpcode {
            category: 12,
            index: 39,
            name: None,
        }),
        (12, 40) => Some(ExtOpcode {
            category: 12,
            index: 40,
            name: None,
        }),
        (12, 41) => Some(ExtOpcode {
            category: 12,
            index: 41,
            name: None,
        }),
        (12, 42) => Some(ExtOpcode {
            category: 12,
            index: 42,
            name: None,
        }),
        (12, 43) => Some(ExtOpcode {
            category: 12,
            index: 43,
            name: Some("tips_get_str"),
        }),
        (12, 44) => Some(ExtOpcode {
            category: 12,
            index: 44,
            name: Some("tips_get_param"),
        }),
        (12, 45) => Some(ExtOpcode {
            category: 12,
            index: 45,
            name: Some("tips_reset"),
        }),
        (12, 46) => Some(ExtOpcode {
            category: 12,
            index: 46,
            name: Some("tips_search"),
        }),
        (12, 47) => Some(ExtOpcode {
            category: 12,
            index: 47,
            name: Some("tips_set_color"),
        }),
        (12, 48) => Some(ExtOpcode {
            category: 12,
            index: 48,
            name: Some("tips_stop"),
        }),
        (12, 49) => Some(ExtOpcode {
            category: 12,
            index: 49,
            name: Some("tips_get_flag"),
        }),
        (12, 50) => Some(ExtOpcode {
            category: 12,
            index: 50,
            name: Some("tips_init"),
        }),
        (12, 51) => Some(ExtOpcode {
            category: 12,
            index: 51,
            name: Some("tips_pause"),
        }),
        (12, 53) => Some(ExtOpcode {
            category: 12,
            index: 53,
            name: Some("voice_play"),
        }),
        (12, 54) => Some(ExtOpcode {
            category: 12,
            index: 54,
            name: Some("voice_stop"),
        }),
        (12, 55) => Some(ExtOpcode {
            category: 12,
            index: 55,
            name: Some("voice_set_volume"),
        }),
        (12, 56) => Some(ExtOpcode {
            category: 12,
            index: 56,
            name: Some("voice_get_volume"),
        }),
        (12, 57) => Some(ExtOpcode {
            category: 12,
            index: 57,
            name: Some("set_voice_info"),
        }),
        (12, 58) => Some(ExtOpcode {
            category: 12,
            index: 58,
            name: Some("voice_enable"),
        }),
        (12, 59) => Some(ExtOpcode {
            category: 12,
            index: 59,
            name: Some("is_voice_enable"),
        }),
        (12, 60) => Some(ExtOpcode {
            category: 12,
            index: 60,
            name: None,
        }),
        (12, 61) => Some(ExtOpcode {
            category: 12,
            index: 61,
            name: Some("bgv_play"),
        }),
        (12, 62) => Some(ExtOpcode {
            category: 12,
            index: 62,
            name: Some("bgv_stop"),
        }),
        (12, 63) => Some(ExtOpcode {
            category: 12,
            index: 63,
            name: Some("bgv_enable"),
        }),
        (12, 64) => Some(ExtOpcode {
            category: 12,
            index: 64,
            name: Some("get_voice_ex_volume"),
        }),
        (12, 65) => Some(ExtOpcode {
            category: 12,
            index: 65,
            name: Some("set_voice_ex_volume"),
        }),
        (12, 66) => Some(ExtOpcode {
            category: 12,
            index: 66,
            name: Some("voice_check_enable"),
        }),
        (12, 67) => Some(ExtOpcode {
            category: 12,
            index: 67,
            name: Some("voice_autopan_initialize"),
        }),
        (12, 68) => Some(ExtOpcode {
            category: 12,
            index: 68,
            name: Some("voice_autopan_enable"),
        }),
        (12, 69) => Some(ExtOpcode {
            category: 12,
            index: 69,
            name: Some("set_voice_autopan_size_over"),
        }),
        (12, 70) => Some(ExtOpcode {
            category: 12,
            index: 70,
            name: Some("is_voice_autopan_enable"),
        }),
        (12, 71) => Some(ExtOpcode {
            category: 12,
            index: 71,
            name: Some("voice_wait"),
        }),
        (12, 72) => Some(ExtOpcode {
            category: 12,
            index: 72,
            name: Some("bgv_pause"),
        }),
        (12, 73) => Some(ExtOpcode {
            category: 12,
            index: 73,
            name: Some("bgv_mute"),
        }),
        (12, 74) => Some(ExtOpcode {
            category: 12,
            index: 74,
            name: Some("set_bgv_volume"),
        }),
        (12, 75) => Some(ExtOpcode {
            category: 12,
            index: 75,
            name: Some("get_bgv_volume"),
        }),
        (12, 76) => Some(ExtOpcode {
            category: 12,
            index: 76,
            name: Some("set_bgv_auto_volume"),
        }),
        (12, 77) => Some(ExtOpcode {
            category: 12,
            index: 77,
            name: Some("voice_mute"),
        }),
        (12, 78) => Some(ExtOpcode {
            category: 12,
            index: 78,
            name: Some("voice_call"),
        }),
        (12, 79) => Some(ExtOpcode {
            category: 12,
            index: 79,
            name: Some("voice_call_clear"),
        }),
        (12, 81) => Some(ExtOpcode {
            category: 12,
            index: 81,
            name: Some("wait"),
        }),
        (12, 82) => Some(ExtOpcode {
            category: 12,
            index: 82,
            name: Some("wait_click"),
        }),
        (12, 83) => Some(ExtOpcode {
            category: 12,
            index: 83,
            name: Some("wait_sync_begin"),
        }),
        (12, 84) => Some(ExtOpcode {
            category: 12,
            index: 84,
            name: Some("wait_sync_release"),
        }),
        (12, 85) => Some(ExtOpcode {
            category: 12,
            index: 85,
            name: Some("wait_sync_end"),
        }),
        (12, 86) => Some(ExtOpcode {
            category: 12,
            index: 86,
            name: None,
        }),
        (12, 87) => Some(ExtOpcode {
            category: 12,
            index: 87,
            name: Some("wait_clear"),
        }),
        (12, 88) => Some(ExtOpcode {
            category: 12,
            index: 88,
            name: Some("wait_click_no_anim"),
        }),
        (12, 89) => Some(ExtOpcode {
            category: 12,
            index: 89,
            name: Some("wait_sync_get_time"),
        }),
        (12, 90) => Some(ExtOpcode {
            category: 12,
            index: 90,
            name: Some("wait_time_push"),
        }),
        (12, 91) => Some(ExtOpcode {
            category: 12,
            index: 91,
            name: Some("wait_time_pop"),
        }),
        (12, 155) => Some(ExtOpcode {
            category: 12,
            index: 155,
            name: None,
        }),
        (12, 158) => Some(ExtOpcode {
            category: 12,
            index: 158,
            name: None,
        }),
        (12, 161) => Some(ExtOpcode {
            category: 12,
            index: 161,
            name: None,
        }),
        (12, 172) => Some(ExtOpcode {
            category: 12,
            index: 172,
            name: None,
        }),
        (12, 223) => Some(ExtOpcode {
            category: 12,
            index: 223,
            name: None,
        }),
        (12, 239) => Some(ExtOpcode {
            category: 12,
            index: 239,
            name: None,
        }),
        (12, 246) => Some(ExtOpcode {
            category: 12,
            index: 246,
            name: None,
        }),
        (12, 253) => Some(ExtOpcode {
            category: 12,
            index: 253,
            name: None,
        }),
        (12, 270) => Some(ExtOpcode {
            category: 12,
            index: 270,
            name: None,
        }),
        (12, 276) => Some(ExtOpcode {
            category: 12,
            index: 276,
            name: None,
        }),
        (12, 288) => Some(ExtOpcode {
            category: 12,
            index: 288,
            name: None,
        }),
        (13, 0) => Some(ExtOpcode {
            category: 13,
            index: 0,
            name: Some("voice_play"),
        }),
        (13, 1) => Some(ExtOpcode {
            category: 13,
            index: 1,
            name: Some("voice_stop"),
        }),
        (13, 2) => Some(ExtOpcode {
            category: 13,
            index: 2,
            name: Some("voice_set_volume"),
        }),
        (13, 3) => Some(ExtOpcode {
            category: 13,
            index: 3,
            name: Some("voice_get_volume"),
        }),
        (13, 4) => Some(ExtOpcode {
            category: 13,
            index: 4,
            name: Some("set_voice_info"),
        }),
        (13, 5) => Some(ExtOpcode {
            category: 13,
            index: 5,
            name: Some("voice_enable"),
        }),
        (13, 6) => Some(ExtOpcode {
            category: 13,
            index: 6,
            name: Some("is_voice_enable"),
        }),
        (13, 7) => Some(ExtOpcode {
            category: 13,
            index: 7,
            name: None,
        }),
        (13, 8) => Some(ExtOpcode {
            category: 13,
            index: 8,
            name: Some("bgv_play"),
        }),
        (13, 9) => Some(ExtOpcode {
            category: 13,
            index: 9,
            name: Some("bgv_stop"),
        }),
        (13, 10) => Some(ExtOpcode {
            category: 13,
            index: 10,
            name: Some("bgv_enable"),
        }),
        (13, 11) => Some(ExtOpcode {
            category: 13,
            index: 11,
            name: Some("get_voice_ex_volume"),
        }),
        (13, 12) => Some(ExtOpcode {
            category: 13,
            index: 12,
            name: Some("set_voice_ex_volume"),
        }),
        (13, 13) => Some(ExtOpcode {
            category: 13,
            index: 13,
            name: Some("voice_check_enable"),
        }),
        (13, 14) => Some(ExtOpcode {
            category: 13,
            index: 14,
            name: Some("voice_autopan_initialize"),
        }),
        (13, 15) => Some(ExtOpcode {
            category: 13,
            index: 15,
            name: Some("voice_autopan_enable"),
        }),
        (13, 16) => Some(ExtOpcode {
            category: 13,
            index: 16,
            name: Some("set_voice_autopan_size_over"),
        }),
        (13, 17) => Some(ExtOpcode {
            category: 13,
            index: 17,
            name: Some("is_voice_autopan_enable"),
        }),
        (13, 18) => Some(ExtOpcode {
            category: 13,
            index: 18,
            name: Some("voice_wait"),
        }),
        (13, 19) => Some(ExtOpcode {
            category: 13,
            index: 19,
            name: Some("bgv_pause"),
        }),
        (13, 20) => Some(ExtOpcode {
            category: 13,
            index: 20,
            name: Some("bgv_mute"),
        }),
        (13, 21) => Some(ExtOpcode {
            category: 13,
            index: 21,
            name: Some("set_bgv_volume"),
        }),
        (13, 22) => Some(ExtOpcode {
            category: 13,
            index: 22,
            name: Some("get_bgv_volume"),
        }),
        (13, 23) => Some(ExtOpcode {
            category: 13,
            index: 23,
            name: Some("set_bgv_auto_volume"),
        }),
        (13, 24) => Some(ExtOpcode {
            category: 13,
            index: 24,
            name: Some("voice_mute"),
        }),
        (13, 25) => Some(ExtOpcode {
            category: 13,
            index: 25,
            name: Some("voice_call"),
        }),
        (13, 26) => Some(ExtOpcode {
            category: 13,
            index: 26,
            name: Some("voice_call_clear"),
        }),
        (13, 28) => Some(ExtOpcode {
            category: 13,
            index: 28,
            name: Some("wait"),
        }),
        (13, 29) => Some(ExtOpcode {
            category: 13,
            index: 29,
            name: Some("wait_click"),
        }),
        (13, 30) => Some(ExtOpcode {
            category: 13,
            index: 30,
            name: Some("wait_sync_begin"),
        }),
        (13, 31) => Some(ExtOpcode {
            category: 13,
            index: 31,
            name: Some("wait_sync_release"),
        }),
        (13, 32) => Some(ExtOpcode {
            category: 13,
            index: 32,
            name: Some("wait_sync_end"),
        }),
        (13, 33) => Some(ExtOpcode {
            category: 13,
            index: 33,
            name: None,
        }),
        (13, 34) => Some(ExtOpcode {
            category: 13,
            index: 34,
            name: Some("wait_clear"),
        }),
        (13, 35) => Some(ExtOpcode {
            category: 13,
            index: 35,
            name: Some("wait_click_no_anim"),
        }),
        (13, 36) => Some(ExtOpcode {
            category: 13,
            index: 36,
            name: Some("wait_sync_get_time"),
        }),
        (13, 37) => Some(ExtOpcode {
            category: 13,
            index: 37,
            name: Some("wait_time_push"),
        }),
        (13, 38) => Some(ExtOpcode {
            category: 13,
            index: 38,
            name: Some("wait_time_pop"),
        }),
        (13, 102) => Some(ExtOpcode {
            category: 13,
            index: 102,
            name: None,
        }),
        (13, 105) => Some(ExtOpcode {
            category: 13,
            index: 105,
            name: None,
        }),
        (13, 108) => Some(ExtOpcode {
            category: 13,
            index: 108,
            name: None,
        }),
        (13, 119) => Some(ExtOpcode {
            category: 13,
            index: 119,
            name: None,
        }),
        (13, 170) => Some(ExtOpcode {
            category: 13,
            index: 170,
            name: None,
        }),
        (13, 186) => Some(ExtOpcode {
            category: 13,
            index: 186,
            name: None,
        }),
        (13, 193) => Some(ExtOpcode {
            category: 13,
            index: 193,
            name: None,
        }),
        (13, 200) => Some(ExtOpcode {
            category: 13,
            index: 200,
            name: None,
        }),
        (13, 217) => Some(ExtOpcode {
            category: 13,
            index: 217,
            name: None,
        }),
        (13, 223) => Some(ExtOpcode {
            category: 13,
            index: 223,
            name: None,
        }),
        (13, 235) => Some(ExtOpcode {
            category: 13,
            index: 235,
            name: None,
        }),
        (14, 0) => Some(ExtOpcode {
            category: 14,
            index: 0,
            name: Some("history_init_0x_0x"),
        }),
        (14, 1) => Some(ExtOpcode {
            category: 14,
            index: 1,
            name: Some("historybegin_lpbyte_ptagdata_sztext"),
        }),
        (14, 2) => Some(ExtOpcode {
            category: 14,
            index: 2,
            name: Some("history_end"),
        }),
        (14, 3) => Some(ExtOpcode {
            category: 14,
            index: 3,
            name: None,
        }),
        (14, 4) => Some(ExtOpcode {
            category: 14,
            index: 4,
            name: None,
        }),
        (14, 5) => Some(ExtOpcode {
            category: 14,
            index: 5,
            name: Some("history_get_height"),
        }),
        (14, 6) => Some(ExtOpcode {
            category: 14,
            index: 6,
            name: None,
        }),
        (14, 7) => Some(ExtOpcode {
            category: 14,
            index: 7,
            name: None,
        }),
        (14, 8) => Some(ExtOpcode {
            category: 14,
            index: 8,
            name: None,
        }),
        (14, 9) => Some(ExtOpcode {
            category: 14,
            index: 9,
            name: None,
        }),
        (14, 10) => Some(ExtOpcode {
            category: 14,
            index: 10,
            name: Some("history_set_rect"),
        }),
        (14, 11) => Some(ExtOpcode {
            category: 14,
            index: 11,
            name: Some("history_clear"),
        }),
        (14, 12) => Some(ExtOpcode {
            category: 14,
            index: 12,
            name: Some("history_set"),
        }),
        (14, 13) => Some(ExtOpcode {
            category: 14,
            index: 13,
            name: None,
        }),
        (14, 14) => Some(ExtOpcode {
            category: 14,
            index: 14,
            name: None,
        }),
        (14, 15) => Some(ExtOpcode {
            category: 14,
            index: 15,
            name: None,
        }),
        (14, 16) => Some(ExtOpcode {
            category: 14,
            index: 16,
            name: None,
        }),
        (14, 17) => Some(ExtOpcode {
            category: 14,
            index: 17,
            name: Some("history_set_face_call"),
        }),
        (14, 18) => Some(ExtOpcode {
            category: 14,
            index: 18,
            name: Some("history_set_face_sound"),
        }),
        (14, 19) => Some(ExtOpcode {
            category: 14,
            index: 19,
            name: Some("history_set_face_sound_release"),
        }),
        (14, 20) => Some(ExtOpcode {
            category: 14,
            index: 20,
            name: Some("history_get_text"),
        }),
        (14, 21) => Some(ExtOpcode {
            category: 14,
            index: 21,
            name: None,
        }),
        (14, 22) => Some(ExtOpcode {
            category: 14,
            index: 22,
            name: None,
        }),
        (14, 23) => Some(ExtOpcode {
            category: 14,
            index: 23,
            name: None,
        }),
        (14, 24) => Some(ExtOpcode {
            category: 14,
            index: 24,
            name: None,
        }),
        (14, 26) => Some(ExtOpcode {
            category: 14,
            index: 26,
            name: Some("movie_play"),
        }),
        (14, 27) => Some(ExtOpcode {
            category: 14,
            index: 27,
            name: Some("msp_set_loop_sp_ep"),
        }),
        (14, 28) => Some(ExtOpcode {
            category: 14,
            index: 28,
            name: Some("msp_cls"),
        }),
        (14, 29) => Some(ExtOpcode {
            category: 14,
            index: 29,
            name: Some("msp_wait"),
        }),
        (14, 30) => Some(ExtOpcode {
            category: 14,
            index: 30,
            name: Some("msp_lock"),
        }),
        (14, 31) => Some(ExtOpcode {
            category: 14,
            index: 31,
            name: Some("msp_unlock"),
        }),
        (14, 32) => Some(ExtOpcode {
            category: 14,
            index: 32,
            name: Some("msp_play"),
        }),
        (14, 33) => Some(ExtOpcode {
            category: 14,
            index: 33,
            name: Some("msp_stop"),
        }),
        (14, 35) => Some(ExtOpcode {
            category: 14,
            index: 35,
            name: Some("create_thread"),
        }),
        (14, 36) => Some(ExtOpcode {
            category: 14,
            index: 36,
            name: Some("exit_thread"),
        }),
        (14, 37) => Some(ExtOpcode {
            category: 14,
            index: 37,
            name: None,
        }),
        (14, 38) => Some(ExtOpcode {
            category: 14,
            index: 38,
            name: Some("get_thread"),
        }),
        (14, 41) => Some(ExtOpcode {
            category: 14,
            index: 41,
            name: Some("mov"),
        }),
        (14, 42) => Some(ExtOpcode {
            category: 14,
            index: 42,
            name: Some("add"),
        }),
        (14, 43) => Some(ExtOpcode {
            category: 14,
            index: 43,
            name: Some("sub"),
        }),
        (14, 44) => Some(ExtOpcode {
            category: 14,
            index: 44,
            name: Some("mul"),
        }),
        (14, 45) => Some(ExtOpcode {
            category: 14,
            index: 45,
            name: Some("div"),
        }),
        (14, 46) => Some(ExtOpcode {
            category: 14,
            index: 46,
            name: Some("bitand"),
        }),
        (14, 47) => Some(ExtOpcode {
            category: 14,
            index: 47,
            name: Some("bitor"),
        }),
        (14, 48) => Some(ExtOpcode {
            category: 14,
            index: 48,
            name: Some("bitxor"),
        }),
        (14, 49) => Some(ExtOpcode {
            category: 14,
            index: 49,
            name: Some("jmp_point"),
        }),
        (14, 50) => Some(ExtOpcode {
            category: 14,
            index: 50,
            name: Some("jf_point"),
        }),
        (14, 51) => Some(ExtOpcode {
            category: 14,
            index: 51,
            name: Some("gosub_point"),
        }),
        (14, 52) => Some(ExtOpcode {
            category: 14,
            index: 52,
            name: Some("eq"),
        }),
        (14, 53) => Some(ExtOpcode {
            category: 14,
            index: 53,
            name: Some("ne"),
        }),
        (14, 54) => Some(ExtOpcode {
            category: 14,
            index: 54,
            name: Some("le"),
        }),
        (14, 55) => Some(ExtOpcode {
            category: 14,
            index: 55,
            name: Some("ge"),
        }),
        (14, 56) => Some(ExtOpcode {
            category: 14,
            index: 56,
            name: Some("lt"),
        }),
        (14, 57) => Some(ExtOpcode {
            category: 14,
            index: 57,
            name: Some("gt"),
        }),
        (14, 58) => Some(ExtOpcode {
            category: 14,
            index: 58,
            name: Some("lor"),
        }),
        (14, 59) => Some(ExtOpcode {
            category: 14,
            index: 59,
            name: Some("land"),
        }),
        (14, 60) => Some(ExtOpcode {
            category: 14,
            index: 60,
            name: Some("lnot_slot"),
        }),
        (14, 61) => Some(ExtOpcode {
            category: 14,
            index: 61,
            name: Some("end"),
        }),
        (14, 62) => Some(ExtOpcode {
            category: 14,
            index: 62,
            name: Some("nop"),
        }),
        (14, 63) => Some(ExtOpcode {
            category: 14,
            index: 63,
            name: Some("extcall"),
        }),
        (14, 64) => Some(ExtOpcode {
            category: 14,
            index: 64,
            name: Some("ret"),
        }),
        (14, 65) => Some(ExtOpcode {
            category: 14,
            index: 65,
            name: Some("reset_adv"),
        }),
        (14, 66) => Some(ExtOpcode {
            category: 14,
            index: 66,
            name: Some("mod"),
        }),
        (14, 67) => Some(ExtOpcode {
            category: 14,
            index: 67,
            name: Some("shl"),
        }),
        (14, 68) => Some(ExtOpcode {
            category: 14,
            index: 68,
            name: Some("shr"),
        }),
        (14, 69) => Some(ExtOpcode {
            category: 14,
            index: 69,
            name: Some("neg_slot"),
        }),
        (14, 70) => Some(ExtOpcode {
            category: 14,
            index: 70,
            name: Some("pop"),
        }),
        (14, 71) => Some(ExtOpcode {
            category: 14,
            index: 71,
            name: Some("push"),
        }),
        (14, 72) => Some(ExtOpcode {
            category: 14,
            index: 72,
            name: Some("pack_args"),
        }),
        (14, 73) => Some(ExtOpcode {
            category: 14,
            index: 73,
            name: Some("drop_args"),
        }),
        (14, 75) => Some(ExtOpcode {
            category: 14,
            index: 75,
            name: Some("create_message"),
        }),
        (14, 76) => Some(ExtOpcode {
            category: 14,
            index: 76,
            name: Some("get_message"),
        }),
        (14, 77) => Some(ExtOpcode {
            category: 14,
            index: 77,
            name: Some("get_message_param"),
        }),
        (14, 80) => Some(ExtOpcode {
            category: 14,
            index: 80,
            name: Some("save"),
        }),
        (14, 81) => Some(ExtOpcode {
            category: 14,
            index: 81,
            name: Some("load"),
        }),
        (14, 82) => Some(ExtOpcode {
            category: 14,
            index: 82,
            name: Some("save_set_title"),
        }),
        (14, 83) => Some(ExtOpcode {
            category: 14,
            index: 83,
            name: Some("save_data"),
        }),
        (14, 84) => Some(ExtOpcode {
            category: 14,
            index: 84,
            name: Some("save_set_thumbnail_size"),
        }),
        (14, 85) => Some(ExtOpcode {
            category: 14,
            index: 85,
            name: Some("thumbnail_set"),
        }),
        (14, 86) => Some(ExtOpcode {
            category: 14,
            index: 86,
            name: Some("savetitledraw"),
        }),
        (14, 87) => Some(ExtOpcode {
            category: 14,
            index: 87,
            name: Some("save_set_font_size"),
        }),
        (14, 88) => Some(ExtOpcode {
            category: 14,
            index: 88,
            name: Some("getsaveday"),
        }),
        (14, 89) => Some(ExtOpcode {
            category: 14,
            index: 89,
            name: Some("is_save"),
        }),
        (14, 90) => Some(ExtOpcode {
            category: 14,
            index: 90,
            name: Some("getsaveusermemory"),
        }),
        (14, 91) => Some(ExtOpcode {
            category: 14,
            index: 91,
            name: Some("savepoint"),
        }),
        (14, 92) => Some(ExtOpcode {
            category: 14,
            index: 92,
            name: Some("save_thumbnail_mosaic_set"),
        }),
        (14, 93) => Some(ExtOpcode {
            category: 14,
            index: 93,
            name: Some("savetimedraw"),
        }),
        (14, 94) => Some(ExtOpcode {
            category: 14,
            index: 94,
            name: Some("savedaydraw"),
        }),
        (14, 95) => Some(ExtOpcode {
            category: 14,
            index: 95,
            name: Some("save_set_text_rect"),
        }),
        (14, 96) => Some(ExtOpcode {
            category: 14,
            index: 96,
            name: Some("savetextdraw"),
        }),
        (14, 97) => Some(ExtOpcode {
            category: 14,
            index: 97,
            name: Some("get_new_savefile"),
        }),
        (14, 101) => Some(ExtOpcode {
            category: 14,
            index: 101,
            name: Some("setsavetext"),
        }),
        (14, 102) => Some(ExtOpcode {
            category: 14,
            index: 102,
            name: Some("thumbnail_renew"),
        }),
        (14, 103) => Some(ExtOpcode {
            category: 14,
            index: 103,
            name: Some("save_set_font_type"),
        }),
        (14, 104) => Some(ExtOpcode {
            category: 14,
            index: 104,
            name: Some("set_load_after_process"),
        }),
        (14, 105) => Some(ExtOpcode {
            category: 14,
            index: 105,
            name: Some("savesystemdata"),
        }),
        (14, 106) => Some(ExtOpcode {
            category: 14,
            index: 106,
            name: Some("save_set_font_effect"),
        }),
        (14, 107) => Some(ExtOpcode {
            category: 14,
            index: 107,
            name: Some("save_set_font_color_0x_0x"),
        }),
        (14, 108) => Some(ExtOpcode {
            category: 14,
            index: 108,
            name: Some("delete_file"),
        }),
        (14, 109) => Some(ExtOpcode {
            category: 14,
            index: 109,
            name: Some("save_tmp_dat"),
        }),
        (14, 110) => Some(ExtOpcode {
            category: 14,
            index: 110,
            name: Some("copy_file"),
        }),
        (14, 111) => Some(ExtOpcode {
            category: 14,
            index: 111,
            name: Some("load_thumbnail"),
        }),
        (14, 112) => Some(ExtOpcode {
            category: 14,
            index: 112,
            name: Some("save_lock_not_open_savefileno"),
        }),
        (14, 113) => Some(ExtOpcode {
            category: 14,
            index: 113,
            name: Some("is_save_lock"),
        }),
        (14, 114) => Some(ExtOpcode {
            category: 14,
            index: 114,
            name: Some("is_prev_data"),
        }),
        (14, 115) => Some(ExtOpcode {
            category: 14,
            index: 115,
            name: Some("save_point_clear"),
        }),
        (14, 116) => Some(ExtOpcode {
            category: 14,
            index: 116,
            name: Some("save_point_lock"),
        }),
        (14, 117) => Some(ExtOpcode {
            category: 14,
            index: 117,
            name: None,
        }),
        (14, 118) => Some(ExtOpcode {
            category: 14,
            index: 118,
            name: Some("histload"),
        }),
        (14, 120) => Some(ExtOpcode {
            category: 14,
            index: 120,
            name: Some("se_load"),
        }),
        (14, 121) => Some(ExtOpcode {
            category: 14,
            index: 121,
            name: Some("se_play"),
        }),
        (14, 122) => Some(ExtOpcode {
            category: 14,
            index: 122,
            name: Some("se_play_ex_ch"),
        }),
        (14, 123) => Some(ExtOpcode {
            category: 14,
            index: 123,
            name: Some("se_stop"),
        }),
        (14, 124) => Some(ExtOpcode {
            category: 14,
            index: 124,
            name: Some("se_set_volume"),
        }),
        (14, 125) => Some(ExtOpcode {
            category: 14,
            index: 125,
            name: Some("se_get_volume"),
        }),
        (14, 126) => Some(ExtOpcode {
            category: 14,
            index: 126,
            name: Some("se_unload"),
        }),
        (14, 127) => Some(ExtOpcode {
            category: 14,
            index: 127,
            name: Some("se_wait"),
        }),
        (14, 128) => Some(ExtOpcode {
            category: 14,
            index: 128,
            name: Some("channel_error_set_se_info"),
        }),
        (14, 129) => Some(ExtOpcode {
            category: 14,
            index: 129,
            name: Some("get_se_ex_volume"),
        }),
        (14, 130) => Some(ExtOpcode {
            category: 14,
            index: 130,
            name: Some("channel_error_set_se_ex_volume"),
        }),
        (14, 131) => Some(ExtOpcode {
            category: 14,
            index: 131,
            name: Some("channel_error_se_enable"),
        }),
        (14, 132) => Some(ExtOpcode {
            category: 14,
            index: 132,
            name: Some("channel_error_is_se_enable"),
        }),
        (14, 133) => Some(ExtOpcode {
            category: 14,
            index: 133,
            name: Some("se_set_pan"),
        }),
        (14, 134) => Some(ExtOpcode {
            category: 14,
            index: 134,
            name: Some("se_mute"),
        }),
        (14, 136) => Some(ExtOpcode {
            category: 14,
            index: 136,
            name: Some("select_init"),
        }),
        (14, 137) => Some(ExtOpcode {
            category: 14,
            index: 137,
            name: Some("select"),
        }),
        (14, 138) => Some(ExtOpcode {
            category: 14,
            index: 138,
            name: None,
        }),
        (14, 139) => Some(ExtOpcode {
            category: 14,
            index: 139,
            name: None,
        }),
        (14, 140) => Some(ExtOpcode {
            category: 14,
            index: 140,
            name: Some("select_clear"),
        }),
        (14, 141) => Some(ExtOpcode {
            category: 14,
            index: 141,
            name: Some("select_set_offset"),
        }),
        (14, 142) => Some(ExtOpcode {
            category: 14,
            index: 142,
            name: Some("select_set_process"),
        }),
        (14, 143) => Some(ExtOpcode {
            category: 14,
            index: 143,
            name: Some("select_lock"),
        }),
        (14, 144) => Some(ExtOpcode {
            category: 14,
            index: 144,
            name: Some("get_select_on_key"),
        }),
        (14, 145) => Some(ExtOpcode {
            category: 14,
            index: 145,
            name: Some("get_select_pull_key"),
        }),
        (14, 146) => Some(ExtOpcode {
            category: 14,
            index: 146,
            name: Some("get_select_push_key"),
        }),
        (14, 148) => Some(ExtOpcode {
            category: 14,
            index: 148,
            name: Some("skip_set"),
        }),
        (14, 149) => Some(ExtOpcode {
            category: 14,
            index: 149,
            name: Some("skip_is"),
        }),
        (14, 150) => Some(ExtOpcode {
            category: 14,
            index: 150,
            name: Some("auto_set"),
        }),
        (14, 151) => Some(ExtOpcode {
            category: 14,
            index: 151,
            name: Some("auto_is"),
        }),
        (14, 152) => Some(ExtOpcode {
            category: 14,
            index: 152,
            name: None,
        }),
        (14, 153) => Some(ExtOpcode {
            category: 14,
            index: 153,
            name: Some("auto_get_time"),
        }),
        (14, 154) => Some(ExtOpcode {
            category: 14,
            index: 154,
            name: None,
        }),
        (14, 155) => Some(ExtOpcode {
            category: 14,
            index: 155,
            name: None,
        }),
        (14, 156) => Some(ExtOpcode {
            category: 14,
            index: 156,
            name: None,
        }),
        (14, 157) => Some(ExtOpcode {
            category: 14,
            index: 157,
            name: None,
        }),
        (14, 158) => Some(ExtOpcode {
            category: 14,
            index: 158,
            name: None,
        }),
        (14, 159) => Some(ExtOpcode {
            category: 14,
            index: 159,
            name: None,
        }),
        (14, 160) => Some(ExtOpcode {
            category: 14,
            index: 160,
            name: None,
        }),
        (14, 161) => Some(ExtOpcode {
            category: 14,
            index: 161,
            name: None,
        }),
        (14, 162) => Some(ExtOpcode {
            category: 14,
            index: 162,
            name: None,
        }),
        (14, 163) => Some(ExtOpcode {
            category: 14,
            index: 163,
            name: Some("load_font"),
        }),
        (14, 164) => Some(ExtOpcode {
            category: 14,
            index: 164,
            name: Some("unload_font"),
        }),
        (14, 165) => Some(ExtOpcode {
            category: 14,
            index: 165,
            name: Some("set_language"),
        }),
        (14, 166) => Some(ExtOpcode {
            category: 14,
            index: 166,
            name: Some("key_canncel"),
        }),
        (14, 167) => Some(ExtOpcode {
            category: 14,
            index: 167,
            name: Some("set_font_color"),
        }),
        (14, 168) => Some(ExtOpcode {
            category: 14,
            index: 168,
            name: Some("load_font_ex"),
        }),
        (14, 169) => Some(ExtOpcode {
            category: 14,
            index: 169,
            name: None,
        }),
        (14, 170) => Some(ExtOpcode {
            category: 14,
            index: 170,
            name: None,
        }),
        (14, 171) => Some(ExtOpcode {
            category: 14,
            index: 171,
            name: None,
        }),
        (14, 172) => Some(ExtOpcode {
            category: 14,
            index: 172,
            name: None,
        }),
        (14, 173) => Some(ExtOpcode {
            category: 14,
            index: 173,
            name: None,
        }),
        (14, 174) => Some(ExtOpcode {
            category: 14,
            index: 174,
            name: None,
        }),
        (14, 175) => Some(ExtOpcode {
            category: 14,
            index: 175,
            name: Some("set_font_size"),
        }),
        (14, 176) => Some(ExtOpcode {
            category: 14,
            index: 176,
            name: Some("get_font_size"),
        }),
        (14, 177) => Some(ExtOpcode {
            category: 14,
            index: 177,
            name: Some("get_font_type"),
        }),
        (14, 178) => Some(ExtOpcode {
            category: 14,
            index: 178,
            name: Some("set_font_effect"),
        }),
        (14, 179) => Some(ExtOpcode {
            category: 14,
            index: 179,
            name: Some("get_font_effect"),
        }),
        (14, 180) => Some(ExtOpcode {
            category: 14,
            index: 180,
            name: Some("get_pull_key"),
        }),
        (14, 181) => Some(ExtOpcode {
            category: 14,
            index: 181,
            name: Some("get_on_key"),
        }),
        (14, 182) => Some(ExtOpcode {
            category: 14,
            index: 182,
            name: Some("get_push_key"),
        }),
        (14, 183) => Some(ExtOpcode {
            category: 14,
            index: 183,
            name: Some("input_clear"),
        }),
        (14, 184) => Some(ExtOpcode {
            category: 14,
            index: 184,
            name: Some("change_window_size"),
        }),
        (14, 185) => Some(ExtOpcode {
            category: 14,
            index: 185,
            name: Some("change_aspect_mode"),
        }),
        (14, 186) => Some(ExtOpcode {
            category: 14,
            index: 186,
            name: Some("aspect_position_enable"),
        }),
        (14, 187) => Some(ExtOpcode {
            category: 14,
            index: 187,
            name: None,
        }),
        (14, 188) => Some(ExtOpcode {
            category: 14,
            index: 188,
            name: Some("get_aspect_mode"),
        }),
        (14, 189) => Some(ExtOpcode {
            category: 14,
            index: 189,
            name: Some("get_monitor_size"),
        }),
        (14, 190) => Some(ExtOpcode {
            category: 14,
            index: 190,
            name: None,
        }),
        (14, 191) => Some(ExtOpcode {
            category: 14,
            index: 191,
            name: Some("get_system_metrics"),
        }),
        (14, 192) => Some(ExtOpcode {
            category: 14,
            index: 192,
            name: Some("set_system_path"),
        }),
        (14, 193) => Some(ExtOpcode {
            category: 14,
            index: 193,
            name: Some("set_allmosaicthumbnail"),
        }),
        (14, 194) => Some(ExtOpcode {
            category: 14,
            index: 194,
            name: Some("enable_window_change"),
        }),
        (14, 195) => Some(ExtOpcode {
            category: 14,
            index: 195,
            name: Some("is_enable_window_change"),
        }),
        (14, 196) => Some(ExtOpcode {
            category: 14,
            index: 196,
            name: Some("set_cursor_null"),
        }),
        (14, 197) => Some(ExtOpcode {
            category: 14,
            index: 197,
            name: Some("set_hide_cursor_time"),
        }),
        (14, 198) => Some(ExtOpcode {
            category: 14,
            index: 198,
            name: Some("get_hide_cursor_time"),
        }),
        (14, 199) => Some(ExtOpcode {
            category: 14,
            index: 199,
            name: Some("scene_skip"),
        }),
        (14, 200) => Some(ExtOpcode {
            category: 14,
            index: 200,
            name: None,
        }),
        (14, 201) => Some(ExtOpcode {
            category: 14,
            index: 201,
            name: None,
        }),
        (14, 202) => Some(ExtOpcode {
            category: 14,
            index: 202,
            name: Some("get_async_key"),
        }),
        (14, 203) => Some(ExtOpcode {
            category: 14,
            index: 203,
            name: Some("get_font_color"),
        }),
        (14, 204) => Some(ExtOpcode {
            category: 14,
            index: 204,
            name: None,
        }),
        (14, 205) => Some(ExtOpcode {
            category: 14,
            index: 205,
            name: Some("history_skip"),
        }),
        (14, 206) => Some(ExtOpcode {
            category: 14,
            index: 206,
            name: None,
        }),
        (14, 207) => Some(ExtOpcode {
            category: 14,
            index: 207,
            name: None,
        }),
        (14, 208) => Some(ExtOpcode {
            category: 14,
            index: 208,
            name: Some("set_language"),
        }),
        (14, 209) => Some(ExtOpcode {
            category: 14,
            index: 209,
            name: Some("set_achievement"),
        }),
        (14, 211) => Some(ExtOpcode {
            category: 14,
            index: 211,
            name: Some("system_btn_set"),
        }),
        (14, 212) => Some(ExtOpcode {
            category: 14,
            index: 212,
            name: Some("system_btn_release"),
        }),
        (14, 213) => Some(ExtOpcode {
            category: 14,
            index: 213,
            name: Some("system_btn_enable"),
        }),
        (14, 216) => Some(ExtOpcode {
            category: 14,
            index: 216,
            name: Some("text_init"),
        }),
        (14, 217) => Some(ExtOpcode {
            category: 14,
            index: 217,
            name: Some("text_set_icon"),
        }),
        (14, 218) => Some(ExtOpcode {
            category: 14,
            index: 218,
            name: Some("text"),
        }),
        (14, 219) => Some(ExtOpcode {
            category: 14,
            index: 219,
            name: Some("text_hide"),
        }),
        (14, 220) => Some(ExtOpcode {
            category: 14,
            index: 220,
            name: Some("text_show"),
        }),
        (14, 221) => Some(ExtOpcode {
            category: 14,
            index: 221,
            name: Some("text_set_btn"),
        }),
        (14, 222) => Some(ExtOpcode {
            category: 14,
            index: 222,
            name: Some("text_uninit"),
        }),
        (14, 223) => Some(ExtOpcode {
            category: 14,
            index: 223,
            name: Some("text_set_rect_invalid_param"),
        }),
        (14, 224) => Some(ExtOpcode {
            category: 14,
            index: 224,
            name: Some("text_clear"),
        }),
        (14, 225) => Some(ExtOpcode {
            category: 14,
            index: 225,
            name: None,
        }),
        (14, 226) => Some(ExtOpcode {
            category: 14,
            index: 226,
            name: Some("text_get_time"),
        }),
        (14, 227) => Some(ExtOpcode {
            category: 14,
            index: 227,
            name: Some("text_window_set_alpha"),
        }),
        (14, 228) => Some(ExtOpcode {
            category: 14,
            index: 228,
            name: Some("text_voice_play"),
        }),
        (14, 229) => Some(ExtOpcode {
            category: 14,
            index: 229,
            name: None,
        }),
        (14, 230) => Some(ExtOpcode {
            category: 14,
            index: 230,
            name: Some("text_set_icon_animation_time"),
        }),
        (14, 231) => Some(ExtOpcode {
            category: 14,
            index: 231,
            name: Some("text_w"),
        }),
        (14, 232) => Some(ExtOpcode {
            category: 14,
            index: 232,
            name: Some("text_a"),
        }),
        (14, 233) => Some(ExtOpcode {
            category: 14,
            index: 233,
            name: Some("text_wa"),
        }),
        (14, 234) => Some(ExtOpcode {
            category: 14,
            index: 234,
            name: Some("text_n"),
        }),
        (14, 235) => Some(ExtOpcode {
            category: 14,
            index: 235,
            name: Some("text_cat"),
        }),
        (14, 236) => Some(ExtOpcode {
            category: 14,
            index: 236,
            name: Some("set_history"),
        }),
        (14, 237) => Some(ExtOpcode {
            category: 14,
            index: 237,
            name: Some("is_text_visible"),
        }),
        (14, 238) => Some(ExtOpcode {
            category: 14,
            index: 238,
            name: Some("text_set_base"),
        }),
        (14, 239) => Some(ExtOpcode {
            category: 14,
            index: 239,
            name: Some("enable_voice_cut"),
        }),
        (14, 240) => Some(ExtOpcode {
            category: 14,
            index: 240,
            name: Some("is_voice_cut"),
        }),
        (14, 241) => Some(ExtOpcode {
            category: 14,
            index: 241,
            name: Some("texttimecheckset"),
        }),
        (14, 242) => Some(ExtOpcode {
            category: 14,
            index: 242,
            name: None,
        }),
        (14, 243) => Some(ExtOpcode {
            category: 14,
            index: 243,
            name: None,
        }),
        (14, 244) => Some(ExtOpcode {
            category: 14,
            index: 244,
            name: Some("text_set_color"),
        }),
        (14, 245) => Some(ExtOpcode {
            category: 14,
            index: 245,
            name: Some("textredraw"),
        }),
        (14, 246) => Some(ExtOpcode {
            category: 14,
            index: 246,
            name: Some("set_text_mode"),
        }),
        (14, 247) => Some(ExtOpcode {
            category: 14,
            index: 247,
            name: Some("text_init_visualnovelmode"),
        }),
        (14, 248) => Some(ExtOpcode {
            category: 14,
            index: 248,
            name: Some("text_set_icon_mode"),
        }),
        (14, 249) => Some(ExtOpcode {
            category: 14,
            index: 249,
            name: Some("text_vn_br"),
        }),
        (14, 250) => Some(ExtOpcode {
            category: 14,
            index: 250,
            name: None,
        }),
        (14, 251) => Some(ExtOpcode {
            category: 14,
            index: 251,
            name: None,
        }),
        (14, 252) => Some(ExtOpcode {
            category: 14,
            index: 252,
            name: None,
        }),
        (14, 253) => Some(ExtOpcode {
            category: 14,
            index: 253,
            name: None,
        }),
        (14, 254) => Some(ExtOpcode {
            category: 14,
            index: 254,
            name: Some("tips_get_str"),
        }),
        (14, 255) => Some(ExtOpcode {
            category: 14,
            index: 255,
            name: Some("tips_get_param"),
        }),
        (14, 256) => Some(ExtOpcode {
            category: 14,
            index: 256,
            name: Some("tips_reset"),
        }),
        (14, 257) => Some(ExtOpcode {
            category: 14,
            index: 257,
            name: Some("tips_search"),
        }),
        (14, 258) => Some(ExtOpcode {
            category: 14,
            index: 258,
            name: Some("tips_set_color"),
        }),
        (14, 259) => Some(ExtOpcode {
            category: 14,
            index: 259,
            name: Some("tips_stop"),
        }),
        (14, 260) => Some(ExtOpcode {
            category: 14,
            index: 260,
            name: Some("tips_get_flag"),
        }),
        (14, 261) => Some(ExtOpcode {
            category: 14,
            index: 261,
            name: Some("tips_init"),
        }),
        (14, 262) => Some(ExtOpcode {
            category: 14,
            index: 262,
            name: Some("tips_pause"),
        }),
        (14, 264) => Some(ExtOpcode {
            category: 14,
            index: 264,
            name: Some("voice_play"),
        }),
        (14, 265) => Some(ExtOpcode {
            category: 14,
            index: 265,
            name: Some("voice_stop"),
        }),
        (14, 266) => Some(ExtOpcode {
            category: 14,
            index: 266,
            name: Some("voice_set_volume"),
        }),
        (14, 267) => Some(ExtOpcode {
            category: 14,
            index: 267,
            name: Some("voice_get_volume"),
        }),
        (14, 268) => Some(ExtOpcode {
            category: 14,
            index: 268,
            name: Some("set_voice_info"),
        }),
        (14, 269) => Some(ExtOpcode {
            category: 14,
            index: 269,
            name: Some("voice_enable"),
        }),
        (14, 270) => Some(ExtOpcode {
            category: 14,
            index: 270,
            name: Some("is_voice_enable"),
        }),
        (14, 271) => Some(ExtOpcode {
            category: 14,
            index: 271,
            name: None,
        }),
        (14, 272) => Some(ExtOpcode {
            category: 14,
            index: 272,
            name: Some("bgv_play"),
        }),
        (14, 273) => Some(ExtOpcode {
            category: 14,
            index: 273,
            name: Some("bgv_stop"),
        }),
        (14, 274) => Some(ExtOpcode {
            category: 14,
            index: 274,
            name: Some("bgv_enable"),
        }),
        (14, 275) => Some(ExtOpcode {
            category: 14,
            index: 275,
            name: Some("get_voice_ex_volume"),
        }),
        (14, 276) => Some(ExtOpcode {
            category: 14,
            index: 276,
            name: Some("set_voice_ex_volume"),
        }),
        (14, 277) => Some(ExtOpcode {
            category: 14,
            index: 277,
            name: Some("voice_check_enable"),
        }),
        (14, 278) => Some(ExtOpcode {
            category: 14,
            index: 278,
            name: Some("voice_autopan_initialize"),
        }),
        (14, 279) => Some(ExtOpcode {
            category: 14,
            index: 279,
            name: Some("voice_autopan_enable"),
        }),
        (14, 280) => Some(ExtOpcode {
            category: 14,
            index: 280,
            name: Some("set_voice_autopan_size_over"),
        }),
        (14, 281) => Some(ExtOpcode {
            category: 14,
            index: 281,
            name: Some("is_voice_autopan_enable"),
        }),
        (14, 282) => Some(ExtOpcode {
            category: 14,
            index: 282,
            name: Some("voice_wait"),
        }),
        (14, 283) => Some(ExtOpcode {
            category: 14,
            index: 283,
            name: Some("bgv_pause"),
        }),
        (14, 284) => Some(ExtOpcode {
            category: 14,
            index: 284,
            name: Some("bgv_mute"),
        }),
        (14, 285) => Some(ExtOpcode {
            category: 14,
            index: 285,
            name: Some("set_bgv_volume"),
        }),
        (14, 286) => Some(ExtOpcode {
            category: 14,
            index: 286,
            name: Some("get_bgv_volume"),
        }),
        (14, 287) => Some(ExtOpcode {
            category: 14,
            index: 287,
            name: Some("set_bgv_auto_volume"),
        }),
        (14, 288) => Some(ExtOpcode {
            category: 14,
            index: 288,
            name: Some("voice_mute"),
        }),
        (14, 289) => Some(ExtOpcode {
            category: 14,
            index: 289,
            name: Some("voice_call"),
        }),
        (14, 290) => Some(ExtOpcode {
            category: 14,
            index: 290,
            name: Some("voice_call_clear"),
        }),
        (14, 292) => Some(ExtOpcode {
            category: 14,
            index: 292,
            name: Some("wait"),
        }),
        (14, 293) => Some(ExtOpcode {
            category: 14,
            index: 293,
            name: Some("wait_click"),
        }),
        (14, 294) => Some(ExtOpcode {
            category: 14,
            index: 294,
            name: Some("wait_sync_begin"),
        }),
        (14, 295) => Some(ExtOpcode {
            category: 14,
            index: 295,
            name: Some("wait_sync_release"),
        }),
        (14, 296) => Some(ExtOpcode {
            category: 14,
            index: 296,
            name: Some("wait_sync_end"),
        }),
        (14, 297) => Some(ExtOpcode {
            category: 14,
            index: 297,
            name: None,
        }),
        (14, 298) => Some(ExtOpcode {
            category: 14,
            index: 298,
            name: Some("wait_clear"),
        }),
        (14, 299) => Some(ExtOpcode {
            category: 14,
            index: 299,
            name: Some("wait_click_no_anim"),
        }),
        (15, 0) => Some(ExtOpcode {
            category: 15,
            index: 0,
            name: None,
        }),
        (15, 1) => Some(ExtOpcode {
            category: 15,
            index: 1,
            name: None,
        }),
        (15, 2) => Some(ExtOpcode {
            category: 15,
            index: 2,
            name: Some("set_window_text"),
        }),
        (15, 3) => Some(ExtOpcode {
            category: 15,
            index: 3,
            name: None,
        }),
        (15, 4) => Some(ExtOpcode {
            category: 15,
            index: 4,
            name: None,
        }),
        (15, 5) => Some(ExtOpcode {
            category: 15,
            index: 5,
            name: None,
        }),
        (15, 6) => Some(ExtOpcode {
            category: 15,
            index: 6,
            name: None,
        }),
        (15, 7) => Some(ExtOpcode {
            category: 15,
            index: 7,
            name: None,
        }),
        (15, 8) => Some(ExtOpcode {
            category: 15,
            index: 8,
            name: None,
        }),
        (15, 9) => Some(ExtOpcode {
            category: 15,
            index: 9,
            name: None,
        }),
        (15, 10) => Some(ExtOpcode {
            category: 15,
            index: 10,
            name: Some("debug_break"),
        }),
        (15, 13) => Some(ExtOpcode {
            category: 15,
            index: 13,
            name: Some("app_exec"),
        }),
        (15, 14) => Some(ExtOpcode {
            category: 15,
            index: 14,
            name: Some("is_playername"),
        }),
        (15, 15) => Some(ExtOpcode {
            category: 15,
            index: 15,
            name: None,
        }),
        (15, 16) => Some(ExtOpcode {
            category: 15,
            index: 16,
            name: None,
        }),
        (15, 17) => Some(ExtOpcode {
            category: 15,
            index: 17,
            name: None,
        }),
        (15, 18) => Some(ExtOpcode {
            category: 15,
            index: 18,
            name: None,
        }),
        (15, 19) => Some(ExtOpcode {
            category: 15,
            index: 19,
            name: None,
        }),
        (15, 20) => Some(ExtOpcode {
            category: 15,
            index: 20,
            name: Some("file_exist"),
        }),
        (15, 21) => Some(ExtOpcode {
            category: 15,
            index: 21,
            name: Some("wsprint"),
        }),
        (15, 22) => Some(ExtOpcode {
            category: 15,
            index: 22,
            name: Some("check_disc"),
        }),
        (15, 23) => Some(ExtOpcode {
            category: 15,
            index: 23,
            name: None,
        }),
        (15, 24) => Some(ExtOpcode {
            category: 15,
            index: 24,
            name: None,
        }),
        (15, 25) => Some(ExtOpcode {
            category: 15,
            index: 25,
            name: None,
        }),
        (15, 26) => Some(ExtOpcode {
            category: 15,
            index: 26,
            name: Some("update_access"),
        }),
        (15, 27) => Some(ExtOpcode {
            category: 15,
            index: 27,
            name: None,
        }),
        (15, 28) => Some(ExtOpcode {
            category: 15,
            index: 28,
            name: None,
        }),
        (15, 29) => Some(ExtOpcode {
            category: 15,
            index: 29,
            name: None,
        }),
        (15, 30) => Some(ExtOpcode {
            category: 15,
            index: 30,
            name: None,
        }),
        (15, 31) => Some(ExtOpcode {
            category: 15,
            index: 31,
            name: None,
        }),
        (15, 32) => Some(ExtOpcode {
            category: 15,
            index: 32,
            name: None,
        }),
        (15, 33) => Some(ExtOpcode {
            category: 15,
            index: 33,
            name: None,
        }),
        (15, 34) => Some(ExtOpcode {
            category: 15,
            index: 34,
            name: Some("player_name_set_begin"),
        }),
        (15, 35) => Some(ExtOpcode {
            category: 15,
            index: 35,
            name: Some("player_name_set_end"),
        }),
        (15, 36) => Some(ExtOpcode {
            category: 15,
            index: 36,
            name: Some("player_name_set_check"),
        }),
        (15, 37) => Some(ExtOpcode {
            category: 15,
            index: 37,
            name: None,
        }),
        (15, 38) => Some(ExtOpcode {
            category: 15,
            index: 38,
            name: Some("player_name_reset"),
        }),
        (15, 39) => Some(ExtOpcode {
            category: 15,
            index: 39,
            name: Some("player_name_set_direct"),
        }),
        (15, 40) => Some(ExtOpcode {
            category: 15,
            index: 40,
            name: None,
        }),
        (15, 41) => Some(ExtOpcode {
            category: 15,
            index: 41,
            name: None,
        }),
        (15, 42) => Some(ExtOpcode {
            category: 15,
            index: 42,
            name: Some("openfile"),
        }),
        (15, 43) => Some(ExtOpcode {
            category: 15,
            index: 43,
            name: Some("read_file"),
        }),
        (15, 44) => Some(ExtOpcode {
            category: 15,
            index: 44,
            name: Some("close_file_not_handle"),
        }),
        (15, 45) => Some(ExtOpcode {
            category: 15,
            index: 45,
            name: Some("set_file_pointer"),
        }),
        (15, 46) => Some(ExtOpcode {
            category: 15,
            index: 46,
            name: Some("file_string"),
        }),
        (15, 47) => Some(ExtOpcode {
            category: 15,
            index: 47,
            name: Some("set_last_process"),
        }),
        (15, 48) => Some(ExtOpcode {
            category: 15,
            index: 48,
            name: Some("sz_buf"),
        }),
        (15, 49) => Some(ExtOpcode {
            category: 15,
            index: 49,
            name: Some("getprivateprofileint"),
        }),
        (15, 50) => Some(ExtOpcode {
            category: 15,
            index: 50,
            name: None,
        }),
        (15, 51) => Some(ExtOpcode {
            category: 15,
            index: 51,
            name: None,
        }),
        (15, 52) => Some(ExtOpcode {
            category: 15,
            index: 52,
            name: None,
        }),
        (15, 53) => Some(ExtOpcode {
            category: 15,
            index: 53,
            name: Some("is_tweet"),
        }),
        (15, 54) => Some(ExtOpcode {
            category: 15,
            index: 54,
            name: None,
        }),
        (15, 55) => Some(ExtOpcode {
            category: 15,
            index: 55,
            name: None,
        }),
        (15, 56) => Some(ExtOpcode {
            category: 15,
            index: 56,
            name: None,
        }),
        (15, 57) => Some(ExtOpcode {
            category: 15,
            index: 57,
            name: None,
        }),
        (15, 58) => Some(ExtOpcode {
            category: 15,
            index: 58,
            name: None,
        }),
        (15, 59) => Some(ExtOpcode {
            category: 15,
            index: 59,
            name: None,
        }),
        (15, 60) => Some(ExtOpcode {
            category: 15,
            index: 60,
            name: None,
        }),
        (15, 61) => Some(ExtOpcode {
            category: 15,
            index: 61,
            name: None,
        }),
        (15, 62) => Some(ExtOpcode {
            category: 15,
            index: 62,
            name: None,
        }),
        (15, 63) => Some(ExtOpcode {
            category: 15,
            index: 63,
            name: None,
        }),
        (15, 64) => Some(ExtOpcode {
            category: 15,
            index: 64,
            name: None,
        }),
        (15, 65) => Some(ExtOpcode {
            category: 15,
            index: 65,
            name: None,
        }),
        (15, 66) => Some(ExtOpcode {
            category: 15,
            index: 66,
            name: None,
        }),
        (15, 67) => Some(ExtOpcode {
            category: 15,
            index: 67,
            name: None,
        }),
        (15, 68) => Some(ExtOpcode {
            category: 15,
            index: 68,
            name: None,
        }),
        (15, 69) => Some(ExtOpcode {
            category: 15,
            index: 69,
            name: None,
        }),
        (15, 70) => Some(ExtOpcode {
            category: 15,
            index: 70,
            name: None,
        }),
        (15, 71) => Some(ExtOpcode {
            category: 15,
            index: 71,
            name: None,
        }),
        (15, 72) => Some(ExtOpcode {
            category: 15,
            index: 72,
            name: None,
        }),
        (15, 73) => Some(ExtOpcode {
            category: 15,
            index: 73,
            name: None,
        }),
        (15, 74) => Some(ExtOpcode {
            category: 15,
            index: 74,
            name: None,
        }),
        (15, 75) => Some(ExtOpcode {
            category: 15,
            index: 75,
            name: None,
        }),
        (15, 76) => Some(ExtOpcode {
            category: 15,
            index: 76,
            name: None,
        }),
        (15, 77) => Some(ExtOpcode {
            category: 15,
            index: 77,
            name: None,
        }),
        (15, 78) => Some(ExtOpcode {
            category: 15,
            index: 78,
            name: None,
        }),
        (15, 79) => Some(ExtOpcode {
            category: 15,
            index: 79,
            name: None,
        }),
        (15, 80) => Some(ExtOpcode {
            category: 15,
            index: 80,
            name: None,
        }),
        (15, 81) => Some(ExtOpcode {
            category: 15,
            index: 81,
            name: None,
        }),
        (15, 82) => Some(ExtOpcode {
            category: 15,
            index: 82,
            name: None,
        }),
        (15, 83) => Some(ExtOpcode {
            category: 15,
            index: 83,
            name: None,
        }),
        (15, 84) => Some(ExtOpcode {
            category: 15,
            index: 84,
            name: None,
        }),
        (15, 85) => Some(ExtOpcode {
            category: 15,
            index: 85,
            name: None,
        }),
        (15, 86) => Some(ExtOpcode {
            category: 15,
            index: 86,
            name: Some("result_tweet"),
        }),
        (15, 87) => Some(ExtOpcode {
            category: 15,
            index: 87,
            name: Some("get_tweet_key"),
        }),
        (15, 88) => Some(ExtOpcode {
            category: 15,
            index: 88,
            name: Some("set_tweet_key"),
        }),
        (15, 89) => Some(ExtOpcode {
            category: 15,
            index: 89,
            name: None,
        }),
        (15, 90) => Some(ExtOpcode {
            category: 15,
            index: 90,
            name: Some("tweet_authorize"),
        }),
        (15, 91) => Some(ExtOpcode {
            category: 15,
            index: 91,
            name: None,
        }),
        (15, 92) => Some(ExtOpcode {
            category: 15,
            index: 92,
            name: None,
        }),
        (15, 93) => Some(ExtOpcode {
            category: 15,
            index: 93,
            name: None,
        }),
        (15, 94) => Some(ExtOpcode {
            category: 15,
            index: 94,
            name: Some("tips_csv_read_error"),
        }),
        (15, 95) => Some(ExtOpcode {
            category: 15,
            index: 95,
            name: Some("tips_csv_get_error"),
        }),
        (15, 96) => Some(ExtOpcode {
            category: 15,
            index: 96,
            name: Some("tips_csv_search_not_found"),
        }),
        (15, 97) => Some(ExtOpcode {
            category: 15,
            index: 97,
            name: None,
        }),
        (15, 98) => Some(ExtOpcode {
            category: 15,
            index: 98,
            name: None,
        }),
        (15, 99) => Some(ExtOpcode {
            category: 15,
            index: 99,
            name: Some("tips_acc_save"),
        }),
        (15, 100) => Some(ExtOpcode {
            category: 15,
            index: 100,
            name: Some("is_network"),
        }),
        (15, 101) => Some(ExtOpcode {
            category: 15,
            index: 101,
            name: Some("is_touch"),
        }),
        (15, 102) => Some(ExtOpcode {
            category: 15,
            index: 102,
            name: None,
        }),
        (15, 103) => Some(ExtOpcode {
            category: 15,
            index: 103,
            name: None,
        }),
        (15, 104) => Some(ExtOpcode {
            category: 15,
            index: 104,
            name: None,
        }),
        (15, 105) => Some(ExtOpcode {
            category: 15,
            index: 105,
            name: None,
        }),
        (15, 106) => Some(ExtOpcode {
            category: 15,
            index: 106,
            name: None,
        }),
        (15, 107) => Some(ExtOpcode {
            category: 15,
            index: 107,
            name: None,
        }),
        (15, 108) => Some(ExtOpcode {
            category: 15,
            index: 108,
            name: None,
        }),
        (15, 109) => Some(ExtOpcode {
            category: 15,
            index: 109,
            name: None,
        }),
        (15, 110) => Some(ExtOpcode {
            category: 15,
            index: 110,
            name: None,
        }),
        (15, 111) => Some(ExtOpcode {
            category: 15,
            index: 111,
            name: None,
        }),
        (15, 112) => Some(ExtOpcode {
            category: 15,
            index: 112,
            name: None,
        }),
        (15, 113) => Some(ExtOpcode {
            category: 15,
            index: 113,
            name: None,
        }),
        (15, 114) => Some(ExtOpcode {
            category: 15,
            index: 114,
            name: None,
        }),
        (15, 115) => Some(ExtOpcode {
            category: 15,
            index: 115,
            name: None,
        }),
        (15, 116) => Some(ExtOpcode {
            category: 15,
            index: 116,
            name: None,
        }),
        (15, 117) => Some(ExtOpcode {
            category: 15,
            index: 117,
            name: None,
        }),
        (15, 118) => Some(ExtOpcode {
            category: 15,
            index: 118,
            name: None,
        }),
        (15, 119) => Some(ExtOpcode {
            category: 15,
            index: 119,
            name: None,
        }),
        (15, 120) => Some(ExtOpcode {
            category: 15,
            index: 120,
            name: None,
        }),
        (15, 121) => Some(ExtOpcode {
            category: 15,
            index: 121,
            name: None,
        }),
        (15, 122) => Some(ExtOpcode {
            category: 15,
            index: 122,
            name: None,
        }),
        (15, 123) => Some(ExtOpcode {
            category: 15,
            index: 123,
            name: None,
        }),
        (15, 124) => Some(ExtOpcode {
            category: 15,
            index: 124,
            name: None,
        }),
        (15, 126) => Some(ExtOpcode {
            category: 15,
            index: 126,
            name: None,
        }),
        (15, 127) => Some(ExtOpcode {
            category: 15,
            index: 127,
            name: Some("run_no_wait"),
        }),
        (15, 128) => Some(ExtOpcode {
            category: 15,
            index: 128,
            name: Some("run_stack"),
        }),
        (15, 130) => Some(ExtOpcode {
            category: 15,
            index: 130,
            name: Some("fx_effect_cls"),
        }),
        (15, 131) => Some(ExtOpcode {
            category: 15,
            index: 131,
            name: Some("fx_raster_stop"),
        }),
        (15, 132) => Some(ExtOpcode {
            category: 15,
            index: 132,
            name: Some("fx_effect_wait"),
        }),
        (15, 133) => Some(ExtOpcode {
            category: 15,
            index: 133,
            name: None,
        }),
        (15, 135) => Some(ExtOpcode {
            category: 15,
            index: 135,
            name: Some("random"),
        }),
        (15, 136) => Some(ExtOpcode {
            category: 15,
            index: 136,
            name: Some("abs"),
        }),
        (15, 137) => Some(ExtOpcode {
            category: 15,
            index: 137,
            name: Some("sin"),
        }),
        (15, 138) => Some(ExtOpcode {
            category: 15,
            index: 138,
            name: Some("cos"),
        }),
        (15, 139) => Some(ExtOpcode {
            category: 15,
            index: 139,
            name: Some("tan"),
        }),
        (15, 140) => Some(ExtOpcode {
            category: 15,
            index: 140,
            name: Some("atan"),
        }),
        (15, 141) => Some(ExtOpcode {
            category: 15,
            index: 141,
            name: Some("log"),
        }),
        (15, 142) => Some(ExtOpcode {
            category: 15,
            index: 142,
            name: Some("log10"),
        }),
        (15, 143) => Some(ExtOpcode {
            category: 15,
            index: 143,
            name: None,
        }),
        (15, 144) => Some(ExtOpcode {
            category: 15,
            index: 144,
            name: Some("sqrt"),
        }),
        (15, 145) => Some(ExtOpcode {
            category: 15,
            index: 145,
            name: None,
        }),
        (15, 146) => Some(ExtOpcode {
            category: 15,
            index: 146,
            name: None,
        }),
        (15, 150) => Some(ExtOpcode {
            category: 15,
            index: 150,
            name: Some("sp_set"),
        }),
        (15, 151) => Some(ExtOpcode {
            category: 15,
            index: 151,
            name: Some("sp_set_ex"),
        }),
        (15, 152) => Some(ExtOpcode {
            category: 15,
            index: 152,
            name: Some("sp_set_pos"),
        }),
        (15, 153) => Some(ExtOpcode {
            category: 15,
            index: 153,
            name: Some("sp_cls"),
        }),
        (15, 154) => Some(ExtOpcode {
            category: 15,
            index: 154,
            name: Some("sp_set_alpha"),
        }),
        (15, 155) => Some(ExtOpcode {
            category: 15,
            index: 155,
            name: Some("set_priority"),
        }),
        (15, 156) => Some(ExtOpcode {
            category: 15,
            index: 156,
            name: None,
        }),
        (15, 157) => Some(ExtOpcode {
            category: 15,
            index: 157,
            name: Some("sp_set_center"),
        }),
        (15, 159) => Some(ExtOpcode {
            category: 15,
            index: 159,
            name: Some("sp_cls_ex"),
        }),
        (15, 160) => Some(ExtOpcode {
            category: 15,
            index: 160,
            name: Some("set_filter"),
        }),
        (15, 161) => Some(ExtOpcode {
            category: 15,
            index: 161,
            name: Some("sp_cls_transition"),
        }),
        (15, 162) => Some(ExtOpcode {
            category: 15,
            index: 162,
            name: Some("sp_set_pos_ex"),
        }),
        (15, 163) => Some(ExtOpcode {
            category: 15,
            index: 163,
            name: Some("sp_set_rect_pos"),
        }),
        (15, 164) => Some(ExtOpcode {
            category: 15,
            index: 164,
            name: None,
        }),
        (15, 165) => Some(ExtOpcode {
            category: 15,
            index: 165,
            name: Some("sp_set_scale"),
        }),
        (15, 166) => Some(ExtOpcode {
            category: 15,
            index: 166,
            name: Some("sp_set_rotate"),
        }),
        (15, 167) => Some(ExtOpcode {
            category: 15,
            index: 167,
            name: Some("face_init"),
        }),
        (15, 168) => Some(ExtOpcode {
            category: 15,
            index: 168,
            name: Some("face_set"),
        }),
        (15, 169) => Some(ExtOpcode {
            category: 15,
            index: 169,
            name: Some("not_image_sp_get_color"),
        }),
        (15, 170) => Some(ExtOpcode {
            category: 15,
            index: 170,
            name: Some("sptext"),
        }),
        (15, 171) => Some(ExtOpcode {
            category: 15,
            index: 171,
            name: Some("face_cls"),
        }),
        (15, 172) => Some(ExtOpcode {
            category: 15,
            index: 172,
            name: Some("sp_set_rect"),
        }),
        (15, 173) => Some(ExtOpcode {
            category: 15,
            index: 173,
            name: Some("sp_set_pos_move"),
        }),
        (15, 174) => Some(ExtOpcode {
            category: 15,
            index: 174,
            name: Some("not_image_sp_get_alpha"),
        }),
        (15, 175) => Some(ExtOpcode {
            category: 15,
            index: 175,
            name: Some("not_image_sp_get_rotate"),
        }),
        (15, 176) => Some(ExtOpcode {
            category: 15,
            index: 176,
            name: None,
        }),
        (15, 177) => Some(ExtOpcode {
            category: 15,
            index: 177,
            name: None,
        }),
        (15, 178) => Some(ExtOpcode {
            category: 15,
            index: 178,
            name: None,
        }),
        (15, 179) => Some(ExtOpcode {
            category: 15,
            index: 179,
            name: None,
        }),
        (15, 180) => Some(ExtOpcode {
            category: 15,
            index: 180,
            name: Some("sp_create"),
        }),
        (15, 181) => Some(ExtOpcode {
            category: 15,
            index: 181,
            name: Some("sp_anime_clear"),
        }),
        (15, 182) => Some(ExtOpcode {
            category: 15,
            index: 182,
            name: None,
        }),
        (15, 183) => Some(ExtOpcode {
            category: 15,
            index: 183,
            name: None,
        }),
        (15, 184) => Some(ExtOpcode {
            category: 15,
            index: 184,
            name: Some("not_image_sp_get_scale"),
        }),
        (15, 185) => Some(ExtOpcode {
            category: 15,
            index: 185,
            name: Some("sp_set_color_0x"),
        }),
        (15, 186) => Some(ExtOpcode {
            category: 15,
            index: 186,
            name: Some("sp_bitblt"),
        }),
        (15, 187) => Some(ExtOpcode {
            category: 15,
            index: 187,
            name: Some("sp_set_shake"),
        }),
        (15, 188) => Some(ExtOpcode {
            category: 15,
            index: 188,
            name: Some("sp_paint"),
        }),
        (15, 189) => Some(ExtOpcode {
            category: 15,
            index: 189,
            name: None,
        }),
        (15, 190) => Some(ExtOpcode {
            category: 15,
            index: 190,
            name: Some("sp_load_wait_time"),
        }),
        (15, 191) => Some(ExtOpcode {
            category: 15,
            index: 191,
            name: Some("sp_draw"),
        }),
        (15, 192) => Some(ExtOpcode {
            category: 15,
            index: 192,
            name: None,
        }),
        (15, 193) => Some(ExtOpcode {
            category: 15,
            index: 193,
            name: Some("sp_unlock"),
        }),
        (15, 194) => Some(ExtOpcode {
            category: 15,
            index: 194,
            name: Some("sp_show"),
        }),
        (15, 195) => Some(ExtOpcode {
            category: 15,
            index: 195,
            name: Some("sp_hide"),
        }),
        (15, 196) => Some(ExtOpcode {
            category: 15,
            index: 196,
            name: None,
        }),
        (15, 197) => Some(ExtOpcode {
            category: 15,
            index: 197,
            name: Some("sp_set_child"),
        }),
        (15, 198) => Some(ExtOpcode {
            category: 15,
            index: 198,
            name: Some("sp_set_transition"),
        }),
        (15, 199) => Some(ExtOpcode {
            category: 15,
            index: 199,
            name: Some("sp_copy_image"),
        }),
        (15, 200) => Some(ExtOpcode {
            category: 15,
            index: 200,
            name: Some("sp_transition"),
        }),
        (15, 201) => Some(ExtOpcode {
            category: 15,
            index: 201,
            name: Some("set_aspect_position_type"),
        }),
        (15, 202) => Some(ExtOpcode {
            category: 15,
            index: 202,
            name: Some("get_backbuffer"),
        }),
        (15, 203) => Some(ExtOpcode {
            category: 15,
            index: 203,
            name: Some("sp_set_mask"),
        }),
        (15, 204) => Some(ExtOpcode {
            category: 15,
            index: 204,
            name: None,
        }),
        (15, 205) => Some(ExtOpcode {
            category: 15,
            index: 205,
            name: Some("spsetanime"),
        }),
        (15, 206) => Some(ExtOpcode {
            category: 15,
            index: 206,
            name: Some("drawtext"),
        }),
        (15, 207) => Some(ExtOpcode {
            category: 15,
            index: 207,
            name: None,
        }),
        (15, 208) => Some(ExtOpcode {
            category: 15,
            index: 208,
            name: None,
        }),
        (15, 210) => Some(ExtOpcode {
            category: 15,
            index: 210,
            name: Some("history_init_0x_0x"),
        }),
        (15, 211) => Some(ExtOpcode {
            category: 15,
            index: 211,
            name: Some("historybegin_lpbyte_ptagdata_sztext"),
        }),
        (15, 212) => Some(ExtOpcode {
            category: 15,
            index: 212,
            name: Some("history_end"),
        }),
        (15, 213) => Some(ExtOpcode {
            category: 15,
            index: 213,
            name: None,
        }),
        (15, 214) => Some(ExtOpcode {
            category: 15,
            index: 214,
            name: None,
        }),
        (15, 215) => Some(ExtOpcode {
            category: 15,
            index: 215,
            name: Some("history_get_height"),
        }),
        (15, 216) => Some(ExtOpcode {
            category: 15,
            index: 216,
            name: None,
        }),
        (15, 217) => Some(ExtOpcode {
            category: 15,
            index: 217,
            name: None,
        }),
        (15, 218) => Some(ExtOpcode {
            category: 15,
            index: 218,
            name: None,
        }),
        (15, 219) => Some(ExtOpcode {
            category: 15,
            index: 219,
            name: None,
        }),
        (15, 220) => Some(ExtOpcode {
            category: 15,
            index: 220,
            name: Some("history_set_rect"),
        }),
        (15, 221) => Some(ExtOpcode {
            category: 15,
            index: 221,
            name: Some("history_clear"),
        }),
        (15, 222) => Some(ExtOpcode {
            category: 15,
            index: 222,
            name: Some("history_set"),
        }),
        (15, 223) => Some(ExtOpcode {
            category: 15,
            index: 223,
            name: None,
        }),
        (15, 224) => Some(ExtOpcode {
            category: 15,
            index: 224,
            name: None,
        }),
        (15, 225) => Some(ExtOpcode {
            category: 15,
            index: 225,
            name: None,
        }),
        (15, 226) => Some(ExtOpcode {
            category: 15,
            index: 226,
            name: None,
        }),
        (15, 227) => Some(ExtOpcode {
            category: 15,
            index: 227,
            name: Some("history_set_face_call"),
        }),
        (15, 228) => Some(ExtOpcode {
            category: 15,
            index: 228,
            name: Some("history_set_face_sound"),
        }),
        (15, 229) => Some(ExtOpcode {
            category: 15,
            index: 229,
            name: Some("history_set_face_sound_release"),
        }),
        (15, 230) => Some(ExtOpcode {
            category: 15,
            index: 230,
            name: Some("history_get_text"),
        }),
        (15, 231) => Some(ExtOpcode {
            category: 15,
            index: 231,
            name: None,
        }),
        (15, 232) => Some(ExtOpcode {
            category: 15,
            index: 232,
            name: None,
        }),
        (15, 233) => Some(ExtOpcode {
            category: 15,
            index: 233,
            name: None,
        }),
        (15, 234) => Some(ExtOpcode {
            category: 15,
            index: 234,
            name: None,
        }),
        (15, 236) => Some(ExtOpcode {
            category: 15,
            index: 236,
            name: Some("movie_play"),
        }),
        (15, 237) => Some(ExtOpcode {
            category: 15,
            index: 237,
            name: Some("msp_set_loop_sp_ep"),
        }),
        (15, 238) => Some(ExtOpcode {
            category: 15,
            index: 238,
            name: Some("msp_cls"),
        }),
        (15, 239) => Some(ExtOpcode {
            category: 15,
            index: 239,
            name: Some("msp_wait"),
        }),
        (15, 240) => Some(ExtOpcode {
            category: 15,
            index: 240,
            name: Some("msp_lock"),
        }),
        (15, 241) => Some(ExtOpcode {
            category: 15,
            index: 241,
            name: Some("msp_unlock"),
        }),
        (15, 242) => Some(ExtOpcode {
            category: 15,
            index: 242,
            name: Some("msp_play"),
        }),
        (15, 243) => Some(ExtOpcode {
            category: 15,
            index: 243,
            name: Some("msp_stop"),
        }),
        (15, 245) => Some(ExtOpcode {
            category: 15,
            index: 245,
            name: Some("create_thread"),
        }),
        (15, 246) => Some(ExtOpcode {
            category: 15,
            index: 246,
            name: Some("exit_thread"),
        }),
        (15, 247) => Some(ExtOpcode {
            category: 15,
            index: 247,
            name: None,
        }),
        (15, 248) => Some(ExtOpcode {
            category: 15,
            index: 248,
            name: Some("get_thread"),
        }),
        (15, 251) => Some(ExtOpcode {
            category: 15,
            index: 251,
            name: Some("mov"),
        }),
        (15, 252) => Some(ExtOpcode {
            category: 15,
            index: 252,
            name: Some("add"),
        }),
        (15, 253) => Some(ExtOpcode {
            category: 15,
            index: 253,
            name: Some("sub"),
        }),
        (15, 254) => Some(ExtOpcode {
            category: 15,
            index: 254,
            name: Some("mul"),
        }),
        (15, 255) => Some(ExtOpcode {
            category: 15,
            index: 255,
            name: Some("div"),
        }),
        (15, 256) => Some(ExtOpcode {
            category: 15,
            index: 256,
            name: Some("bitand"),
        }),
        (15, 257) => Some(ExtOpcode {
            category: 15,
            index: 257,
            name: Some("bitor"),
        }),
        (15, 258) => Some(ExtOpcode {
            category: 15,
            index: 258,
            name: Some("bitxor"),
        }),
        (15, 259) => Some(ExtOpcode {
            category: 15,
            index: 259,
            name: Some("jmp_point"),
        }),
        (15, 260) => Some(ExtOpcode {
            category: 15,
            index: 260,
            name: Some("jf_point"),
        }),
        (15, 261) => Some(ExtOpcode {
            category: 15,
            index: 261,
            name: Some("gosub_point"),
        }),
        (15, 262) => Some(ExtOpcode {
            category: 15,
            index: 262,
            name: Some("eq"),
        }),
        (15, 263) => Some(ExtOpcode {
            category: 15,
            index: 263,
            name: Some("ne"),
        }),
        (15, 264) => Some(ExtOpcode {
            category: 15,
            index: 264,
            name: Some("le"),
        }),
        (15, 265) => Some(ExtOpcode {
            category: 15,
            index: 265,
            name: Some("ge"),
        }),
        (15, 266) => Some(ExtOpcode {
            category: 15,
            index: 266,
            name: Some("lt"),
        }),
        (15, 267) => Some(ExtOpcode {
            category: 15,
            index: 267,
            name: Some("gt"),
        }),
        (15, 268) => Some(ExtOpcode {
            category: 15,
            index: 268,
            name: Some("lor"),
        }),
        (15, 269) => Some(ExtOpcode {
            category: 15,
            index: 269,
            name: Some("land"),
        }),
        (15, 270) => Some(ExtOpcode {
            category: 15,
            index: 270,
            name: Some("lnot_slot"),
        }),
        (15, 271) => Some(ExtOpcode {
            category: 15,
            index: 271,
            name: Some("end"),
        }),
        (15, 272) => Some(ExtOpcode {
            category: 15,
            index: 272,
            name: Some("nop"),
        }),
        (15, 273) => Some(ExtOpcode {
            category: 15,
            index: 273,
            name: Some("extcall"),
        }),
        (15, 274) => Some(ExtOpcode {
            category: 15,
            index: 274,
            name: Some("ret"),
        }),
        (15, 275) => Some(ExtOpcode {
            category: 15,
            index: 275,
            name: Some("reset_adv"),
        }),
        (15, 276) => Some(ExtOpcode {
            category: 15,
            index: 276,
            name: Some("mod"),
        }),
        (15, 277) => Some(ExtOpcode {
            category: 15,
            index: 277,
            name: Some("shl"),
        }),
        (15, 278) => Some(ExtOpcode {
            category: 15,
            index: 278,
            name: Some("shr"),
        }),
        (15, 279) => Some(ExtOpcode {
            category: 15,
            index: 279,
            name: Some("neg_slot"),
        }),
        (15, 280) => Some(ExtOpcode {
            category: 15,
            index: 280,
            name: Some("pop"),
        }),
        (15, 281) => Some(ExtOpcode {
            category: 15,
            index: 281,
            name: Some("push"),
        }),
        (15, 282) => Some(ExtOpcode {
            category: 15,
            index: 282,
            name: Some("pack_args"),
        }),
        (15, 283) => Some(ExtOpcode {
            category: 15,
            index: 283,
            name: Some("drop_args"),
        }),
        (15, 285) => Some(ExtOpcode {
            category: 15,
            index: 285,
            name: Some("create_message"),
        }),
        (15, 286) => Some(ExtOpcode {
            category: 15,
            index: 286,
            name: Some("get_message"),
        }),
        (15, 287) => Some(ExtOpcode {
            category: 15,
            index: 287,
            name: Some("get_message_param"),
        }),
        (15, 290) => Some(ExtOpcode {
            category: 15,
            index: 290,
            name: Some("save"),
        }),
        (15, 291) => Some(ExtOpcode {
            category: 15,
            index: 291,
            name: Some("load"),
        }),
        (15, 292) => Some(ExtOpcode {
            category: 15,
            index: 292,
            name: Some("save_set_title"),
        }),
        (15, 293) => Some(ExtOpcode {
            category: 15,
            index: 293,
            name: Some("save_data"),
        }),
        (15, 294) => Some(ExtOpcode {
            category: 15,
            index: 294,
            name: Some("save_set_thumbnail_size"),
        }),
        (15, 295) => Some(ExtOpcode {
            category: 15,
            index: 295,
            name: Some("thumbnail_set"),
        }),
        (15, 296) => Some(ExtOpcode {
            category: 15,
            index: 296,
            name: Some("savetitledraw"),
        }),
        (15, 297) => Some(ExtOpcode {
            category: 15,
            index: 297,
            name: Some("save_set_font_size"),
        }),
        (15, 298) => Some(ExtOpcode {
            category: 15,
            index: 298,
            name: Some("getsaveday"),
        }),
        (15, 299) => Some(ExtOpcode {
            category: 15,
            index: 299,
            name: Some("is_save"),
        }),
        (16, 0) => Some(ExtOpcode {
            category: 16,
            index: 0,
            name: Some("effect_stop_skip"),
        }),
        (16, 1) => Some(ExtOpcode {
            category: 16,
            index: 1,
            name: None,
        }),
        (16, 2) => Some(ExtOpcode {
            category: 16,
            index: 2,
            name: None,
        }),
        (16, 3) => Some(ExtOpcode {
            category: 16,
            index: 3,
            name: None,
        }),
        (16, 4) => Some(ExtOpcode {
            category: 16,
            index: 4,
            name: None,
        }),
        (16, 5) => Some(ExtOpcode {
            category: 16,
            index: 5,
            name: None,
        }),
        (16, 7) => Some(ExtOpcode {
            category: 16,
            index: 7,
            name: Some("bgm_play"),
        }),
        (16, 8) => Some(ExtOpcode {
            category: 16,
            index: 8,
            name: Some("bgm_stop"),
        }),
        (16, 9) => Some(ExtOpcode {
            category: 16,
            index: 9,
            name: Some("bgm_set_volume"),
        }),
        (16, 10) => Some(ExtOpcode {
            category: 16,
            index: 10,
            name: Some("bgm_get_volume"),
        }),
        (16, 11) => Some(ExtOpcode {
            category: 16,
            index: 11,
            name: Some("bgm_get_auto_volume"),
        }),
        (16, 12) => Some(ExtOpcode {
            category: 16,
            index: 12,
            name: Some("bgm_set_volume_users"),
        }),
        (16, 13) => Some(ExtOpcode {
            category: 16,
            index: 13,
            name: Some("bgm_set_auto_volume"),
        }),
        (16, 14) => Some(ExtOpcode {
            category: 16,
            index: 14,
            name: Some("bgm_pause"),
        }),
        (16, 15) => Some(ExtOpcode {
            category: 16,
            index: 15,
            name: Some("get_bgm_filename"),
        }),
        (16, 16) => Some(ExtOpcode {
            category: 16,
            index: 16,
            name: Some("bgm_load"),
        }),
        (16, 17) => Some(ExtOpcode {
            category: 16,
            index: 17,
            name: Some("bgm_play2"),
        }),
        (16, 18) => Some(ExtOpcode {
            category: 16,
            index: 18,
            name: Some("set_master_volume"),
        }),
        (16, 19) => Some(ExtOpcode {
            category: 16,
            index: 19,
            name: Some("get_master_volume"),
        }),
        (16, 20) => Some(ExtOpcode {
            category: 16,
            index: 20,
            name: Some("mute_master_volume"),
        }),
        (16, 21) => Some(ExtOpcode {
            category: 16,
            index: 21,
            name: Some("bgm_mute"),
        }),
        (16, 22) => Some(ExtOpcode {
            category: 16,
            index: 22,
            name: Some("mute_bgm_auto_volume"),
        }),
        (16, 23) => Some(ExtOpcode {
            category: 16,
            index: 23,
            name: Some("1_get_bgm_status"),
        }),
        (16, 24) => Some(ExtOpcode {
            category: 16,
            index: 24,
            name: Some("1_get_bgm_pos"),
        }),
        (16, 25) => Some(ExtOpcode {
            category: 16,
            index: 25,
            name: Some("get_bgm_ch"),
        }),
        (16, 27) => Some(ExtOpcode {
            category: 16,
            index: 27,
            name: Some("btn_init"),
        }),
        (16, 28) => Some(ExtOpcode {
            category: 16,
            index: 28,
            name: Some("btn_uninit"),
        }),
        (16, 30) => Some(ExtOpcode {
            category: 16,
            index: 30,
            name: Some("btn_set"),
        }),
        (16, 31) => Some(ExtOpcode {
            category: 16,
            index: 31,
            name: Some("btn_hide"),
        }),
        (16, 32) => Some(ExtOpcode {
            category: 16,
            index: 32,
            name: Some("btn_show"),
        }),
        (16, 33) => Some(ExtOpcode {
            category: 16,
            index: 33,
            name: Some("btn_set_pos"),
        }),
        (16, 34) => Some(ExtOpcode {
            category: 16,
            index: 34,
            name: Some("btn_set_rect"),
        }),
        (16, 35) => Some(ExtOpcode {
            category: 16,
            index: 35,
            name: Some("btn_release"),
        }),
        (16, 36) => Some(ExtOpcode {
            category: 16,
            index: 36,
            name: None,
        }),
        (16, 37) => Some(ExtOpcode {
            category: 16,
            index: 37,
            name: None,
        }),
        (16, 38) => Some(ExtOpcode {
            category: 16,
            index: 38,
            name: None,
        }),
        (16, 39) => Some(ExtOpcode {
            category: 16,
            index: 39,
            name: None,
        }),
        (16, 40) => Some(ExtOpcode {
            category: 16,
            index: 40,
            name: Some("btn_set_toggle"),
        }),
        (16, 41) => Some(ExtOpcode {
            category: 16,
            index: 41,
            name: None,
        }),
        (16, 42) => Some(ExtOpcode {
            category: 16,
            index: 42,
            name: Some("btn_enable"),
        }),
        (16, 43) => Some(ExtOpcode {
            category: 16,
            index: 43,
            name: Some("btn_set_alpha_0x"),
        }),
        (16, 44) => Some(ExtOpcode {
            category: 16,
            index: 44,
            name: Some("btn_get_push"),
        }),
        (16, 45) => Some(ExtOpcode {
            category: 16,
            index: 45,
            name: Some("error_btn_expansion"),
        }),
        (16, 46) => Some(ExtOpcode {
            category: 16,
            index: 46,
            name: Some("btn_lock"),
        }),
        (16, 47) => Some(ExtOpcode {
            category: 16,
            index: 47,
            name: Some("btn_unlock"),
        }),
        (16, 48) => Some(ExtOpcode {
            category: 16,
            index: 48,
            name: Some("btn_set_anim"),
        }),
        (16, 49) => Some(ExtOpcode {
            category: 16,
            index: 49,
            name: Some("btn_set_hit"),
        }),
        (16, 50) => Some(ExtOpcode {
            category: 16,
            index: 50,
            name: Some("btn_get_onmouse"),
        }),
        (16, 51) => Some(ExtOpcode {
            category: 16,
            index: 51,
            name: Some("btn_anim_clear"),
        }),
        (16, 52) => Some(ExtOpcode {
            category: 16,
            index: 52,
            name: Some("btn_get_offmouse"),
        }),
        (16, 53) => Some(ExtOpcode {
            category: 16,
            index: 53,
            name: Some("btn_onmouse_clear"),
        }),
        (16, 54) => Some(ExtOpcode {
            category: 16,
            index: 54,
            name: Some("btn_blt"),
        }),
        (16, 55) => Some(ExtOpcode {
            category: 16,
            index: 55,
            name: Some("btn_link"),
        }),
        (16, 56) => Some(ExtOpcode {
            category: 16,
            index: 56,
            name: Some("btn_set_state"),
        }),
        (16, 57) => Some(ExtOpcode {
            category: 16,
            index: 57,
            name: Some("btn_get_link"),
        }),
        (16, 58) => Some(ExtOpcode {
            category: 16,
            index: 58,
            name: Some("btn_set_tips"),
        }),
        (16, 59) => Some(ExtOpcode {
            category: 16,
            index: 59,
            name: Some("btn_get_tips"),
        }),
        (16, 60) => Some(ExtOpcode {
            category: 16,
            index: 60,
            name: Some("btn_anime_is_true"),
        }),
        (16, 61) => Some(ExtOpcode {
            category: 16,
            index: 61,
            name: Some("btn_anime_get_status"),
        }),
        (16, 62) => Some(ExtOpcode {
            category: 16,
            index: 62,
            name: Some("btn_anime_finish"),
        }),
        (16, 63) => Some(ExtOpcode {
            category: 16,
            index: 63,
            name: Some("btn_mode"),
        }),
        (16, 64) => Some(ExtOpcode {
            category: 16,
            index: 64,
            name: Some("btn_get_alpha_0x"),
        }),
        (16, 65) => Some(ExtOpcode {
            category: 16,
            index: 65,
            name: Some("btn_on_check_0x"),
        }),
        (16, 67) => Some(ExtOpcode {
            category: 16,
            index: 67,
            name: None,
        }),
        (16, 68) => Some(ExtOpcode {
            category: 16,
            index: 68,
            name: None,
        }),
        (16, 69) => Some(ExtOpcode {
            category: 16,
            index: 69,
            name: Some("set_window_text"),
        }),
        (16, 70) => Some(ExtOpcode {
            category: 16,
            index: 70,
            name: None,
        }),
        (16, 71) => Some(ExtOpcode {
            category: 16,
            index: 71,
            name: None,
        }),
        (16, 72) => Some(ExtOpcode {
            category: 16,
            index: 72,
            name: None,
        }),
        (16, 73) => Some(ExtOpcode {
            category: 16,
            index: 73,
            name: None,
        }),
        (16, 74) => Some(ExtOpcode {
            category: 16,
            index: 74,
            name: None,
        }),
        (16, 75) => Some(ExtOpcode {
            category: 16,
            index: 75,
            name: None,
        }),
        (16, 76) => Some(ExtOpcode {
            category: 16,
            index: 76,
            name: None,
        }),
        (16, 77) => Some(ExtOpcode {
            category: 16,
            index: 77,
            name: Some("debug_break"),
        }),
        (16, 80) => Some(ExtOpcode {
            category: 16,
            index: 80,
            name: Some("app_exec"),
        }),
        (16, 81) => Some(ExtOpcode {
            category: 16,
            index: 81,
            name: Some("is_playername"),
        }),
        (16, 82) => Some(ExtOpcode {
            category: 16,
            index: 82,
            name: None,
        }),
        (16, 83) => Some(ExtOpcode {
            category: 16,
            index: 83,
            name: None,
        }),
        (16, 84) => Some(ExtOpcode {
            category: 16,
            index: 84,
            name: None,
        }),
        (16, 85) => Some(ExtOpcode {
            category: 16,
            index: 85,
            name: None,
        }),
        (16, 86) => Some(ExtOpcode {
            category: 16,
            index: 86,
            name: None,
        }),
        (16, 87) => Some(ExtOpcode {
            category: 16,
            index: 87,
            name: Some("file_exist"),
        }),
        (16, 88) => Some(ExtOpcode {
            category: 16,
            index: 88,
            name: Some("wsprint"),
        }),
        (16, 89) => Some(ExtOpcode {
            category: 16,
            index: 89,
            name: Some("check_disc"),
        }),
        (16, 90) => Some(ExtOpcode {
            category: 16,
            index: 90,
            name: None,
        }),
        (16, 91) => Some(ExtOpcode {
            category: 16,
            index: 91,
            name: None,
        }),
        (16, 92) => Some(ExtOpcode {
            category: 16,
            index: 92,
            name: None,
        }),
        (16, 93) => Some(ExtOpcode {
            category: 16,
            index: 93,
            name: Some("update_access"),
        }),
        (16, 94) => Some(ExtOpcode {
            category: 16,
            index: 94,
            name: None,
        }),
        (16, 95) => Some(ExtOpcode {
            category: 16,
            index: 95,
            name: None,
        }),
        (16, 96) => Some(ExtOpcode {
            category: 16,
            index: 96,
            name: None,
        }),
        (16, 97) => Some(ExtOpcode {
            category: 16,
            index: 97,
            name: None,
        }),
        (16, 98) => Some(ExtOpcode {
            category: 16,
            index: 98,
            name: None,
        }),
        (16, 99) => Some(ExtOpcode {
            category: 16,
            index: 99,
            name: None,
        }),
        (16, 100) => Some(ExtOpcode {
            category: 16,
            index: 100,
            name: None,
        }),
        (16, 101) => Some(ExtOpcode {
            category: 16,
            index: 101,
            name: Some("player_name_set_begin"),
        }),
        (16, 102) => Some(ExtOpcode {
            category: 16,
            index: 102,
            name: Some("player_name_set_end"),
        }),
        (16, 103) => Some(ExtOpcode {
            category: 16,
            index: 103,
            name: Some("player_name_set_check"),
        }),
        (16, 104) => Some(ExtOpcode {
            category: 16,
            index: 104,
            name: None,
        }),
        (16, 105) => Some(ExtOpcode {
            category: 16,
            index: 105,
            name: Some("player_name_reset"),
        }),
        (16, 106) => Some(ExtOpcode {
            category: 16,
            index: 106,
            name: Some("player_name_set_direct"),
        }),
        (16, 107) => Some(ExtOpcode {
            category: 16,
            index: 107,
            name: None,
        }),
        (16, 108) => Some(ExtOpcode {
            category: 16,
            index: 108,
            name: None,
        }),
        (16, 109) => Some(ExtOpcode {
            category: 16,
            index: 109,
            name: Some("openfile"),
        }),
        (16, 110) => Some(ExtOpcode {
            category: 16,
            index: 110,
            name: Some("read_file"),
        }),
        (16, 111) => Some(ExtOpcode {
            category: 16,
            index: 111,
            name: Some("close_file_not_handle"),
        }),
        (16, 112) => Some(ExtOpcode {
            category: 16,
            index: 112,
            name: Some("set_file_pointer"),
        }),
        (16, 113) => Some(ExtOpcode {
            category: 16,
            index: 113,
            name: Some("file_string"),
        }),
        (16, 114) => Some(ExtOpcode {
            category: 16,
            index: 114,
            name: Some("set_last_process"),
        }),
        (16, 115) => Some(ExtOpcode {
            category: 16,
            index: 115,
            name: Some("sz_buf"),
        }),
        (16, 116) => Some(ExtOpcode {
            category: 16,
            index: 116,
            name: Some("getprivateprofileint"),
        }),
        (16, 117) => Some(ExtOpcode {
            category: 16,
            index: 117,
            name: None,
        }),
        (16, 118) => Some(ExtOpcode {
            category: 16,
            index: 118,
            name: None,
        }),
        (16, 119) => Some(ExtOpcode {
            category: 16,
            index: 119,
            name: None,
        }),
        (16, 120) => Some(ExtOpcode {
            category: 16,
            index: 120,
            name: Some("is_tweet"),
        }),
        (16, 121) => Some(ExtOpcode {
            category: 16,
            index: 121,
            name: None,
        }),
        (16, 122) => Some(ExtOpcode {
            category: 16,
            index: 122,
            name: None,
        }),
        (16, 123) => Some(ExtOpcode {
            category: 16,
            index: 123,
            name: None,
        }),
        (16, 124) => Some(ExtOpcode {
            category: 16,
            index: 124,
            name: None,
        }),
        (16, 125) => Some(ExtOpcode {
            category: 16,
            index: 125,
            name: None,
        }),
        (16, 126) => Some(ExtOpcode {
            category: 16,
            index: 126,
            name: None,
        }),
        (16, 127) => Some(ExtOpcode {
            category: 16,
            index: 127,
            name: None,
        }),
        (16, 128) => Some(ExtOpcode {
            category: 16,
            index: 128,
            name: None,
        }),
        (16, 129) => Some(ExtOpcode {
            category: 16,
            index: 129,
            name: None,
        }),
        (16, 130) => Some(ExtOpcode {
            category: 16,
            index: 130,
            name: None,
        }),
        (16, 131) => Some(ExtOpcode {
            category: 16,
            index: 131,
            name: None,
        }),
        (16, 132) => Some(ExtOpcode {
            category: 16,
            index: 132,
            name: None,
        }),
        (16, 133) => Some(ExtOpcode {
            category: 16,
            index: 133,
            name: None,
        }),
        (16, 134) => Some(ExtOpcode {
            category: 16,
            index: 134,
            name: None,
        }),
        (16, 135) => Some(ExtOpcode {
            category: 16,
            index: 135,
            name: None,
        }),
        (16, 136) => Some(ExtOpcode {
            category: 16,
            index: 136,
            name: None,
        }),
        (16, 137) => Some(ExtOpcode {
            category: 16,
            index: 137,
            name: None,
        }),
        (16, 138) => Some(ExtOpcode {
            category: 16,
            index: 138,
            name: None,
        }),
        (16, 139) => Some(ExtOpcode {
            category: 16,
            index: 139,
            name: None,
        }),
        (16, 140) => Some(ExtOpcode {
            category: 16,
            index: 140,
            name: None,
        }),
        (16, 141) => Some(ExtOpcode {
            category: 16,
            index: 141,
            name: None,
        }),
        (16, 142) => Some(ExtOpcode {
            category: 16,
            index: 142,
            name: None,
        }),
        (16, 143) => Some(ExtOpcode {
            category: 16,
            index: 143,
            name: None,
        }),
        (16, 144) => Some(ExtOpcode {
            category: 16,
            index: 144,
            name: None,
        }),
        (16, 145) => Some(ExtOpcode {
            category: 16,
            index: 145,
            name: None,
        }),
        (16, 146) => Some(ExtOpcode {
            category: 16,
            index: 146,
            name: None,
        }),
        (16, 147) => Some(ExtOpcode {
            category: 16,
            index: 147,
            name: None,
        }),
        (16, 148) => Some(ExtOpcode {
            category: 16,
            index: 148,
            name: None,
        }),
        (16, 149) => Some(ExtOpcode {
            category: 16,
            index: 149,
            name: None,
        }),
        (16, 150) => Some(ExtOpcode {
            category: 16,
            index: 150,
            name: None,
        }),
        (16, 151) => Some(ExtOpcode {
            category: 16,
            index: 151,
            name: None,
        }),
        (16, 152) => Some(ExtOpcode {
            category: 16,
            index: 152,
            name: None,
        }),
        (16, 153) => Some(ExtOpcode {
            category: 16,
            index: 153,
            name: Some("result_tweet"),
        }),
        (16, 154) => Some(ExtOpcode {
            category: 16,
            index: 154,
            name: Some("get_tweet_key"),
        }),
        (16, 155) => Some(ExtOpcode {
            category: 16,
            index: 155,
            name: Some("set_tweet_key"),
        }),
        (16, 156) => Some(ExtOpcode {
            category: 16,
            index: 156,
            name: None,
        }),
        (16, 157) => Some(ExtOpcode {
            category: 16,
            index: 157,
            name: Some("tweet_authorize"),
        }),
        (16, 158) => Some(ExtOpcode {
            category: 16,
            index: 158,
            name: None,
        }),
        (16, 159) => Some(ExtOpcode {
            category: 16,
            index: 159,
            name: None,
        }),
        (16, 160) => Some(ExtOpcode {
            category: 16,
            index: 160,
            name: None,
        }),
        (16, 161) => Some(ExtOpcode {
            category: 16,
            index: 161,
            name: Some("tips_csv_read_error"),
        }),
        (16, 162) => Some(ExtOpcode {
            category: 16,
            index: 162,
            name: Some("tips_csv_get_error"),
        }),
        (16, 163) => Some(ExtOpcode {
            category: 16,
            index: 163,
            name: Some("tips_csv_search_not_found"),
        }),
        (16, 164) => Some(ExtOpcode {
            category: 16,
            index: 164,
            name: None,
        }),
        (16, 165) => Some(ExtOpcode {
            category: 16,
            index: 165,
            name: None,
        }),
        (16, 166) => Some(ExtOpcode {
            category: 16,
            index: 166,
            name: Some("tips_acc_save"),
        }),
        (16, 167) => Some(ExtOpcode {
            category: 16,
            index: 167,
            name: Some("is_network"),
        }),
        (16, 168) => Some(ExtOpcode {
            category: 16,
            index: 168,
            name: Some("is_touch"),
        }),
        (16, 169) => Some(ExtOpcode {
            category: 16,
            index: 169,
            name: None,
        }),
        (16, 170) => Some(ExtOpcode {
            category: 16,
            index: 170,
            name: None,
        }),
        (16, 171) => Some(ExtOpcode {
            category: 16,
            index: 171,
            name: None,
        }),
        (16, 172) => Some(ExtOpcode {
            category: 16,
            index: 172,
            name: None,
        }),
        (16, 173) => Some(ExtOpcode {
            category: 16,
            index: 173,
            name: None,
        }),
        (16, 174) => Some(ExtOpcode {
            category: 16,
            index: 174,
            name: None,
        }),
        (16, 175) => Some(ExtOpcode {
            category: 16,
            index: 175,
            name: None,
        }),
        (16, 176) => Some(ExtOpcode {
            category: 16,
            index: 176,
            name: None,
        }),
        (16, 177) => Some(ExtOpcode {
            category: 16,
            index: 177,
            name: None,
        }),
        (16, 178) => Some(ExtOpcode {
            category: 16,
            index: 178,
            name: None,
        }),
        (16, 179) => Some(ExtOpcode {
            category: 16,
            index: 179,
            name: None,
        }),
        (16, 180) => Some(ExtOpcode {
            category: 16,
            index: 180,
            name: None,
        }),
        (16, 181) => Some(ExtOpcode {
            category: 16,
            index: 181,
            name: None,
        }),
        (16, 182) => Some(ExtOpcode {
            category: 16,
            index: 182,
            name: None,
        }),
        (16, 183) => Some(ExtOpcode {
            category: 16,
            index: 183,
            name: None,
        }),
        (16, 184) => Some(ExtOpcode {
            category: 16,
            index: 184,
            name: None,
        }),
        (16, 185) => Some(ExtOpcode {
            category: 16,
            index: 185,
            name: None,
        }),
        (16, 186) => Some(ExtOpcode {
            category: 16,
            index: 186,
            name: None,
        }),
        (16, 187) => Some(ExtOpcode {
            category: 16,
            index: 187,
            name: None,
        }),
        (16, 188) => Some(ExtOpcode {
            category: 16,
            index: 188,
            name: None,
        }),
        (16, 189) => Some(ExtOpcode {
            category: 16,
            index: 189,
            name: None,
        }),
        (16, 190) => Some(ExtOpcode {
            category: 16,
            index: 190,
            name: None,
        }),
        (16, 191) => Some(ExtOpcode {
            category: 16,
            index: 191,
            name: None,
        }),
        (16, 193) => Some(ExtOpcode {
            category: 16,
            index: 193,
            name: None,
        }),
        (16, 194) => Some(ExtOpcode {
            category: 16,
            index: 194,
            name: Some("run_no_wait"),
        }),
        (16, 195) => Some(ExtOpcode {
            category: 16,
            index: 195,
            name: Some("run_stack"),
        }),
        (16, 197) => Some(ExtOpcode {
            category: 16,
            index: 197,
            name: Some("fx_effect_cls"),
        }),
        (16, 198) => Some(ExtOpcode {
            category: 16,
            index: 198,
            name: Some("fx_raster_stop"),
        }),
        (16, 199) => Some(ExtOpcode {
            category: 16,
            index: 199,
            name: Some("fx_effect_wait"),
        }),
        (16, 200) => Some(ExtOpcode {
            category: 16,
            index: 200,
            name: None,
        }),
        (16, 202) => Some(ExtOpcode {
            category: 16,
            index: 202,
            name: Some("random"),
        }),
        (16, 203) => Some(ExtOpcode {
            category: 16,
            index: 203,
            name: Some("abs"),
        }),
        (16, 204) => Some(ExtOpcode {
            category: 16,
            index: 204,
            name: Some("sin"),
        }),
        (16, 205) => Some(ExtOpcode {
            category: 16,
            index: 205,
            name: Some("cos"),
        }),
        (16, 206) => Some(ExtOpcode {
            category: 16,
            index: 206,
            name: Some("tan"),
        }),
        (16, 207) => Some(ExtOpcode {
            category: 16,
            index: 207,
            name: Some("atan"),
        }),
        (16, 208) => Some(ExtOpcode {
            category: 16,
            index: 208,
            name: Some("log"),
        }),
        (16, 209) => Some(ExtOpcode {
            category: 16,
            index: 209,
            name: Some("log10"),
        }),
        (16, 210) => Some(ExtOpcode {
            category: 16,
            index: 210,
            name: None,
        }),
        (16, 211) => Some(ExtOpcode {
            category: 16,
            index: 211,
            name: Some("sqrt"),
        }),
        (16, 212) => Some(ExtOpcode {
            category: 16,
            index: 212,
            name: None,
        }),
        (16, 213) => Some(ExtOpcode {
            category: 16,
            index: 213,
            name: None,
        }),
        (16, 217) => Some(ExtOpcode {
            category: 16,
            index: 217,
            name: Some("sp_set"),
        }),
        (16, 218) => Some(ExtOpcode {
            category: 16,
            index: 218,
            name: Some("sp_set_ex"),
        }),
        (16, 219) => Some(ExtOpcode {
            category: 16,
            index: 219,
            name: Some("sp_set_pos"),
        }),
        (16, 220) => Some(ExtOpcode {
            category: 16,
            index: 220,
            name: Some("sp_cls"),
        }),
        (16, 221) => Some(ExtOpcode {
            category: 16,
            index: 221,
            name: Some("sp_set_alpha"),
        }),
        (16, 222) => Some(ExtOpcode {
            category: 16,
            index: 222,
            name: Some("set_priority"),
        }),
        (16, 223) => Some(ExtOpcode {
            category: 16,
            index: 223,
            name: None,
        }),
        (16, 224) => Some(ExtOpcode {
            category: 16,
            index: 224,
            name: Some("sp_set_center"),
        }),
        (16, 226) => Some(ExtOpcode {
            category: 16,
            index: 226,
            name: Some("sp_cls_ex"),
        }),
        (16, 227) => Some(ExtOpcode {
            category: 16,
            index: 227,
            name: Some("set_filter"),
        }),
        (16, 228) => Some(ExtOpcode {
            category: 16,
            index: 228,
            name: Some("sp_cls_transition"),
        }),
        (16, 229) => Some(ExtOpcode {
            category: 16,
            index: 229,
            name: Some("sp_set_pos_ex"),
        }),
        (16, 230) => Some(ExtOpcode {
            category: 16,
            index: 230,
            name: Some("sp_set_rect_pos"),
        }),
        (16, 231) => Some(ExtOpcode {
            category: 16,
            index: 231,
            name: None,
        }),
        (16, 232) => Some(ExtOpcode {
            category: 16,
            index: 232,
            name: Some("sp_set_scale"),
        }),
        (16, 233) => Some(ExtOpcode {
            category: 16,
            index: 233,
            name: Some("sp_set_rotate"),
        }),
        (16, 234) => Some(ExtOpcode {
            category: 16,
            index: 234,
            name: Some("face_init"),
        }),
        (16, 235) => Some(ExtOpcode {
            category: 16,
            index: 235,
            name: Some("face_set"),
        }),
        (16, 236) => Some(ExtOpcode {
            category: 16,
            index: 236,
            name: Some("not_image_sp_get_color"),
        }),
        (16, 237) => Some(ExtOpcode {
            category: 16,
            index: 237,
            name: Some("sptext"),
        }),
        (16, 238) => Some(ExtOpcode {
            category: 16,
            index: 238,
            name: Some("face_cls"),
        }),
        (16, 239) => Some(ExtOpcode {
            category: 16,
            index: 239,
            name: Some("sp_set_rect"),
        }),
        (16, 240) => Some(ExtOpcode {
            category: 16,
            index: 240,
            name: Some("sp_set_pos_move"),
        }),
        (16, 241) => Some(ExtOpcode {
            category: 16,
            index: 241,
            name: Some("not_image_sp_get_alpha"),
        }),
        (16, 242) => Some(ExtOpcode {
            category: 16,
            index: 242,
            name: Some("not_image_sp_get_rotate"),
        }),
        (16, 243) => Some(ExtOpcode {
            category: 16,
            index: 243,
            name: None,
        }),
        (16, 244) => Some(ExtOpcode {
            category: 16,
            index: 244,
            name: None,
        }),
        (16, 245) => Some(ExtOpcode {
            category: 16,
            index: 245,
            name: None,
        }),
        (16, 246) => Some(ExtOpcode {
            category: 16,
            index: 246,
            name: None,
        }),
        (16, 247) => Some(ExtOpcode {
            category: 16,
            index: 247,
            name: Some("sp_create"),
        }),
        (16, 248) => Some(ExtOpcode {
            category: 16,
            index: 248,
            name: Some("sp_anime_clear"),
        }),
        (16, 249) => Some(ExtOpcode {
            category: 16,
            index: 249,
            name: None,
        }),
        (16, 250) => Some(ExtOpcode {
            category: 16,
            index: 250,
            name: None,
        }),
        (16, 251) => Some(ExtOpcode {
            category: 16,
            index: 251,
            name: Some("not_image_sp_get_scale"),
        }),
        (16, 252) => Some(ExtOpcode {
            category: 16,
            index: 252,
            name: Some("sp_set_color_0x"),
        }),
        (16, 253) => Some(ExtOpcode {
            category: 16,
            index: 253,
            name: Some("sp_bitblt"),
        }),
        (16, 254) => Some(ExtOpcode {
            category: 16,
            index: 254,
            name: Some("sp_set_shake"),
        }),
        (16, 255) => Some(ExtOpcode {
            category: 16,
            index: 255,
            name: Some("sp_paint"),
        }),
        (16, 256) => Some(ExtOpcode {
            category: 16,
            index: 256,
            name: None,
        }),
        (16, 257) => Some(ExtOpcode {
            category: 16,
            index: 257,
            name: Some("sp_load_wait_time"),
        }),
        (16, 258) => Some(ExtOpcode {
            category: 16,
            index: 258,
            name: Some("sp_draw"),
        }),
        (16, 259) => Some(ExtOpcode {
            category: 16,
            index: 259,
            name: None,
        }),
        (16, 260) => Some(ExtOpcode {
            category: 16,
            index: 260,
            name: Some("sp_unlock"),
        }),
        (16, 261) => Some(ExtOpcode {
            category: 16,
            index: 261,
            name: Some("sp_show"),
        }),
        (16, 262) => Some(ExtOpcode {
            category: 16,
            index: 262,
            name: Some("sp_hide"),
        }),
        (16, 263) => Some(ExtOpcode {
            category: 16,
            index: 263,
            name: None,
        }),
        (16, 264) => Some(ExtOpcode {
            category: 16,
            index: 264,
            name: Some("sp_set_child"),
        }),
        (16, 265) => Some(ExtOpcode {
            category: 16,
            index: 265,
            name: Some("sp_set_transition"),
        }),
        (16, 266) => Some(ExtOpcode {
            category: 16,
            index: 266,
            name: Some("sp_copy_image"),
        }),
        (16, 267) => Some(ExtOpcode {
            category: 16,
            index: 267,
            name: Some("sp_transition"),
        }),
        (16, 268) => Some(ExtOpcode {
            category: 16,
            index: 268,
            name: Some("set_aspect_position_type"),
        }),
        (16, 269) => Some(ExtOpcode {
            category: 16,
            index: 269,
            name: Some("get_backbuffer"),
        }),
        (16, 270) => Some(ExtOpcode {
            category: 16,
            index: 270,
            name: Some("sp_set_mask"),
        }),
        (16, 271) => Some(ExtOpcode {
            category: 16,
            index: 271,
            name: None,
        }),
        (16, 272) => Some(ExtOpcode {
            category: 16,
            index: 272,
            name: Some("spsetanime"),
        }),
        (16, 273) => Some(ExtOpcode {
            category: 16,
            index: 273,
            name: Some("drawtext"),
        }),
        (16, 274) => Some(ExtOpcode {
            category: 16,
            index: 274,
            name: None,
        }),
        (16, 275) => Some(ExtOpcode {
            category: 16,
            index: 275,
            name: None,
        }),
        (16, 277) => Some(ExtOpcode {
            category: 16,
            index: 277,
            name: Some("history_init_0x_0x"),
        }),
        (16, 278) => Some(ExtOpcode {
            category: 16,
            index: 278,
            name: Some("historybegin_lpbyte_ptagdata_sztext"),
        }),
        (16, 279) => Some(ExtOpcode {
            category: 16,
            index: 279,
            name: Some("history_end"),
        }),
        (16, 280) => Some(ExtOpcode {
            category: 16,
            index: 280,
            name: None,
        }),
        (16, 281) => Some(ExtOpcode {
            category: 16,
            index: 281,
            name: None,
        }),
        (16, 282) => Some(ExtOpcode {
            category: 16,
            index: 282,
            name: Some("history_get_height"),
        }),
        (16, 283) => Some(ExtOpcode {
            category: 16,
            index: 283,
            name: None,
        }),
        (16, 284) => Some(ExtOpcode {
            category: 16,
            index: 284,
            name: None,
        }),
        (16, 285) => Some(ExtOpcode {
            category: 16,
            index: 285,
            name: None,
        }),
        (16, 286) => Some(ExtOpcode {
            category: 16,
            index: 286,
            name: None,
        }),
        (16, 287) => Some(ExtOpcode {
            category: 16,
            index: 287,
            name: Some("history_set_rect"),
        }),
        (16, 288) => Some(ExtOpcode {
            category: 16,
            index: 288,
            name: Some("history_clear"),
        }),
        (16, 289) => Some(ExtOpcode {
            category: 16,
            index: 289,
            name: Some("history_set"),
        }),
        (16, 290) => Some(ExtOpcode {
            category: 16,
            index: 290,
            name: None,
        }),
        (16, 291) => Some(ExtOpcode {
            category: 16,
            index: 291,
            name: None,
        }),
        (16, 292) => Some(ExtOpcode {
            category: 16,
            index: 292,
            name: None,
        }),
        (16, 293) => Some(ExtOpcode {
            category: 16,
            index: 293,
            name: None,
        }),
        (16, 294) => Some(ExtOpcode {
            category: 16,
            index: 294,
            name: Some("history_set_face_call"),
        }),
        (16, 295) => Some(ExtOpcode {
            category: 16,
            index: 295,
            name: Some("history_set_face_sound"),
        }),
        (16, 296) => Some(ExtOpcode {
            category: 16,
            index: 296,
            name: Some("history_set_face_sound_release"),
        }),
        (16, 297) => Some(ExtOpcode {
            category: 16,
            index: 297,
            name: Some("history_get_text"),
        }),
        (16, 298) => Some(ExtOpcode {
            category: 16,
            index: 298,
            name: None,
        }),
        (16, 299) => Some(ExtOpcode {
            category: 16,
            index: 299,
            name: None,
        }),
        (17, 0) => Some(ExtOpcode {
            category: 17,
            index: 0,
            name: Some("action_run_count_over"),
        }),
        (17, 1) => Some(ExtOpcode {
            category: 17,
            index: 1,
            name: Some("action_sync_run_count_over"),
        }),
        (17, 2) => Some(ExtOpcode {
            category: 17,
            index: 2,
            name: Some("action_loop_run_count_over"),
        }),
        (17, 3) => Some(ExtOpcode {
            category: 17,
            index: 3,
            name: Some("action_clear_count_over"),
        }),
        (17, 4) => Some(ExtOpcode {
            category: 17,
            index: 4,
            name: Some("action_sync_wait_count_over"),
        }),
        (17, 5) => Some(ExtOpcode {
            category: 17,
            index: 5,
            name: None,
        }),
        (17, 6) => Some(ExtOpcode {
            category: 17,
            index: 6,
            name: None,
        }),
        (17, 7) => Some(ExtOpcode {
            category: 17,
            index: 7,
            name: None,
        }),
        (17, 8) => Some(ExtOpcode {
            category: 17,
            index: 8,
            name: None,
        }),
        (17, 9) => Some(ExtOpcode {
            category: 17,
            index: 9,
            name: None,
        }),
        (17, 10) => Some(ExtOpcode {
            category: 17,
            index: 10,
            name: None,
        }),
        (17, 11) => Some(ExtOpcode {
            category: 17,
            index: 11,
            name: None,
        }),
        (17, 12) => Some(ExtOpcode {
            category: 17,
            index: 12,
            name: Some("action_push"),
        }),
        (17, 13) => Some(ExtOpcode {
            category: 17,
            index: 13,
            name: Some("action_pop"),
        }),
        (17, 14) => Some(ExtOpcode {
            category: 17,
            index: 14,
            name: None,
        }),
        (17, 15) => Some(ExtOpcode {
            category: 17,
            index: 15,
            name: None,
        }),
        (17, 16) => Some(ExtOpcode {
            category: 17,
            index: 16,
            name: None,
        }),
        (17, 17) => Some(ExtOpcode {
            category: 17,
            index: 17,
            name: None,
        }),
        (17, 18) => Some(ExtOpcode {
            category: 17,
            index: 18,
            name: None,
        }),
        (17, 19) => Some(ExtOpcode {
            category: 17,
            index: 19,
            name: None,
        }),
        (17, 20) => Some(ExtOpcode {
            category: 17,
            index: 20,
            name: None,
        }),
        (17, 21) => Some(ExtOpcode {
            category: 17,
            index: 21,
            name: None,
        }),
        (17, 23) => Some(ExtOpcode {
            category: 17,
            index: 23,
            name: Some("set_active_action"),
        }),
        (17, 24) => Some(ExtOpcode {
            category: 17,
            index: 24,
            name: Some("get_active_action"),
        }),
        (17, 25) => Some(ExtOpcode {
            category: 17,
            index: 25,
            name: Some("action_run_all"),
        }),
        (17, 26) => Some(ExtOpcode {
            category: 17,
            index: 26,
            name: Some("action_sync_run_all"),
        }),
        (17, 27) => Some(ExtOpcode {
            category: 17,
            index: 27,
            name: Some("action_loop_run_all"),
        }),
        (17, 28) => Some(ExtOpcode {
            category: 17,
            index: 28,
            name: Some("action_uninit"),
        }),
        (17, 29) => Some(ExtOpcode {
            category: 17,
            index: 29,
            name: None,
        }),
        (17, 30) => Some(ExtOpcode {
            category: 17,
            index: 30,
            name: Some("set_action_clear"),
        }),
        (17, 31) => Some(ExtOpcode {
            category: 17,
            index: 31,
            name: None,
        }),
        (17, 32) => Some(ExtOpcode {
            category: 17,
            index: 32,
            name: None,
        }),
        (17, 33) => Some(ExtOpcode {
            category: 17,
            index: 33,
            name: None,
        }),
        (17, 34) => Some(ExtOpcode {
            category: 17,
            index: 34,
            name: None,
        }),
        (17, 35) => Some(ExtOpcode {
            category: 17,
            index: 35,
            name: None,
        }),
        (17, 37) => Some(ExtOpcode {
            category: 17,
            index: 37,
            name: Some("effect_stop_skip"),
        }),
        (17, 38) => Some(ExtOpcode {
            category: 17,
            index: 38,
            name: None,
        }),
        (17, 39) => Some(ExtOpcode {
            category: 17,
            index: 39,
            name: None,
        }),
        (17, 40) => Some(ExtOpcode {
            category: 17,
            index: 40,
            name: None,
        }),
        (17, 41) => Some(ExtOpcode {
            category: 17,
            index: 41,
            name: None,
        }),
        (17, 42) => Some(ExtOpcode {
            category: 17,
            index: 42,
            name: None,
        }),
        (17, 44) => Some(ExtOpcode {
            category: 17,
            index: 44,
            name: Some("bgm_play"),
        }),
        (17, 45) => Some(ExtOpcode {
            category: 17,
            index: 45,
            name: Some("bgm_stop"),
        }),
        (17, 46) => Some(ExtOpcode {
            category: 17,
            index: 46,
            name: Some("bgm_set_volume"),
        }),
        (17, 47) => Some(ExtOpcode {
            category: 17,
            index: 47,
            name: Some("bgm_get_volume"),
        }),
        (17, 48) => Some(ExtOpcode {
            category: 17,
            index: 48,
            name: Some("bgm_get_auto_volume"),
        }),
        (17, 49) => Some(ExtOpcode {
            category: 17,
            index: 49,
            name: Some("bgm_set_volume_users"),
        }),
        (17, 50) => Some(ExtOpcode {
            category: 17,
            index: 50,
            name: Some("bgm_set_auto_volume"),
        }),
        (17, 51) => Some(ExtOpcode {
            category: 17,
            index: 51,
            name: Some("bgm_pause"),
        }),
        (17, 52) => Some(ExtOpcode {
            category: 17,
            index: 52,
            name: Some("get_bgm_filename"),
        }),
        (17, 53) => Some(ExtOpcode {
            category: 17,
            index: 53,
            name: Some("bgm_load"),
        }),
        (17, 54) => Some(ExtOpcode {
            category: 17,
            index: 54,
            name: Some("bgm_play2"),
        }),
        (17, 55) => Some(ExtOpcode {
            category: 17,
            index: 55,
            name: Some("set_master_volume"),
        }),
        (17, 56) => Some(ExtOpcode {
            category: 17,
            index: 56,
            name: Some("get_master_volume"),
        }),
        (17, 57) => Some(ExtOpcode {
            category: 17,
            index: 57,
            name: Some("mute_master_volume"),
        }),
        (17, 58) => Some(ExtOpcode {
            category: 17,
            index: 58,
            name: Some("bgm_mute"),
        }),
        (17, 59) => Some(ExtOpcode {
            category: 17,
            index: 59,
            name: Some("mute_bgm_auto_volume"),
        }),
        (17, 60) => Some(ExtOpcode {
            category: 17,
            index: 60,
            name: Some("1_get_bgm_status"),
        }),
        (17, 61) => Some(ExtOpcode {
            category: 17,
            index: 61,
            name: Some("1_get_bgm_pos"),
        }),
        (17, 62) => Some(ExtOpcode {
            category: 17,
            index: 62,
            name: Some("get_bgm_ch"),
        }),
        (17, 64) => Some(ExtOpcode {
            category: 17,
            index: 64,
            name: Some("btn_init"),
        }),
        (17, 65) => Some(ExtOpcode {
            category: 17,
            index: 65,
            name: Some("btn_uninit"),
        }),
        (17, 67) => Some(ExtOpcode {
            category: 17,
            index: 67,
            name: Some("btn_set"),
        }),
        (17, 68) => Some(ExtOpcode {
            category: 17,
            index: 68,
            name: Some("btn_hide"),
        }),
        (17, 69) => Some(ExtOpcode {
            category: 17,
            index: 69,
            name: Some("btn_show"),
        }),
        (17, 70) => Some(ExtOpcode {
            category: 17,
            index: 70,
            name: Some("btn_set_pos"),
        }),
        (17, 71) => Some(ExtOpcode {
            category: 17,
            index: 71,
            name: Some("btn_set_rect"),
        }),
        (17, 72) => Some(ExtOpcode {
            category: 17,
            index: 72,
            name: Some("btn_release"),
        }),
        (17, 73) => Some(ExtOpcode {
            category: 17,
            index: 73,
            name: None,
        }),
        (17, 74) => Some(ExtOpcode {
            category: 17,
            index: 74,
            name: None,
        }),
        (17, 75) => Some(ExtOpcode {
            category: 17,
            index: 75,
            name: None,
        }),
        (17, 76) => Some(ExtOpcode {
            category: 17,
            index: 76,
            name: None,
        }),
        (17, 77) => Some(ExtOpcode {
            category: 17,
            index: 77,
            name: Some("btn_set_toggle"),
        }),
        (17, 78) => Some(ExtOpcode {
            category: 17,
            index: 78,
            name: None,
        }),
        (17, 79) => Some(ExtOpcode {
            category: 17,
            index: 79,
            name: Some("btn_enable"),
        }),
        (17, 80) => Some(ExtOpcode {
            category: 17,
            index: 80,
            name: Some("btn_set_alpha_0x"),
        }),
        (17, 81) => Some(ExtOpcode {
            category: 17,
            index: 81,
            name: Some("btn_get_push"),
        }),
        (17, 82) => Some(ExtOpcode {
            category: 17,
            index: 82,
            name: Some("error_btn_expansion"),
        }),
        (17, 83) => Some(ExtOpcode {
            category: 17,
            index: 83,
            name: Some("btn_lock"),
        }),
        (17, 84) => Some(ExtOpcode {
            category: 17,
            index: 84,
            name: Some("btn_unlock"),
        }),
        (17, 85) => Some(ExtOpcode {
            category: 17,
            index: 85,
            name: Some("btn_set_anim"),
        }),
        (17, 86) => Some(ExtOpcode {
            category: 17,
            index: 86,
            name: Some("btn_set_hit"),
        }),
        (17, 87) => Some(ExtOpcode {
            category: 17,
            index: 87,
            name: Some("btn_get_onmouse"),
        }),
        (17, 88) => Some(ExtOpcode {
            category: 17,
            index: 88,
            name: Some("btn_anim_clear"),
        }),
        (17, 89) => Some(ExtOpcode {
            category: 17,
            index: 89,
            name: Some("btn_get_offmouse"),
        }),
        (17, 90) => Some(ExtOpcode {
            category: 17,
            index: 90,
            name: Some("btn_onmouse_clear"),
        }),
        (17, 91) => Some(ExtOpcode {
            category: 17,
            index: 91,
            name: Some("btn_blt"),
        }),
        (17, 92) => Some(ExtOpcode {
            category: 17,
            index: 92,
            name: Some("btn_link"),
        }),
        (17, 93) => Some(ExtOpcode {
            category: 17,
            index: 93,
            name: Some("btn_set_state"),
        }),
        (17, 94) => Some(ExtOpcode {
            category: 17,
            index: 94,
            name: Some("btn_get_link"),
        }),
        (17, 95) => Some(ExtOpcode {
            category: 17,
            index: 95,
            name: Some("btn_set_tips"),
        }),
        (17, 96) => Some(ExtOpcode {
            category: 17,
            index: 96,
            name: Some("btn_get_tips"),
        }),
        (17, 97) => Some(ExtOpcode {
            category: 17,
            index: 97,
            name: Some("btn_anime_is_true"),
        }),
        (17, 98) => Some(ExtOpcode {
            category: 17,
            index: 98,
            name: Some("btn_anime_get_status"),
        }),
        (17, 99) => Some(ExtOpcode {
            category: 17,
            index: 99,
            name: Some("btn_anime_finish"),
        }),
        (17, 100) => Some(ExtOpcode {
            category: 17,
            index: 100,
            name: Some("btn_mode"),
        }),
        (17, 101) => Some(ExtOpcode {
            category: 17,
            index: 101,
            name: Some("btn_get_alpha_0x"),
        }),
        (17, 102) => Some(ExtOpcode {
            category: 17,
            index: 102,
            name: Some("btn_on_check_0x"),
        }),
        (17, 104) => Some(ExtOpcode {
            category: 17,
            index: 104,
            name: None,
        }),
        (17, 105) => Some(ExtOpcode {
            category: 17,
            index: 105,
            name: None,
        }),
        (17, 106) => Some(ExtOpcode {
            category: 17,
            index: 106,
            name: Some("set_window_text"),
        }),
        (17, 107) => Some(ExtOpcode {
            category: 17,
            index: 107,
            name: None,
        }),
        (17, 108) => Some(ExtOpcode {
            category: 17,
            index: 108,
            name: None,
        }),
        (17, 109) => Some(ExtOpcode {
            category: 17,
            index: 109,
            name: None,
        }),
        (17, 110) => Some(ExtOpcode {
            category: 17,
            index: 110,
            name: None,
        }),
        (17, 111) => Some(ExtOpcode {
            category: 17,
            index: 111,
            name: None,
        }),
        (17, 112) => Some(ExtOpcode {
            category: 17,
            index: 112,
            name: None,
        }),
        (17, 113) => Some(ExtOpcode {
            category: 17,
            index: 113,
            name: None,
        }),
        (17, 114) => Some(ExtOpcode {
            category: 17,
            index: 114,
            name: Some("debug_break"),
        }),
        (17, 117) => Some(ExtOpcode {
            category: 17,
            index: 117,
            name: Some("app_exec"),
        }),
        (17, 118) => Some(ExtOpcode {
            category: 17,
            index: 118,
            name: Some("is_playername"),
        }),
        (17, 119) => Some(ExtOpcode {
            category: 17,
            index: 119,
            name: None,
        }),
        (17, 120) => Some(ExtOpcode {
            category: 17,
            index: 120,
            name: None,
        }),
        (17, 121) => Some(ExtOpcode {
            category: 17,
            index: 121,
            name: None,
        }),
        (17, 122) => Some(ExtOpcode {
            category: 17,
            index: 122,
            name: None,
        }),
        (17, 123) => Some(ExtOpcode {
            category: 17,
            index: 123,
            name: None,
        }),
        (17, 124) => Some(ExtOpcode {
            category: 17,
            index: 124,
            name: Some("file_exist"),
        }),
        (17, 125) => Some(ExtOpcode {
            category: 17,
            index: 125,
            name: Some("wsprint"),
        }),
        (17, 126) => Some(ExtOpcode {
            category: 17,
            index: 126,
            name: Some("check_disc"),
        }),
        (17, 127) => Some(ExtOpcode {
            category: 17,
            index: 127,
            name: None,
        }),
        (17, 128) => Some(ExtOpcode {
            category: 17,
            index: 128,
            name: None,
        }),
        (17, 129) => Some(ExtOpcode {
            category: 17,
            index: 129,
            name: None,
        }),
        (17, 130) => Some(ExtOpcode {
            category: 17,
            index: 130,
            name: Some("update_access"),
        }),
        (17, 131) => Some(ExtOpcode {
            category: 17,
            index: 131,
            name: None,
        }),
        (17, 132) => Some(ExtOpcode {
            category: 17,
            index: 132,
            name: None,
        }),
        (17, 133) => Some(ExtOpcode {
            category: 17,
            index: 133,
            name: None,
        }),
        (17, 134) => Some(ExtOpcode {
            category: 17,
            index: 134,
            name: None,
        }),
        (17, 135) => Some(ExtOpcode {
            category: 17,
            index: 135,
            name: None,
        }),
        (17, 136) => Some(ExtOpcode {
            category: 17,
            index: 136,
            name: None,
        }),
        (17, 137) => Some(ExtOpcode {
            category: 17,
            index: 137,
            name: None,
        }),
        (17, 138) => Some(ExtOpcode {
            category: 17,
            index: 138,
            name: Some("player_name_set_begin"),
        }),
        (17, 139) => Some(ExtOpcode {
            category: 17,
            index: 139,
            name: Some("player_name_set_end"),
        }),
        (17, 140) => Some(ExtOpcode {
            category: 17,
            index: 140,
            name: Some("player_name_set_check"),
        }),
        (17, 141) => Some(ExtOpcode {
            category: 17,
            index: 141,
            name: None,
        }),
        (17, 142) => Some(ExtOpcode {
            category: 17,
            index: 142,
            name: Some("player_name_reset"),
        }),
        (17, 143) => Some(ExtOpcode {
            category: 17,
            index: 143,
            name: Some("player_name_set_direct"),
        }),
        (17, 144) => Some(ExtOpcode {
            category: 17,
            index: 144,
            name: None,
        }),
        (17, 145) => Some(ExtOpcode {
            category: 17,
            index: 145,
            name: None,
        }),
        (17, 146) => Some(ExtOpcode {
            category: 17,
            index: 146,
            name: Some("openfile"),
        }),
        (17, 147) => Some(ExtOpcode {
            category: 17,
            index: 147,
            name: Some("read_file"),
        }),
        (17, 148) => Some(ExtOpcode {
            category: 17,
            index: 148,
            name: Some("close_file_not_handle"),
        }),
        (17, 149) => Some(ExtOpcode {
            category: 17,
            index: 149,
            name: Some("set_file_pointer"),
        }),
        (17, 150) => Some(ExtOpcode {
            category: 17,
            index: 150,
            name: Some("file_string"),
        }),
        (17, 151) => Some(ExtOpcode {
            category: 17,
            index: 151,
            name: Some("set_last_process"),
        }),
        (17, 152) => Some(ExtOpcode {
            category: 17,
            index: 152,
            name: Some("sz_buf"),
        }),
        (17, 153) => Some(ExtOpcode {
            category: 17,
            index: 153,
            name: Some("getprivateprofileint"),
        }),
        (17, 154) => Some(ExtOpcode {
            category: 17,
            index: 154,
            name: None,
        }),
        (17, 155) => Some(ExtOpcode {
            category: 17,
            index: 155,
            name: None,
        }),
        (17, 156) => Some(ExtOpcode {
            category: 17,
            index: 156,
            name: None,
        }),
        (17, 157) => Some(ExtOpcode {
            category: 17,
            index: 157,
            name: Some("is_tweet"),
        }),
        (17, 158) => Some(ExtOpcode {
            category: 17,
            index: 158,
            name: None,
        }),
        (17, 159) => Some(ExtOpcode {
            category: 17,
            index: 159,
            name: None,
        }),
        (17, 160) => Some(ExtOpcode {
            category: 17,
            index: 160,
            name: None,
        }),
        (17, 161) => Some(ExtOpcode {
            category: 17,
            index: 161,
            name: None,
        }),
        (17, 162) => Some(ExtOpcode {
            category: 17,
            index: 162,
            name: None,
        }),
        (17, 163) => Some(ExtOpcode {
            category: 17,
            index: 163,
            name: None,
        }),
        (17, 164) => Some(ExtOpcode {
            category: 17,
            index: 164,
            name: None,
        }),
        (17, 165) => Some(ExtOpcode {
            category: 17,
            index: 165,
            name: None,
        }),
        (17, 166) => Some(ExtOpcode {
            category: 17,
            index: 166,
            name: None,
        }),
        (17, 167) => Some(ExtOpcode {
            category: 17,
            index: 167,
            name: None,
        }),
        (17, 168) => Some(ExtOpcode {
            category: 17,
            index: 168,
            name: None,
        }),
        (17, 169) => Some(ExtOpcode {
            category: 17,
            index: 169,
            name: None,
        }),
        (17, 170) => Some(ExtOpcode {
            category: 17,
            index: 170,
            name: None,
        }),
        (17, 171) => Some(ExtOpcode {
            category: 17,
            index: 171,
            name: None,
        }),
        (17, 172) => Some(ExtOpcode {
            category: 17,
            index: 172,
            name: None,
        }),
        (17, 173) => Some(ExtOpcode {
            category: 17,
            index: 173,
            name: None,
        }),
        (17, 174) => Some(ExtOpcode {
            category: 17,
            index: 174,
            name: None,
        }),
        (17, 175) => Some(ExtOpcode {
            category: 17,
            index: 175,
            name: None,
        }),
        (17, 176) => Some(ExtOpcode {
            category: 17,
            index: 176,
            name: None,
        }),
        (17, 177) => Some(ExtOpcode {
            category: 17,
            index: 177,
            name: None,
        }),
        (17, 178) => Some(ExtOpcode {
            category: 17,
            index: 178,
            name: None,
        }),
        (17, 179) => Some(ExtOpcode {
            category: 17,
            index: 179,
            name: None,
        }),
        (17, 180) => Some(ExtOpcode {
            category: 17,
            index: 180,
            name: None,
        }),
        (17, 181) => Some(ExtOpcode {
            category: 17,
            index: 181,
            name: None,
        }),
        (17, 182) => Some(ExtOpcode {
            category: 17,
            index: 182,
            name: None,
        }),
        (17, 183) => Some(ExtOpcode {
            category: 17,
            index: 183,
            name: None,
        }),
        (17, 184) => Some(ExtOpcode {
            category: 17,
            index: 184,
            name: None,
        }),
        (17, 185) => Some(ExtOpcode {
            category: 17,
            index: 185,
            name: None,
        }),
        (17, 186) => Some(ExtOpcode {
            category: 17,
            index: 186,
            name: None,
        }),
        (17, 187) => Some(ExtOpcode {
            category: 17,
            index: 187,
            name: None,
        }),
        (17, 188) => Some(ExtOpcode {
            category: 17,
            index: 188,
            name: None,
        }),
        (17, 189) => Some(ExtOpcode {
            category: 17,
            index: 189,
            name: None,
        }),
        (17, 190) => Some(ExtOpcode {
            category: 17,
            index: 190,
            name: Some("result_tweet"),
        }),
        (17, 191) => Some(ExtOpcode {
            category: 17,
            index: 191,
            name: Some("get_tweet_key"),
        }),
        (17, 192) => Some(ExtOpcode {
            category: 17,
            index: 192,
            name: Some("set_tweet_key"),
        }),
        (17, 193) => Some(ExtOpcode {
            category: 17,
            index: 193,
            name: None,
        }),
        (17, 194) => Some(ExtOpcode {
            category: 17,
            index: 194,
            name: Some("tweet_authorize"),
        }),
        (17, 195) => Some(ExtOpcode {
            category: 17,
            index: 195,
            name: None,
        }),
        (17, 196) => Some(ExtOpcode {
            category: 17,
            index: 196,
            name: None,
        }),
        (17, 197) => Some(ExtOpcode {
            category: 17,
            index: 197,
            name: None,
        }),
        (17, 198) => Some(ExtOpcode {
            category: 17,
            index: 198,
            name: Some("tips_csv_read_error"),
        }),
        (17, 199) => Some(ExtOpcode {
            category: 17,
            index: 199,
            name: Some("tips_csv_get_error"),
        }),
        (17, 200) => Some(ExtOpcode {
            category: 17,
            index: 200,
            name: Some("tips_csv_search_not_found"),
        }),
        (17, 201) => Some(ExtOpcode {
            category: 17,
            index: 201,
            name: None,
        }),
        (17, 202) => Some(ExtOpcode {
            category: 17,
            index: 202,
            name: None,
        }),
        (17, 203) => Some(ExtOpcode {
            category: 17,
            index: 203,
            name: Some("tips_acc_save"),
        }),
        (17, 204) => Some(ExtOpcode {
            category: 17,
            index: 204,
            name: Some("is_network"),
        }),
        (17, 205) => Some(ExtOpcode {
            category: 17,
            index: 205,
            name: Some("is_touch"),
        }),
        (17, 206) => Some(ExtOpcode {
            category: 17,
            index: 206,
            name: None,
        }),
        (17, 207) => Some(ExtOpcode {
            category: 17,
            index: 207,
            name: None,
        }),
        (17, 208) => Some(ExtOpcode {
            category: 17,
            index: 208,
            name: None,
        }),
        (17, 209) => Some(ExtOpcode {
            category: 17,
            index: 209,
            name: None,
        }),
        (17, 210) => Some(ExtOpcode {
            category: 17,
            index: 210,
            name: None,
        }),
        (17, 211) => Some(ExtOpcode {
            category: 17,
            index: 211,
            name: None,
        }),
        (17, 212) => Some(ExtOpcode {
            category: 17,
            index: 212,
            name: None,
        }),
        (17, 213) => Some(ExtOpcode {
            category: 17,
            index: 213,
            name: None,
        }),
        (17, 214) => Some(ExtOpcode {
            category: 17,
            index: 214,
            name: None,
        }),
        (17, 215) => Some(ExtOpcode {
            category: 17,
            index: 215,
            name: None,
        }),
        (17, 216) => Some(ExtOpcode {
            category: 17,
            index: 216,
            name: None,
        }),
        (17, 217) => Some(ExtOpcode {
            category: 17,
            index: 217,
            name: None,
        }),
        (17, 218) => Some(ExtOpcode {
            category: 17,
            index: 218,
            name: None,
        }),
        (17, 219) => Some(ExtOpcode {
            category: 17,
            index: 219,
            name: None,
        }),
        (17, 220) => Some(ExtOpcode {
            category: 17,
            index: 220,
            name: None,
        }),
        (17, 221) => Some(ExtOpcode {
            category: 17,
            index: 221,
            name: None,
        }),
        (17, 222) => Some(ExtOpcode {
            category: 17,
            index: 222,
            name: None,
        }),
        (17, 223) => Some(ExtOpcode {
            category: 17,
            index: 223,
            name: None,
        }),
        (17, 224) => Some(ExtOpcode {
            category: 17,
            index: 224,
            name: None,
        }),
        (17, 225) => Some(ExtOpcode {
            category: 17,
            index: 225,
            name: None,
        }),
        (17, 226) => Some(ExtOpcode {
            category: 17,
            index: 226,
            name: None,
        }),
        (17, 227) => Some(ExtOpcode {
            category: 17,
            index: 227,
            name: None,
        }),
        (17, 228) => Some(ExtOpcode {
            category: 17,
            index: 228,
            name: None,
        }),
        (17, 230) => Some(ExtOpcode {
            category: 17,
            index: 230,
            name: None,
        }),
        (17, 231) => Some(ExtOpcode {
            category: 17,
            index: 231,
            name: Some("run_no_wait"),
        }),
        (17, 232) => Some(ExtOpcode {
            category: 17,
            index: 232,
            name: Some("run_stack"),
        }),
        (17, 234) => Some(ExtOpcode {
            category: 17,
            index: 234,
            name: Some("fx_effect_cls"),
        }),
        (17, 235) => Some(ExtOpcode {
            category: 17,
            index: 235,
            name: Some("fx_raster_stop"),
        }),
        (17, 236) => Some(ExtOpcode {
            category: 17,
            index: 236,
            name: Some("fx_effect_wait"),
        }),
        (17, 237) => Some(ExtOpcode {
            category: 17,
            index: 237,
            name: None,
        }),
        (17, 239) => Some(ExtOpcode {
            category: 17,
            index: 239,
            name: Some("random"),
        }),
        (17, 240) => Some(ExtOpcode {
            category: 17,
            index: 240,
            name: Some("abs"),
        }),
        (17, 241) => Some(ExtOpcode {
            category: 17,
            index: 241,
            name: Some("sin"),
        }),
        (17, 242) => Some(ExtOpcode {
            category: 17,
            index: 242,
            name: Some("cos"),
        }),
        (17, 243) => Some(ExtOpcode {
            category: 17,
            index: 243,
            name: Some("tan"),
        }),
        (17, 244) => Some(ExtOpcode {
            category: 17,
            index: 244,
            name: Some("atan"),
        }),
        (17, 245) => Some(ExtOpcode {
            category: 17,
            index: 245,
            name: Some("log"),
        }),
        (17, 246) => Some(ExtOpcode {
            category: 17,
            index: 246,
            name: Some("log10"),
        }),
        (17, 247) => Some(ExtOpcode {
            category: 17,
            index: 247,
            name: None,
        }),
        (17, 248) => Some(ExtOpcode {
            category: 17,
            index: 248,
            name: Some("sqrt"),
        }),
        (17, 249) => Some(ExtOpcode {
            category: 17,
            index: 249,
            name: None,
        }),
        (17, 250) => Some(ExtOpcode {
            category: 17,
            index: 250,
            name: None,
        }),
        (17, 254) => Some(ExtOpcode {
            category: 17,
            index: 254,
            name: Some("sp_set"),
        }),
        (17, 255) => Some(ExtOpcode {
            category: 17,
            index: 255,
            name: Some("sp_set_ex"),
        }),
        (17, 256) => Some(ExtOpcode {
            category: 17,
            index: 256,
            name: Some("sp_set_pos"),
        }),
        (17, 257) => Some(ExtOpcode {
            category: 17,
            index: 257,
            name: Some("sp_cls"),
        }),
        (17, 258) => Some(ExtOpcode {
            category: 17,
            index: 258,
            name: Some("sp_set_alpha"),
        }),
        (17, 259) => Some(ExtOpcode {
            category: 17,
            index: 259,
            name: Some("set_priority"),
        }),
        (17, 260) => Some(ExtOpcode {
            category: 17,
            index: 260,
            name: None,
        }),
        (17, 261) => Some(ExtOpcode {
            category: 17,
            index: 261,
            name: Some("sp_set_center"),
        }),
        (17, 263) => Some(ExtOpcode {
            category: 17,
            index: 263,
            name: Some("sp_cls_ex"),
        }),
        (17, 264) => Some(ExtOpcode {
            category: 17,
            index: 264,
            name: Some("set_filter"),
        }),
        (17, 265) => Some(ExtOpcode {
            category: 17,
            index: 265,
            name: Some("sp_cls_transition"),
        }),
        (17, 266) => Some(ExtOpcode {
            category: 17,
            index: 266,
            name: Some("sp_set_pos_ex"),
        }),
        (17, 267) => Some(ExtOpcode {
            category: 17,
            index: 267,
            name: Some("sp_set_rect_pos"),
        }),
        (17, 268) => Some(ExtOpcode {
            category: 17,
            index: 268,
            name: None,
        }),
        (17, 269) => Some(ExtOpcode {
            category: 17,
            index: 269,
            name: Some("sp_set_scale"),
        }),
        (17, 270) => Some(ExtOpcode {
            category: 17,
            index: 270,
            name: Some("sp_set_rotate"),
        }),
        (17, 271) => Some(ExtOpcode {
            category: 17,
            index: 271,
            name: Some("face_init"),
        }),
        (17, 272) => Some(ExtOpcode {
            category: 17,
            index: 272,
            name: Some("face_set"),
        }),
        (17, 273) => Some(ExtOpcode {
            category: 17,
            index: 273,
            name: Some("not_image_sp_get_color"),
        }),
        (17, 274) => Some(ExtOpcode {
            category: 17,
            index: 274,
            name: Some("sptext"),
        }),
        (17, 275) => Some(ExtOpcode {
            category: 17,
            index: 275,
            name: Some("face_cls"),
        }),
        (17, 276) => Some(ExtOpcode {
            category: 17,
            index: 276,
            name: Some("sp_set_rect"),
        }),
        (17, 277) => Some(ExtOpcode {
            category: 17,
            index: 277,
            name: Some("sp_set_pos_move"),
        }),
        (17, 278) => Some(ExtOpcode {
            category: 17,
            index: 278,
            name: Some("not_image_sp_get_alpha"),
        }),
        (17, 279) => Some(ExtOpcode {
            category: 17,
            index: 279,
            name: Some("not_image_sp_get_rotate"),
        }),
        (17, 280) => Some(ExtOpcode {
            category: 17,
            index: 280,
            name: None,
        }),
        (17, 281) => Some(ExtOpcode {
            category: 17,
            index: 281,
            name: None,
        }),
        (17, 282) => Some(ExtOpcode {
            category: 17,
            index: 282,
            name: None,
        }),
        (17, 283) => Some(ExtOpcode {
            category: 17,
            index: 283,
            name: None,
        }),
        (17, 284) => Some(ExtOpcode {
            category: 17,
            index: 284,
            name: Some("sp_create"),
        }),
        (17, 285) => Some(ExtOpcode {
            category: 17,
            index: 285,
            name: Some("sp_anime_clear"),
        }),
        (17, 286) => Some(ExtOpcode {
            category: 17,
            index: 286,
            name: None,
        }),
        (17, 287) => Some(ExtOpcode {
            category: 17,
            index: 287,
            name: None,
        }),
        (17, 288) => Some(ExtOpcode {
            category: 17,
            index: 288,
            name: Some("not_image_sp_get_scale"),
        }),
        (17, 289) => Some(ExtOpcode {
            category: 17,
            index: 289,
            name: Some("sp_set_color_0x"),
        }),
        (17, 290) => Some(ExtOpcode {
            category: 17,
            index: 290,
            name: Some("sp_bitblt"),
        }),
        (17, 291) => Some(ExtOpcode {
            category: 17,
            index: 291,
            name: Some("sp_set_shake"),
        }),
        (17, 292) => Some(ExtOpcode {
            category: 17,
            index: 292,
            name: Some("sp_paint"),
        }),
        (17, 293) => Some(ExtOpcode {
            category: 17,
            index: 293,
            name: None,
        }),
        (17, 294) => Some(ExtOpcode {
            category: 17,
            index: 294,
            name: Some("sp_load_wait_time"),
        }),
        (17, 295) => Some(ExtOpcode {
            category: 17,
            index: 295,
            name: Some("sp_draw"),
        }),
        (17, 296) => Some(ExtOpcode {
            category: 17,
            index: 296,
            name: None,
        }),
        (17, 297) => Some(ExtOpcode {
            category: 17,
            index: 297,
            name: Some("sp_unlock"),
        }),
        (17, 298) => Some(ExtOpcode {
            category: 17,
            index: 298,
            name: Some("sp_show"),
        }),
        (17, 299) => Some(ExtOpcode {
            category: 17,
            index: 299,
            name: Some("sp_hide"),
        }),
        (18, 1) => Some(ExtOpcode {
            category: 18,
            index: 1,
            name: Some("app_exec"),
        }),
        (18, 2) => Some(ExtOpcode {
            category: 18,
            index: 2,
            name: Some("is_playername"),
        }),
        (18, 3) => Some(ExtOpcode {
            category: 18,
            index: 3,
            name: None,
        }),
        (18, 4) => Some(ExtOpcode {
            category: 18,
            index: 4,
            name: None,
        }),
        (18, 5) => Some(ExtOpcode {
            category: 18,
            index: 5,
            name: None,
        }),
        (18, 6) => Some(ExtOpcode {
            category: 18,
            index: 6,
            name: Some("string_alloc"),
        }),
        (18, 7) => Some(ExtOpcode {
            category: 18,
            index: 7,
            name: None,
        }),
        (18, 8) => Some(ExtOpcode {
            category: 18,
            index: 8,
            name: Some("file_exist"),
        }),
        (18, 9) => Some(ExtOpcode {
            category: 18,
            index: 9,
            name: Some("wsprint"),
        }),
        (18, 10) => Some(ExtOpcode {
            category: 18,
            index: 10,
            name: Some("check_disc"),
        }),
        (18, 11) => Some(ExtOpcode {
            category: 18,
            index: 11,
            name: None,
        }),
        (18, 12) => Some(ExtOpcode {
            category: 18,
            index: 12,
            name: None,
        }),
        (18, 13) => Some(ExtOpcode {
            category: 18,
            index: 13,
            name: None,
        }),
        (18, 14) => Some(ExtOpcode {
            category: 18,
            index: 14,
            name: Some("update_access"),
        }),
        (18, 15) => Some(ExtOpcode {
            category: 18,
            index: 15,
            name: None,
        }),
        (18, 16) => Some(ExtOpcode {
            category: 18,
            index: 16,
            name: None,
        }),
        (18, 17) => Some(ExtOpcode {
            category: 18,
            index: 17,
            name: None,
        }),
        (18, 18) => Some(ExtOpcode {
            category: 18,
            index: 18,
            name: None,
        }),
        (18, 19) => Some(ExtOpcode {
            category: 18,
            index: 19,
            name: None,
        }),
        (18, 20) => Some(ExtOpcode {
            category: 18,
            index: 20,
            name: None,
        }),
        (18, 21) => Some(ExtOpcode {
            category: 18,
            index: 21,
            name: Some("strlenf"),
        }),
        (18, 22) => Some(ExtOpcode {
            category: 18,
            index: 22,
            name: Some("player_name_set_begin"),
        }),
        (18, 23) => Some(ExtOpcode {
            category: 18,
            index: 23,
            name: Some("player_name_set_end"),
        }),
        (18, 24) => Some(ExtOpcode {
            category: 18,
            index: 24,
            name: Some("player_name_set_check"),
        }),
        (18, 25) => Some(ExtOpcode {
            category: 18,
            index: 25,
            name: None,
        }),
        (18, 26) => Some(ExtOpcode {
            category: 18,
            index: 26,
            name: Some("player_name_reset"),
        }),
        (18, 27) => Some(ExtOpcode {
            category: 18,
            index: 27,
            name: Some("player_name_set_direct"),
        }),
        (18, 28) => Some(ExtOpcode {
            category: 18,
            index: 28,
            name: None,
        }),
        (18, 29) => Some(ExtOpcode {
            category: 18,
            index: 29,
            name: None,
        }),
        (18, 30) => Some(ExtOpcode {
            category: 18,
            index: 30,
            name: Some("openfile"),
        }),
        (18, 31) => Some(ExtOpcode {
            category: 18,
            index: 31,
            name: Some("read_file"),
        }),
        (18, 32) => Some(ExtOpcode {
            category: 18,
            index: 32,
            name: Some("close_file_not_handle"),
        }),
        (18, 33) => Some(ExtOpcode {
            category: 18,
            index: 33,
            name: Some("set_file_pointer"),
        }),
        (18, 34) => Some(ExtOpcode {
            category: 18,
            index: 34,
            name: Some("file_string"),
        }),
        (18, 35) => Some(ExtOpcode {
            category: 18,
            index: 35,
            name: Some("set_last_process"),
        }),
        (18, 36) => Some(ExtOpcode {
            category: 18,
            index: 36,
            name: Some("sz_buf"),
        }),
        (18, 37) => Some(ExtOpcode {
            category: 18,
            index: 37,
            name: Some("getprivateprofileint"),
        }),
        (18, 38) => Some(ExtOpcode {
            category: 18,
            index: 38,
            name: None,
        }),
        (18, 39) => Some(ExtOpcode {
            category: 18,
            index: 39,
            name: None,
        }),
        (18, 40) => Some(ExtOpcode {
            category: 18,
            index: 40,
            name: None,
        }),
        (18, 41) => Some(ExtOpcode {
            category: 18,
            index: 41,
            name: Some("is_tweet"),
        }),
        (18, 42) => Some(ExtOpcode {
            category: 18,
            index: 42,
            name: None,
        }),
        (18, 43) => Some(ExtOpcode {
            category: 18,
            index: 43,
            name: None,
        }),
        (18, 44) => Some(ExtOpcode {
            category: 18,
            index: 44,
            name: None,
        }),
        (18, 45) => Some(ExtOpcode {
            category: 18,
            index: 45,
            name: None,
        }),
        (18, 46) => Some(ExtOpcode {
            category: 18,
            index: 46,
            name: None,
        }),
        (18, 47) => Some(ExtOpcode {
            category: 18,
            index: 47,
            name: None,
        }),
        (18, 48) => Some(ExtOpcode {
            category: 18,
            index: 48,
            name: None,
        }),
        (18, 49) => Some(ExtOpcode {
            category: 18,
            index: 49,
            name: None,
        }),
        (18, 50) => Some(ExtOpcode {
            category: 18,
            index: 50,
            name: None,
        }),
        (18, 51) => Some(ExtOpcode {
            category: 18,
            index: 51,
            name: None,
        }),
        (18, 52) => Some(ExtOpcode {
            category: 18,
            index: 52,
            name: None,
        }),
        (18, 53) => Some(ExtOpcode {
            category: 18,
            index: 53,
            name: None,
        }),
        (18, 54) => Some(ExtOpcode {
            category: 18,
            index: 54,
            name: None,
        }),
        (18, 55) => Some(ExtOpcode {
            category: 18,
            index: 55,
            name: None,
        }),
        (18, 56) => Some(ExtOpcode {
            category: 18,
            index: 56,
            name: None,
        }),
        (18, 57) => Some(ExtOpcode {
            category: 18,
            index: 57,
            name: None,
        }),
        (18, 58) => Some(ExtOpcode {
            category: 18,
            index: 58,
            name: None,
        }),
        (18, 59) => Some(ExtOpcode {
            category: 18,
            index: 59,
            name: None,
        }),
        (18, 60) => Some(ExtOpcode {
            category: 18,
            index: 60,
            name: None,
        }),
        (18, 61) => Some(ExtOpcode {
            category: 18,
            index: 61,
            name: None,
        }),
        (18, 62) => Some(ExtOpcode {
            category: 18,
            index: 62,
            name: None,
        }),
        (18, 63) => Some(ExtOpcode {
            category: 18,
            index: 63,
            name: None,
        }),
        (18, 64) => Some(ExtOpcode {
            category: 18,
            index: 64,
            name: None,
        }),
        (18, 65) => Some(ExtOpcode {
            category: 18,
            index: 65,
            name: None,
        }),
        (18, 66) => Some(ExtOpcode {
            category: 18,
            index: 66,
            name: None,
        }),
        (18, 67) => Some(ExtOpcode {
            category: 18,
            index: 67,
            name: None,
        }),
        (18, 68) => Some(ExtOpcode {
            category: 18,
            index: 68,
            name: None,
        }),
        (18, 69) => Some(ExtOpcode {
            category: 18,
            index: 69,
            name: None,
        }),
        (18, 70) => Some(ExtOpcode {
            category: 18,
            index: 70,
            name: None,
        }),
        (18, 71) => Some(ExtOpcode {
            category: 18,
            index: 71,
            name: None,
        }),
        (18, 72) => Some(ExtOpcode {
            category: 18,
            index: 72,
            name: None,
        }),
        (18, 73) => Some(ExtOpcode {
            category: 18,
            index: 73,
            name: None,
        }),
        (18, 74) => Some(ExtOpcode {
            category: 18,
            index: 74,
            name: Some("result_tweet"),
        }),
        (18, 75) => Some(ExtOpcode {
            category: 18,
            index: 75,
            name: Some("get_tweet_key"),
        }),
        (18, 76) => Some(ExtOpcode {
            category: 18,
            index: 76,
            name: Some("set_tweet_key"),
        }),
        (18, 77) => Some(ExtOpcode {
            category: 18,
            index: 77,
            name: None,
        }),
        (18, 78) => Some(ExtOpcode {
            category: 18,
            index: 78,
            name: Some("tweet_authorize"),
        }),
        (18, 79) => Some(ExtOpcode {
            category: 18,
            index: 79,
            name: None,
        }),
        (18, 80) => Some(ExtOpcode {
            category: 18,
            index: 80,
            name: None,
        }),
        (18, 81) => Some(ExtOpcode {
            category: 18,
            index: 81,
            name: None,
        }),
        (18, 82) => Some(ExtOpcode {
            category: 18,
            index: 82,
            name: Some("tips_csv_read_error"),
        }),
        (18, 83) => Some(ExtOpcode {
            category: 18,
            index: 83,
            name: Some("tips_csv_get_error"),
        }),
        (18, 84) => Some(ExtOpcode {
            category: 18,
            index: 84,
            name: Some("tips_csv_search_not_found"),
        }),
        (18, 85) => Some(ExtOpcode {
            category: 18,
            index: 85,
            name: None,
        }),
        (18, 86) => Some(ExtOpcode {
            category: 18,
            index: 86,
            name: None,
        }),
        (18, 87) => Some(ExtOpcode {
            category: 18,
            index: 87,
            name: Some("tips_acc_save"),
        }),
        (18, 88) => Some(ExtOpcode {
            category: 18,
            index: 88,
            name: Some("is_network"),
        }),
        (18, 89) => Some(ExtOpcode {
            category: 18,
            index: 89,
            name: Some("is_touch"),
        }),
        (18, 90) => Some(ExtOpcode {
            category: 18,
            index: 90,
            name: None,
        }),
        (18, 91) => Some(ExtOpcode {
            category: 18,
            index: 91,
            name: None,
        }),
        (18, 92) => Some(ExtOpcode {
            category: 18,
            index: 92,
            name: None,
        }),
        (18, 93) => Some(ExtOpcode {
            category: 18,
            index: 93,
            name: None,
        }),
        (18, 94) => Some(ExtOpcode {
            category: 18,
            index: 94,
            name: None,
        }),
        (18, 95) => Some(ExtOpcode {
            category: 18,
            index: 95,
            name: None,
        }),
        (18, 96) => Some(ExtOpcode {
            category: 18,
            index: 96,
            name: None,
        }),
        (18, 97) => Some(ExtOpcode {
            category: 18,
            index: 97,
            name: None,
        }),
        (18, 98) => Some(ExtOpcode {
            category: 18,
            index: 98,
            name: None,
        }),
        (18, 99) => Some(ExtOpcode {
            category: 18,
            index: 99,
            name: None,
        }),
        (18, 100) => Some(ExtOpcode {
            category: 18,
            index: 100,
            name: None,
        }),
        (18, 101) => Some(ExtOpcode {
            category: 18,
            index: 101,
            name: None,
        }),
        (18, 102) => Some(ExtOpcode {
            category: 18,
            index: 102,
            name: None,
        }),
        (18, 103) => Some(ExtOpcode {
            category: 18,
            index: 103,
            name: None,
        }),
        (18, 104) => Some(ExtOpcode {
            category: 18,
            index: 104,
            name: None,
        }),
        (18, 105) => Some(ExtOpcode {
            category: 18,
            index: 105,
            name: None,
        }),
        (18, 106) => Some(ExtOpcode {
            category: 18,
            index: 106,
            name: None,
        }),
        (18, 107) => Some(ExtOpcode {
            category: 18,
            index: 107,
            name: None,
        }),
        (18, 108) => Some(ExtOpcode {
            category: 18,
            index: 108,
            name: None,
        }),
        (18, 109) => Some(ExtOpcode {
            category: 18,
            index: 109,
            name: None,
        }),
        (18, 110) => Some(ExtOpcode {
            category: 18,
            index: 110,
            name: None,
        }),
        (18, 111) => Some(ExtOpcode {
            category: 18,
            index: 111,
            name: None,
        }),
        (18, 112) => Some(ExtOpcode {
            category: 18,
            index: 112,
            name: None,
        }),
        (18, 114) => Some(ExtOpcode {
            category: 18,
            index: 114,
            name: None,
        }),
        (18, 115) => Some(ExtOpcode {
            category: 18,
            index: 115,
            name: Some("run_no_wait"),
        }),
        (18, 116) => Some(ExtOpcode {
            category: 18,
            index: 116,
            name: Some("run_stack"),
        }),
        (18, 118) => Some(ExtOpcode {
            category: 18,
            index: 118,
            name: Some("fx_effect_cls"),
        }),
        (18, 119) => Some(ExtOpcode {
            category: 18,
            index: 119,
            name: Some("fx_raster_stop"),
        }),
        (18, 120) => Some(ExtOpcode {
            category: 18,
            index: 120,
            name: Some("fx_effect_wait"),
        }),
        (18, 121) => Some(ExtOpcode {
            category: 18,
            index: 121,
            name: Some("pal_time_ms"),
        }),
        (18, 123) => Some(ExtOpcode {
            category: 18,
            index: 123,
            name: Some("random"),
        }),
        (18, 124) => Some(ExtOpcode {
            category: 18,
            index: 124,
            name: Some("abs"),
        }),
        (18, 125) => Some(ExtOpcode {
            category: 18,
            index: 125,
            name: Some("sin"),
        }),
        (18, 126) => Some(ExtOpcode {
            category: 18,
            index: 126,
            name: Some("cos"),
        }),
        (18, 127) => Some(ExtOpcode {
            category: 18,
            index: 127,
            name: Some("tan"),
        }),
        (18, 128) => Some(ExtOpcode {
            category: 18,
            index: 128,
            name: Some("atan"),
        }),
        (18, 129) => Some(ExtOpcode {
            category: 18,
            index: 129,
            name: Some("log"),
        }),
        (18, 130) => Some(ExtOpcode {
            category: 18,
            index: 130,
            name: Some("log10"),
        }),
        (18, 131) => Some(ExtOpcode {
            category: 18,
            index: 131,
            name: None,
        }),
        (18, 132) => Some(ExtOpcode {
            category: 18,
            index: 132,
            name: Some("sqrt"),
        }),
        (18, 133) => Some(ExtOpcode {
            category: 18,
            index: 133,
            name: None,
        }),
        (18, 134) => Some(ExtOpcode {
            category: 18,
            index: 134,
            name: None,
        }),
        (18, 138) => Some(ExtOpcode {
            category: 18,
            index: 138,
            name: Some("sp_set"),
        }),
        (18, 139) => Some(ExtOpcode {
            category: 18,
            index: 139,
            name: Some("sp_set_ex"),
        }),
        (18, 140) => Some(ExtOpcode {
            category: 18,
            index: 140,
            name: Some("sp_set_pos"),
        }),
        (18, 141) => Some(ExtOpcode {
            category: 18,
            index: 141,
            name: Some("sp_cls"),
        }),
        (18, 142) => Some(ExtOpcode {
            category: 18,
            index: 142,
            name: Some("sp_set_alpha"),
        }),
        (18, 143) => Some(ExtOpcode {
            category: 18,
            index: 143,
            name: Some("set_priority"),
        }),
        (18, 144) => Some(ExtOpcode {
            category: 18,
            index: 144,
            name: None,
        }),
        (18, 145) => Some(ExtOpcode {
            category: 18,
            index: 145,
            name: Some("sp_set_center"),
        }),
        (18, 147) => Some(ExtOpcode {
            category: 18,
            index: 147,
            name: Some("sp_cls_ex"),
        }),
        (18, 148) => Some(ExtOpcode {
            category: 18,
            index: 148,
            name: Some("set_filter"),
        }),
        (18, 149) => Some(ExtOpcode {
            category: 18,
            index: 149,
            name: Some("sp_cls_transition"),
        }),
        (18, 150) => Some(ExtOpcode {
            category: 18,
            index: 150,
            name: Some("sp_set_pos_ex"),
        }),
        (18, 151) => Some(ExtOpcode {
            category: 18,
            index: 151,
            name: Some("sp_set_rect_pos"),
        }),
        (18, 152) => Some(ExtOpcode {
            category: 18,
            index: 152,
            name: None,
        }),
        (18, 153) => Some(ExtOpcode {
            category: 18,
            index: 153,
            name: Some("sp_set_scale"),
        }),
        (18, 154) => Some(ExtOpcode {
            category: 18,
            index: 154,
            name: Some("sp_set_rotate"),
        }),
        (18, 155) => Some(ExtOpcode {
            category: 18,
            index: 155,
            name: Some("face_init"),
        }),
        (18, 156) => Some(ExtOpcode {
            category: 18,
            index: 156,
            name: Some("face_set"),
        }),
        (18, 157) => Some(ExtOpcode {
            category: 18,
            index: 157,
            name: Some("not_image_sp_get_color"),
        }),
        (18, 158) => Some(ExtOpcode {
            category: 18,
            index: 158,
            name: Some("sptext"),
        }),
        (18, 159) => Some(ExtOpcode {
            category: 18,
            index: 159,
            name: Some("face_cls"),
        }),
        (18, 160) => Some(ExtOpcode {
            category: 18,
            index: 160,
            name: Some("sp_set_rect"),
        }),
        (18, 161) => Some(ExtOpcode {
            category: 18,
            index: 161,
            name: Some("sp_set_pos_move"),
        }),
        (18, 162) => Some(ExtOpcode {
            category: 18,
            index: 162,
            name: Some("not_image_sp_get_alpha"),
        }),
        (18, 163) => Some(ExtOpcode {
            category: 18,
            index: 163,
            name: Some("not_image_sp_get_rotate"),
        }),
        (18, 164) => Some(ExtOpcode {
            category: 18,
            index: 164,
            name: None,
        }),
        (18, 165) => Some(ExtOpcode {
            category: 18,
            index: 165,
            name: None,
        }),
        (18, 166) => Some(ExtOpcode {
            category: 18,
            index: 166,
            name: None,
        }),
        (18, 167) => Some(ExtOpcode {
            category: 18,
            index: 167,
            name: None,
        }),
        (18, 168) => Some(ExtOpcode {
            category: 18,
            index: 168,
            name: Some("sp_create"),
        }),
        (18, 169) => Some(ExtOpcode {
            category: 18,
            index: 169,
            name: Some("sp_anime_clear"),
        }),
        (18, 170) => Some(ExtOpcode {
            category: 18,
            index: 170,
            name: None,
        }),
        (18, 171) => Some(ExtOpcode {
            category: 18,
            index: 171,
            name: None,
        }),
        (18, 172) => Some(ExtOpcode {
            category: 18,
            index: 172,
            name: Some("not_image_sp_get_scale"),
        }),
        (18, 173) => Some(ExtOpcode {
            category: 18,
            index: 173,
            name: Some("sp_set_color_0x"),
        }),
        (18, 174) => Some(ExtOpcode {
            category: 18,
            index: 174,
            name: Some("sp_bitblt"),
        }),
        (18, 175) => Some(ExtOpcode {
            category: 18,
            index: 175,
            name: Some("sp_set_shake"),
        }),
        (18, 176) => Some(ExtOpcode {
            category: 18,
            index: 176,
            name: Some("sp_paint"),
        }),
        (18, 177) => Some(ExtOpcode {
            category: 18,
            index: 177,
            name: None,
        }),
        (18, 178) => Some(ExtOpcode {
            category: 18,
            index: 178,
            name: Some("sp_load_wait_time"),
        }),
        (18, 179) => Some(ExtOpcode {
            category: 18,
            index: 179,
            name: Some("sp_draw"),
        }),
        (18, 180) => Some(ExtOpcode {
            category: 18,
            index: 180,
            name: None,
        }),
        (18, 181) => Some(ExtOpcode {
            category: 18,
            index: 181,
            name: Some("sp_unlock"),
        }),
        (18, 182) => Some(ExtOpcode {
            category: 18,
            index: 182,
            name: Some("sp_show"),
        }),
        (18, 183) => Some(ExtOpcode {
            category: 18,
            index: 183,
            name: Some("sp_hide"),
        }),
        (18, 184) => Some(ExtOpcode {
            category: 18,
            index: 184,
            name: None,
        }),
        (18, 185) => Some(ExtOpcode {
            category: 18,
            index: 185,
            name: Some("sp_set_child"),
        }),
        (18, 186) => Some(ExtOpcode {
            category: 18,
            index: 186,
            name: Some("sp_set_transition"),
        }),
        (18, 187) => Some(ExtOpcode {
            category: 18,
            index: 187,
            name: Some("sp_copy_image"),
        }),
        (18, 188) => Some(ExtOpcode {
            category: 18,
            index: 188,
            name: Some("sp_transition"),
        }),
        (18, 189) => Some(ExtOpcode {
            category: 18,
            index: 189,
            name: Some("set_aspect_position_type"),
        }),
        (18, 190) => Some(ExtOpcode {
            category: 18,
            index: 190,
            name: Some("get_backbuffer"),
        }),
        (18, 191) => Some(ExtOpcode {
            category: 18,
            index: 191,
            name: Some("sp_set_mask"),
        }),
        (18, 192) => Some(ExtOpcode {
            category: 18,
            index: 192,
            name: None,
        }),
        (18, 193) => Some(ExtOpcode {
            category: 18,
            index: 193,
            name: Some("spsetanime"),
        }),
        (18, 194) => Some(ExtOpcode {
            category: 18,
            index: 194,
            name: Some("drawtext"),
        }),
        (18, 195) => Some(ExtOpcode {
            category: 18,
            index: 195,
            name: None,
        }),
        (18, 196) => Some(ExtOpcode {
            category: 18,
            index: 196,
            name: None,
        }),
        (18, 198) => Some(ExtOpcode {
            category: 18,
            index: 198,
            name: Some("history_init_0x_0x"),
        }),
        (18, 199) => Some(ExtOpcode {
            category: 18,
            index: 199,
            name: Some("historybegin_lpbyte_ptagdata_sztext"),
        }),
        (18, 200) => Some(ExtOpcode {
            category: 18,
            index: 200,
            name: Some("history_end"),
        }),
        (18, 201) => Some(ExtOpcode {
            category: 18,
            index: 201,
            name: None,
        }),
        (18, 202) => Some(ExtOpcode {
            category: 18,
            index: 202,
            name: None,
        }),
        (18, 203) => Some(ExtOpcode {
            category: 18,
            index: 203,
            name: Some("history_get_height"),
        }),
        (18, 204) => Some(ExtOpcode {
            category: 18,
            index: 204,
            name: None,
        }),
        (18, 205) => Some(ExtOpcode {
            category: 18,
            index: 205,
            name: None,
        }),
        (18, 206) => Some(ExtOpcode {
            category: 18,
            index: 206,
            name: None,
        }),
        (18, 207) => Some(ExtOpcode {
            category: 18,
            index: 207,
            name: None,
        }),
        (18, 208) => Some(ExtOpcode {
            category: 18,
            index: 208,
            name: Some("history_set_rect"),
        }),
        (18, 209) => Some(ExtOpcode {
            category: 18,
            index: 209,
            name: Some("history_clear"),
        }),
        (18, 210) => Some(ExtOpcode {
            category: 18,
            index: 210,
            name: Some("history_set"),
        }),
        (18, 211) => Some(ExtOpcode {
            category: 18,
            index: 211,
            name: None,
        }),
        (18, 212) => Some(ExtOpcode {
            category: 18,
            index: 212,
            name: None,
        }),
        (18, 213) => Some(ExtOpcode {
            category: 18,
            index: 213,
            name: None,
        }),
        (18, 214) => Some(ExtOpcode {
            category: 18,
            index: 214,
            name: None,
        }),
        (18, 215) => Some(ExtOpcode {
            category: 18,
            index: 215,
            name: Some("history_set_face_call"),
        }),
        (18, 216) => Some(ExtOpcode {
            category: 18,
            index: 216,
            name: Some("history_set_face_sound"),
        }),
        (18, 217) => Some(ExtOpcode {
            category: 18,
            index: 217,
            name: Some("history_set_face_sound_release"),
        }),
        (18, 218) => Some(ExtOpcode {
            category: 18,
            index: 218,
            name: Some("history_get_text"),
        }),
        (18, 219) => Some(ExtOpcode {
            category: 18,
            index: 219,
            name: None,
        }),
        (18, 220) => Some(ExtOpcode {
            category: 18,
            index: 220,
            name: None,
        }),
        (18, 221) => Some(ExtOpcode {
            category: 18,
            index: 221,
            name: None,
        }),
        (18, 222) => Some(ExtOpcode {
            category: 18,
            index: 222,
            name: None,
        }),
        (18, 224) => Some(ExtOpcode {
            category: 18,
            index: 224,
            name: Some("movie_play"),
        }),
        (18, 225) => Some(ExtOpcode {
            category: 18,
            index: 225,
            name: Some("msp_set_loop_sp_ep"),
        }),
        (18, 226) => Some(ExtOpcode {
            category: 18,
            index: 226,
            name: Some("msp_cls"),
        }),
        (18, 227) => Some(ExtOpcode {
            category: 18,
            index: 227,
            name: Some("msp_wait"),
        }),
        (18, 228) => Some(ExtOpcode {
            category: 18,
            index: 228,
            name: Some("msp_lock"),
        }),
        (18, 229) => Some(ExtOpcode {
            category: 18,
            index: 229,
            name: Some("msp_unlock"),
        }),
        (18, 230) => Some(ExtOpcode {
            category: 18,
            index: 230,
            name: Some("msp_play"),
        }),
        (18, 231) => Some(ExtOpcode {
            category: 18,
            index: 231,
            name: Some("msp_stop"),
        }),
        (18, 233) => Some(ExtOpcode {
            category: 18,
            index: 233,
            name: Some("create_thread"),
        }),
        (18, 234) => Some(ExtOpcode {
            category: 18,
            index: 234,
            name: Some("exit_thread"),
        }),
        (18, 235) => Some(ExtOpcode {
            category: 18,
            index: 235,
            name: None,
        }),
        (18, 236) => Some(ExtOpcode {
            category: 18,
            index: 236,
            name: Some("get_thread"),
        }),
        (18, 239) => Some(ExtOpcode {
            category: 18,
            index: 239,
            name: Some("mov"),
        }),
        (18, 240) => Some(ExtOpcode {
            category: 18,
            index: 240,
            name: Some("add"),
        }),
        (18, 241) => Some(ExtOpcode {
            category: 18,
            index: 241,
            name: Some("sub"),
        }),
        (18, 242) => Some(ExtOpcode {
            category: 18,
            index: 242,
            name: Some("mul"),
        }),
        (18, 243) => Some(ExtOpcode {
            category: 18,
            index: 243,
            name: Some("div"),
        }),
        (18, 244) => Some(ExtOpcode {
            category: 18,
            index: 244,
            name: Some("bitand"),
        }),
        (18, 245) => Some(ExtOpcode {
            category: 18,
            index: 245,
            name: Some("bitor"),
        }),
        (18, 246) => Some(ExtOpcode {
            category: 18,
            index: 246,
            name: Some("bitxor"),
        }),
        (18, 247) => Some(ExtOpcode {
            category: 18,
            index: 247,
            name: Some("jmp_point"),
        }),
        (18, 248) => Some(ExtOpcode {
            category: 18,
            index: 248,
            name: Some("jf_point"),
        }),
        (18, 249) => Some(ExtOpcode {
            category: 18,
            index: 249,
            name: Some("gosub_point"),
        }),
        (18, 250) => Some(ExtOpcode {
            category: 18,
            index: 250,
            name: Some("eq"),
        }),
        (18, 251) => Some(ExtOpcode {
            category: 18,
            index: 251,
            name: Some("ne"),
        }),
        (18, 252) => Some(ExtOpcode {
            category: 18,
            index: 252,
            name: Some("le"),
        }),
        (18, 253) => Some(ExtOpcode {
            category: 18,
            index: 253,
            name: Some("ge"),
        }),
        (18, 254) => Some(ExtOpcode {
            category: 18,
            index: 254,
            name: Some("lt"),
        }),
        (18, 255) => Some(ExtOpcode {
            category: 18,
            index: 255,
            name: Some("gt"),
        }),
        (18, 256) => Some(ExtOpcode {
            category: 18,
            index: 256,
            name: Some("lor"),
        }),
        (18, 257) => Some(ExtOpcode {
            category: 18,
            index: 257,
            name: Some("land"),
        }),
        (18, 258) => Some(ExtOpcode {
            category: 18,
            index: 258,
            name: Some("lnot_slot"),
        }),
        (18, 259) => Some(ExtOpcode {
            category: 18,
            index: 259,
            name: Some("end"),
        }),
        (18, 260) => Some(ExtOpcode {
            category: 18,
            index: 260,
            name: Some("nop"),
        }),
        (18, 261) => Some(ExtOpcode {
            category: 18,
            index: 261,
            name: Some("extcall"),
        }),
        (18, 262) => Some(ExtOpcode {
            category: 18,
            index: 262,
            name: Some("ret"),
        }),
        (18, 263) => Some(ExtOpcode {
            category: 18,
            index: 263,
            name: Some("reset_adv"),
        }),
        (18, 264) => Some(ExtOpcode {
            category: 18,
            index: 264,
            name: Some("mod"),
        }),
        (18, 265) => Some(ExtOpcode {
            category: 18,
            index: 265,
            name: Some("shl"),
        }),
        (18, 266) => Some(ExtOpcode {
            category: 18,
            index: 266,
            name: Some("shr"),
        }),
        (18, 267) => Some(ExtOpcode {
            category: 18,
            index: 267,
            name: Some("neg_slot"),
        }),
        (18, 268) => Some(ExtOpcode {
            category: 18,
            index: 268,
            name: Some("pop"),
        }),
        (18, 269) => Some(ExtOpcode {
            category: 18,
            index: 269,
            name: Some("push"),
        }),
        (18, 270) => Some(ExtOpcode {
            category: 18,
            index: 270,
            name: Some("pack_args"),
        }),
        (18, 271) => Some(ExtOpcode {
            category: 18,
            index: 271,
            name: Some("drop_args"),
        }),
        (18, 273) => Some(ExtOpcode {
            category: 18,
            index: 273,
            name: Some("create_message"),
        }),
        (18, 274) => Some(ExtOpcode {
            category: 18,
            index: 274,
            name: Some("get_message"),
        }),
        (18, 275) => Some(ExtOpcode {
            category: 18,
            index: 275,
            name: Some("get_message_param"),
        }),
        (18, 278) => Some(ExtOpcode {
            category: 18,
            index: 278,
            name: Some("save"),
        }),
        (18, 279) => Some(ExtOpcode {
            category: 18,
            index: 279,
            name: Some("load"),
        }),
        (18, 280) => Some(ExtOpcode {
            category: 18,
            index: 280,
            name: Some("save_set_title"),
        }),
        (18, 281) => Some(ExtOpcode {
            category: 18,
            index: 281,
            name: Some("save_data"),
        }),
        (18, 282) => Some(ExtOpcode {
            category: 18,
            index: 282,
            name: Some("save_set_thumbnail_size"),
        }),
        (18, 283) => Some(ExtOpcode {
            category: 18,
            index: 283,
            name: Some("thumbnail_set"),
        }),
        (18, 284) => Some(ExtOpcode {
            category: 18,
            index: 284,
            name: Some("savetitledraw"),
        }),
        (18, 285) => Some(ExtOpcode {
            category: 18,
            index: 285,
            name: Some("save_set_font_size"),
        }),
        (18, 286) => Some(ExtOpcode {
            category: 18,
            index: 286,
            name: Some("getsaveday"),
        }),
        (18, 287) => Some(ExtOpcode {
            category: 18,
            index: 287,
            name: Some("is_save"),
        }),
        (18, 288) => Some(ExtOpcode {
            category: 18,
            index: 288,
            name: Some("getsaveusermemory"),
        }),
        (18, 289) => Some(ExtOpcode {
            category: 18,
            index: 289,
            name: Some("savepoint"),
        }),
        (18, 290) => Some(ExtOpcode {
            category: 18,
            index: 290,
            name: Some("save_thumbnail_mosaic_set"),
        }),
        (18, 291) => Some(ExtOpcode {
            category: 18,
            index: 291,
            name: Some("savetimedraw"),
        }),
        (18, 292) => Some(ExtOpcode {
            category: 18,
            index: 292,
            name: Some("savedaydraw"),
        }),
        (18, 293) => Some(ExtOpcode {
            category: 18,
            index: 293,
            name: Some("save_set_text_rect"),
        }),
        (18, 294) => Some(ExtOpcode {
            category: 18,
            index: 294,
            name: Some("savetextdraw"),
        }),
        (18, 295) => Some(ExtOpcode {
            category: 18,
            index: 295,
            name: Some("get_new_savefile"),
        }),
        (18, 299) => Some(ExtOpcode {
            category: 18,
            index: 299,
            name: Some("setsavetext"),
        }),
        (19, 0) => Some(ExtOpcode {
            category: 19,
            index: 0,
            name: Some("fx_effect_cls"),
        }),
        (19, 1) => Some(ExtOpcode {
            category: 19,
            index: 1,
            name: Some("fx_raster_stop"),
        }),
        (19, 2) => Some(ExtOpcode {
            category: 19,
            index: 2,
            name: Some("fx_effect_wait"),
        }),
        (19, 3) => Some(ExtOpcode {
            category: 19,
            index: 3,
            name: None,
        }),
        (19, 5) => Some(ExtOpcode {
            category: 19,
            index: 5,
            name: Some("random"),
        }),
        (19, 6) => Some(ExtOpcode {
            category: 19,
            index: 6,
            name: Some("abs"),
        }),
        (19, 7) => Some(ExtOpcode {
            category: 19,
            index: 7,
            name: Some("sin"),
        }),
        (19, 8) => Some(ExtOpcode {
            category: 19,
            index: 8,
            name: Some("cos"),
        }),
        (19, 9) => Some(ExtOpcode {
            category: 19,
            index: 9,
            name: Some("tan"),
        }),
        (19, 10) => Some(ExtOpcode {
            category: 19,
            index: 10,
            name: Some("atan"),
        }),
        (19, 11) => Some(ExtOpcode {
            category: 19,
            index: 11,
            name: Some("log"),
        }),
        (19, 12) => Some(ExtOpcode {
            category: 19,
            index: 12,
            name: Some("log10"),
        }),
        (19, 13) => Some(ExtOpcode {
            category: 19,
            index: 13,
            name: None,
        }),
        (19, 14) => Some(ExtOpcode {
            category: 19,
            index: 14,
            name: Some("sqrt"),
        }),
        (19, 15) => Some(ExtOpcode {
            category: 19,
            index: 15,
            name: None,
        }),
        (19, 16) => Some(ExtOpcode {
            category: 19,
            index: 16,
            name: None,
        }),
        (19, 20) => Some(ExtOpcode {
            category: 19,
            index: 20,
            name: Some("sp_set"),
        }),
        (19, 21) => Some(ExtOpcode {
            category: 19,
            index: 21,
            name: Some("sp_set_ex"),
        }),
        (19, 22) => Some(ExtOpcode {
            category: 19,
            index: 22,
            name: Some("sp_set_pos"),
        }),
        (19, 23) => Some(ExtOpcode {
            category: 19,
            index: 23,
            name: Some("sp_cls"),
        }),
        (19, 24) => Some(ExtOpcode {
            category: 19,
            index: 24,
            name: Some("sp_set_alpha"),
        }),
        (19, 25) => Some(ExtOpcode {
            category: 19,
            index: 25,
            name: Some("set_priority"),
        }),
        (19, 26) => Some(ExtOpcode {
            category: 19,
            index: 26,
            name: None,
        }),
        (19, 27) => Some(ExtOpcode {
            category: 19,
            index: 27,
            name: Some("sp_set_center"),
        }),
        (19, 29) => Some(ExtOpcode {
            category: 19,
            index: 29,
            name: Some("sp_cls_ex"),
        }),
        (19, 30) => Some(ExtOpcode {
            category: 19,
            index: 30,
            name: Some("set_filter"),
        }),
        (19, 31) => Some(ExtOpcode {
            category: 19,
            index: 31,
            name: Some("sp_cls_transition"),
        }),
        (19, 32) => Some(ExtOpcode {
            category: 19,
            index: 32,
            name: Some("sp_set_pos_ex"),
        }),
        (19, 33) => Some(ExtOpcode {
            category: 19,
            index: 33,
            name: Some("sp_set_rect_pos"),
        }),
        (19, 34) => Some(ExtOpcode {
            category: 19,
            index: 34,
            name: None,
        }),
        (19, 35) => Some(ExtOpcode {
            category: 19,
            index: 35,
            name: Some("sp_set_scale"),
        }),
        (19, 36) => Some(ExtOpcode {
            category: 19,
            index: 36,
            name: Some("sp_set_rotate"),
        }),
        (19, 37) => Some(ExtOpcode {
            category: 19,
            index: 37,
            name: Some("face_init"),
        }),
        (19, 38) => Some(ExtOpcode {
            category: 19,
            index: 38,
            name: Some("face_set"),
        }),
        (19, 39) => Some(ExtOpcode {
            category: 19,
            index: 39,
            name: Some("not_image_sp_get_color"),
        }),
        (19, 40) => Some(ExtOpcode {
            category: 19,
            index: 40,
            name: Some("sptext"),
        }),
        (19, 41) => Some(ExtOpcode {
            category: 19,
            index: 41,
            name: Some("face_cls"),
        }),
        (19, 42) => Some(ExtOpcode {
            category: 19,
            index: 42,
            name: Some("sp_set_rect"),
        }),
        (19, 43) => Some(ExtOpcode {
            category: 19,
            index: 43,
            name: Some("sp_set_pos_move"),
        }),
        (19, 44) => Some(ExtOpcode {
            category: 19,
            index: 44,
            name: Some("not_image_sp_get_alpha"),
        }),
        (19, 45) => Some(ExtOpcode {
            category: 19,
            index: 45,
            name: Some("not_image_sp_get_rotate"),
        }),
        (19, 46) => Some(ExtOpcode {
            category: 19,
            index: 46,
            name: None,
        }),
        (19, 47) => Some(ExtOpcode {
            category: 19,
            index: 47,
            name: None,
        }),
        (19, 48) => Some(ExtOpcode {
            category: 19,
            index: 48,
            name: None,
        }),
        (19, 49) => Some(ExtOpcode {
            category: 19,
            index: 49,
            name: None,
        }),
        (19, 50) => Some(ExtOpcode {
            category: 19,
            index: 50,
            name: Some("sp_create"),
        }),
        (19, 51) => Some(ExtOpcode {
            category: 19,
            index: 51,
            name: Some("sp_anime_clear"),
        }),
        (19, 52) => Some(ExtOpcode {
            category: 19,
            index: 52,
            name: None,
        }),
        (19, 53) => Some(ExtOpcode {
            category: 19,
            index: 53,
            name: None,
        }),
        (19, 54) => Some(ExtOpcode {
            category: 19,
            index: 54,
            name: Some("not_image_sp_get_scale"),
        }),
        (19, 55) => Some(ExtOpcode {
            category: 19,
            index: 55,
            name: Some("sp_set_color_0x"),
        }),
        (19, 56) => Some(ExtOpcode {
            category: 19,
            index: 56,
            name: Some("sp_bitblt"),
        }),
        (19, 57) => Some(ExtOpcode {
            category: 19,
            index: 57,
            name: Some("sp_set_shake"),
        }),
        (19, 58) => Some(ExtOpcode {
            category: 19,
            index: 58,
            name: Some("sp_paint"),
        }),
        (19, 59) => Some(ExtOpcode {
            category: 19,
            index: 59,
            name: None,
        }),
        (19, 60) => Some(ExtOpcode {
            category: 19,
            index: 60,
            name: Some("sp_load_wait_time"),
        }),
        (19, 61) => Some(ExtOpcode {
            category: 19,
            index: 61,
            name: Some("sp_draw"),
        }),
        (19, 62) => Some(ExtOpcode {
            category: 19,
            index: 62,
            name: None,
        }),
        (19, 63) => Some(ExtOpcode {
            category: 19,
            index: 63,
            name: Some("sp_unlock"),
        }),
        (19, 64) => Some(ExtOpcode {
            category: 19,
            index: 64,
            name: Some("sp_show"),
        }),
        (19, 65) => Some(ExtOpcode {
            category: 19,
            index: 65,
            name: Some("sp_hide"),
        }),
        (19, 66) => Some(ExtOpcode {
            category: 19,
            index: 66,
            name: None,
        }),
        (19, 67) => Some(ExtOpcode {
            category: 19,
            index: 67,
            name: Some("sp_set_child"),
        }),
        (19, 68) => Some(ExtOpcode {
            category: 19,
            index: 68,
            name: Some("sp_set_transition"),
        }),
        (19, 69) => Some(ExtOpcode {
            category: 19,
            index: 69,
            name: Some("sp_copy_image"),
        }),
        (19, 70) => Some(ExtOpcode {
            category: 19,
            index: 70,
            name: Some("sp_transition"),
        }),
        (19, 71) => Some(ExtOpcode {
            category: 19,
            index: 71,
            name: Some("set_aspect_position_type"),
        }),
        (19, 72) => Some(ExtOpcode {
            category: 19,
            index: 72,
            name: Some("get_backbuffer"),
        }),
        (19, 73) => Some(ExtOpcode {
            category: 19,
            index: 73,
            name: Some("sp_set_mask"),
        }),
        (19, 74) => Some(ExtOpcode {
            category: 19,
            index: 74,
            name: None,
        }),
        (19, 75) => Some(ExtOpcode {
            category: 19,
            index: 75,
            name: Some("spsetanime"),
        }),
        (19, 76) => Some(ExtOpcode {
            category: 19,
            index: 76,
            name: Some("drawtext"),
        }),
        (19, 77) => Some(ExtOpcode {
            category: 19,
            index: 77,
            name: None,
        }),
        (19, 78) => Some(ExtOpcode {
            category: 19,
            index: 78,
            name: None,
        }),
        (19, 80) => Some(ExtOpcode {
            category: 19,
            index: 80,
            name: Some("history_init_0x_0x"),
        }),
        (19, 81) => Some(ExtOpcode {
            category: 19,
            index: 81,
            name: Some("historybegin_lpbyte_ptagdata_sztext"),
        }),
        (19, 82) => Some(ExtOpcode {
            category: 19,
            index: 82,
            name: Some("history_end"),
        }),
        (19, 83) => Some(ExtOpcode {
            category: 19,
            index: 83,
            name: None,
        }),
        (19, 84) => Some(ExtOpcode {
            category: 19,
            index: 84,
            name: None,
        }),
        (19, 85) => Some(ExtOpcode {
            category: 19,
            index: 85,
            name: Some("history_get_height"),
        }),
        (19, 86) => Some(ExtOpcode {
            category: 19,
            index: 86,
            name: None,
        }),
        (19, 87) => Some(ExtOpcode {
            category: 19,
            index: 87,
            name: None,
        }),
        (19, 88) => Some(ExtOpcode {
            category: 19,
            index: 88,
            name: None,
        }),
        (19, 89) => Some(ExtOpcode {
            category: 19,
            index: 89,
            name: None,
        }),
        (19, 90) => Some(ExtOpcode {
            category: 19,
            index: 90,
            name: Some("history_set_rect"),
        }),
        (19, 91) => Some(ExtOpcode {
            category: 19,
            index: 91,
            name: Some("history_clear"),
        }),
        (19, 92) => Some(ExtOpcode {
            category: 19,
            index: 92,
            name: Some("history_set"),
        }),
        (19, 93) => Some(ExtOpcode {
            category: 19,
            index: 93,
            name: None,
        }),
        (19, 94) => Some(ExtOpcode {
            category: 19,
            index: 94,
            name: None,
        }),
        (19, 95) => Some(ExtOpcode {
            category: 19,
            index: 95,
            name: None,
        }),
        (19, 96) => Some(ExtOpcode {
            category: 19,
            index: 96,
            name: None,
        }),
        (19, 97) => Some(ExtOpcode {
            category: 19,
            index: 97,
            name: Some("history_set_face_call"),
        }),
        (19, 98) => Some(ExtOpcode {
            category: 19,
            index: 98,
            name: Some("history_set_face_sound"),
        }),
        (19, 99) => Some(ExtOpcode {
            category: 19,
            index: 99,
            name: Some("history_set_face_sound_release"),
        }),
        (19, 100) => Some(ExtOpcode {
            category: 19,
            index: 100,
            name: Some("history_get_text"),
        }),
        (19, 101) => Some(ExtOpcode {
            category: 19,
            index: 101,
            name: None,
        }),
        (19, 102) => Some(ExtOpcode {
            category: 19,
            index: 102,
            name: None,
        }),
        (19, 103) => Some(ExtOpcode {
            category: 19,
            index: 103,
            name: None,
        }),
        (19, 104) => Some(ExtOpcode {
            category: 19,
            index: 104,
            name: None,
        }),
        (19, 106) => Some(ExtOpcode {
            category: 19,
            index: 106,
            name: Some("movie_play"),
        }),
        (19, 107) => Some(ExtOpcode {
            category: 19,
            index: 107,
            name: Some("msp_set_loop_sp_ep"),
        }),
        (19, 108) => Some(ExtOpcode {
            category: 19,
            index: 108,
            name: Some("msp_cls"),
        }),
        (19, 109) => Some(ExtOpcode {
            category: 19,
            index: 109,
            name: Some("msp_wait"),
        }),
        (19, 110) => Some(ExtOpcode {
            category: 19,
            index: 110,
            name: Some("msp_lock"),
        }),
        (19, 111) => Some(ExtOpcode {
            category: 19,
            index: 111,
            name: Some("msp_unlock"),
        }),
        (19, 112) => Some(ExtOpcode {
            category: 19,
            index: 112,
            name: Some("msp_play"),
        }),
        (19, 113) => Some(ExtOpcode {
            category: 19,
            index: 113,
            name: Some("msp_stop"),
        }),
        (19, 115) => Some(ExtOpcode {
            category: 19,
            index: 115,
            name: Some("create_thread"),
        }),
        (19, 116) => Some(ExtOpcode {
            category: 19,
            index: 116,
            name: Some("exit_thread"),
        }),
        (19, 117) => Some(ExtOpcode {
            category: 19,
            index: 117,
            name: None,
        }),
        (19, 118) => Some(ExtOpcode {
            category: 19,
            index: 118,
            name: Some("get_thread"),
        }),
        (19, 121) => Some(ExtOpcode {
            category: 19,
            index: 121,
            name: Some("mov"),
        }),
        (19, 122) => Some(ExtOpcode {
            category: 19,
            index: 122,
            name: Some("add"),
        }),
        (19, 123) => Some(ExtOpcode {
            category: 19,
            index: 123,
            name: Some("sub"),
        }),
        (19, 124) => Some(ExtOpcode {
            category: 19,
            index: 124,
            name: Some("mul"),
        }),
        (19, 125) => Some(ExtOpcode {
            category: 19,
            index: 125,
            name: Some("div"),
        }),
        (19, 126) => Some(ExtOpcode {
            category: 19,
            index: 126,
            name: Some("bitand"),
        }),
        (19, 127) => Some(ExtOpcode {
            category: 19,
            index: 127,
            name: Some("bitor"),
        }),
        (19, 128) => Some(ExtOpcode {
            category: 19,
            index: 128,
            name: Some("bitxor"),
        }),
        (19, 129) => Some(ExtOpcode {
            category: 19,
            index: 129,
            name: Some("jmp_point"),
        }),
        (19, 130) => Some(ExtOpcode {
            category: 19,
            index: 130,
            name: Some("jf_point"),
        }),
        (19, 131) => Some(ExtOpcode {
            category: 19,
            index: 131,
            name: Some("gosub_point"),
        }),
        (19, 132) => Some(ExtOpcode {
            category: 19,
            index: 132,
            name: Some("eq"),
        }),
        (19, 133) => Some(ExtOpcode {
            category: 19,
            index: 133,
            name: Some("ne"),
        }),
        (19, 134) => Some(ExtOpcode {
            category: 19,
            index: 134,
            name: Some("le"),
        }),
        (19, 135) => Some(ExtOpcode {
            category: 19,
            index: 135,
            name: Some("ge"),
        }),
        (19, 136) => Some(ExtOpcode {
            category: 19,
            index: 136,
            name: Some("lt"),
        }),
        (19, 137) => Some(ExtOpcode {
            category: 19,
            index: 137,
            name: Some("gt"),
        }),
        (19, 138) => Some(ExtOpcode {
            category: 19,
            index: 138,
            name: Some("lor"),
        }),
        (19, 139) => Some(ExtOpcode {
            category: 19,
            index: 139,
            name: Some("land"),
        }),
        (19, 140) => Some(ExtOpcode {
            category: 19,
            index: 140,
            name: Some("lnot_slot"),
        }),
        (19, 141) => Some(ExtOpcode {
            category: 19,
            index: 141,
            name: Some("end"),
        }),
        (19, 142) => Some(ExtOpcode {
            category: 19,
            index: 142,
            name: Some("nop"),
        }),
        (19, 143) => Some(ExtOpcode {
            category: 19,
            index: 143,
            name: Some("extcall"),
        }),
        (19, 144) => Some(ExtOpcode {
            category: 19,
            index: 144,
            name: Some("ret"),
        }),
        (19, 145) => Some(ExtOpcode {
            category: 19,
            index: 145,
            name: Some("reset_adv"),
        }),
        (19, 146) => Some(ExtOpcode {
            category: 19,
            index: 146,
            name: Some("mod"),
        }),
        (19, 147) => Some(ExtOpcode {
            category: 19,
            index: 147,
            name: Some("shl"),
        }),
        (19, 148) => Some(ExtOpcode {
            category: 19,
            index: 148,
            name: Some("shr"),
        }),
        (19, 149) => Some(ExtOpcode {
            category: 19,
            index: 149,
            name: Some("neg_slot"),
        }),
        (19, 150) => Some(ExtOpcode {
            category: 19,
            index: 150,
            name: Some("pop"),
        }),
        (19, 151) => Some(ExtOpcode {
            category: 19,
            index: 151,
            name: Some("push"),
        }),
        (19, 152) => Some(ExtOpcode {
            category: 19,
            index: 152,
            name: Some("pack_args"),
        }),
        (19, 153) => Some(ExtOpcode {
            category: 19,
            index: 153,
            name: Some("drop_args"),
        }),
        (19, 155) => Some(ExtOpcode {
            category: 19,
            index: 155,
            name: Some("create_message"),
        }),
        (19, 156) => Some(ExtOpcode {
            category: 19,
            index: 156,
            name: Some("get_message"),
        }),
        (19, 157) => Some(ExtOpcode {
            category: 19,
            index: 157,
            name: Some("get_message_param"),
        }),
        (19, 160) => Some(ExtOpcode {
            category: 19,
            index: 160,
            name: Some("save"),
        }),
        (19, 161) => Some(ExtOpcode {
            category: 19,
            index: 161,
            name: Some("load"),
        }),
        (19, 162) => Some(ExtOpcode {
            category: 19,
            index: 162,
            name: Some("save_set_title"),
        }),
        (19, 163) => Some(ExtOpcode {
            category: 19,
            index: 163,
            name: Some("save_data"),
        }),
        (19, 164) => Some(ExtOpcode {
            category: 19,
            index: 164,
            name: Some("save_set_thumbnail_size"),
        }),
        (19, 165) => Some(ExtOpcode {
            category: 19,
            index: 165,
            name: Some("thumbnail_set"),
        }),
        (19, 166) => Some(ExtOpcode {
            category: 19,
            index: 166,
            name: Some("savetitledraw"),
        }),
        (19, 167) => Some(ExtOpcode {
            category: 19,
            index: 167,
            name: Some("save_set_font_size"),
        }),
        (19, 168) => Some(ExtOpcode {
            category: 19,
            index: 168,
            name: Some("getsaveday"),
        }),
        (19, 169) => Some(ExtOpcode {
            category: 19,
            index: 169,
            name: Some("is_save"),
        }),
        (19, 170) => Some(ExtOpcode {
            category: 19,
            index: 170,
            name: Some("getsaveusermemory"),
        }),
        (19, 171) => Some(ExtOpcode {
            category: 19,
            index: 171,
            name: Some("savepoint"),
        }),
        (19, 172) => Some(ExtOpcode {
            category: 19,
            index: 172,
            name: Some("save_thumbnail_mosaic_set"),
        }),
        (19, 173) => Some(ExtOpcode {
            category: 19,
            index: 173,
            name: Some("savetimedraw"),
        }),
        (19, 174) => Some(ExtOpcode {
            category: 19,
            index: 174,
            name: Some("savedaydraw"),
        }),
        (19, 175) => Some(ExtOpcode {
            category: 19,
            index: 175,
            name: Some("save_set_text_rect"),
        }),
        (19, 176) => Some(ExtOpcode {
            category: 19,
            index: 176,
            name: Some("savetextdraw"),
        }),
        (19, 177) => Some(ExtOpcode {
            category: 19,
            index: 177,
            name: Some("get_new_savefile"),
        }),
        (19, 181) => Some(ExtOpcode {
            category: 19,
            index: 181,
            name: Some("setsavetext"),
        }),
        (19, 182) => Some(ExtOpcode {
            category: 19,
            index: 182,
            name: Some("thumbnail_renew"),
        }),
        (19, 183) => Some(ExtOpcode {
            category: 19,
            index: 183,
            name: Some("save_set_font_type"),
        }),
        (19, 184) => Some(ExtOpcode {
            category: 19,
            index: 184,
            name: Some("set_load_after_process"),
        }),
        (19, 185) => Some(ExtOpcode {
            category: 19,
            index: 185,
            name: Some("savesystemdata"),
        }),
        (19, 186) => Some(ExtOpcode {
            category: 19,
            index: 186,
            name: Some("save_set_font_effect"),
        }),
        (19, 187) => Some(ExtOpcode {
            category: 19,
            index: 187,
            name: Some("save_set_font_color_0x_0x"),
        }),
        (19, 188) => Some(ExtOpcode {
            category: 19,
            index: 188,
            name: Some("delete_file"),
        }),
        (19, 189) => Some(ExtOpcode {
            category: 19,
            index: 189,
            name: Some("save_tmp_dat"),
        }),
        (19, 190) => Some(ExtOpcode {
            category: 19,
            index: 190,
            name: Some("copy_file"),
        }),
        (19, 191) => Some(ExtOpcode {
            category: 19,
            index: 191,
            name: Some("load_thumbnail"),
        }),
        (19, 192) => Some(ExtOpcode {
            category: 19,
            index: 192,
            name: Some("save_lock_not_open_savefileno"),
        }),
        (19, 193) => Some(ExtOpcode {
            category: 19,
            index: 193,
            name: Some("is_save_lock"),
        }),
        (19, 194) => Some(ExtOpcode {
            category: 19,
            index: 194,
            name: Some("is_prev_data"),
        }),
        (19, 195) => Some(ExtOpcode {
            category: 19,
            index: 195,
            name: Some("save_point_clear"),
        }),
        (19, 196) => Some(ExtOpcode {
            category: 19,
            index: 196,
            name: Some("save_point_lock"),
        }),
        (19, 197) => Some(ExtOpcode {
            category: 19,
            index: 197,
            name: None,
        }),
        (19, 198) => Some(ExtOpcode {
            category: 19,
            index: 198,
            name: Some("histload"),
        }),
        (19, 200) => Some(ExtOpcode {
            category: 19,
            index: 200,
            name: Some("se_load"),
        }),
        (19, 201) => Some(ExtOpcode {
            category: 19,
            index: 201,
            name: Some("se_play"),
        }),
        (19, 202) => Some(ExtOpcode {
            category: 19,
            index: 202,
            name: Some("se_play_ex_ch"),
        }),
        (19, 203) => Some(ExtOpcode {
            category: 19,
            index: 203,
            name: Some("se_stop"),
        }),
        (19, 204) => Some(ExtOpcode {
            category: 19,
            index: 204,
            name: Some("se_set_volume"),
        }),
        (19, 205) => Some(ExtOpcode {
            category: 19,
            index: 205,
            name: Some("se_get_volume"),
        }),
        (19, 206) => Some(ExtOpcode {
            category: 19,
            index: 206,
            name: Some("se_unload"),
        }),
        (19, 207) => Some(ExtOpcode {
            category: 19,
            index: 207,
            name: Some("se_wait"),
        }),
        (19, 208) => Some(ExtOpcode {
            category: 19,
            index: 208,
            name: Some("channel_error_set_se_info"),
        }),
        (19, 209) => Some(ExtOpcode {
            category: 19,
            index: 209,
            name: Some("get_se_ex_volume"),
        }),
        (19, 210) => Some(ExtOpcode {
            category: 19,
            index: 210,
            name: Some("channel_error_set_se_ex_volume"),
        }),
        (19, 211) => Some(ExtOpcode {
            category: 19,
            index: 211,
            name: Some("channel_error_se_enable"),
        }),
        (19, 212) => Some(ExtOpcode {
            category: 19,
            index: 212,
            name: Some("channel_error_is_se_enable"),
        }),
        (19, 213) => Some(ExtOpcode {
            category: 19,
            index: 213,
            name: Some("se_set_pan"),
        }),
        (19, 214) => Some(ExtOpcode {
            category: 19,
            index: 214,
            name: Some("se_mute"),
        }),
        (19, 216) => Some(ExtOpcode {
            category: 19,
            index: 216,
            name: Some("select_init"),
        }),
        (19, 217) => Some(ExtOpcode {
            category: 19,
            index: 217,
            name: Some("select"),
        }),
        (19, 218) => Some(ExtOpcode {
            category: 19,
            index: 218,
            name: None,
        }),
        (19, 219) => Some(ExtOpcode {
            category: 19,
            index: 219,
            name: None,
        }),
        (19, 220) => Some(ExtOpcode {
            category: 19,
            index: 220,
            name: Some("select_clear"),
        }),
        (19, 221) => Some(ExtOpcode {
            category: 19,
            index: 221,
            name: Some("select_set_offset"),
        }),
        (19, 222) => Some(ExtOpcode {
            category: 19,
            index: 222,
            name: Some("select_set_process"),
        }),
        (19, 223) => Some(ExtOpcode {
            category: 19,
            index: 223,
            name: Some("select_lock"),
        }),
        (19, 224) => Some(ExtOpcode {
            category: 19,
            index: 224,
            name: Some("get_select_on_key"),
        }),
        (19, 225) => Some(ExtOpcode {
            category: 19,
            index: 225,
            name: Some("get_select_pull_key"),
        }),
        (19, 226) => Some(ExtOpcode {
            category: 19,
            index: 226,
            name: Some("get_select_push_key"),
        }),
        (19, 228) => Some(ExtOpcode {
            category: 19,
            index: 228,
            name: Some("skip_set"),
        }),
        (19, 229) => Some(ExtOpcode {
            category: 19,
            index: 229,
            name: Some("skip_is"),
        }),
        (19, 230) => Some(ExtOpcode {
            category: 19,
            index: 230,
            name: Some("auto_set"),
        }),
        (19, 231) => Some(ExtOpcode {
            category: 19,
            index: 231,
            name: Some("auto_is"),
        }),
        (19, 232) => Some(ExtOpcode {
            category: 19,
            index: 232,
            name: None,
        }),
        (19, 233) => Some(ExtOpcode {
            category: 19,
            index: 233,
            name: Some("auto_get_time"),
        }),
        (19, 234) => Some(ExtOpcode {
            category: 19,
            index: 234,
            name: None,
        }),
        (19, 235) => Some(ExtOpcode {
            category: 19,
            index: 235,
            name: None,
        }),
        (19, 236) => Some(ExtOpcode {
            category: 19,
            index: 236,
            name: None,
        }),
        (19, 237) => Some(ExtOpcode {
            category: 19,
            index: 237,
            name: None,
        }),
        (19, 238) => Some(ExtOpcode {
            category: 19,
            index: 238,
            name: None,
        }),
        (19, 239) => Some(ExtOpcode {
            category: 19,
            index: 239,
            name: None,
        }),
        (19, 240) => Some(ExtOpcode {
            category: 19,
            index: 240,
            name: None,
        }),
        (19, 241) => Some(ExtOpcode {
            category: 19,
            index: 241,
            name: None,
        }),
        (19, 242) => Some(ExtOpcode {
            category: 19,
            index: 242,
            name: None,
        }),
        (19, 243) => Some(ExtOpcode {
            category: 19,
            index: 243,
            name: Some("load_font"),
        }),
        (19, 244) => Some(ExtOpcode {
            category: 19,
            index: 244,
            name: Some("unload_font"),
        }),
        (19, 245) => Some(ExtOpcode {
            category: 19,
            index: 245,
            name: Some("set_language"),
        }),
        (19, 246) => Some(ExtOpcode {
            category: 19,
            index: 246,
            name: Some("key_canncel"),
        }),
        (19, 247) => Some(ExtOpcode {
            category: 19,
            index: 247,
            name: Some("set_font_color"),
        }),
        (19, 248) => Some(ExtOpcode {
            category: 19,
            index: 248,
            name: Some("load_font_ex"),
        }),
        (19, 249) => Some(ExtOpcode {
            category: 19,
            index: 249,
            name: None,
        }),
        (19, 250) => Some(ExtOpcode {
            category: 19,
            index: 250,
            name: None,
        }),
        (19, 251) => Some(ExtOpcode {
            category: 19,
            index: 251,
            name: None,
        }),
        (19, 252) => Some(ExtOpcode {
            category: 19,
            index: 252,
            name: None,
        }),
        (19, 253) => Some(ExtOpcode {
            category: 19,
            index: 253,
            name: None,
        }),
        (19, 254) => Some(ExtOpcode {
            category: 19,
            index: 254,
            name: None,
        }),
        (19, 255) => Some(ExtOpcode {
            category: 19,
            index: 255,
            name: Some("set_font_size"),
        }),
        (19, 256) => Some(ExtOpcode {
            category: 19,
            index: 256,
            name: Some("get_font_size"),
        }),
        (19, 257) => Some(ExtOpcode {
            category: 19,
            index: 257,
            name: Some("get_font_type"),
        }),
        (19, 258) => Some(ExtOpcode {
            category: 19,
            index: 258,
            name: Some("set_font_effect"),
        }),
        (19, 259) => Some(ExtOpcode {
            category: 19,
            index: 259,
            name: Some("get_font_effect"),
        }),
        (19, 260) => Some(ExtOpcode {
            category: 19,
            index: 260,
            name: Some("get_pull_key"),
        }),
        (19, 261) => Some(ExtOpcode {
            category: 19,
            index: 261,
            name: Some("get_on_key"),
        }),
        (19, 262) => Some(ExtOpcode {
            category: 19,
            index: 262,
            name: Some("get_push_key"),
        }),
        (19, 263) => Some(ExtOpcode {
            category: 19,
            index: 263,
            name: Some("input_clear"),
        }),
        (19, 264) => Some(ExtOpcode {
            category: 19,
            index: 264,
            name: Some("change_window_size"),
        }),
        (19, 265) => Some(ExtOpcode {
            category: 19,
            index: 265,
            name: Some("change_aspect_mode"),
        }),
        (19, 266) => Some(ExtOpcode {
            category: 19,
            index: 266,
            name: Some("aspect_position_enable"),
        }),
        (19, 267) => Some(ExtOpcode {
            category: 19,
            index: 267,
            name: None,
        }),
        (19, 268) => Some(ExtOpcode {
            category: 19,
            index: 268,
            name: Some("get_aspect_mode"),
        }),
        (19, 269) => Some(ExtOpcode {
            category: 19,
            index: 269,
            name: Some("get_monitor_size"),
        }),
        (19, 270) => Some(ExtOpcode {
            category: 19,
            index: 270,
            name: None,
        }),
        (19, 271) => Some(ExtOpcode {
            category: 19,
            index: 271,
            name: Some("get_system_metrics"),
        }),
        (19, 272) => Some(ExtOpcode {
            category: 19,
            index: 272,
            name: Some("set_system_path"),
        }),
        (19, 273) => Some(ExtOpcode {
            category: 19,
            index: 273,
            name: Some("set_allmosaicthumbnail"),
        }),
        (19, 274) => Some(ExtOpcode {
            category: 19,
            index: 274,
            name: Some("enable_window_change"),
        }),
        (19, 275) => Some(ExtOpcode {
            category: 19,
            index: 275,
            name: Some("is_enable_window_change"),
        }),
        (19, 276) => Some(ExtOpcode {
            category: 19,
            index: 276,
            name: Some("set_cursor_null"),
        }),
        (19, 277) => Some(ExtOpcode {
            category: 19,
            index: 277,
            name: Some("set_hide_cursor_time"),
        }),
        (19, 278) => Some(ExtOpcode {
            category: 19,
            index: 278,
            name: Some("get_hide_cursor_time"),
        }),
        (19, 279) => Some(ExtOpcode {
            category: 19,
            index: 279,
            name: Some("scene_skip"),
        }),
        (19, 280) => Some(ExtOpcode {
            category: 19,
            index: 280,
            name: None,
        }),
        (19, 281) => Some(ExtOpcode {
            category: 19,
            index: 281,
            name: None,
        }),
        (19, 282) => Some(ExtOpcode {
            category: 19,
            index: 282,
            name: Some("get_async_key"),
        }),
        (19, 283) => Some(ExtOpcode {
            category: 19,
            index: 283,
            name: Some("get_font_color"),
        }),
        (19, 284) => Some(ExtOpcode {
            category: 19,
            index: 284,
            name: None,
        }),
        (19, 285) => Some(ExtOpcode {
            category: 19,
            index: 285,
            name: Some("history_skip"),
        }),
        (19, 286) => Some(ExtOpcode {
            category: 19,
            index: 286,
            name: None,
        }),
        (19, 287) => Some(ExtOpcode {
            category: 19,
            index: 287,
            name: None,
        }),
        (19, 288) => Some(ExtOpcode {
            category: 19,
            index: 288,
            name: Some("set_language"),
        }),
        (19, 289) => Some(ExtOpcode {
            category: 19,
            index: 289,
            name: Some("set_achievement"),
        }),
        (19, 291) => Some(ExtOpcode {
            category: 19,
            index: 291,
            name: Some("system_btn_set"),
        }),
        (19, 292) => Some(ExtOpcode {
            category: 19,
            index: 292,
            name: Some("system_btn_release"),
        }),
        (19, 293) => Some(ExtOpcode {
            category: 19,
            index: 293,
            name: Some("system_btn_enable"),
        }),
        (19, 296) => Some(ExtOpcode {
            category: 19,
            index: 296,
            name: Some("text_init"),
        }),
        (19, 297) => Some(ExtOpcode {
            category: 19,
            index: 297,
            name: Some("text_set_icon"),
        }),
        (19, 298) => Some(ExtOpcode {
            category: 19,
            index: 298,
            name: Some("text"),
        }),
        (19, 299) => Some(ExtOpcode {
            category: 19,
            index: 299,
            name: Some("text_hide"),
        }),
        (20, 0) => Some(ExtOpcode {
            category: 20,
            index: 0,
            name: Some("random"),
        }),
        (20, 1) => Some(ExtOpcode {
            category: 20,
            index: 1,
            name: Some("abs"),
        }),
        (20, 2) => Some(ExtOpcode {
            category: 20,
            index: 2,
            name: Some("sin"),
        }),
        (20, 3) => Some(ExtOpcode {
            category: 20,
            index: 3,
            name: Some("cos"),
        }),
        (20, 4) => Some(ExtOpcode {
            category: 20,
            index: 4,
            name: Some("tan"),
        }),
        (20, 5) => Some(ExtOpcode {
            category: 20,
            index: 5,
            name: Some("atan"),
        }),
        (20, 6) => Some(ExtOpcode {
            category: 20,
            index: 6,
            name: Some("log"),
        }),
        (20, 7) => Some(ExtOpcode {
            category: 20,
            index: 7,
            name: Some("log10"),
        }),
        (20, 8) => Some(ExtOpcode {
            category: 20,
            index: 8,
            name: None,
        }),
        (20, 9) => Some(ExtOpcode {
            category: 20,
            index: 9,
            name: Some("sqrt"),
        }),
        (20, 10) => Some(ExtOpcode {
            category: 20,
            index: 10,
            name: None,
        }),
        (20, 11) => Some(ExtOpcode {
            category: 20,
            index: 11,
            name: None,
        }),
        (20, 15) => Some(ExtOpcode {
            category: 20,
            index: 15,
            name: Some("sp_set"),
        }),
        (20, 16) => Some(ExtOpcode {
            category: 20,
            index: 16,
            name: Some("sp_set_ex"),
        }),
        (20, 17) => Some(ExtOpcode {
            category: 20,
            index: 17,
            name: Some("sp_set_pos"),
        }),
        (20, 18) => Some(ExtOpcode {
            category: 20,
            index: 18,
            name: Some("sp_cls"),
        }),
        (20, 19) => Some(ExtOpcode {
            category: 20,
            index: 19,
            name: Some("sp_set_alpha"),
        }),
        (20, 20) => Some(ExtOpcode {
            category: 20,
            index: 20,
            name: Some("set_priority"),
        }),
        (20, 21) => Some(ExtOpcode {
            category: 20,
            index: 21,
            name: None,
        }),
        (20, 22) => Some(ExtOpcode {
            category: 20,
            index: 22,
            name: Some("sp_set_center"),
        }),
        (20, 24) => Some(ExtOpcode {
            category: 20,
            index: 24,
            name: Some("sp_cls_ex"),
        }),
        (20, 25) => Some(ExtOpcode {
            category: 20,
            index: 25,
            name: Some("set_filter"),
        }),
        (20, 26) => Some(ExtOpcode {
            category: 20,
            index: 26,
            name: Some("sp_cls_transition"),
        }),
        (20, 27) => Some(ExtOpcode {
            category: 20,
            index: 27,
            name: Some("sp_set_pos_ex"),
        }),
        (20, 28) => Some(ExtOpcode {
            category: 20,
            index: 28,
            name: Some("sp_set_rect_pos"),
        }),
        (20, 29) => Some(ExtOpcode {
            category: 20,
            index: 29,
            name: None,
        }),
        (20, 30) => Some(ExtOpcode {
            category: 20,
            index: 30,
            name: Some("sp_set_scale"),
        }),
        (20, 31) => Some(ExtOpcode {
            category: 20,
            index: 31,
            name: Some("sp_set_rotate"),
        }),
        (20, 32) => Some(ExtOpcode {
            category: 20,
            index: 32,
            name: Some("face_init"),
        }),
        (20, 33) => Some(ExtOpcode {
            category: 20,
            index: 33,
            name: Some("face_set"),
        }),
        (20, 34) => Some(ExtOpcode {
            category: 20,
            index: 34,
            name: Some("not_image_sp_get_color"),
        }),
        (20, 35) => Some(ExtOpcode {
            category: 20,
            index: 35,
            name: Some("sptext"),
        }),
        (20, 36) => Some(ExtOpcode {
            category: 20,
            index: 36,
            name: Some("face_cls"),
        }),
        (20, 37) => Some(ExtOpcode {
            category: 20,
            index: 37,
            name: Some("sp_set_rect"),
        }),
        (20, 38) => Some(ExtOpcode {
            category: 20,
            index: 38,
            name: Some("sp_set_pos_move"),
        }),
        (20, 39) => Some(ExtOpcode {
            category: 20,
            index: 39,
            name: Some("not_image_sp_get_alpha"),
        }),
        (20, 40) => Some(ExtOpcode {
            category: 20,
            index: 40,
            name: Some("not_image_sp_get_rotate"),
        }),
        (20, 41) => Some(ExtOpcode {
            category: 20,
            index: 41,
            name: None,
        }),
        (20, 42) => Some(ExtOpcode {
            category: 20,
            index: 42,
            name: None,
        }),
        (20, 43) => Some(ExtOpcode {
            category: 20,
            index: 43,
            name: None,
        }),
        (20, 44) => Some(ExtOpcode {
            category: 20,
            index: 44,
            name: None,
        }),
        (20, 45) => Some(ExtOpcode {
            category: 20,
            index: 45,
            name: Some("sp_create"),
        }),
        (20, 46) => Some(ExtOpcode {
            category: 20,
            index: 46,
            name: Some("sp_anime_clear"),
        }),
        (20, 47) => Some(ExtOpcode {
            category: 20,
            index: 47,
            name: None,
        }),
        (20, 48) => Some(ExtOpcode {
            category: 20,
            index: 48,
            name: None,
        }),
        (20, 49) => Some(ExtOpcode {
            category: 20,
            index: 49,
            name: Some("not_image_sp_get_scale"),
        }),
        (20, 50) => Some(ExtOpcode {
            category: 20,
            index: 50,
            name: Some("sp_set_color_0x"),
        }),
        (20, 51) => Some(ExtOpcode {
            category: 20,
            index: 51,
            name: Some("sp_bitblt"),
        }),
        (20, 52) => Some(ExtOpcode {
            category: 20,
            index: 52,
            name: Some("sp_set_shake"),
        }),
        (20, 53) => Some(ExtOpcode {
            category: 20,
            index: 53,
            name: Some("sp_paint"),
        }),
        (20, 54) => Some(ExtOpcode {
            category: 20,
            index: 54,
            name: None,
        }),
        (20, 55) => Some(ExtOpcode {
            category: 20,
            index: 55,
            name: Some("sp_load_wait_time"),
        }),
        (20, 56) => Some(ExtOpcode {
            category: 20,
            index: 56,
            name: Some("sp_draw"),
        }),
        (20, 57) => Some(ExtOpcode {
            category: 20,
            index: 57,
            name: None,
        }),
        (20, 58) => Some(ExtOpcode {
            category: 20,
            index: 58,
            name: Some("sp_unlock"),
        }),
        (20, 59) => Some(ExtOpcode {
            category: 20,
            index: 59,
            name: Some("sp_show"),
        }),
        (20, 60) => Some(ExtOpcode {
            category: 20,
            index: 60,
            name: Some("sp_hide"),
        }),
        (20, 61) => Some(ExtOpcode {
            category: 20,
            index: 61,
            name: None,
        }),
        (20, 62) => Some(ExtOpcode {
            category: 20,
            index: 62,
            name: Some("sp_set_child"),
        }),
        (20, 63) => Some(ExtOpcode {
            category: 20,
            index: 63,
            name: Some("sp_set_transition"),
        }),
        (20, 64) => Some(ExtOpcode {
            category: 20,
            index: 64,
            name: Some("sp_copy_image"),
        }),
        (20, 65) => Some(ExtOpcode {
            category: 20,
            index: 65,
            name: Some("sp_transition"),
        }),
        (20, 66) => Some(ExtOpcode {
            category: 20,
            index: 66,
            name: Some("set_aspect_position_type"),
        }),
        (20, 67) => Some(ExtOpcode {
            category: 20,
            index: 67,
            name: Some("get_backbuffer"),
        }),
        (20, 68) => Some(ExtOpcode {
            category: 20,
            index: 68,
            name: Some("sp_set_mask"),
        }),
        (20, 69) => Some(ExtOpcode {
            category: 20,
            index: 69,
            name: None,
        }),
        (20, 70) => Some(ExtOpcode {
            category: 20,
            index: 70,
            name: Some("spsetanime"),
        }),
        (20, 71) => Some(ExtOpcode {
            category: 20,
            index: 71,
            name: Some("drawtext"),
        }),
        (20, 72) => Some(ExtOpcode {
            category: 20,
            index: 72,
            name: None,
        }),
        (20, 73) => Some(ExtOpcode {
            category: 20,
            index: 73,
            name: None,
        }),
        (20, 75) => Some(ExtOpcode {
            category: 20,
            index: 75,
            name: Some("history_init_0x_0x"),
        }),
        (20, 76) => Some(ExtOpcode {
            category: 20,
            index: 76,
            name: Some("historybegin_lpbyte_ptagdata_sztext"),
        }),
        (20, 77) => Some(ExtOpcode {
            category: 20,
            index: 77,
            name: Some("history_end"),
        }),
        (20, 78) => Some(ExtOpcode {
            category: 20,
            index: 78,
            name: None,
        }),
        (20, 79) => Some(ExtOpcode {
            category: 20,
            index: 79,
            name: None,
        }),
        (20, 80) => Some(ExtOpcode {
            category: 20,
            index: 80,
            name: Some("history_get_height"),
        }),
        (20, 81) => Some(ExtOpcode {
            category: 20,
            index: 81,
            name: None,
        }),
        (20, 82) => Some(ExtOpcode {
            category: 20,
            index: 82,
            name: None,
        }),
        (20, 83) => Some(ExtOpcode {
            category: 20,
            index: 83,
            name: None,
        }),
        (20, 84) => Some(ExtOpcode {
            category: 20,
            index: 84,
            name: None,
        }),
        (20, 85) => Some(ExtOpcode {
            category: 20,
            index: 85,
            name: Some("history_set_rect"),
        }),
        (20, 86) => Some(ExtOpcode {
            category: 20,
            index: 86,
            name: Some("history_clear"),
        }),
        (20, 87) => Some(ExtOpcode {
            category: 20,
            index: 87,
            name: Some("history_set"),
        }),
        (20, 88) => Some(ExtOpcode {
            category: 20,
            index: 88,
            name: None,
        }),
        (20, 89) => Some(ExtOpcode {
            category: 20,
            index: 89,
            name: None,
        }),
        (20, 90) => Some(ExtOpcode {
            category: 20,
            index: 90,
            name: None,
        }),
        (20, 91) => Some(ExtOpcode {
            category: 20,
            index: 91,
            name: None,
        }),
        (20, 92) => Some(ExtOpcode {
            category: 20,
            index: 92,
            name: Some("history_set_face_call"),
        }),
        (20, 93) => Some(ExtOpcode {
            category: 20,
            index: 93,
            name: Some("history_set_face_sound"),
        }),
        (20, 94) => Some(ExtOpcode {
            category: 20,
            index: 94,
            name: Some("history_set_face_sound_release"),
        }),
        (20, 95) => Some(ExtOpcode {
            category: 20,
            index: 95,
            name: Some("history_get_text"),
        }),
        (20, 96) => Some(ExtOpcode {
            category: 20,
            index: 96,
            name: None,
        }),
        (20, 97) => Some(ExtOpcode {
            category: 20,
            index: 97,
            name: None,
        }),
        (20, 98) => Some(ExtOpcode {
            category: 20,
            index: 98,
            name: None,
        }),
        (20, 99) => Some(ExtOpcode {
            category: 20,
            index: 99,
            name: None,
        }),
        (20, 101) => Some(ExtOpcode {
            category: 20,
            index: 101,
            name: Some("movie_play"),
        }),
        (20, 102) => Some(ExtOpcode {
            category: 20,
            index: 102,
            name: Some("msp_set_loop_sp_ep"),
        }),
        (20, 103) => Some(ExtOpcode {
            category: 20,
            index: 103,
            name: Some("msp_cls"),
        }),
        (20, 104) => Some(ExtOpcode {
            category: 20,
            index: 104,
            name: Some("msp_wait"),
        }),
        (20, 105) => Some(ExtOpcode {
            category: 20,
            index: 105,
            name: Some("msp_lock"),
        }),
        (20, 106) => Some(ExtOpcode {
            category: 20,
            index: 106,
            name: Some("msp_unlock"),
        }),
        (20, 107) => Some(ExtOpcode {
            category: 20,
            index: 107,
            name: Some("msp_play"),
        }),
        (20, 108) => Some(ExtOpcode {
            category: 20,
            index: 108,
            name: Some("msp_stop"),
        }),
        (20, 110) => Some(ExtOpcode {
            category: 20,
            index: 110,
            name: Some("create_thread"),
        }),
        (20, 111) => Some(ExtOpcode {
            category: 20,
            index: 111,
            name: Some("exit_thread"),
        }),
        (20, 112) => Some(ExtOpcode {
            category: 20,
            index: 112,
            name: None,
        }),
        (20, 113) => Some(ExtOpcode {
            category: 20,
            index: 113,
            name: Some("get_thread"),
        }),
        (20, 116) => Some(ExtOpcode {
            category: 20,
            index: 116,
            name: Some("mov"),
        }),
        (20, 117) => Some(ExtOpcode {
            category: 20,
            index: 117,
            name: Some("add"),
        }),
        (20, 118) => Some(ExtOpcode {
            category: 20,
            index: 118,
            name: Some("sub"),
        }),
        (20, 119) => Some(ExtOpcode {
            category: 20,
            index: 119,
            name: Some("mul"),
        }),
        (20, 120) => Some(ExtOpcode {
            category: 20,
            index: 120,
            name: Some("div"),
        }),
        (20, 121) => Some(ExtOpcode {
            category: 20,
            index: 121,
            name: Some("bitand"),
        }),
        (20, 122) => Some(ExtOpcode {
            category: 20,
            index: 122,
            name: Some("bitor"),
        }),
        (20, 123) => Some(ExtOpcode {
            category: 20,
            index: 123,
            name: Some("bitxor"),
        }),
        (20, 124) => Some(ExtOpcode {
            category: 20,
            index: 124,
            name: Some("jmp_point"),
        }),
        (20, 125) => Some(ExtOpcode {
            category: 20,
            index: 125,
            name: Some("jf_point"),
        }),
        (20, 126) => Some(ExtOpcode {
            category: 20,
            index: 126,
            name: Some("gosub_point"),
        }),
        (20, 127) => Some(ExtOpcode {
            category: 20,
            index: 127,
            name: Some("eq"),
        }),
        (20, 128) => Some(ExtOpcode {
            category: 20,
            index: 128,
            name: Some("ne"),
        }),
        (20, 129) => Some(ExtOpcode {
            category: 20,
            index: 129,
            name: Some("le"),
        }),
        (20, 130) => Some(ExtOpcode {
            category: 20,
            index: 130,
            name: Some("ge"),
        }),
        (20, 131) => Some(ExtOpcode {
            category: 20,
            index: 131,
            name: Some("lt"),
        }),
        (20, 132) => Some(ExtOpcode {
            category: 20,
            index: 132,
            name: Some("gt"),
        }),
        (20, 133) => Some(ExtOpcode {
            category: 20,
            index: 133,
            name: Some("lor"),
        }),
        (20, 134) => Some(ExtOpcode {
            category: 20,
            index: 134,
            name: Some("land"),
        }),
        (20, 135) => Some(ExtOpcode {
            category: 20,
            index: 135,
            name: Some("lnot_slot"),
        }),
        (20, 136) => Some(ExtOpcode {
            category: 20,
            index: 136,
            name: Some("end"),
        }),
        (20, 137) => Some(ExtOpcode {
            category: 20,
            index: 137,
            name: Some("nop"),
        }),
        (20, 138) => Some(ExtOpcode {
            category: 20,
            index: 138,
            name: Some("extcall"),
        }),
        (20, 139) => Some(ExtOpcode {
            category: 20,
            index: 139,
            name: Some("ret"),
        }),
        (20, 140) => Some(ExtOpcode {
            category: 20,
            index: 140,
            name: Some("reset_adv"),
        }),
        (20, 141) => Some(ExtOpcode {
            category: 20,
            index: 141,
            name: Some("mod"),
        }),
        (20, 142) => Some(ExtOpcode {
            category: 20,
            index: 142,
            name: Some("shl"),
        }),
        (20, 143) => Some(ExtOpcode {
            category: 20,
            index: 143,
            name: Some("shr"),
        }),
        (20, 144) => Some(ExtOpcode {
            category: 20,
            index: 144,
            name: Some("neg_slot"),
        }),
        (20, 145) => Some(ExtOpcode {
            category: 20,
            index: 145,
            name: Some("pop"),
        }),
        (20, 146) => Some(ExtOpcode {
            category: 20,
            index: 146,
            name: Some("push"),
        }),
        (20, 147) => Some(ExtOpcode {
            category: 20,
            index: 147,
            name: Some("pack_args"),
        }),
        (20, 148) => Some(ExtOpcode {
            category: 20,
            index: 148,
            name: Some("drop_args"),
        }),
        (20, 150) => Some(ExtOpcode {
            category: 20,
            index: 150,
            name: Some("create_message"),
        }),
        (20, 151) => Some(ExtOpcode {
            category: 20,
            index: 151,
            name: Some("get_message"),
        }),
        (20, 152) => Some(ExtOpcode {
            category: 20,
            index: 152,
            name: Some("get_message_param"),
        }),
        (20, 155) => Some(ExtOpcode {
            category: 20,
            index: 155,
            name: Some("save"),
        }),
        (20, 156) => Some(ExtOpcode {
            category: 20,
            index: 156,
            name: Some("load"),
        }),
        (20, 157) => Some(ExtOpcode {
            category: 20,
            index: 157,
            name: Some("save_set_title"),
        }),
        (20, 158) => Some(ExtOpcode {
            category: 20,
            index: 158,
            name: Some("save_data"),
        }),
        (20, 159) => Some(ExtOpcode {
            category: 20,
            index: 159,
            name: Some("save_set_thumbnail_size"),
        }),
        (20, 160) => Some(ExtOpcode {
            category: 20,
            index: 160,
            name: Some("thumbnail_set"),
        }),
        (20, 161) => Some(ExtOpcode {
            category: 20,
            index: 161,
            name: Some("savetitledraw"),
        }),
        (20, 162) => Some(ExtOpcode {
            category: 20,
            index: 162,
            name: Some("save_set_font_size"),
        }),
        (20, 163) => Some(ExtOpcode {
            category: 20,
            index: 163,
            name: Some("getsaveday"),
        }),
        (20, 164) => Some(ExtOpcode {
            category: 20,
            index: 164,
            name: Some("is_save"),
        }),
        (20, 165) => Some(ExtOpcode {
            category: 20,
            index: 165,
            name: Some("getsaveusermemory"),
        }),
        (20, 166) => Some(ExtOpcode {
            category: 20,
            index: 166,
            name: Some("savepoint"),
        }),
        (20, 167) => Some(ExtOpcode {
            category: 20,
            index: 167,
            name: Some("save_thumbnail_mosaic_set"),
        }),
        (20, 168) => Some(ExtOpcode {
            category: 20,
            index: 168,
            name: Some("savetimedraw"),
        }),
        (20, 169) => Some(ExtOpcode {
            category: 20,
            index: 169,
            name: Some("savedaydraw"),
        }),
        (20, 170) => Some(ExtOpcode {
            category: 20,
            index: 170,
            name: Some("save_set_text_rect"),
        }),
        (20, 171) => Some(ExtOpcode {
            category: 20,
            index: 171,
            name: Some("savetextdraw"),
        }),
        (20, 172) => Some(ExtOpcode {
            category: 20,
            index: 172,
            name: Some("get_new_savefile"),
        }),
        (20, 176) => Some(ExtOpcode {
            category: 20,
            index: 176,
            name: Some("setsavetext"),
        }),
        (20, 177) => Some(ExtOpcode {
            category: 20,
            index: 177,
            name: Some("thumbnail_renew"),
        }),
        (20, 178) => Some(ExtOpcode {
            category: 20,
            index: 178,
            name: Some("save_set_font_type"),
        }),
        (20, 179) => Some(ExtOpcode {
            category: 20,
            index: 179,
            name: Some("set_load_after_process"),
        }),
        (20, 180) => Some(ExtOpcode {
            category: 20,
            index: 180,
            name: Some("savesystemdata"),
        }),
        (20, 181) => Some(ExtOpcode {
            category: 20,
            index: 181,
            name: Some("save_set_font_effect"),
        }),
        (20, 182) => Some(ExtOpcode {
            category: 20,
            index: 182,
            name: Some("save_set_font_color_0x_0x"),
        }),
        (20, 183) => Some(ExtOpcode {
            category: 20,
            index: 183,
            name: Some("delete_file"),
        }),
        (20, 184) => Some(ExtOpcode {
            category: 20,
            index: 184,
            name: Some("save_tmp_dat"),
        }),
        (20, 185) => Some(ExtOpcode {
            category: 20,
            index: 185,
            name: Some("copy_file"),
        }),
        (20, 186) => Some(ExtOpcode {
            category: 20,
            index: 186,
            name: Some("load_thumbnail"),
        }),
        (20, 187) => Some(ExtOpcode {
            category: 20,
            index: 187,
            name: Some("save_lock_not_open_savefileno"),
        }),
        (20, 188) => Some(ExtOpcode {
            category: 20,
            index: 188,
            name: Some("is_save_lock"),
        }),
        (20, 189) => Some(ExtOpcode {
            category: 20,
            index: 189,
            name: Some("is_prev_data"),
        }),
        (20, 190) => Some(ExtOpcode {
            category: 20,
            index: 190,
            name: Some("save_point_clear"),
        }),
        (20, 191) => Some(ExtOpcode {
            category: 20,
            index: 191,
            name: Some("save_point_lock"),
        }),
        (20, 192) => Some(ExtOpcode {
            category: 20,
            index: 192,
            name: None,
        }),
        (20, 193) => Some(ExtOpcode {
            category: 20,
            index: 193,
            name: Some("histload"),
        }),
        (20, 195) => Some(ExtOpcode {
            category: 20,
            index: 195,
            name: Some("se_load"),
        }),
        (20, 196) => Some(ExtOpcode {
            category: 20,
            index: 196,
            name: Some("se_play"),
        }),
        (20, 197) => Some(ExtOpcode {
            category: 20,
            index: 197,
            name: Some("se_play_ex_ch"),
        }),
        (20, 198) => Some(ExtOpcode {
            category: 20,
            index: 198,
            name: Some("se_stop"),
        }),
        (20, 199) => Some(ExtOpcode {
            category: 20,
            index: 199,
            name: Some("se_set_volume"),
        }),
        (20, 200) => Some(ExtOpcode {
            category: 20,
            index: 200,
            name: Some("se_get_volume"),
        }),
        (20, 201) => Some(ExtOpcode {
            category: 20,
            index: 201,
            name: Some("se_unload"),
        }),
        (20, 202) => Some(ExtOpcode {
            category: 20,
            index: 202,
            name: Some("se_wait"),
        }),
        (20, 203) => Some(ExtOpcode {
            category: 20,
            index: 203,
            name: Some("channel_error_set_se_info"),
        }),
        (20, 204) => Some(ExtOpcode {
            category: 20,
            index: 204,
            name: Some("get_se_ex_volume"),
        }),
        (20, 205) => Some(ExtOpcode {
            category: 20,
            index: 205,
            name: Some("channel_error_set_se_ex_volume"),
        }),
        (20, 206) => Some(ExtOpcode {
            category: 20,
            index: 206,
            name: Some("channel_error_se_enable"),
        }),
        (20, 207) => Some(ExtOpcode {
            category: 20,
            index: 207,
            name: Some("channel_error_is_se_enable"),
        }),
        (20, 208) => Some(ExtOpcode {
            category: 20,
            index: 208,
            name: Some("se_set_pan"),
        }),
        (20, 209) => Some(ExtOpcode {
            category: 20,
            index: 209,
            name: Some("se_mute"),
        }),
        (20, 211) => Some(ExtOpcode {
            category: 20,
            index: 211,
            name: Some("select_init"),
        }),
        (20, 212) => Some(ExtOpcode {
            category: 20,
            index: 212,
            name: Some("select"),
        }),
        (20, 213) => Some(ExtOpcode {
            category: 20,
            index: 213,
            name: None,
        }),
        (20, 214) => Some(ExtOpcode {
            category: 20,
            index: 214,
            name: None,
        }),
        (20, 215) => Some(ExtOpcode {
            category: 20,
            index: 215,
            name: Some("select_clear"),
        }),
        (20, 216) => Some(ExtOpcode {
            category: 20,
            index: 216,
            name: Some("select_set_offset"),
        }),
        (20, 217) => Some(ExtOpcode {
            category: 20,
            index: 217,
            name: Some("select_set_process"),
        }),
        (20, 218) => Some(ExtOpcode {
            category: 20,
            index: 218,
            name: Some("select_lock"),
        }),
        (20, 219) => Some(ExtOpcode {
            category: 20,
            index: 219,
            name: Some("get_select_on_key"),
        }),
        (20, 220) => Some(ExtOpcode {
            category: 20,
            index: 220,
            name: Some("get_select_pull_key"),
        }),
        (20, 221) => Some(ExtOpcode {
            category: 20,
            index: 221,
            name: Some("get_select_push_key"),
        }),
        (20, 223) => Some(ExtOpcode {
            category: 20,
            index: 223,
            name: Some("skip_set"),
        }),
        (20, 224) => Some(ExtOpcode {
            category: 20,
            index: 224,
            name: Some("skip_is"),
        }),
        (20, 225) => Some(ExtOpcode {
            category: 20,
            index: 225,
            name: Some("auto_set"),
        }),
        (20, 226) => Some(ExtOpcode {
            category: 20,
            index: 226,
            name: Some("auto_is"),
        }),
        (20, 227) => Some(ExtOpcode {
            category: 20,
            index: 227,
            name: None,
        }),
        (20, 228) => Some(ExtOpcode {
            category: 20,
            index: 228,
            name: Some("auto_get_time"),
        }),
        (20, 229) => Some(ExtOpcode {
            category: 20,
            index: 229,
            name: None,
        }),
        (20, 230) => Some(ExtOpcode {
            category: 20,
            index: 230,
            name: None,
        }),
        (20, 231) => Some(ExtOpcode {
            category: 20,
            index: 231,
            name: None,
        }),
        (20, 232) => Some(ExtOpcode {
            category: 20,
            index: 232,
            name: None,
        }),
        (20, 233) => Some(ExtOpcode {
            category: 20,
            index: 233,
            name: None,
        }),
        (20, 234) => Some(ExtOpcode {
            category: 20,
            index: 234,
            name: None,
        }),
        (20, 235) => Some(ExtOpcode {
            category: 20,
            index: 235,
            name: None,
        }),
        (20, 236) => Some(ExtOpcode {
            category: 20,
            index: 236,
            name: None,
        }),
        (20, 237) => Some(ExtOpcode {
            category: 20,
            index: 237,
            name: None,
        }),
        (20, 238) => Some(ExtOpcode {
            category: 20,
            index: 238,
            name: Some("load_font"),
        }),
        (20, 239) => Some(ExtOpcode {
            category: 20,
            index: 239,
            name: Some("unload_font"),
        }),
        (20, 240) => Some(ExtOpcode {
            category: 20,
            index: 240,
            name: Some("set_language"),
        }),
        (20, 241) => Some(ExtOpcode {
            category: 20,
            index: 241,
            name: Some("key_canncel"),
        }),
        (20, 242) => Some(ExtOpcode {
            category: 20,
            index: 242,
            name: Some("set_font_color"),
        }),
        (20, 243) => Some(ExtOpcode {
            category: 20,
            index: 243,
            name: Some("load_font_ex"),
        }),
        (20, 244) => Some(ExtOpcode {
            category: 20,
            index: 244,
            name: None,
        }),
        (20, 245) => Some(ExtOpcode {
            category: 20,
            index: 245,
            name: None,
        }),
        (20, 246) => Some(ExtOpcode {
            category: 20,
            index: 246,
            name: None,
        }),
        (20, 247) => Some(ExtOpcode {
            category: 20,
            index: 247,
            name: None,
        }),
        (20, 248) => Some(ExtOpcode {
            category: 20,
            index: 248,
            name: None,
        }),
        (20, 249) => Some(ExtOpcode {
            category: 20,
            index: 249,
            name: None,
        }),
        (20, 250) => Some(ExtOpcode {
            category: 20,
            index: 250,
            name: Some("set_font_size"),
        }),
        (20, 251) => Some(ExtOpcode {
            category: 20,
            index: 251,
            name: Some("get_font_size"),
        }),
        (20, 252) => Some(ExtOpcode {
            category: 20,
            index: 252,
            name: Some("get_font_type"),
        }),
        (20, 253) => Some(ExtOpcode {
            category: 20,
            index: 253,
            name: Some("set_font_effect"),
        }),
        (20, 254) => Some(ExtOpcode {
            category: 20,
            index: 254,
            name: Some("get_font_effect"),
        }),
        (20, 255) => Some(ExtOpcode {
            category: 20,
            index: 255,
            name: Some("get_pull_key"),
        }),
        (20, 256) => Some(ExtOpcode {
            category: 20,
            index: 256,
            name: Some("get_on_key"),
        }),
        (20, 257) => Some(ExtOpcode {
            category: 20,
            index: 257,
            name: Some("get_push_key"),
        }),
        (20, 258) => Some(ExtOpcode {
            category: 20,
            index: 258,
            name: Some("input_clear"),
        }),
        (20, 259) => Some(ExtOpcode {
            category: 20,
            index: 259,
            name: Some("change_window_size"),
        }),
        (20, 260) => Some(ExtOpcode {
            category: 20,
            index: 260,
            name: Some("change_aspect_mode"),
        }),
        (20, 261) => Some(ExtOpcode {
            category: 20,
            index: 261,
            name: Some("aspect_position_enable"),
        }),
        (20, 262) => Some(ExtOpcode {
            category: 20,
            index: 262,
            name: None,
        }),
        (20, 263) => Some(ExtOpcode {
            category: 20,
            index: 263,
            name: Some("get_aspect_mode"),
        }),
        (20, 264) => Some(ExtOpcode {
            category: 20,
            index: 264,
            name: Some("get_monitor_size"),
        }),
        (20, 265) => Some(ExtOpcode {
            category: 20,
            index: 265,
            name: None,
        }),
        (20, 266) => Some(ExtOpcode {
            category: 20,
            index: 266,
            name: Some("get_system_metrics"),
        }),
        (20, 267) => Some(ExtOpcode {
            category: 20,
            index: 267,
            name: Some("set_system_path"),
        }),
        (20, 268) => Some(ExtOpcode {
            category: 20,
            index: 268,
            name: Some("set_allmosaicthumbnail"),
        }),
        (20, 269) => Some(ExtOpcode {
            category: 20,
            index: 269,
            name: Some("enable_window_change"),
        }),
        (20, 270) => Some(ExtOpcode {
            category: 20,
            index: 270,
            name: Some("is_enable_window_change"),
        }),
        (20, 271) => Some(ExtOpcode {
            category: 20,
            index: 271,
            name: Some("set_cursor_null"),
        }),
        (20, 272) => Some(ExtOpcode {
            category: 20,
            index: 272,
            name: Some("set_hide_cursor_time"),
        }),
        (20, 273) => Some(ExtOpcode {
            category: 20,
            index: 273,
            name: Some("get_hide_cursor_time"),
        }),
        (20, 274) => Some(ExtOpcode {
            category: 20,
            index: 274,
            name: Some("scene_skip"),
        }),
        (20, 275) => Some(ExtOpcode {
            category: 20,
            index: 275,
            name: None,
        }),
        (20, 276) => Some(ExtOpcode {
            category: 20,
            index: 276,
            name: None,
        }),
        (20, 277) => Some(ExtOpcode {
            category: 20,
            index: 277,
            name: Some("get_async_key"),
        }),
        (20, 278) => Some(ExtOpcode {
            category: 20,
            index: 278,
            name: Some("get_font_color"),
        }),
        (20, 279) => Some(ExtOpcode {
            category: 20,
            index: 279,
            name: None,
        }),
        (20, 280) => Some(ExtOpcode {
            category: 20,
            index: 280,
            name: Some("history_skip"),
        }),
        (20, 281) => Some(ExtOpcode {
            category: 20,
            index: 281,
            name: None,
        }),
        (20, 282) => Some(ExtOpcode {
            category: 20,
            index: 282,
            name: None,
        }),
        (20, 283) => Some(ExtOpcode {
            category: 20,
            index: 283,
            name: Some("set_language"),
        }),
        (20, 284) => Some(ExtOpcode {
            category: 20,
            index: 284,
            name: Some("set_achievement"),
        }),
        (20, 286) => Some(ExtOpcode {
            category: 20,
            index: 286,
            name: Some("system_btn_set"),
        }),
        (20, 287) => Some(ExtOpcode {
            category: 20,
            index: 287,
            name: Some("system_btn_release"),
        }),
        (20, 288) => Some(ExtOpcode {
            category: 20,
            index: 288,
            name: Some("system_btn_enable"),
        }),
        (20, 291) => Some(ExtOpcode {
            category: 20,
            index: 291,
            name: Some("text_init"),
        }),
        (20, 292) => Some(ExtOpcode {
            category: 20,
            index: 292,
            name: Some("text_set_icon"),
        }),
        (20, 293) => Some(ExtOpcode {
            category: 20,
            index: 293,
            name: Some("text"),
        }),
        (20, 294) => Some(ExtOpcode {
            category: 20,
            index: 294,
            name: Some("text_hide"),
        }),
        (20, 295) => Some(ExtOpcode {
            category: 20,
            index: 295,
            name: Some("text_show"),
        }),
        (20, 296) => Some(ExtOpcode {
            category: 20,
            index: 296,
            name: Some("text_set_btn"),
        }),
        (20, 297) => Some(ExtOpcode {
            category: 20,
            index: 297,
            name: Some("text_uninit"),
        }),
        (20, 298) => Some(ExtOpcode {
            category: 20,
            index: 298,
            name: Some("text_set_rect_invalid_param"),
        }),
        (20, 299) => Some(ExtOpcode {
            category: 20,
            index: 299,
            name: Some("text_clear"),
        }),
        (21, 0) => Some(ExtOpcode {
            category: 21,
            index: 0,
            name: Some("create_thread"),
        }),
        (21, 1) => Some(ExtOpcode {
            category: 21,
            index: 1,
            name: Some("exit_thread"),
        }),
        (21, 2) => Some(ExtOpcode {
            category: 21,
            index: 2,
            name: None,
        }),
        (21, 3) => Some(ExtOpcode {
            category: 21,
            index: 3,
            name: Some("get_thread"),
        }),
        (21, 6) => Some(ExtOpcode {
            category: 21,
            index: 6,
            name: Some("mov"),
        }),
        (21, 7) => Some(ExtOpcode {
            category: 21,
            index: 7,
            name: Some("add"),
        }),
        (21, 8) => Some(ExtOpcode {
            category: 21,
            index: 8,
            name: Some("sub"),
        }),
        (21, 9) => Some(ExtOpcode {
            category: 21,
            index: 9,
            name: Some("mul"),
        }),
        (21, 10) => Some(ExtOpcode {
            category: 21,
            index: 10,
            name: Some("div"),
        }),
        (21, 11) => Some(ExtOpcode {
            category: 21,
            index: 11,
            name: Some("bitand"),
        }),
        (21, 12) => Some(ExtOpcode {
            category: 21,
            index: 12,
            name: Some("bitor"),
        }),
        (21, 13) => Some(ExtOpcode {
            category: 21,
            index: 13,
            name: Some("bitxor"),
        }),
        (21, 14) => Some(ExtOpcode {
            category: 21,
            index: 14,
            name: Some("jmp_point"),
        }),
        (21, 15) => Some(ExtOpcode {
            category: 21,
            index: 15,
            name: Some("jf_point"),
        }),
        (21, 16) => Some(ExtOpcode {
            category: 21,
            index: 16,
            name: Some("gosub_point"),
        }),
        (21, 17) => Some(ExtOpcode {
            category: 21,
            index: 17,
            name: Some("eq"),
        }),
        (21, 18) => Some(ExtOpcode {
            category: 21,
            index: 18,
            name: Some("ne"),
        }),
        (21, 19) => Some(ExtOpcode {
            category: 21,
            index: 19,
            name: Some("le"),
        }),
        (21, 20) => Some(ExtOpcode {
            category: 21,
            index: 20,
            name: Some("ge"),
        }),
        (21, 21) => Some(ExtOpcode {
            category: 21,
            index: 21,
            name: Some("lt"),
        }),
        (21, 22) => Some(ExtOpcode {
            category: 21,
            index: 22,
            name: Some("gt"),
        }),
        (21, 23) => Some(ExtOpcode {
            category: 21,
            index: 23,
            name: Some("lor"),
        }),
        (21, 24) => Some(ExtOpcode {
            category: 21,
            index: 24,
            name: Some("land"),
        }),
        (21, 25) => Some(ExtOpcode {
            category: 21,
            index: 25,
            name: Some("lnot_slot"),
        }),
        (21, 26) => Some(ExtOpcode {
            category: 21,
            index: 26,
            name: Some("end"),
        }),
        (21, 27) => Some(ExtOpcode {
            category: 21,
            index: 27,
            name: Some("nop"),
        }),
        (21, 28) => Some(ExtOpcode {
            category: 21,
            index: 28,
            name: Some("extcall"),
        }),
        (21, 29) => Some(ExtOpcode {
            category: 21,
            index: 29,
            name: Some("ret"),
        }),
        (21, 30) => Some(ExtOpcode {
            category: 21,
            index: 30,
            name: Some("reset_adv"),
        }),
        (21, 31) => Some(ExtOpcode {
            category: 21,
            index: 31,
            name: Some("mod"),
        }),
        (21, 32) => Some(ExtOpcode {
            category: 21,
            index: 32,
            name: Some("shl"),
        }),
        (21, 33) => Some(ExtOpcode {
            category: 21,
            index: 33,
            name: Some("shr"),
        }),
        (21, 34) => Some(ExtOpcode {
            category: 21,
            index: 34,
            name: Some("neg_slot"),
        }),
        (21, 35) => Some(ExtOpcode {
            category: 21,
            index: 35,
            name: Some("pop"),
        }),
        (21, 36) => Some(ExtOpcode {
            category: 21,
            index: 36,
            name: Some("push"),
        }),
        (21, 37) => Some(ExtOpcode {
            category: 21,
            index: 37,
            name: Some("pack_args"),
        }),
        (21, 38) => Some(ExtOpcode {
            category: 21,
            index: 38,
            name: Some("drop_args"),
        }),
        (21, 40) => Some(ExtOpcode {
            category: 21,
            index: 40,
            name: Some("create_message"),
        }),
        (21, 41) => Some(ExtOpcode {
            category: 21,
            index: 41,
            name: Some("get_message"),
        }),
        (21, 42) => Some(ExtOpcode {
            category: 21,
            index: 42,
            name: Some("get_message_param"),
        }),
        (21, 45) => Some(ExtOpcode {
            category: 21,
            index: 45,
            name: Some("save"),
        }),
        (21, 46) => Some(ExtOpcode {
            category: 21,
            index: 46,
            name: Some("load"),
        }),
        (21, 47) => Some(ExtOpcode {
            category: 21,
            index: 47,
            name: Some("save_set_title"),
        }),
        (21, 48) => Some(ExtOpcode {
            category: 21,
            index: 48,
            name: Some("save_data"),
        }),
        (21, 49) => Some(ExtOpcode {
            category: 21,
            index: 49,
            name: Some("save_set_thumbnail_size"),
        }),
        (21, 50) => Some(ExtOpcode {
            category: 21,
            index: 50,
            name: Some("thumbnail_set"),
        }),
        (21, 51) => Some(ExtOpcode {
            category: 21,
            index: 51,
            name: Some("savetitledraw"),
        }),
        (21, 52) => Some(ExtOpcode {
            category: 21,
            index: 52,
            name: Some("save_set_font_size"),
        }),
        (21, 53) => Some(ExtOpcode {
            category: 21,
            index: 53,
            name: Some("getsaveday"),
        }),
        (21, 54) => Some(ExtOpcode {
            category: 21,
            index: 54,
            name: Some("is_save"),
        }),
        (21, 55) => Some(ExtOpcode {
            category: 21,
            index: 55,
            name: Some("getsaveusermemory"),
        }),
        (21, 56) => Some(ExtOpcode {
            category: 21,
            index: 56,
            name: Some("savepoint"),
        }),
        (21, 57) => Some(ExtOpcode {
            category: 21,
            index: 57,
            name: Some("save_thumbnail_mosaic_set"),
        }),
        (21, 58) => Some(ExtOpcode {
            category: 21,
            index: 58,
            name: Some("savetimedraw"),
        }),
        (21, 59) => Some(ExtOpcode {
            category: 21,
            index: 59,
            name: Some("savedaydraw"),
        }),
        (21, 60) => Some(ExtOpcode {
            category: 21,
            index: 60,
            name: Some("save_set_text_rect"),
        }),
        (21, 61) => Some(ExtOpcode {
            category: 21,
            index: 61,
            name: Some("savetextdraw"),
        }),
        (21, 62) => Some(ExtOpcode {
            category: 21,
            index: 62,
            name: Some("get_new_savefile"),
        }),
        (21, 66) => Some(ExtOpcode {
            category: 21,
            index: 66,
            name: Some("setsavetext"),
        }),
        (21, 67) => Some(ExtOpcode {
            category: 21,
            index: 67,
            name: Some("thumbnail_renew"),
        }),
        (21, 68) => Some(ExtOpcode {
            category: 21,
            index: 68,
            name: Some("save_set_font_type"),
        }),
        (21, 69) => Some(ExtOpcode {
            category: 21,
            index: 69,
            name: Some("set_load_after_process"),
        }),
        (21, 70) => Some(ExtOpcode {
            category: 21,
            index: 70,
            name: Some("savesystemdata"),
        }),
        (21, 71) => Some(ExtOpcode {
            category: 21,
            index: 71,
            name: Some("save_set_font_effect"),
        }),
        (21, 72) => Some(ExtOpcode {
            category: 21,
            index: 72,
            name: Some("save_set_font_color_0x_0x"),
        }),
        (21, 73) => Some(ExtOpcode {
            category: 21,
            index: 73,
            name: Some("delete_file"),
        }),
        (21, 74) => Some(ExtOpcode {
            category: 21,
            index: 74,
            name: Some("save_tmp_dat"),
        }),
        (21, 75) => Some(ExtOpcode {
            category: 21,
            index: 75,
            name: Some("copy_file"),
        }),
        (21, 76) => Some(ExtOpcode {
            category: 21,
            index: 76,
            name: Some("load_thumbnail"),
        }),
        (21, 77) => Some(ExtOpcode {
            category: 21,
            index: 77,
            name: Some("save_lock_not_open_savefileno"),
        }),
        (21, 78) => Some(ExtOpcode {
            category: 21,
            index: 78,
            name: Some("is_save_lock"),
        }),
        (21, 79) => Some(ExtOpcode {
            category: 21,
            index: 79,
            name: Some("is_prev_data"),
        }),
        (21, 80) => Some(ExtOpcode {
            category: 21,
            index: 80,
            name: Some("save_point_clear"),
        }),
        (21, 81) => Some(ExtOpcode {
            category: 21,
            index: 81,
            name: Some("save_point_lock"),
        }),
        (21, 82) => Some(ExtOpcode {
            category: 21,
            index: 82,
            name: None,
        }),
        (21, 83) => Some(ExtOpcode {
            category: 21,
            index: 83,
            name: Some("histload"),
        }),
        (21, 85) => Some(ExtOpcode {
            category: 21,
            index: 85,
            name: Some("se_load"),
        }),
        (21, 86) => Some(ExtOpcode {
            category: 21,
            index: 86,
            name: Some("se_play"),
        }),
        (21, 87) => Some(ExtOpcode {
            category: 21,
            index: 87,
            name: Some("se_play_ex_ch"),
        }),
        (21, 88) => Some(ExtOpcode {
            category: 21,
            index: 88,
            name: Some("se_stop"),
        }),
        (21, 89) => Some(ExtOpcode {
            category: 21,
            index: 89,
            name: Some("se_set_volume"),
        }),
        (21, 90) => Some(ExtOpcode {
            category: 21,
            index: 90,
            name: Some("se_get_volume"),
        }),
        (21, 91) => Some(ExtOpcode {
            category: 21,
            index: 91,
            name: Some("se_unload"),
        }),
        (21, 92) => Some(ExtOpcode {
            category: 21,
            index: 92,
            name: Some("se_wait"),
        }),
        (21, 93) => Some(ExtOpcode {
            category: 21,
            index: 93,
            name: Some("channel_error_set_se_info"),
        }),
        (21, 94) => Some(ExtOpcode {
            category: 21,
            index: 94,
            name: Some("get_se_ex_volume"),
        }),
        (21, 95) => Some(ExtOpcode {
            category: 21,
            index: 95,
            name: Some("channel_error_set_se_ex_volume"),
        }),
        (21, 96) => Some(ExtOpcode {
            category: 21,
            index: 96,
            name: Some("channel_error_se_enable"),
        }),
        (21, 97) => Some(ExtOpcode {
            category: 21,
            index: 97,
            name: Some("channel_error_is_se_enable"),
        }),
        (21, 98) => Some(ExtOpcode {
            category: 21,
            index: 98,
            name: Some("se_set_pan"),
        }),
        (21, 99) => Some(ExtOpcode {
            category: 21,
            index: 99,
            name: Some("se_mute"),
        }),
        (21, 101) => Some(ExtOpcode {
            category: 21,
            index: 101,
            name: Some("select_init"),
        }),
        (21, 102) => Some(ExtOpcode {
            category: 21,
            index: 102,
            name: Some("select"),
        }),
        (21, 103) => Some(ExtOpcode {
            category: 21,
            index: 103,
            name: None,
        }),
        (21, 104) => Some(ExtOpcode {
            category: 21,
            index: 104,
            name: None,
        }),
        (21, 105) => Some(ExtOpcode {
            category: 21,
            index: 105,
            name: Some("select_clear"),
        }),
        (21, 106) => Some(ExtOpcode {
            category: 21,
            index: 106,
            name: Some("select_set_offset"),
        }),
        (21, 107) => Some(ExtOpcode {
            category: 21,
            index: 107,
            name: Some("select_set_process"),
        }),
        (21, 108) => Some(ExtOpcode {
            category: 21,
            index: 108,
            name: Some("select_lock"),
        }),
        (21, 109) => Some(ExtOpcode {
            category: 21,
            index: 109,
            name: Some("get_select_on_key"),
        }),
        (21, 110) => Some(ExtOpcode {
            category: 21,
            index: 110,
            name: Some("get_select_pull_key"),
        }),
        (21, 111) => Some(ExtOpcode {
            category: 21,
            index: 111,
            name: Some("get_select_push_key"),
        }),
        (21, 113) => Some(ExtOpcode {
            category: 21,
            index: 113,
            name: Some("skip_set"),
        }),
        (21, 114) => Some(ExtOpcode {
            category: 21,
            index: 114,
            name: Some("skip_is"),
        }),
        (21, 115) => Some(ExtOpcode {
            category: 21,
            index: 115,
            name: Some("auto_set"),
        }),
        (21, 116) => Some(ExtOpcode {
            category: 21,
            index: 116,
            name: Some("auto_is"),
        }),
        (21, 117) => Some(ExtOpcode {
            category: 21,
            index: 117,
            name: None,
        }),
        (21, 118) => Some(ExtOpcode {
            category: 21,
            index: 118,
            name: Some("auto_get_time"),
        }),
        (21, 119) => Some(ExtOpcode {
            category: 21,
            index: 119,
            name: None,
        }),
        (21, 120) => Some(ExtOpcode {
            category: 21,
            index: 120,
            name: None,
        }),
        (21, 121) => Some(ExtOpcode {
            category: 21,
            index: 121,
            name: None,
        }),
        (21, 122) => Some(ExtOpcode {
            category: 21,
            index: 122,
            name: None,
        }),
        (21, 123) => Some(ExtOpcode {
            category: 21,
            index: 123,
            name: None,
        }),
        (21, 124) => Some(ExtOpcode {
            category: 21,
            index: 124,
            name: None,
        }),
        (21, 125) => Some(ExtOpcode {
            category: 21,
            index: 125,
            name: None,
        }),
        (21, 126) => Some(ExtOpcode {
            category: 21,
            index: 126,
            name: None,
        }),
        (21, 127) => Some(ExtOpcode {
            category: 21,
            index: 127,
            name: None,
        }),
        (21, 128) => Some(ExtOpcode {
            category: 21,
            index: 128,
            name: Some("load_font"),
        }),
        (21, 129) => Some(ExtOpcode {
            category: 21,
            index: 129,
            name: Some("unload_font"),
        }),
        (21, 130) => Some(ExtOpcode {
            category: 21,
            index: 130,
            name: Some("set_language"),
        }),
        (21, 131) => Some(ExtOpcode {
            category: 21,
            index: 131,
            name: Some("key_canncel"),
        }),
        (21, 132) => Some(ExtOpcode {
            category: 21,
            index: 132,
            name: Some("set_font_color"),
        }),
        (21, 133) => Some(ExtOpcode {
            category: 21,
            index: 133,
            name: Some("load_font_ex"),
        }),
        (21, 134) => Some(ExtOpcode {
            category: 21,
            index: 134,
            name: None,
        }),
        (21, 135) => Some(ExtOpcode {
            category: 21,
            index: 135,
            name: None,
        }),
        (21, 136) => Some(ExtOpcode {
            category: 21,
            index: 136,
            name: None,
        }),
        (21, 137) => Some(ExtOpcode {
            category: 21,
            index: 137,
            name: None,
        }),
        (21, 138) => Some(ExtOpcode {
            category: 21,
            index: 138,
            name: None,
        }),
        (21, 139) => Some(ExtOpcode {
            category: 21,
            index: 139,
            name: None,
        }),
        (21, 140) => Some(ExtOpcode {
            category: 21,
            index: 140,
            name: Some("set_font_size"),
        }),
        (21, 141) => Some(ExtOpcode {
            category: 21,
            index: 141,
            name: Some("get_font_size"),
        }),
        (21, 142) => Some(ExtOpcode {
            category: 21,
            index: 142,
            name: Some("get_font_type"),
        }),
        (21, 143) => Some(ExtOpcode {
            category: 21,
            index: 143,
            name: Some("set_font_effect"),
        }),
        (21, 144) => Some(ExtOpcode {
            category: 21,
            index: 144,
            name: Some("get_font_effect"),
        }),
        (21, 145) => Some(ExtOpcode {
            category: 21,
            index: 145,
            name: Some("get_pull_key"),
        }),
        (21, 146) => Some(ExtOpcode {
            category: 21,
            index: 146,
            name: Some("get_on_key"),
        }),
        (21, 147) => Some(ExtOpcode {
            category: 21,
            index: 147,
            name: Some("get_push_key"),
        }),
        (21, 148) => Some(ExtOpcode {
            category: 21,
            index: 148,
            name: Some("input_clear"),
        }),
        (21, 149) => Some(ExtOpcode {
            category: 21,
            index: 149,
            name: Some("change_window_size"),
        }),
        (21, 150) => Some(ExtOpcode {
            category: 21,
            index: 150,
            name: Some("change_aspect_mode"),
        }),
        (21, 151) => Some(ExtOpcode {
            category: 21,
            index: 151,
            name: Some("aspect_position_enable"),
        }),
        (21, 152) => Some(ExtOpcode {
            category: 21,
            index: 152,
            name: None,
        }),
        (21, 153) => Some(ExtOpcode {
            category: 21,
            index: 153,
            name: Some("get_aspect_mode"),
        }),
        (21, 154) => Some(ExtOpcode {
            category: 21,
            index: 154,
            name: Some("get_monitor_size"),
        }),
        (21, 155) => Some(ExtOpcode {
            category: 21,
            index: 155,
            name: None,
        }),
        (21, 156) => Some(ExtOpcode {
            category: 21,
            index: 156,
            name: Some("get_system_metrics"),
        }),
        (21, 157) => Some(ExtOpcode {
            category: 21,
            index: 157,
            name: Some("set_system_path"),
        }),
        (21, 158) => Some(ExtOpcode {
            category: 21,
            index: 158,
            name: Some("set_allmosaicthumbnail"),
        }),
        (21, 159) => Some(ExtOpcode {
            category: 21,
            index: 159,
            name: Some("enable_window_change"),
        }),
        (21, 160) => Some(ExtOpcode {
            category: 21,
            index: 160,
            name: Some("is_enable_window_change"),
        }),
        (21, 161) => Some(ExtOpcode {
            category: 21,
            index: 161,
            name: Some("set_cursor_null"),
        }),
        (21, 162) => Some(ExtOpcode {
            category: 21,
            index: 162,
            name: Some("set_hide_cursor_time"),
        }),
        (21, 163) => Some(ExtOpcode {
            category: 21,
            index: 163,
            name: Some("get_hide_cursor_time"),
        }),
        (21, 164) => Some(ExtOpcode {
            category: 21,
            index: 164,
            name: Some("scene_skip"),
        }),
        (21, 165) => Some(ExtOpcode {
            category: 21,
            index: 165,
            name: None,
        }),
        (21, 166) => Some(ExtOpcode {
            category: 21,
            index: 166,
            name: None,
        }),
        (21, 167) => Some(ExtOpcode {
            category: 21,
            index: 167,
            name: Some("get_async_key"),
        }),
        (21, 168) => Some(ExtOpcode {
            category: 21,
            index: 168,
            name: Some("get_font_color"),
        }),
        (21, 169) => Some(ExtOpcode {
            category: 21,
            index: 169,
            name: None,
        }),
        (21, 170) => Some(ExtOpcode {
            category: 21,
            index: 170,
            name: Some("history_skip"),
        }),
        (21, 171) => Some(ExtOpcode {
            category: 21,
            index: 171,
            name: None,
        }),
        (21, 172) => Some(ExtOpcode {
            category: 21,
            index: 172,
            name: None,
        }),
        (21, 173) => Some(ExtOpcode {
            category: 21,
            index: 173,
            name: Some("set_language"),
        }),
        (21, 174) => Some(ExtOpcode {
            category: 21,
            index: 174,
            name: Some("set_achievement"),
        }),
        (21, 176) => Some(ExtOpcode {
            category: 21,
            index: 176,
            name: Some("system_btn_set"),
        }),
        (21, 177) => Some(ExtOpcode {
            category: 21,
            index: 177,
            name: Some("system_btn_release"),
        }),
        (21, 178) => Some(ExtOpcode {
            category: 21,
            index: 178,
            name: Some("system_btn_enable"),
        }),
        (21, 181) => Some(ExtOpcode {
            category: 21,
            index: 181,
            name: Some("text_init"),
        }),
        (21, 182) => Some(ExtOpcode {
            category: 21,
            index: 182,
            name: Some("text_set_icon"),
        }),
        (21, 183) => Some(ExtOpcode {
            category: 21,
            index: 183,
            name: Some("text"),
        }),
        (21, 184) => Some(ExtOpcode {
            category: 21,
            index: 184,
            name: Some("text_hide"),
        }),
        (21, 185) => Some(ExtOpcode {
            category: 21,
            index: 185,
            name: Some("text_show"),
        }),
        (21, 186) => Some(ExtOpcode {
            category: 21,
            index: 186,
            name: Some("text_set_btn"),
        }),
        (21, 187) => Some(ExtOpcode {
            category: 21,
            index: 187,
            name: Some("text_uninit"),
        }),
        (21, 188) => Some(ExtOpcode {
            category: 21,
            index: 188,
            name: Some("text_set_rect_invalid_param"),
        }),
        (21, 189) => Some(ExtOpcode {
            category: 21,
            index: 189,
            name: Some("text_clear"),
        }),
        (21, 190) => Some(ExtOpcode {
            category: 21,
            index: 190,
            name: None,
        }),
        (21, 191) => Some(ExtOpcode {
            category: 21,
            index: 191,
            name: Some("text_get_time"),
        }),
        (21, 192) => Some(ExtOpcode {
            category: 21,
            index: 192,
            name: Some("text_window_set_alpha"),
        }),
        (21, 193) => Some(ExtOpcode {
            category: 21,
            index: 193,
            name: Some("text_voice_play"),
        }),
        (21, 194) => Some(ExtOpcode {
            category: 21,
            index: 194,
            name: None,
        }),
        (21, 195) => Some(ExtOpcode {
            category: 21,
            index: 195,
            name: Some("text_set_icon_animation_time"),
        }),
        (21, 196) => Some(ExtOpcode {
            category: 21,
            index: 196,
            name: Some("text_w"),
        }),
        (21, 197) => Some(ExtOpcode {
            category: 21,
            index: 197,
            name: Some("text_a"),
        }),
        (21, 198) => Some(ExtOpcode {
            category: 21,
            index: 198,
            name: Some("text_wa"),
        }),
        (21, 199) => Some(ExtOpcode {
            category: 21,
            index: 199,
            name: Some("text_n"),
        }),
        (21, 200) => Some(ExtOpcode {
            category: 21,
            index: 200,
            name: Some("text_cat"),
        }),
        (21, 201) => Some(ExtOpcode {
            category: 21,
            index: 201,
            name: Some("set_history"),
        }),
        (21, 202) => Some(ExtOpcode {
            category: 21,
            index: 202,
            name: Some("is_text_visible"),
        }),
        (21, 203) => Some(ExtOpcode {
            category: 21,
            index: 203,
            name: Some("text_set_base"),
        }),
        (21, 204) => Some(ExtOpcode {
            category: 21,
            index: 204,
            name: Some("enable_voice_cut"),
        }),
        (21, 205) => Some(ExtOpcode {
            category: 21,
            index: 205,
            name: Some("is_voice_cut"),
        }),
        (21, 206) => Some(ExtOpcode {
            category: 21,
            index: 206,
            name: Some("texttimecheckset"),
        }),
        (21, 207) => Some(ExtOpcode {
            category: 21,
            index: 207,
            name: None,
        }),
        (21, 208) => Some(ExtOpcode {
            category: 21,
            index: 208,
            name: None,
        }),
        (21, 209) => Some(ExtOpcode {
            category: 21,
            index: 209,
            name: Some("text_set_color"),
        }),
        (21, 210) => Some(ExtOpcode {
            category: 21,
            index: 210,
            name: Some("textredraw"),
        }),
        (21, 211) => Some(ExtOpcode {
            category: 21,
            index: 211,
            name: Some("set_text_mode"),
        }),
        (21, 212) => Some(ExtOpcode {
            category: 21,
            index: 212,
            name: Some("text_init_visualnovelmode"),
        }),
        (21, 213) => Some(ExtOpcode {
            category: 21,
            index: 213,
            name: Some("text_set_icon_mode"),
        }),
        (21, 214) => Some(ExtOpcode {
            category: 21,
            index: 214,
            name: Some("text_vn_br"),
        }),
        (21, 215) => Some(ExtOpcode {
            category: 21,
            index: 215,
            name: None,
        }),
        (21, 216) => Some(ExtOpcode {
            category: 21,
            index: 216,
            name: None,
        }),
        (21, 217) => Some(ExtOpcode {
            category: 21,
            index: 217,
            name: None,
        }),
        (21, 218) => Some(ExtOpcode {
            category: 21,
            index: 218,
            name: None,
        }),
        (21, 219) => Some(ExtOpcode {
            category: 21,
            index: 219,
            name: Some("tips_get_str"),
        }),
        (21, 220) => Some(ExtOpcode {
            category: 21,
            index: 220,
            name: Some("tips_get_param"),
        }),
        (21, 221) => Some(ExtOpcode {
            category: 21,
            index: 221,
            name: Some("tips_reset"),
        }),
        (21, 222) => Some(ExtOpcode {
            category: 21,
            index: 222,
            name: Some("tips_search"),
        }),
        (21, 223) => Some(ExtOpcode {
            category: 21,
            index: 223,
            name: Some("tips_set_color"),
        }),
        (21, 224) => Some(ExtOpcode {
            category: 21,
            index: 224,
            name: Some("tips_stop"),
        }),
        (21, 225) => Some(ExtOpcode {
            category: 21,
            index: 225,
            name: Some("tips_get_flag"),
        }),
        (21, 226) => Some(ExtOpcode {
            category: 21,
            index: 226,
            name: Some("tips_init"),
        }),
        (21, 227) => Some(ExtOpcode {
            category: 21,
            index: 227,
            name: Some("tips_pause"),
        }),
        (21, 229) => Some(ExtOpcode {
            category: 21,
            index: 229,
            name: Some("voice_play"),
        }),
        (21, 230) => Some(ExtOpcode {
            category: 21,
            index: 230,
            name: Some("voice_stop"),
        }),
        (21, 231) => Some(ExtOpcode {
            category: 21,
            index: 231,
            name: Some("voice_set_volume"),
        }),
        (21, 232) => Some(ExtOpcode {
            category: 21,
            index: 232,
            name: Some("voice_get_volume"),
        }),
        (21, 233) => Some(ExtOpcode {
            category: 21,
            index: 233,
            name: Some("set_voice_info"),
        }),
        (21, 234) => Some(ExtOpcode {
            category: 21,
            index: 234,
            name: Some("voice_enable"),
        }),
        (21, 235) => Some(ExtOpcode {
            category: 21,
            index: 235,
            name: Some("is_voice_enable"),
        }),
        (21, 236) => Some(ExtOpcode {
            category: 21,
            index: 236,
            name: None,
        }),
        (21, 237) => Some(ExtOpcode {
            category: 21,
            index: 237,
            name: Some("bgv_play"),
        }),
        (21, 238) => Some(ExtOpcode {
            category: 21,
            index: 238,
            name: Some("bgv_stop"),
        }),
        (21, 239) => Some(ExtOpcode {
            category: 21,
            index: 239,
            name: Some("bgv_enable"),
        }),
        (21, 240) => Some(ExtOpcode {
            category: 21,
            index: 240,
            name: Some("get_voice_ex_volume"),
        }),
        (21, 241) => Some(ExtOpcode {
            category: 21,
            index: 241,
            name: Some("set_voice_ex_volume"),
        }),
        (21, 242) => Some(ExtOpcode {
            category: 21,
            index: 242,
            name: Some("voice_check_enable"),
        }),
        (21, 243) => Some(ExtOpcode {
            category: 21,
            index: 243,
            name: Some("voice_autopan_initialize"),
        }),
        (21, 244) => Some(ExtOpcode {
            category: 21,
            index: 244,
            name: Some("voice_autopan_enable"),
        }),
        (21, 245) => Some(ExtOpcode {
            category: 21,
            index: 245,
            name: Some("set_voice_autopan_size_over"),
        }),
        (21, 246) => Some(ExtOpcode {
            category: 21,
            index: 246,
            name: Some("is_voice_autopan_enable"),
        }),
        (21, 247) => Some(ExtOpcode {
            category: 21,
            index: 247,
            name: Some("voice_wait"),
        }),
        (21, 248) => Some(ExtOpcode {
            category: 21,
            index: 248,
            name: Some("bgv_pause"),
        }),
        (21, 249) => Some(ExtOpcode {
            category: 21,
            index: 249,
            name: Some("bgv_mute"),
        }),
        (21, 250) => Some(ExtOpcode {
            category: 21,
            index: 250,
            name: Some("set_bgv_volume"),
        }),
        (21, 251) => Some(ExtOpcode {
            category: 21,
            index: 251,
            name: Some("get_bgv_volume"),
        }),
        (21, 252) => Some(ExtOpcode {
            category: 21,
            index: 252,
            name: Some("set_bgv_auto_volume"),
        }),
        (21, 253) => Some(ExtOpcode {
            category: 21,
            index: 253,
            name: Some("voice_mute"),
        }),
        (21, 254) => Some(ExtOpcode {
            category: 21,
            index: 254,
            name: Some("voice_call"),
        }),
        (21, 255) => Some(ExtOpcode {
            category: 21,
            index: 255,
            name: Some("voice_call_clear"),
        }),
        (21, 257) => Some(ExtOpcode {
            category: 21,
            index: 257,
            name: Some("wait"),
        }),
        (21, 258) => Some(ExtOpcode {
            category: 21,
            index: 258,
            name: Some("wait_click"),
        }),
        (21, 259) => Some(ExtOpcode {
            category: 21,
            index: 259,
            name: Some("wait_sync_begin"),
        }),
        (21, 260) => Some(ExtOpcode {
            category: 21,
            index: 260,
            name: Some("wait_sync_release"),
        }),
        (21, 261) => Some(ExtOpcode {
            category: 21,
            index: 261,
            name: Some("wait_sync_end"),
        }),
        (21, 262) => Some(ExtOpcode {
            category: 21,
            index: 262,
            name: None,
        }),
        (21, 263) => Some(ExtOpcode {
            category: 21,
            index: 263,
            name: Some("wait_clear"),
        }),
        (21, 264) => Some(ExtOpcode {
            category: 21,
            index: 264,
            name: Some("wait_click_no_anim"),
        }),
        (21, 265) => Some(ExtOpcode {
            category: 21,
            index: 265,
            name: Some("wait_sync_get_time"),
        }),
        (21, 266) => Some(ExtOpcode {
            category: 21,
            index: 266,
            name: Some("wait_time_push"),
        }),
        (21, 267) => Some(ExtOpcode {
            category: 21,
            index: 267,
            name: Some("wait_time_pop"),
        }),
        (22, 0) => Some(ExtOpcode {
            category: 22,
            index: 0,
            name: None,
        }),
        (22, 1) => Some(ExtOpcode {
            category: 22,
            index: 1,
            name: Some("run_no_wait"),
        }),
        (22, 2) => Some(ExtOpcode {
            category: 22,
            index: 2,
            name: Some("run_stack"),
        }),
        (22, 4) => Some(ExtOpcode {
            category: 22,
            index: 4,
            name: Some("fx_effect_cls"),
        }),
        (22, 5) => Some(ExtOpcode {
            category: 22,
            index: 5,
            name: Some("fx_raster_stop"),
        }),
        (22, 6) => Some(ExtOpcode {
            category: 22,
            index: 6,
            name: Some("fx_effect_wait"),
        }),
        (22, 7) => Some(ExtOpcode {
            category: 22,
            index: 7,
            name: None,
        }),
        (22, 9) => Some(ExtOpcode {
            category: 22,
            index: 9,
            name: Some("random"),
        }),
        (22, 10) => Some(ExtOpcode {
            category: 22,
            index: 10,
            name: Some("abs"),
        }),
        (22, 11) => Some(ExtOpcode {
            category: 22,
            index: 11,
            name: Some("sin"),
        }),
        (22, 12) => Some(ExtOpcode {
            category: 22,
            index: 12,
            name: Some("cos"),
        }),
        (22, 13) => Some(ExtOpcode {
            category: 22,
            index: 13,
            name: Some("tan"),
        }),
        (22, 14) => Some(ExtOpcode {
            category: 22,
            index: 14,
            name: Some("atan"),
        }),
        (22, 15) => Some(ExtOpcode {
            category: 22,
            index: 15,
            name: Some("log"),
        }),
        (22, 16) => Some(ExtOpcode {
            category: 22,
            index: 16,
            name: Some("log10"),
        }),
        (22, 17) => Some(ExtOpcode {
            category: 22,
            index: 17,
            name: None,
        }),
        (22, 18) => Some(ExtOpcode {
            category: 22,
            index: 18,
            name: Some("sqrt"),
        }),
        (22, 19) => Some(ExtOpcode {
            category: 22,
            index: 19,
            name: None,
        }),
        (22, 20) => Some(ExtOpcode {
            category: 22,
            index: 20,
            name: None,
        }),
        (22, 24) => Some(ExtOpcode {
            category: 22,
            index: 24,
            name: Some("sp_set"),
        }),
        (22, 25) => Some(ExtOpcode {
            category: 22,
            index: 25,
            name: Some("sp_set_ex"),
        }),
        (22, 26) => Some(ExtOpcode {
            category: 22,
            index: 26,
            name: Some("sp_set_pos"),
        }),
        (22, 27) => Some(ExtOpcode {
            category: 22,
            index: 27,
            name: Some("sp_cls"),
        }),
        (22, 28) => Some(ExtOpcode {
            category: 22,
            index: 28,
            name: Some("sp_set_alpha"),
        }),
        (22, 29) => Some(ExtOpcode {
            category: 22,
            index: 29,
            name: Some("set_priority"),
        }),
        (22, 30) => Some(ExtOpcode {
            category: 22,
            index: 30,
            name: None,
        }),
        (22, 31) => Some(ExtOpcode {
            category: 22,
            index: 31,
            name: Some("sp_set_center"),
        }),
        (22, 33) => Some(ExtOpcode {
            category: 22,
            index: 33,
            name: Some("sp_cls_ex"),
        }),
        (22, 34) => Some(ExtOpcode {
            category: 22,
            index: 34,
            name: Some("set_filter"),
        }),
        (22, 35) => Some(ExtOpcode {
            category: 22,
            index: 35,
            name: Some("sp_cls_transition"),
        }),
        (22, 36) => Some(ExtOpcode {
            category: 22,
            index: 36,
            name: Some("sp_set_pos_ex"),
        }),
        (22, 37) => Some(ExtOpcode {
            category: 22,
            index: 37,
            name: Some("sp_set_rect_pos"),
        }),
        (22, 38) => Some(ExtOpcode {
            category: 22,
            index: 38,
            name: None,
        }),
        (22, 39) => Some(ExtOpcode {
            category: 22,
            index: 39,
            name: Some("sp_set_scale"),
        }),
        (22, 40) => Some(ExtOpcode {
            category: 22,
            index: 40,
            name: Some("sp_set_rotate"),
        }),
        (22, 41) => Some(ExtOpcode {
            category: 22,
            index: 41,
            name: Some("face_init"),
        }),
        (22, 42) => Some(ExtOpcode {
            category: 22,
            index: 42,
            name: Some("face_set"),
        }),
        (22, 43) => Some(ExtOpcode {
            category: 22,
            index: 43,
            name: Some("not_image_sp_get_color"),
        }),
        (22, 44) => Some(ExtOpcode {
            category: 22,
            index: 44,
            name: Some("sptext"),
        }),
        (22, 45) => Some(ExtOpcode {
            category: 22,
            index: 45,
            name: Some("face_cls"),
        }),
        (22, 46) => Some(ExtOpcode {
            category: 22,
            index: 46,
            name: Some("sp_set_rect"),
        }),
        (22, 47) => Some(ExtOpcode {
            category: 22,
            index: 47,
            name: Some("sp_set_pos_move"),
        }),
        (22, 48) => Some(ExtOpcode {
            category: 22,
            index: 48,
            name: Some("not_image_sp_get_alpha"),
        }),
        (22, 49) => Some(ExtOpcode {
            category: 22,
            index: 49,
            name: Some("not_image_sp_get_rotate"),
        }),
        (22, 50) => Some(ExtOpcode {
            category: 22,
            index: 50,
            name: None,
        }),
        (22, 51) => Some(ExtOpcode {
            category: 22,
            index: 51,
            name: None,
        }),
        (22, 52) => Some(ExtOpcode {
            category: 22,
            index: 52,
            name: None,
        }),
        (22, 53) => Some(ExtOpcode {
            category: 22,
            index: 53,
            name: None,
        }),
        (22, 54) => Some(ExtOpcode {
            category: 22,
            index: 54,
            name: Some("sp_create"),
        }),
        (22, 55) => Some(ExtOpcode {
            category: 22,
            index: 55,
            name: Some("sp_anime_clear"),
        }),
        (22, 56) => Some(ExtOpcode {
            category: 22,
            index: 56,
            name: None,
        }),
        (22, 57) => Some(ExtOpcode {
            category: 22,
            index: 57,
            name: None,
        }),
        (22, 58) => Some(ExtOpcode {
            category: 22,
            index: 58,
            name: Some("not_image_sp_get_scale"),
        }),
        (22, 59) => Some(ExtOpcode {
            category: 22,
            index: 59,
            name: Some("sp_set_color_0x"),
        }),
        (22, 60) => Some(ExtOpcode {
            category: 22,
            index: 60,
            name: Some("sp_bitblt"),
        }),
        (22, 61) => Some(ExtOpcode {
            category: 22,
            index: 61,
            name: Some("sp_set_shake"),
        }),
        (22, 62) => Some(ExtOpcode {
            category: 22,
            index: 62,
            name: Some("sp_paint"),
        }),
        (22, 63) => Some(ExtOpcode {
            category: 22,
            index: 63,
            name: None,
        }),
        (22, 64) => Some(ExtOpcode {
            category: 22,
            index: 64,
            name: Some("sp_load_wait_time"),
        }),
        (22, 65) => Some(ExtOpcode {
            category: 22,
            index: 65,
            name: Some("sp_draw"),
        }),
        (22, 66) => Some(ExtOpcode {
            category: 22,
            index: 66,
            name: None,
        }),
        (22, 67) => Some(ExtOpcode {
            category: 22,
            index: 67,
            name: Some("sp_unlock"),
        }),
        (22, 68) => Some(ExtOpcode {
            category: 22,
            index: 68,
            name: Some("sp_show"),
        }),
        (22, 69) => Some(ExtOpcode {
            category: 22,
            index: 69,
            name: Some("sp_hide"),
        }),
        (22, 70) => Some(ExtOpcode {
            category: 22,
            index: 70,
            name: None,
        }),
        (22, 71) => Some(ExtOpcode {
            category: 22,
            index: 71,
            name: Some("sp_set_child"),
        }),
        (22, 72) => Some(ExtOpcode {
            category: 22,
            index: 72,
            name: Some("sp_set_transition"),
        }),
        (22, 73) => Some(ExtOpcode {
            category: 22,
            index: 73,
            name: Some("sp_copy_image"),
        }),
        (22, 74) => Some(ExtOpcode {
            category: 22,
            index: 74,
            name: Some("sp_transition"),
        }),
        (22, 75) => Some(ExtOpcode {
            category: 22,
            index: 75,
            name: Some("set_aspect_position_type"),
        }),
        (22, 76) => Some(ExtOpcode {
            category: 22,
            index: 76,
            name: Some("get_backbuffer"),
        }),
        (22, 77) => Some(ExtOpcode {
            category: 22,
            index: 77,
            name: Some("sp_set_mask"),
        }),
        (22, 78) => Some(ExtOpcode {
            category: 22,
            index: 78,
            name: None,
        }),
        (22, 79) => Some(ExtOpcode {
            category: 22,
            index: 79,
            name: Some("spsetanime"),
        }),
        (22, 80) => Some(ExtOpcode {
            category: 22,
            index: 80,
            name: Some("drawtext"),
        }),
        (22, 81) => Some(ExtOpcode {
            category: 22,
            index: 81,
            name: None,
        }),
        (22, 82) => Some(ExtOpcode {
            category: 22,
            index: 82,
            name: None,
        }),
        (22, 84) => Some(ExtOpcode {
            category: 22,
            index: 84,
            name: Some("history_init_0x_0x"),
        }),
        (22, 85) => Some(ExtOpcode {
            category: 22,
            index: 85,
            name: Some("historybegin_lpbyte_ptagdata_sztext"),
        }),
        (22, 86) => Some(ExtOpcode {
            category: 22,
            index: 86,
            name: Some("history_end"),
        }),
        (22, 87) => Some(ExtOpcode {
            category: 22,
            index: 87,
            name: None,
        }),
        (22, 88) => Some(ExtOpcode {
            category: 22,
            index: 88,
            name: None,
        }),
        (22, 89) => Some(ExtOpcode {
            category: 22,
            index: 89,
            name: Some("history_get_height"),
        }),
        (22, 90) => Some(ExtOpcode {
            category: 22,
            index: 90,
            name: None,
        }),
        (22, 91) => Some(ExtOpcode {
            category: 22,
            index: 91,
            name: None,
        }),
        (22, 92) => Some(ExtOpcode {
            category: 22,
            index: 92,
            name: None,
        }),
        (22, 93) => Some(ExtOpcode {
            category: 22,
            index: 93,
            name: None,
        }),
        (22, 94) => Some(ExtOpcode {
            category: 22,
            index: 94,
            name: Some("history_set_rect"),
        }),
        (22, 95) => Some(ExtOpcode {
            category: 22,
            index: 95,
            name: Some("history_clear"),
        }),
        (22, 96) => Some(ExtOpcode {
            category: 22,
            index: 96,
            name: Some("history_set"),
        }),
        (22, 97) => Some(ExtOpcode {
            category: 22,
            index: 97,
            name: None,
        }),
        (22, 98) => Some(ExtOpcode {
            category: 22,
            index: 98,
            name: None,
        }),
        (22, 99) => Some(ExtOpcode {
            category: 22,
            index: 99,
            name: None,
        }),
        (22, 100) => Some(ExtOpcode {
            category: 22,
            index: 100,
            name: None,
        }),
        (22, 101) => Some(ExtOpcode {
            category: 22,
            index: 101,
            name: Some("history_set_face_call"),
        }),
        (22, 102) => Some(ExtOpcode {
            category: 22,
            index: 102,
            name: Some("history_set_face_sound"),
        }),
        (22, 103) => Some(ExtOpcode {
            category: 22,
            index: 103,
            name: Some("history_set_face_sound_release"),
        }),
        (22, 104) => Some(ExtOpcode {
            category: 22,
            index: 104,
            name: Some("history_get_text"),
        }),
        (22, 105) => Some(ExtOpcode {
            category: 22,
            index: 105,
            name: None,
        }),
        (22, 106) => Some(ExtOpcode {
            category: 22,
            index: 106,
            name: None,
        }),
        (22, 107) => Some(ExtOpcode {
            category: 22,
            index: 107,
            name: None,
        }),
        (22, 108) => Some(ExtOpcode {
            category: 22,
            index: 108,
            name: None,
        }),
        (22, 110) => Some(ExtOpcode {
            category: 22,
            index: 110,
            name: Some("movie_play"),
        }),
        (22, 111) => Some(ExtOpcode {
            category: 22,
            index: 111,
            name: Some("msp_set_loop_sp_ep"),
        }),
        (22, 112) => Some(ExtOpcode {
            category: 22,
            index: 112,
            name: Some("msp_cls"),
        }),
        (22, 113) => Some(ExtOpcode {
            category: 22,
            index: 113,
            name: Some("msp_wait"),
        }),
        (22, 114) => Some(ExtOpcode {
            category: 22,
            index: 114,
            name: Some("msp_lock"),
        }),
        (22, 115) => Some(ExtOpcode {
            category: 22,
            index: 115,
            name: Some("msp_unlock"),
        }),
        (22, 116) => Some(ExtOpcode {
            category: 22,
            index: 116,
            name: Some("msp_play"),
        }),
        (22, 117) => Some(ExtOpcode {
            category: 22,
            index: 117,
            name: Some("msp_stop"),
        }),
        (22, 119) => Some(ExtOpcode {
            category: 22,
            index: 119,
            name: Some("create_thread"),
        }),
        (22, 120) => Some(ExtOpcode {
            category: 22,
            index: 120,
            name: Some("exit_thread"),
        }),
        (22, 121) => Some(ExtOpcode {
            category: 22,
            index: 121,
            name: None,
        }),
        (22, 122) => Some(ExtOpcode {
            category: 22,
            index: 122,
            name: Some("get_thread"),
        }),
        (22, 125) => Some(ExtOpcode {
            category: 22,
            index: 125,
            name: Some("mov"),
        }),
        (22, 126) => Some(ExtOpcode {
            category: 22,
            index: 126,
            name: Some("add"),
        }),
        (22, 127) => Some(ExtOpcode {
            category: 22,
            index: 127,
            name: Some("sub"),
        }),
        (22, 128) => Some(ExtOpcode {
            category: 22,
            index: 128,
            name: Some("mul"),
        }),
        (22, 129) => Some(ExtOpcode {
            category: 22,
            index: 129,
            name: Some("div"),
        }),
        (22, 130) => Some(ExtOpcode {
            category: 22,
            index: 130,
            name: Some("bitand"),
        }),
        (22, 131) => Some(ExtOpcode {
            category: 22,
            index: 131,
            name: Some("bitor"),
        }),
        (22, 132) => Some(ExtOpcode {
            category: 22,
            index: 132,
            name: Some("bitxor"),
        }),
        (22, 133) => Some(ExtOpcode {
            category: 22,
            index: 133,
            name: Some("jmp_point"),
        }),
        (22, 134) => Some(ExtOpcode {
            category: 22,
            index: 134,
            name: Some("jf_point"),
        }),
        (22, 135) => Some(ExtOpcode {
            category: 22,
            index: 135,
            name: Some("gosub_point"),
        }),
        (22, 136) => Some(ExtOpcode {
            category: 22,
            index: 136,
            name: Some("eq"),
        }),
        (22, 137) => Some(ExtOpcode {
            category: 22,
            index: 137,
            name: Some("ne"),
        }),
        (22, 138) => Some(ExtOpcode {
            category: 22,
            index: 138,
            name: Some("le"),
        }),
        (22, 139) => Some(ExtOpcode {
            category: 22,
            index: 139,
            name: Some("ge"),
        }),
        (22, 140) => Some(ExtOpcode {
            category: 22,
            index: 140,
            name: Some("lt"),
        }),
        (22, 141) => Some(ExtOpcode {
            category: 22,
            index: 141,
            name: Some("gt"),
        }),
        (22, 142) => Some(ExtOpcode {
            category: 22,
            index: 142,
            name: Some("lor"),
        }),
        (22, 143) => Some(ExtOpcode {
            category: 22,
            index: 143,
            name: Some("land"),
        }),
        (22, 144) => Some(ExtOpcode {
            category: 22,
            index: 144,
            name: Some("lnot_slot"),
        }),
        (22, 145) => Some(ExtOpcode {
            category: 22,
            index: 145,
            name: Some("end"),
        }),
        (22, 146) => Some(ExtOpcode {
            category: 22,
            index: 146,
            name: Some("nop"),
        }),
        (22, 147) => Some(ExtOpcode {
            category: 22,
            index: 147,
            name: Some("extcall"),
        }),
        (22, 148) => Some(ExtOpcode {
            category: 22,
            index: 148,
            name: Some("ret"),
        }),
        (22, 149) => Some(ExtOpcode {
            category: 22,
            index: 149,
            name: Some("reset_adv"),
        }),
        (22, 150) => Some(ExtOpcode {
            category: 22,
            index: 150,
            name: Some("mod"),
        }),
        (22, 151) => Some(ExtOpcode {
            category: 22,
            index: 151,
            name: Some("shl"),
        }),
        (22, 152) => Some(ExtOpcode {
            category: 22,
            index: 152,
            name: Some("shr"),
        }),
        (22, 153) => Some(ExtOpcode {
            category: 22,
            index: 153,
            name: Some("neg_slot"),
        }),
        (22, 154) => Some(ExtOpcode {
            category: 22,
            index: 154,
            name: Some("pop"),
        }),
        (22, 155) => Some(ExtOpcode {
            category: 22,
            index: 155,
            name: Some("push"),
        }),
        (22, 156) => Some(ExtOpcode {
            category: 22,
            index: 156,
            name: Some("pack_args"),
        }),
        (22, 157) => Some(ExtOpcode {
            category: 22,
            index: 157,
            name: Some("drop_args"),
        }),
        (22, 159) => Some(ExtOpcode {
            category: 22,
            index: 159,
            name: Some("create_message"),
        }),
        (22, 160) => Some(ExtOpcode {
            category: 22,
            index: 160,
            name: Some("get_message"),
        }),
        (22, 161) => Some(ExtOpcode {
            category: 22,
            index: 161,
            name: Some("get_message_param"),
        }),
        (22, 164) => Some(ExtOpcode {
            category: 22,
            index: 164,
            name: Some("save"),
        }),
        (22, 165) => Some(ExtOpcode {
            category: 22,
            index: 165,
            name: Some("load"),
        }),
        (22, 166) => Some(ExtOpcode {
            category: 22,
            index: 166,
            name: Some("save_set_title"),
        }),
        (22, 167) => Some(ExtOpcode {
            category: 22,
            index: 167,
            name: Some("save_data"),
        }),
        (22, 168) => Some(ExtOpcode {
            category: 22,
            index: 168,
            name: Some("save_set_thumbnail_size"),
        }),
        (22, 169) => Some(ExtOpcode {
            category: 22,
            index: 169,
            name: Some("thumbnail_set"),
        }),
        (22, 170) => Some(ExtOpcode {
            category: 22,
            index: 170,
            name: Some("savetitledraw"),
        }),
        (22, 171) => Some(ExtOpcode {
            category: 22,
            index: 171,
            name: Some("save_set_font_size"),
        }),
        (22, 172) => Some(ExtOpcode {
            category: 22,
            index: 172,
            name: Some("getsaveday"),
        }),
        (22, 173) => Some(ExtOpcode {
            category: 22,
            index: 173,
            name: Some("is_save"),
        }),
        (22, 174) => Some(ExtOpcode {
            category: 22,
            index: 174,
            name: Some("getsaveusermemory"),
        }),
        (22, 175) => Some(ExtOpcode {
            category: 22,
            index: 175,
            name: Some("savepoint"),
        }),
        (22, 176) => Some(ExtOpcode {
            category: 22,
            index: 176,
            name: Some("save_thumbnail_mosaic_set"),
        }),
        (22, 177) => Some(ExtOpcode {
            category: 22,
            index: 177,
            name: Some("savetimedraw"),
        }),
        (22, 178) => Some(ExtOpcode {
            category: 22,
            index: 178,
            name: Some("savedaydraw"),
        }),
        (22, 179) => Some(ExtOpcode {
            category: 22,
            index: 179,
            name: Some("save_set_text_rect"),
        }),
        (22, 180) => Some(ExtOpcode {
            category: 22,
            index: 180,
            name: Some("savetextdraw"),
        }),
        (22, 181) => Some(ExtOpcode {
            category: 22,
            index: 181,
            name: Some("get_new_savefile"),
        }),
        (22, 185) => Some(ExtOpcode {
            category: 22,
            index: 185,
            name: Some("setsavetext"),
        }),
        (22, 186) => Some(ExtOpcode {
            category: 22,
            index: 186,
            name: Some("thumbnail_renew"),
        }),
        (22, 187) => Some(ExtOpcode {
            category: 22,
            index: 187,
            name: Some("save_set_font_type"),
        }),
        (22, 188) => Some(ExtOpcode {
            category: 22,
            index: 188,
            name: Some("set_load_after_process"),
        }),
        (22, 189) => Some(ExtOpcode {
            category: 22,
            index: 189,
            name: Some("savesystemdata"),
        }),
        (22, 190) => Some(ExtOpcode {
            category: 22,
            index: 190,
            name: Some("save_set_font_effect"),
        }),
        (22, 191) => Some(ExtOpcode {
            category: 22,
            index: 191,
            name: Some("save_set_font_color_0x_0x"),
        }),
        (22, 192) => Some(ExtOpcode {
            category: 22,
            index: 192,
            name: Some("delete_file"),
        }),
        (22, 193) => Some(ExtOpcode {
            category: 22,
            index: 193,
            name: Some("save_tmp_dat"),
        }),
        (22, 194) => Some(ExtOpcode {
            category: 22,
            index: 194,
            name: Some("copy_file"),
        }),
        (22, 195) => Some(ExtOpcode {
            category: 22,
            index: 195,
            name: Some("load_thumbnail"),
        }),
        (22, 196) => Some(ExtOpcode {
            category: 22,
            index: 196,
            name: Some("save_lock_not_open_savefileno"),
        }),
        (22, 197) => Some(ExtOpcode {
            category: 22,
            index: 197,
            name: Some("is_save_lock"),
        }),
        (22, 198) => Some(ExtOpcode {
            category: 22,
            index: 198,
            name: Some("is_prev_data"),
        }),
        (22, 199) => Some(ExtOpcode {
            category: 22,
            index: 199,
            name: Some("save_point_clear"),
        }),
        (22, 200) => Some(ExtOpcode {
            category: 22,
            index: 200,
            name: Some("save_point_lock"),
        }),
        (22, 201) => Some(ExtOpcode {
            category: 22,
            index: 201,
            name: None,
        }),
        (22, 202) => Some(ExtOpcode {
            category: 22,
            index: 202,
            name: Some("histload"),
        }),
        (22, 204) => Some(ExtOpcode {
            category: 22,
            index: 204,
            name: Some("se_load"),
        }),
        (22, 205) => Some(ExtOpcode {
            category: 22,
            index: 205,
            name: Some("se_play"),
        }),
        (22, 206) => Some(ExtOpcode {
            category: 22,
            index: 206,
            name: Some("se_play_ex_ch"),
        }),
        (22, 207) => Some(ExtOpcode {
            category: 22,
            index: 207,
            name: Some("se_stop"),
        }),
        (22, 208) => Some(ExtOpcode {
            category: 22,
            index: 208,
            name: Some("se_set_volume"),
        }),
        (22, 209) => Some(ExtOpcode {
            category: 22,
            index: 209,
            name: Some("se_get_volume"),
        }),
        (22, 210) => Some(ExtOpcode {
            category: 22,
            index: 210,
            name: Some("se_unload"),
        }),
        (22, 211) => Some(ExtOpcode {
            category: 22,
            index: 211,
            name: Some("se_wait"),
        }),
        (22, 212) => Some(ExtOpcode {
            category: 22,
            index: 212,
            name: Some("channel_error_set_se_info"),
        }),
        (22, 213) => Some(ExtOpcode {
            category: 22,
            index: 213,
            name: Some("get_se_ex_volume"),
        }),
        (22, 214) => Some(ExtOpcode {
            category: 22,
            index: 214,
            name: Some("channel_error_set_se_ex_volume"),
        }),
        (22, 215) => Some(ExtOpcode {
            category: 22,
            index: 215,
            name: Some("channel_error_se_enable"),
        }),
        (22, 216) => Some(ExtOpcode {
            category: 22,
            index: 216,
            name: Some("channel_error_is_se_enable"),
        }),
        (22, 217) => Some(ExtOpcode {
            category: 22,
            index: 217,
            name: Some("se_set_pan"),
        }),
        (22, 218) => Some(ExtOpcode {
            category: 22,
            index: 218,
            name: Some("se_mute"),
        }),
        (22, 220) => Some(ExtOpcode {
            category: 22,
            index: 220,
            name: Some("select_init"),
        }),
        (22, 221) => Some(ExtOpcode {
            category: 22,
            index: 221,
            name: Some("select"),
        }),
        (22, 222) => Some(ExtOpcode {
            category: 22,
            index: 222,
            name: None,
        }),
        (22, 223) => Some(ExtOpcode {
            category: 22,
            index: 223,
            name: None,
        }),
        (22, 224) => Some(ExtOpcode {
            category: 22,
            index: 224,
            name: Some("select_clear"),
        }),
        (22, 225) => Some(ExtOpcode {
            category: 22,
            index: 225,
            name: Some("select_set_offset"),
        }),
        (22, 226) => Some(ExtOpcode {
            category: 22,
            index: 226,
            name: Some("select_set_process"),
        }),
        (22, 227) => Some(ExtOpcode {
            category: 22,
            index: 227,
            name: Some("select_lock"),
        }),
        (22, 228) => Some(ExtOpcode {
            category: 22,
            index: 228,
            name: Some("get_select_on_key"),
        }),
        (22, 229) => Some(ExtOpcode {
            category: 22,
            index: 229,
            name: Some("get_select_pull_key"),
        }),
        (22, 230) => Some(ExtOpcode {
            category: 22,
            index: 230,
            name: Some("get_select_push_key"),
        }),
        (22, 232) => Some(ExtOpcode {
            category: 22,
            index: 232,
            name: Some("skip_set"),
        }),
        (22, 233) => Some(ExtOpcode {
            category: 22,
            index: 233,
            name: Some("skip_is"),
        }),
        (22, 234) => Some(ExtOpcode {
            category: 22,
            index: 234,
            name: Some("auto_set"),
        }),
        (22, 235) => Some(ExtOpcode {
            category: 22,
            index: 235,
            name: Some("auto_is"),
        }),
        (22, 236) => Some(ExtOpcode {
            category: 22,
            index: 236,
            name: None,
        }),
        (22, 237) => Some(ExtOpcode {
            category: 22,
            index: 237,
            name: Some("auto_get_time"),
        }),
        (22, 238) => Some(ExtOpcode {
            category: 22,
            index: 238,
            name: None,
        }),
        (22, 239) => Some(ExtOpcode {
            category: 22,
            index: 239,
            name: None,
        }),
        (22, 240) => Some(ExtOpcode {
            category: 22,
            index: 240,
            name: None,
        }),
        (22, 241) => Some(ExtOpcode {
            category: 22,
            index: 241,
            name: None,
        }),
        (22, 242) => Some(ExtOpcode {
            category: 22,
            index: 242,
            name: None,
        }),
        (22, 243) => Some(ExtOpcode {
            category: 22,
            index: 243,
            name: None,
        }),
        (22, 244) => Some(ExtOpcode {
            category: 22,
            index: 244,
            name: None,
        }),
        (22, 245) => Some(ExtOpcode {
            category: 22,
            index: 245,
            name: None,
        }),
        (22, 246) => Some(ExtOpcode {
            category: 22,
            index: 246,
            name: None,
        }),
        (22, 247) => Some(ExtOpcode {
            category: 22,
            index: 247,
            name: Some("load_font"),
        }),
        (22, 248) => Some(ExtOpcode {
            category: 22,
            index: 248,
            name: Some("unload_font"),
        }),
        (22, 249) => Some(ExtOpcode {
            category: 22,
            index: 249,
            name: Some("set_language"),
        }),
        (22, 250) => Some(ExtOpcode {
            category: 22,
            index: 250,
            name: Some("key_canncel"),
        }),
        (22, 251) => Some(ExtOpcode {
            category: 22,
            index: 251,
            name: Some("set_font_color"),
        }),
        (22, 252) => Some(ExtOpcode {
            category: 22,
            index: 252,
            name: Some("load_font_ex"),
        }),
        (22, 253) => Some(ExtOpcode {
            category: 22,
            index: 253,
            name: None,
        }),
        (22, 254) => Some(ExtOpcode {
            category: 22,
            index: 254,
            name: None,
        }),
        (22, 255) => Some(ExtOpcode {
            category: 22,
            index: 255,
            name: None,
        }),
        (22, 256) => Some(ExtOpcode {
            category: 22,
            index: 256,
            name: None,
        }),
        (22, 257) => Some(ExtOpcode {
            category: 22,
            index: 257,
            name: None,
        }),
        (22, 258) => Some(ExtOpcode {
            category: 22,
            index: 258,
            name: None,
        }),
        (22, 259) => Some(ExtOpcode {
            category: 22,
            index: 259,
            name: Some("set_font_size"),
        }),
        (22, 260) => Some(ExtOpcode {
            category: 22,
            index: 260,
            name: Some("get_font_size"),
        }),
        (22, 261) => Some(ExtOpcode {
            category: 22,
            index: 261,
            name: Some("get_font_type"),
        }),
        (22, 262) => Some(ExtOpcode {
            category: 22,
            index: 262,
            name: Some("set_font_effect"),
        }),
        (22, 263) => Some(ExtOpcode {
            category: 22,
            index: 263,
            name: Some("get_font_effect"),
        }),
        (22, 264) => Some(ExtOpcode {
            category: 22,
            index: 264,
            name: Some("get_pull_key"),
        }),
        (22, 265) => Some(ExtOpcode {
            category: 22,
            index: 265,
            name: Some("get_on_key"),
        }),
        (22, 266) => Some(ExtOpcode {
            category: 22,
            index: 266,
            name: Some("get_push_key"),
        }),
        (22, 267) => Some(ExtOpcode {
            category: 22,
            index: 267,
            name: Some("input_clear"),
        }),
        (22, 268) => Some(ExtOpcode {
            category: 22,
            index: 268,
            name: Some("change_window_size"),
        }),
        (22, 269) => Some(ExtOpcode {
            category: 22,
            index: 269,
            name: Some("change_aspect_mode"),
        }),
        (22, 270) => Some(ExtOpcode {
            category: 22,
            index: 270,
            name: Some("aspect_position_enable"),
        }),
        (22, 271) => Some(ExtOpcode {
            category: 22,
            index: 271,
            name: None,
        }),
        (22, 272) => Some(ExtOpcode {
            category: 22,
            index: 272,
            name: Some("get_aspect_mode"),
        }),
        (22, 273) => Some(ExtOpcode {
            category: 22,
            index: 273,
            name: Some("get_monitor_size"),
        }),
        (22, 274) => Some(ExtOpcode {
            category: 22,
            index: 274,
            name: None,
        }),
        (22, 275) => Some(ExtOpcode {
            category: 22,
            index: 275,
            name: Some("get_system_metrics"),
        }),
        (22, 276) => Some(ExtOpcode {
            category: 22,
            index: 276,
            name: Some("set_system_path"),
        }),
        (22, 277) => Some(ExtOpcode {
            category: 22,
            index: 277,
            name: Some("set_allmosaicthumbnail"),
        }),
        (22, 278) => Some(ExtOpcode {
            category: 22,
            index: 278,
            name: Some("enable_window_change"),
        }),
        (22, 279) => Some(ExtOpcode {
            category: 22,
            index: 279,
            name: Some("is_enable_window_change"),
        }),
        (22, 280) => Some(ExtOpcode {
            category: 22,
            index: 280,
            name: Some("set_cursor_null"),
        }),
        (22, 281) => Some(ExtOpcode {
            category: 22,
            index: 281,
            name: Some("set_hide_cursor_time"),
        }),
        (22, 282) => Some(ExtOpcode {
            category: 22,
            index: 282,
            name: Some("get_hide_cursor_time"),
        }),
        (22, 283) => Some(ExtOpcode {
            category: 22,
            index: 283,
            name: Some("scene_skip"),
        }),
        (22, 284) => Some(ExtOpcode {
            category: 22,
            index: 284,
            name: None,
        }),
        (22, 285) => Some(ExtOpcode {
            category: 22,
            index: 285,
            name: None,
        }),
        (22, 286) => Some(ExtOpcode {
            category: 22,
            index: 286,
            name: Some("get_async_key"),
        }),
        (22, 287) => Some(ExtOpcode {
            category: 22,
            index: 287,
            name: Some("get_font_color"),
        }),
        (22, 288) => Some(ExtOpcode {
            category: 22,
            index: 288,
            name: None,
        }),
        (22, 289) => Some(ExtOpcode {
            category: 22,
            index: 289,
            name: Some("history_skip"),
        }),
        (22, 290) => Some(ExtOpcode {
            category: 22,
            index: 290,
            name: None,
        }),
        (22, 291) => Some(ExtOpcode {
            category: 22,
            index: 291,
            name: None,
        }),
        (22, 292) => Some(ExtOpcode {
            category: 22,
            index: 292,
            name: Some("set_language"),
        }),
        (22, 293) => Some(ExtOpcode {
            category: 22,
            index: 293,
            name: Some("set_achievement"),
        }),
        (22, 295) => Some(ExtOpcode {
            category: 22,
            index: 295,
            name: Some("system_btn_set"),
        }),
        (22, 296) => Some(ExtOpcode {
            category: 22,
            index: 296,
            name: Some("system_btn_release"),
        }),
        (22, 297) => Some(ExtOpcode {
            category: 22,
            index: 297,
            name: Some("system_btn_enable"),
        }),
        (23, 0) => Some(ExtOpcode {
            category: 23,
            index: 0,
            name: Some("create_message"),
        }),
        (23, 1) => Some(ExtOpcode {
            category: 23,
            index: 1,
            name: Some("get_message"),
        }),
        (23, 2) => Some(ExtOpcode {
            category: 23,
            index: 2,
            name: Some("get_message_param"),
        }),
        (23, 5) => Some(ExtOpcode {
            category: 23,
            index: 5,
            name: Some("save"),
        }),
        (23, 6) => Some(ExtOpcode {
            category: 23,
            index: 6,
            name: Some("load"),
        }),
        (23, 7) => Some(ExtOpcode {
            category: 23,
            index: 7,
            name: Some("save_set_title"),
        }),
        (23, 8) => Some(ExtOpcode {
            category: 23,
            index: 8,
            name: Some("save_data"),
        }),
        (23, 9) => Some(ExtOpcode {
            category: 23,
            index: 9,
            name: Some("save_set_thumbnail_size"),
        }),
        (23, 10) => Some(ExtOpcode {
            category: 23,
            index: 10,
            name: Some("thumbnail_set"),
        }),
        (23, 11) => Some(ExtOpcode {
            category: 23,
            index: 11,
            name: Some("savetitledraw"),
        }),
        (23, 12) => Some(ExtOpcode {
            category: 23,
            index: 12,
            name: Some("save_set_font_size"),
        }),
        (23, 13) => Some(ExtOpcode {
            category: 23,
            index: 13,
            name: Some("getsaveday"),
        }),
        (23, 14) => Some(ExtOpcode {
            category: 23,
            index: 14,
            name: Some("is_save"),
        }),
        (23, 15) => Some(ExtOpcode {
            category: 23,
            index: 15,
            name: Some("getsaveusermemory"),
        }),
        (23, 16) => Some(ExtOpcode {
            category: 23,
            index: 16,
            name: Some("savepoint"),
        }),
        (23, 17) => Some(ExtOpcode {
            category: 23,
            index: 17,
            name: Some("save_thumbnail_mosaic_set"),
        }),
        (23, 18) => Some(ExtOpcode {
            category: 23,
            index: 18,
            name: Some("savetimedraw"),
        }),
        (23, 19) => Some(ExtOpcode {
            category: 23,
            index: 19,
            name: Some("savedaydraw"),
        }),
        (23, 20) => Some(ExtOpcode {
            category: 23,
            index: 20,
            name: Some("save_set_text_rect"),
        }),
        (23, 21) => Some(ExtOpcode {
            category: 23,
            index: 21,
            name: Some("savetextdraw"),
        }),
        (23, 22) => Some(ExtOpcode {
            category: 23,
            index: 22,
            name: Some("get_new_savefile"),
        }),
        (23, 26) => Some(ExtOpcode {
            category: 23,
            index: 26,
            name: Some("setsavetext"),
        }),
        (23, 27) => Some(ExtOpcode {
            category: 23,
            index: 27,
            name: Some("thumbnail_renew"),
        }),
        (23, 28) => Some(ExtOpcode {
            category: 23,
            index: 28,
            name: Some("save_set_font_type"),
        }),
        (23, 29) => Some(ExtOpcode {
            category: 23,
            index: 29,
            name: Some("set_load_after_process"),
        }),
        (23, 30) => Some(ExtOpcode {
            category: 23,
            index: 30,
            name: Some("savesystemdata"),
        }),
        (23, 31) => Some(ExtOpcode {
            category: 23,
            index: 31,
            name: Some("save_set_font_effect"),
        }),
        (23, 32) => Some(ExtOpcode {
            category: 23,
            index: 32,
            name: Some("save_set_font_color_0x_0x"),
        }),
        (23, 33) => Some(ExtOpcode {
            category: 23,
            index: 33,
            name: Some("delete_file"),
        }),
        (23, 34) => Some(ExtOpcode {
            category: 23,
            index: 34,
            name: Some("save_tmp_dat"),
        }),
        (23, 35) => Some(ExtOpcode {
            category: 23,
            index: 35,
            name: Some("copy_file"),
        }),
        (23, 36) => Some(ExtOpcode {
            category: 23,
            index: 36,
            name: Some("load_thumbnail"),
        }),
        (23, 37) => Some(ExtOpcode {
            category: 23,
            index: 37,
            name: Some("save_lock_not_open_savefileno"),
        }),
        (23, 38) => Some(ExtOpcode {
            category: 23,
            index: 38,
            name: Some("is_save_lock"),
        }),
        (23, 39) => Some(ExtOpcode {
            category: 23,
            index: 39,
            name: Some("is_prev_data"),
        }),
        (23, 40) => Some(ExtOpcode {
            category: 23,
            index: 40,
            name: Some("save_point_clear"),
        }),
        (23, 41) => Some(ExtOpcode {
            category: 23,
            index: 41,
            name: Some("save_point_lock"),
        }),
        (23, 42) => Some(ExtOpcode {
            category: 23,
            index: 42,
            name: None,
        }),
        (23, 43) => Some(ExtOpcode {
            category: 23,
            index: 43,
            name: Some("histload"),
        }),
        (23, 45) => Some(ExtOpcode {
            category: 23,
            index: 45,
            name: Some("se_load"),
        }),
        (23, 46) => Some(ExtOpcode {
            category: 23,
            index: 46,
            name: Some("se_play"),
        }),
        (23, 47) => Some(ExtOpcode {
            category: 23,
            index: 47,
            name: Some("se_play_ex_ch"),
        }),
        (23, 48) => Some(ExtOpcode {
            category: 23,
            index: 48,
            name: Some("se_stop"),
        }),
        (23, 49) => Some(ExtOpcode {
            category: 23,
            index: 49,
            name: Some("se_set_volume"),
        }),
        (23, 50) => Some(ExtOpcode {
            category: 23,
            index: 50,
            name: Some("se_get_volume"),
        }),
        (23, 51) => Some(ExtOpcode {
            category: 23,
            index: 51,
            name: Some("se_unload"),
        }),
        (23, 52) => Some(ExtOpcode {
            category: 23,
            index: 52,
            name: Some("se_wait"),
        }),
        (23, 53) => Some(ExtOpcode {
            category: 23,
            index: 53,
            name: Some("channel_error_set_se_info"),
        }),
        (23, 54) => Some(ExtOpcode {
            category: 23,
            index: 54,
            name: Some("get_se_ex_volume"),
        }),
        (23, 55) => Some(ExtOpcode {
            category: 23,
            index: 55,
            name: Some("channel_error_set_se_ex_volume"),
        }),
        (23, 56) => Some(ExtOpcode {
            category: 23,
            index: 56,
            name: Some("channel_error_se_enable"),
        }),
        (23, 57) => Some(ExtOpcode {
            category: 23,
            index: 57,
            name: Some("channel_error_is_se_enable"),
        }),
        (23, 58) => Some(ExtOpcode {
            category: 23,
            index: 58,
            name: Some("se_set_pan"),
        }),
        (23, 59) => Some(ExtOpcode {
            category: 23,
            index: 59,
            name: Some("se_mute"),
        }),
        (23, 61) => Some(ExtOpcode {
            category: 23,
            index: 61,
            name: Some("select_init"),
        }),
        (23, 62) => Some(ExtOpcode {
            category: 23,
            index: 62,
            name: Some("select"),
        }),
        (23, 63) => Some(ExtOpcode {
            category: 23,
            index: 63,
            name: None,
        }),
        (23, 64) => Some(ExtOpcode {
            category: 23,
            index: 64,
            name: None,
        }),
        (23, 65) => Some(ExtOpcode {
            category: 23,
            index: 65,
            name: Some("select_clear"),
        }),
        (23, 66) => Some(ExtOpcode {
            category: 23,
            index: 66,
            name: Some("select_set_offset"),
        }),
        (23, 67) => Some(ExtOpcode {
            category: 23,
            index: 67,
            name: Some("select_set_process"),
        }),
        (23, 68) => Some(ExtOpcode {
            category: 23,
            index: 68,
            name: Some("select_lock"),
        }),
        (23, 69) => Some(ExtOpcode {
            category: 23,
            index: 69,
            name: Some("get_select_on_key"),
        }),
        (23, 70) => Some(ExtOpcode {
            category: 23,
            index: 70,
            name: Some("get_select_pull_key"),
        }),
        (23, 71) => Some(ExtOpcode {
            category: 23,
            index: 71,
            name: Some("get_select_push_key"),
        }),
        (23, 73) => Some(ExtOpcode {
            category: 23,
            index: 73,
            name: Some("skip_set"),
        }),
        (23, 74) => Some(ExtOpcode {
            category: 23,
            index: 74,
            name: Some("skip_is"),
        }),
        (23, 75) => Some(ExtOpcode {
            category: 23,
            index: 75,
            name: Some("auto_set"),
        }),
        (23, 76) => Some(ExtOpcode {
            category: 23,
            index: 76,
            name: Some("auto_is"),
        }),
        (23, 77) => Some(ExtOpcode {
            category: 23,
            index: 77,
            name: None,
        }),
        (23, 78) => Some(ExtOpcode {
            category: 23,
            index: 78,
            name: Some("auto_get_time"),
        }),
        (23, 79) => Some(ExtOpcode {
            category: 23,
            index: 79,
            name: None,
        }),
        (23, 80) => Some(ExtOpcode {
            category: 23,
            index: 80,
            name: None,
        }),
        (23, 81) => Some(ExtOpcode {
            category: 23,
            index: 81,
            name: None,
        }),
        (23, 82) => Some(ExtOpcode {
            category: 23,
            index: 82,
            name: None,
        }),
        (23, 83) => Some(ExtOpcode {
            category: 23,
            index: 83,
            name: None,
        }),
        (23, 84) => Some(ExtOpcode {
            category: 23,
            index: 84,
            name: None,
        }),
        (23, 85) => Some(ExtOpcode {
            category: 23,
            index: 85,
            name: None,
        }),
        (23, 86) => Some(ExtOpcode {
            category: 23,
            index: 86,
            name: None,
        }),
        (23, 87) => Some(ExtOpcode {
            category: 23,
            index: 87,
            name: None,
        }),
        (23, 88) => Some(ExtOpcode {
            category: 23,
            index: 88,
            name: Some("load_font"),
        }),
        (23, 89) => Some(ExtOpcode {
            category: 23,
            index: 89,
            name: Some("unload_font"),
        }),
        (23, 90) => Some(ExtOpcode {
            category: 23,
            index: 90,
            name: Some("set_language"),
        }),
        (23, 91) => Some(ExtOpcode {
            category: 23,
            index: 91,
            name: Some("key_canncel"),
        }),
        (23, 92) => Some(ExtOpcode {
            category: 23,
            index: 92,
            name: Some("set_font_color"),
        }),
        (23, 93) => Some(ExtOpcode {
            category: 23,
            index: 93,
            name: Some("load_font_ex"),
        }),
        (23, 94) => Some(ExtOpcode {
            category: 23,
            index: 94,
            name: None,
        }),
        (23, 95) => Some(ExtOpcode {
            category: 23,
            index: 95,
            name: None,
        }),
        (23, 96) => Some(ExtOpcode {
            category: 23,
            index: 96,
            name: None,
        }),
        (23, 97) => Some(ExtOpcode {
            category: 23,
            index: 97,
            name: None,
        }),
        (23, 98) => Some(ExtOpcode {
            category: 23,
            index: 98,
            name: None,
        }),
        (23, 99) => Some(ExtOpcode {
            category: 23,
            index: 99,
            name: None,
        }),
        (23, 100) => Some(ExtOpcode {
            category: 23,
            index: 100,
            name: Some("set_font_size"),
        }),
        (23, 101) => Some(ExtOpcode {
            category: 23,
            index: 101,
            name: Some("get_font_size"),
        }),
        (23, 102) => Some(ExtOpcode {
            category: 23,
            index: 102,
            name: Some("get_font_type"),
        }),
        (23, 103) => Some(ExtOpcode {
            category: 23,
            index: 103,
            name: Some("set_font_effect"),
        }),
        (23, 104) => Some(ExtOpcode {
            category: 23,
            index: 104,
            name: Some("get_font_effect"),
        }),
        (23, 105) => Some(ExtOpcode {
            category: 23,
            index: 105,
            name: Some("get_pull_key"),
        }),
        (23, 106) => Some(ExtOpcode {
            category: 23,
            index: 106,
            name: Some("get_on_key"),
        }),
        (23, 107) => Some(ExtOpcode {
            category: 23,
            index: 107,
            name: Some("get_push_key"),
        }),
        (23, 108) => Some(ExtOpcode {
            category: 23,
            index: 108,
            name: Some("input_clear"),
        }),
        (23, 109) => Some(ExtOpcode {
            category: 23,
            index: 109,
            name: Some("change_window_size"),
        }),
        (23, 110) => Some(ExtOpcode {
            category: 23,
            index: 110,
            name: Some("change_aspect_mode"),
        }),
        (23, 111) => Some(ExtOpcode {
            category: 23,
            index: 111,
            name: Some("aspect_position_enable"),
        }),
        (23, 112) => Some(ExtOpcode {
            category: 23,
            index: 112,
            name: None,
        }),
        (23, 113) => Some(ExtOpcode {
            category: 23,
            index: 113,
            name: Some("get_aspect_mode"),
        }),
        (23, 114) => Some(ExtOpcode {
            category: 23,
            index: 114,
            name: Some("get_monitor_size"),
        }),
        (23, 115) => Some(ExtOpcode {
            category: 23,
            index: 115,
            name: None,
        }),
        (23, 116) => Some(ExtOpcode {
            category: 23,
            index: 116,
            name: Some("get_system_metrics"),
        }),
        (23, 117) => Some(ExtOpcode {
            category: 23,
            index: 117,
            name: Some("set_system_path"),
        }),
        (23, 118) => Some(ExtOpcode {
            category: 23,
            index: 118,
            name: Some("set_allmosaicthumbnail"),
        }),
        (23, 119) => Some(ExtOpcode {
            category: 23,
            index: 119,
            name: Some("enable_window_change"),
        }),
        (23, 120) => Some(ExtOpcode {
            category: 23,
            index: 120,
            name: Some("is_enable_window_change"),
        }),
        (23, 121) => Some(ExtOpcode {
            category: 23,
            index: 121,
            name: Some("set_cursor_null"),
        }),
        (23, 122) => Some(ExtOpcode {
            category: 23,
            index: 122,
            name: Some("set_hide_cursor_time"),
        }),
        (23, 123) => Some(ExtOpcode {
            category: 23,
            index: 123,
            name: Some("get_hide_cursor_time"),
        }),
        (23, 124) => Some(ExtOpcode {
            category: 23,
            index: 124,
            name: Some("scene_skip"),
        }),
        (23, 125) => Some(ExtOpcode {
            category: 23,
            index: 125,
            name: None,
        }),
        (23, 126) => Some(ExtOpcode {
            category: 23,
            index: 126,
            name: None,
        }),
        (23, 127) => Some(ExtOpcode {
            category: 23,
            index: 127,
            name: Some("get_async_key"),
        }),
        (23, 128) => Some(ExtOpcode {
            category: 23,
            index: 128,
            name: Some("get_font_color"),
        }),
        (23, 129) => Some(ExtOpcode {
            category: 23,
            index: 129,
            name: None,
        }),
        (23, 130) => Some(ExtOpcode {
            category: 23,
            index: 130,
            name: Some("history_skip"),
        }),
        (23, 131) => Some(ExtOpcode {
            category: 23,
            index: 131,
            name: None,
        }),
        (23, 132) => Some(ExtOpcode {
            category: 23,
            index: 132,
            name: None,
        }),
        (23, 133) => Some(ExtOpcode {
            category: 23,
            index: 133,
            name: Some("set_language"),
        }),
        (23, 134) => Some(ExtOpcode {
            category: 23,
            index: 134,
            name: Some("set_achievement"),
        }),
        (23, 136) => Some(ExtOpcode {
            category: 23,
            index: 136,
            name: Some("system_btn_set"),
        }),
        (23, 137) => Some(ExtOpcode {
            category: 23,
            index: 137,
            name: Some("system_btn_release"),
        }),
        (23, 138) => Some(ExtOpcode {
            category: 23,
            index: 138,
            name: Some("system_btn_enable"),
        }),
        (23, 141) => Some(ExtOpcode {
            category: 23,
            index: 141,
            name: Some("text_init"),
        }),
        (23, 142) => Some(ExtOpcode {
            category: 23,
            index: 142,
            name: Some("text_set_icon"),
        }),
        (23, 143) => Some(ExtOpcode {
            category: 23,
            index: 143,
            name: Some("text"),
        }),
        (23, 144) => Some(ExtOpcode {
            category: 23,
            index: 144,
            name: Some("text_hide"),
        }),
        (23, 145) => Some(ExtOpcode {
            category: 23,
            index: 145,
            name: Some("text_show"),
        }),
        (23, 146) => Some(ExtOpcode {
            category: 23,
            index: 146,
            name: Some("text_set_btn"),
        }),
        (23, 147) => Some(ExtOpcode {
            category: 23,
            index: 147,
            name: Some("text_uninit"),
        }),
        (23, 148) => Some(ExtOpcode {
            category: 23,
            index: 148,
            name: Some("text_set_rect_invalid_param"),
        }),
        (23, 149) => Some(ExtOpcode {
            category: 23,
            index: 149,
            name: Some("text_clear"),
        }),
        (23, 150) => Some(ExtOpcode {
            category: 23,
            index: 150,
            name: None,
        }),
        (23, 151) => Some(ExtOpcode {
            category: 23,
            index: 151,
            name: Some("text_get_time"),
        }),
        (23, 152) => Some(ExtOpcode {
            category: 23,
            index: 152,
            name: Some("text_window_set_alpha"),
        }),
        (23, 153) => Some(ExtOpcode {
            category: 23,
            index: 153,
            name: Some("text_voice_play"),
        }),
        (23, 154) => Some(ExtOpcode {
            category: 23,
            index: 154,
            name: None,
        }),
        (23, 155) => Some(ExtOpcode {
            category: 23,
            index: 155,
            name: Some("text_set_icon_animation_time"),
        }),
        (23, 156) => Some(ExtOpcode {
            category: 23,
            index: 156,
            name: Some("text_w"),
        }),
        (23, 157) => Some(ExtOpcode {
            category: 23,
            index: 157,
            name: Some("text_a"),
        }),
        (23, 158) => Some(ExtOpcode {
            category: 23,
            index: 158,
            name: Some("text_wa"),
        }),
        (23, 159) => Some(ExtOpcode {
            category: 23,
            index: 159,
            name: Some("text_n"),
        }),
        (23, 160) => Some(ExtOpcode {
            category: 23,
            index: 160,
            name: Some("text_cat"),
        }),
        (23, 161) => Some(ExtOpcode {
            category: 23,
            index: 161,
            name: Some("set_history"),
        }),
        (23, 162) => Some(ExtOpcode {
            category: 23,
            index: 162,
            name: Some("is_text_visible"),
        }),
        (23, 163) => Some(ExtOpcode {
            category: 23,
            index: 163,
            name: Some("text_set_base"),
        }),
        (23, 164) => Some(ExtOpcode {
            category: 23,
            index: 164,
            name: Some("enable_voice_cut"),
        }),
        (23, 165) => Some(ExtOpcode {
            category: 23,
            index: 165,
            name: Some("is_voice_cut"),
        }),
        (23, 166) => Some(ExtOpcode {
            category: 23,
            index: 166,
            name: Some("texttimecheckset"),
        }),
        (23, 167) => Some(ExtOpcode {
            category: 23,
            index: 167,
            name: None,
        }),
        (23, 168) => Some(ExtOpcode {
            category: 23,
            index: 168,
            name: None,
        }),
        (23, 169) => Some(ExtOpcode {
            category: 23,
            index: 169,
            name: Some("text_set_color"),
        }),
        (23, 170) => Some(ExtOpcode {
            category: 23,
            index: 170,
            name: Some("textredraw"),
        }),
        (23, 171) => Some(ExtOpcode {
            category: 23,
            index: 171,
            name: Some("set_text_mode"),
        }),
        (23, 172) => Some(ExtOpcode {
            category: 23,
            index: 172,
            name: Some("text_init_visualnovelmode"),
        }),
        (23, 173) => Some(ExtOpcode {
            category: 23,
            index: 173,
            name: Some("text_set_icon_mode"),
        }),
        (23, 174) => Some(ExtOpcode {
            category: 23,
            index: 174,
            name: Some("text_vn_br"),
        }),
        (23, 175) => Some(ExtOpcode {
            category: 23,
            index: 175,
            name: None,
        }),
        (23, 176) => Some(ExtOpcode {
            category: 23,
            index: 176,
            name: None,
        }),
        (23, 177) => Some(ExtOpcode {
            category: 23,
            index: 177,
            name: None,
        }),
        (23, 178) => Some(ExtOpcode {
            category: 23,
            index: 178,
            name: None,
        }),
        (23, 179) => Some(ExtOpcode {
            category: 23,
            index: 179,
            name: Some("tips_get_str"),
        }),
        (23, 180) => Some(ExtOpcode {
            category: 23,
            index: 180,
            name: Some("tips_get_param"),
        }),
        (23, 181) => Some(ExtOpcode {
            category: 23,
            index: 181,
            name: Some("tips_reset"),
        }),
        (23, 182) => Some(ExtOpcode {
            category: 23,
            index: 182,
            name: Some("tips_search"),
        }),
        (23, 183) => Some(ExtOpcode {
            category: 23,
            index: 183,
            name: Some("tips_set_color"),
        }),
        (23, 184) => Some(ExtOpcode {
            category: 23,
            index: 184,
            name: Some("tips_stop"),
        }),
        (23, 185) => Some(ExtOpcode {
            category: 23,
            index: 185,
            name: Some("tips_get_flag"),
        }),
        (23, 186) => Some(ExtOpcode {
            category: 23,
            index: 186,
            name: Some("tips_init"),
        }),
        (23, 187) => Some(ExtOpcode {
            category: 23,
            index: 187,
            name: Some("tips_pause"),
        }),
        (23, 189) => Some(ExtOpcode {
            category: 23,
            index: 189,
            name: Some("voice_play"),
        }),
        (23, 190) => Some(ExtOpcode {
            category: 23,
            index: 190,
            name: Some("voice_stop"),
        }),
        (23, 191) => Some(ExtOpcode {
            category: 23,
            index: 191,
            name: Some("voice_set_volume"),
        }),
        (23, 192) => Some(ExtOpcode {
            category: 23,
            index: 192,
            name: Some("voice_get_volume"),
        }),
        (23, 193) => Some(ExtOpcode {
            category: 23,
            index: 193,
            name: Some("set_voice_info"),
        }),
        (23, 194) => Some(ExtOpcode {
            category: 23,
            index: 194,
            name: Some("voice_enable"),
        }),
        (23, 195) => Some(ExtOpcode {
            category: 23,
            index: 195,
            name: Some("is_voice_enable"),
        }),
        (23, 196) => Some(ExtOpcode {
            category: 23,
            index: 196,
            name: None,
        }),
        (23, 197) => Some(ExtOpcode {
            category: 23,
            index: 197,
            name: Some("bgv_play"),
        }),
        (23, 198) => Some(ExtOpcode {
            category: 23,
            index: 198,
            name: Some("bgv_stop"),
        }),
        (23, 199) => Some(ExtOpcode {
            category: 23,
            index: 199,
            name: Some("bgv_enable"),
        }),
        (23, 200) => Some(ExtOpcode {
            category: 23,
            index: 200,
            name: Some("get_voice_ex_volume"),
        }),
        (23, 201) => Some(ExtOpcode {
            category: 23,
            index: 201,
            name: Some("set_voice_ex_volume"),
        }),
        (23, 202) => Some(ExtOpcode {
            category: 23,
            index: 202,
            name: Some("voice_check_enable"),
        }),
        (23, 203) => Some(ExtOpcode {
            category: 23,
            index: 203,
            name: Some("voice_autopan_initialize"),
        }),
        (23, 204) => Some(ExtOpcode {
            category: 23,
            index: 204,
            name: Some("voice_autopan_enable"),
        }),
        (23, 205) => Some(ExtOpcode {
            category: 23,
            index: 205,
            name: Some("set_voice_autopan_size_over"),
        }),
        (23, 206) => Some(ExtOpcode {
            category: 23,
            index: 206,
            name: Some("is_voice_autopan_enable"),
        }),
        (23, 207) => Some(ExtOpcode {
            category: 23,
            index: 207,
            name: Some("voice_wait"),
        }),
        (23, 208) => Some(ExtOpcode {
            category: 23,
            index: 208,
            name: Some("bgv_pause"),
        }),
        (23, 209) => Some(ExtOpcode {
            category: 23,
            index: 209,
            name: Some("bgv_mute"),
        }),
        (23, 210) => Some(ExtOpcode {
            category: 23,
            index: 210,
            name: Some("set_bgv_volume"),
        }),
        (23, 211) => Some(ExtOpcode {
            category: 23,
            index: 211,
            name: Some("get_bgv_volume"),
        }),
        (23, 212) => Some(ExtOpcode {
            category: 23,
            index: 212,
            name: Some("set_bgv_auto_volume"),
        }),
        (23, 213) => Some(ExtOpcode {
            category: 23,
            index: 213,
            name: Some("voice_mute"),
        }),
        (23, 214) => Some(ExtOpcode {
            category: 23,
            index: 214,
            name: Some("voice_call"),
        }),
        (23, 215) => Some(ExtOpcode {
            category: 23,
            index: 215,
            name: Some("voice_call_clear"),
        }),
        (23, 217) => Some(ExtOpcode {
            category: 23,
            index: 217,
            name: Some("wait"),
        }),
        (23, 218) => Some(ExtOpcode {
            category: 23,
            index: 218,
            name: Some("wait_click"),
        }),
        (23, 219) => Some(ExtOpcode {
            category: 23,
            index: 219,
            name: Some("wait_sync_begin"),
        }),
        (23, 220) => Some(ExtOpcode {
            category: 23,
            index: 220,
            name: Some("wait_sync_release"),
        }),
        (23, 221) => Some(ExtOpcode {
            category: 23,
            index: 221,
            name: Some("wait_sync_end"),
        }),
        (23, 222) => Some(ExtOpcode {
            category: 23,
            index: 222,
            name: None,
        }),
        (23, 223) => Some(ExtOpcode {
            category: 23,
            index: 223,
            name: Some("wait_clear"),
        }),
        (23, 224) => Some(ExtOpcode {
            category: 23,
            index: 224,
            name: Some("wait_click_no_anim"),
        }),
        (23, 225) => Some(ExtOpcode {
            category: 23,
            index: 225,
            name: Some("wait_sync_get_time"),
        }),
        (23, 226) => Some(ExtOpcode {
            category: 23,
            index: 226,
            name: Some("wait_time_push"),
        }),
        (23, 227) => Some(ExtOpcode {
            category: 23,
            index: 227,
            name: Some("wait_time_pop"),
        }),
        (23, 291) => Some(ExtOpcode {
            category: 23,
            index: 291,
            name: None,
        }),
        (23, 294) => Some(ExtOpcode {
            category: 23,
            index: 294,
            name: None,
        }),
        (23, 297) => Some(ExtOpcode {
            category: 23,
            index: 297,
            name: None,
        }),
        _ => None,
    }
}
