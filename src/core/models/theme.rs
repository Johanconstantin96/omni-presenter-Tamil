use eframe::egui::Color32;

/// Representation of a Presentation Theme with deep visual controls
#[derive(Debug, Clone, PartialEq)]
pub struct PresentationTheme {
    pub id: String,
    pub name: String,

    // Plano de Fundo: "color", "gradient", "image", "video"
    pub bg_type: String,
    pub bg_top: Color32,
    pub bg_bottom: Color32,
    pub bg_image_path: String,

    // Transição: "fade", "slide_left", "slide_right", "slide_up", "slide_down", "cut"
    pub transition: String,

    // Tipografia & Formatação
    pub font_family: String, // "Montserrat", "Inter", "Roboto", "Proportional", "Serif", "Monospace"
    pub font_size: f32,      // Tamanho base de fonte
    pub font_weight: u16, // 100..=900 (100 Thin, 300 Light, 400 Regular, 500 Medium, 600 SemiBold, 700 Bold, 800 ExtraBold, 900 Black)
    pub font_bold: bool,
    pub font_italic: bool,
    pub font_underline: bool,
    pub font_strikethrough: bool,
    pub text_color: Color32,
    pub text_case: String, // "original", "uppercase", "lowercase", "titlecase"
    pub line_spacing: f32, // Multiplicador de entrelinha (ex: 1.35)
    pub letter_spacing: f32, // Espaçamento entre letras
    pub align_h: String,   // "left", "center", "right", "justify"
    pub align_v: String,   // "top", "center", "bottom"

    // Contorno do texto (Outline / Stroke)
    pub outline_enabled: bool,
    pub outline_color: Color32,
    pub outline_width: f32,

    // Sombra do texto
    pub shadow_enabled: bool,
    pub shadow_color: Color32,
    pub shadow_blur: f32,
    pub shadow_offset_x: f32,
    pub shadow_offset_y: f32,

    // Bounding Box interativo do texto (coordenadas normalizadas 0.0 a 1.0)
    pub box_x: f32,
    pub box_y: f32,
    pub box_w: f32,
    pub box_h: f32,

    // Configurações da Caixa de Texto
    pub box_bg_enabled: bool,
    pub box_bg_color: Color32,
    pub box_border_enabled: bool,
    pub box_border_color: Color32,
    pub box_border_width: f32,
    pub box_radius: f32,
    pub box_padding: f32,

    // Bandeiras auxiliares
    pub is_gradient: bool,
    pub is_animated: bool,
    pub is_favorite: bool,
    pub sort_order: i32,
    pub options: super::theme_options::ThemeOptions,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ThemeRecord {
    pub id: String,
    pub name: String,
    pub bg_type: String,
    pub bg_top: String,
    pub bg_bottom: String,
    pub bg_image_path: String,
    pub transition: String,
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: u16,
    pub font_bold: bool,
    pub font_italic: bool,
    pub font_underline: bool,
    pub font_strikethrough: bool,
    pub text_color: String,
    pub text_case: String,
    pub line_spacing: f32,
    pub letter_spacing: f32,
    pub align_h: String,
    pub align_v: String,
    pub box_x: f32,
    pub box_y: f32,
    pub box_w: f32,
    pub box_h: f32,
    pub outline_enabled: bool,
    pub outline_color: String,
    pub outline_width: f32,
    pub shadow_enabled: bool,
    pub shadow_color: String,
    pub shadow_blur: f32,
    pub shadow_offset_x: f32,
    pub shadow_offset_y: f32,
    pub box_bg_enabled: bool,
    pub box_bg_color: String,
    pub box_border_enabled: bool,
    pub box_border_color: String,
    pub box_border_width: f32,
    pub box_radius: f32,
    pub box_padding: f32,
    pub is_gradient: bool,
    pub is_animated: bool,
    pub is_favorite: bool,
    pub sort_order: i32,
    #[serde(default)]
    pub options: super::theme_options::ThemeOptions,
}

impl From<&PresentationTheme> for ThemeRecord {
    fn from(t: &PresentationTheme) -> Self {
        Self {
            id: t.id.clone(),
            name: t.name.clone(),
            bg_type: t.bg_type.clone(),
            bg_top: color_to_hex(t.bg_top),
            bg_bottom: color_to_hex(t.bg_bottom),
            bg_image_path: t.bg_image_path.clone(),
            transition: t.transition.clone(),
            font_family: t.font_family.clone(),
            font_size: t.font_size,
            font_weight: t.font_weight,
            font_bold: t.font_bold,
            font_italic: t.font_italic,
            font_underline: t.font_underline,
            font_strikethrough: t.font_strikethrough,
            text_color: color_to_hex(t.text_color),
            text_case: t.text_case.clone(),
            line_spacing: t.line_spacing,
            letter_spacing: t.letter_spacing,
            align_h: t.align_h.clone(),
            align_v: t.align_v.clone(),
            box_x: t.box_x,
            box_y: t.box_y,
            box_w: t.box_w,
            box_h: t.box_h,
            outline_enabled: t.outline_enabled,
            outline_color: color_to_hex(t.outline_color),
            outline_width: t.outline_width,
            shadow_enabled: t.shadow_enabled,
            shadow_color: color_to_hex(t.shadow_color),
            shadow_blur: t.shadow_blur,
            shadow_offset_x: t.shadow_offset_x,
            shadow_offset_y: t.shadow_offset_y,
            box_bg_enabled: t.box_bg_enabled,
            box_bg_color: color_to_hex(t.box_bg_color),
            box_border_enabled: t.box_border_enabled,
            box_border_color: color_to_hex(t.box_border_color),
            box_border_width: t.box_border_width,
            box_radius: t.box_radius,
            box_padding: t.box_padding,
            is_gradient: t.is_gradient,
            is_animated: t.is_animated,
            is_favorite: t.is_favorite,
            sort_order: t.sort_order,
            options: t.options.clone(),
        }
    }
}

impl From<PresentationTheme> for ThemeRecord {
    fn from(t: PresentationTheme) -> Self {
        Self::from(&t)
    }
}

impl From<ThemeRecord> for PresentationTheme {
    fn from(r: ThemeRecord) -> Self {
        Self {
            id: r.id,
            name: r.name,
            bg_type: r.bg_type,
            bg_top: hex_to_color(&r.bg_top),
            bg_bottom: hex_to_color(&r.bg_bottom),
            bg_image_path: r.bg_image_path,
            transition: r.transition,
            font_family: r.font_family,
            font_size: r.font_size,
            font_weight: r.font_weight,
            font_bold: r.font_bold,
            font_italic: r.font_italic,
            font_underline: r.font_underline,
            font_strikethrough: r.font_strikethrough,
            text_color: hex_to_color(&r.text_color),
            text_case: r.text_case,
            line_spacing: r.line_spacing,
            letter_spacing: r.letter_spacing,
            align_h: r.align_h,
            align_v: r.align_v,
            box_x: r.box_x,
            box_y: r.box_y,
            box_w: r.box_w,
            box_h: r.box_h,
            outline_enabled: r.outline_enabled,
            outline_color: hex_to_color(&r.outline_color),
            outline_width: r.outline_width,
            shadow_enabled: r.shadow_enabled,
            shadow_color: hex_to_color(&r.shadow_color),
            shadow_blur: r.shadow_blur,
            shadow_offset_x: r.shadow_offset_x,
            shadow_offset_y: r.shadow_offset_y,
            box_bg_enabled: r.box_bg_enabled,
            box_bg_color: hex_to_color(&r.box_bg_color),
            box_border_enabled: r.box_border_enabled,
            box_border_color: hex_to_color(&r.box_border_color),
            box_border_width: r.box_border_width,
            box_radius: r.box_radius,
            box_padding: r.box_padding,
            is_gradient: r.is_gradient,
            is_animated: r.is_animated,
            is_favorite: r.is_favorite,
            sort_order: r.sort_order,
            options: r.options,
        }
    }
}

impl Default for PresentationTheme {
    fn default() -> Self {
        Self {
            id: "default_theme".to_owned(),
            name: "services-new-theme".to_owned(),
            bg_type: "gradient".to_owned(),
            bg_top: Color32::from_rgb(17, 24, 39),
            bg_bottom: Color32::from_rgb(3, 7, 18),
            bg_image_path: String::new(),
            transition: "fade".to_owned(),
            font_family: "Montserrat".to_owned(),
            font_size: 44.0,
            font_weight: 700,
            font_bold: true,
            font_italic: false,
            font_underline: false,
            font_strikethrough: false,
            text_color: Color32::WHITE,
            text_case: "original".to_owned(),
            line_spacing: 1.36,
            letter_spacing: 0.6,
            align_h: "center".to_owned(),
            align_v: "center".to_owned(),
            outline_enabled: false,
            outline_color: Color32::from_black_alpha(220),
            outline_width: 2.5,
            shadow_enabled: true,
            shadow_color: Color32::from_black_alpha(220),
            shadow_blur: 10.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 3.0,
            box_x: 0.12,
            box_y: 0.18,
            box_w: 0.76,
            box_h: 0.64,
            box_bg_enabled: false,
            box_bg_color: Color32::from_black_alpha(130),
            box_border_enabled: false,
            box_border_color: Color32::from_white_alpha(120),
            box_border_width: 1.5,
            box_radius: 8.0,
            box_padding: 16.0,
            is_gradient: true,
            is_animated: false,
            is_favorite: false,
            sort_order: 0,
            options: super::theme_options::ThemeOptions {
                gradient_angle: 145.0,
                emphasis_color: [250, 204, 21],
                ..Default::default()
            },
        }
    }
}

impl PresentationTheme {
    /// Aradhanai: schlichte Hintergründe „Schwarz“ (weiße Schrift) und
    /// „Weiß“ (schwarze Schrift), Schrift Hind Madurai SemiBold.
    pub fn aradhanai_plain_themes() -> Vec<Self> {
        let plain = |id: &str, name: &str, bg: Color32, text: Color32, sort: i32| Self {
            id: id.to_owned(),
            name: name.to_owned(),
            bg_type: "gradient".to_owned(),
            bg_top: bg,
            bg_bottom: bg,
            font_family: "Hind Madurai".to_owned(),
            font_size: 46.0,
            font_weight: 600,
            font_bold: true,
            text_color: text,
            text_case: "original".to_owned(),
            letter_spacing: 0.0,
            shadow_enabled: false,
            shadow_color: Color32::TRANSPARENT,
            shadow_blur: 0.0,
            shadow_offset_y: 0.0,
            is_gradient: false,
            sort_order: sort,
            ..Default::default()
        };
        vec![
            plain("aradhanai_black", "theme-aradhanai-black", Color32::BLACK, Color32::WHITE, -2),
            plain("aradhanai_white", "theme-aradhanai-white", Color32::WHITE, Color32::BLACK, -1),
        ]
    }

    /// Conjunto de temas padrão ricos prontos para projeção
    pub fn default_themes() -> Vec<Self> {
        vec![
            Self {
                id: "obsidian_pro".to_owned(),
                name: "theme-obsidian-pro".to_owned(),
                bg_type: "gradient".to_owned(),
                bg_top: Color32::from_rgb(26, 29, 38),
                bg_bottom: Color32::from_rgb(11, 12, 16),
                bg_image_path: String::new(),
                transition: "fade".to_owned(),
                font_family: "Inter".to_owned(),
                font_size: 48.0,
                font_weight: 800,
                font_bold: true,
                font_italic: false,
                font_underline: false,
                font_strikethrough: false,
                text_color: Color32::WHITE,
                text_case: "uppercase".to_owned(),
                line_spacing: 1.36,
                letter_spacing: 2.2,
                align_h: "center".to_owned(),
                align_v: "center".to_owned(),
                outline_enabled: false,
                outline_color: Color32::from_black_alpha(220),
                outline_width: 2.5,
                shadow_enabled: true,
                shadow_color: Color32::from_black_alpha(220),
                shadow_blur: 12.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 3.0,
                box_x: 0.12,
                box_y: 0.18,
                box_w: 0.76,
                box_h: 0.64,
                box_bg_enabled: false,
                box_bg_color: Color32::from_black_alpha(130),
                box_border_enabled: false,
                box_border_color: Color32::from_white_alpha(120),
                box_border_width: 1.5,
                box_radius: 8.0,
                box_padding: 16.0,
                is_gradient: true,
                is_animated: false,
                is_favorite: false,
                sort_order: 20,
                options: super::theme_options::ThemeOptions {
                    gradient_angle: 145.0,
                    emphasis_color: [250, 204, 21],
                    ..Default::default()
                },
            },
            Self {
                id: "midnight_sapphire".to_owned(),
                name: "theme-midnight-sapphire".to_owned(),
                bg_type: "gradient".to_owned(),
                bg_top: Color32::from_rgb(10, 28, 60),
                bg_bottom: Color32::from_rgb(14, 114, 138),
                bg_image_path: String::new(),
                transition: "fade".to_owned(),
                font_family: "Montserrat".to_owned(),
                font_size: 48.0,
                font_weight: 700,
                font_bold: true,
                font_italic: false,
                font_underline: false,
                font_strikethrough: false,
                text_color: Color32::WHITE,
                text_case: "uppercase".to_owned(),
                line_spacing: 1.36,
                letter_spacing: 1.8,
                align_h: "center".to_owned(),
                align_v: "center".to_owned(),
                outline_enabled: false,
                outline_color: Color32::from_black_alpha(200),
                outline_width: 2.5,
                shadow_enabled: true,
                shadow_color: Color32::from_black_alpha(200),
                shadow_blur: 10.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 2.5,
                box_x: 0.12,
                box_y: 0.18,
                box_w: 0.76,
                box_h: 0.64,
                box_bg_enabled: false,
                box_bg_color: Color32::from_black_alpha(130),
                box_border_enabled: false,
                box_border_color: Color32::from_white_alpha(120),
                box_border_width: 1.5,
                box_radius: 8.0,
                box_padding: 16.0,
                is_gradient: true,
                is_animated: false,
                is_favorite: false,
                sort_order: 22,
                options: super::theme_options::ThemeOptions {
                    gradient_angle: 135.0,
                    emphasis_color: [56, 189, 248],
                    ..Default::default()
                },
            },
            Self {
                id: "golden_champagne".to_owned(),
                name: "theme-golden-champagne".to_owned(),
                bg_type: "gradient".to_owned(),
                bg_top: Color32::from_rgb(186, 131, 84),
                bg_bottom: Color32::from_rgb(245, 227, 207),
                bg_image_path: String::new(),
                transition: "fade".to_owned(),
                font_family: "Montserrat".to_owned(),
                font_size: 48.0,
                font_weight: 700,
                font_bold: true,
                font_italic: false,
                font_underline: false,
                font_strikethrough: false,
                text_color: Color32::WHITE,
                text_case: "uppercase".to_owned(),
                line_spacing: 1.35,
                letter_spacing: 1.8,
                align_h: "center".to_owned(),
                align_v: "center".to_owned(),
                outline_enabled: false,
                outline_color: Color32::from_black_alpha(220),
                outline_width: 2.5,
                shadow_enabled: true,
                shadow_color: Color32::from_rgba_unmultiplied(65, 30, 10, 160),
                shadow_blur: 8.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 2.0,
                box_x: 0.12,
                box_y: 0.18,
                box_w: 0.76,
                box_h: 0.64,
                box_bg_enabled: false,
                box_bg_color: Color32::from_black_alpha(130),
                box_border_enabled: false,
                box_border_color: Color32::from_white_alpha(120),
                box_border_width: 1.5,
                box_radius: 8.0,
                box_padding: 16.0,
                is_gradient: true,
                is_animated: false,
                is_favorite: false,
                sort_order: 21,
                options: super::theme_options::ThemeOptions {
                    gradient_angle: 135.0,
                    emphasis_color: [254, 240, 138],
                    ..Default::default()
                },
            },
            Self {
                id: "warm_ember".to_owned(),
                name: "theme-warm-ember".to_owned(),
                bg_type: "gradient".to_owned(),
                bg_top: Color32::from_rgb(168, 68, 44),
                bg_bottom: Color32::from_rgb(230, 164, 110),
                bg_image_path: String::new(),
                transition: "fade".to_owned(),
                font_family: "Montserrat".to_owned(),
                font_size: 48.0,
                font_weight: 700,
                font_bold: true,
                font_italic: false,
                font_underline: false,
                font_strikethrough: false,
                text_color: Color32::WHITE,
                text_case: "uppercase".to_owned(),
                line_spacing: 1.35,
                letter_spacing: 1.8,
                align_h: "center".to_owned(),
                align_v: "center".to_owned(),
                outline_enabled: false,
                outline_color: Color32::from_black_alpha(220),
                outline_width: 2.5,
                shadow_enabled: true,
                shadow_color: Color32::from_rgba_unmultiplied(50, 15, 10, 170),
                shadow_blur: 8.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 2.0,
                box_x: 0.12,
                box_y: 0.18,
                box_w: 0.76,
                box_h: 0.64,
                box_bg_enabled: false,
                box_bg_color: Color32::from_black_alpha(130),
                box_border_enabled: false,
                box_border_color: Color32::from_white_alpha(120),
                box_border_width: 1.5,
                box_radius: 8.0,
                box_padding: 16.0,
                is_gradient: true,
                is_animated: false,
                is_favorite: false,
                sort_order: 23,
                options: super::theme_options::ThemeOptions {
                    gradient_angle: 140.0,
                    emphasis_color: [251, 191, 36],
                    ..Default::default()
                },
            },
            Self {
                id: "nordic_sage".to_owned(),
                name: "theme-nordic-sage".to_owned(),
                bg_type: "gradient".to_owned(),
                bg_top: Color32::from_rgb(60, 85, 72),
                bg_bottom: Color32::from_rgb(148, 170, 156),
                bg_image_path: String::new(),
                transition: "fade".to_owned(),
                font_family: "Montserrat".to_owned(),
                font_size: 48.0,
                font_weight: 700,
                font_bold: true,
                font_italic: false,
                font_underline: false,
                font_strikethrough: false,
                text_color: Color32::WHITE,
                text_case: "uppercase".to_owned(),
                line_spacing: 1.36,
                letter_spacing: 1.8,
                align_h: "center".to_owned(),
                align_v: "center".to_owned(),
                outline_enabled: false,
                outline_color: Color32::from_black_alpha(200),
                outline_width: 2.5,
                shadow_enabled: true,
                shadow_color: Color32::from_rgba_unmultiplied(20, 35, 25, 160),
                shadow_blur: 8.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 2.0,
                box_x: 0.12,
                box_y: 0.18,
                box_w: 0.76,
                box_h: 0.64,
                box_bg_enabled: false,
                box_bg_color: Color32::from_black_alpha(130),
                box_border_enabled: false,
                box_border_color: Color32::from_white_alpha(120),
                box_border_width: 1.5,
                box_radius: 8.0,
                box_padding: 16.0,
                is_gradient: true,
                is_animated: false,
                is_favorite: false,
                sort_order: 25,
                options: super::theme_options::ThemeOptions {
                    gradient_angle: 150.0,
                    emphasis_color: [253, 230, 138],
                    ..Default::default()
                },
            },
            Self {
                id: "clean_alabaster".to_owned(),
                name: "theme-clean-alabaster".to_owned(),
                bg_type: "gradient".to_owned(),
                bg_top: Color32::from_rgb(246, 241, 233),
                bg_bottom: Color32::from_rgb(230, 220, 206),
                bg_image_path: String::new(),
                transition: "fade".to_owned(),
                font_family: "Montserrat".to_owned(),
                font_size: 48.0,
                font_weight: 700,
                font_bold: true,
                font_italic: false,
                font_underline: false,
                font_strikethrough: false,
                text_color: Color32::from_rgb(24, 28, 36),
                text_case: "uppercase".to_owned(),
                line_spacing: 1.38,
                letter_spacing: 1.8,
                align_h: "center".to_owned(),
                align_v: "center".to_owned(),
                outline_enabled: false,
                outline_color: Color32::TRANSPARENT,
                outline_width: 0.0,
                shadow_enabled: false,
                shadow_color: Color32::TRANSPARENT,
                shadow_blur: 0.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 0.0,
                box_x: 0.12,
                box_y: 0.18,
                box_w: 0.76,
                box_h: 0.64,
                box_bg_enabled: false,
                box_bg_color: Color32::from_black_alpha(130),
                box_border_enabled: false,
                box_border_color: Color32::from_white_alpha(120),
                box_border_width: 1.5,
                box_radius: 8.0,
                box_padding: 16.0,
                is_gradient: true,
                is_animated: false,
                is_favorite: false,
                sort_order: 27,
                options: super::theme_options::ThemeOptions {
                    gradient_angle: 160.0,
                    emphasis_color: [180, 83, 9],
                    ..Default::default()
                },
            },
            Self {
                id: "bold_anthem".to_owned(),
                name: "theme-bold-anthem".to_owned(),
                bg_type: "gradient".to_owned(),
                bg_top: Color32::from_rgb(22, 27, 36),
                bg_bottom: Color32::from_rgb(10, 12, 17),
                bg_image_path: String::new(),
                transition: "fade".to_owned(),
                font_family: "Oswald".to_owned(),
                font_size: 52.0,
                font_weight: 800,
                font_bold: true,
                font_italic: false,
                font_underline: false,
                font_strikethrough: false,
                text_color: Color32::WHITE,
                text_case: "uppercase".to_owned(),
                line_spacing: 1.26,
                letter_spacing: 2.4,
                align_h: "center".to_owned(),
                align_v: "center".to_owned(),
                outline_enabled: false,
                outline_color: Color32::from_black_alpha(220),
                outline_width: 2.5,
                shadow_enabled: true,
                shadow_color: Color32::from_black_alpha(230),
                shadow_blur: 12.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 3.0,
                box_x: 0.12,
                box_y: 0.18,
                box_w: 0.76,
                box_h: 0.64,
                box_bg_enabled: false,
                box_bg_color: Color32::from_black_alpha(130),
                box_border_enabled: false,
                box_border_color: Color32::from_white_alpha(120),
                box_border_width: 1.5,
                box_radius: 8.0,
                box_padding: 16.0,
                is_gradient: true,
                is_animated: false,
                is_favorite: false,
                sort_order: 28,
                options: super::theme_options::ThemeOptions {
                    gradient_angle: 130.0,
                    emphasis_color: [252, 211, 77],
                    ..Default::default()
                },
            },
            Self {
                id: "frosted_glass_card".to_owned(),
                name: "theme-frosted-glass-card".to_owned(),
                bg_type: "gradient".to_owned(),
                bg_top: Color32::from_rgb(20, 24, 38),
                bg_bottom: Color32::from_rgb(8, 10, 16),
                bg_image_path: String::new(),
                transition: "fade".to_owned(),
                font_family: "Montserrat".to_owned(),
                font_size: 46.0,
                font_weight: 700,
                font_bold: true,
                font_italic: false,
                font_underline: false,
                font_strikethrough: false,
                text_color: Color32::WHITE,
                text_case: "uppercase".to_owned(),
                line_spacing: 1.36,
                letter_spacing: 1.6,
                align_h: "center".to_owned(),
                align_v: "center".to_owned(),
                outline_enabled: false,
                outline_color: Color32::from_black_alpha(200),
                outline_width: 2.5,
                shadow_enabled: true,
                shadow_color: Color32::from_black_alpha(200),
                shadow_blur: 8.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 2.5,
                box_x: 0.08,
                box_y: 0.14,
                box_w: 0.84,
                box_h: 0.72,
                box_bg_enabled: true,
                box_bg_color: Color32::from_rgba_unmultiplied(255, 255, 255, 18),
                box_border_enabled: true,
                box_border_color: Color32::from_white_alpha(50),
                box_border_width: 1.2,
                box_radius: 16.0,
                box_padding: 24.0,
                is_gradient: true,
                is_animated: false,
                is_favorite: false,
                sort_order: 30,
                options: super::theme_options::ThemeOptions {
                    gradient_angle: 135.0,
                    ..Default::default()
                },
            },
            Self {
                id: "broadcast_lower_third".to_owned(),
                name: "theme-broadcast-lower-third".to_owned(),
                bg_type: "color".to_owned(),
                bg_top: Color32::TRANSPARENT,
                bg_bottom: Color32::TRANSPARENT,
                bg_image_path: String::new(),
                transition: "fade".to_owned(),
                font_family: "Montserrat".to_owned(),
                font_size: 34.0,
                font_weight: 700,
                font_bold: true,
                font_italic: false,
                font_underline: false,
                font_strikethrough: false,
                text_color: Color32::WHITE,
                text_case: "original".to_owned(),
                line_spacing: 1.32,
                letter_spacing: 0.6,
                align_h: "center".to_owned(),
                align_v: "center".to_owned(),
                outline_enabled: false,
                outline_color: Color32::from_black_alpha(200),
                outline_width: 2.0,
                shadow_enabled: true,
                shadow_color: Color32::from_black_alpha(180),
                shadow_blur: 6.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 2.0,
                box_x: 0.05,
                box_y: 0.74,
                box_w: 0.90,
                box_h: 0.20,
                box_bg_enabled: true,
                box_bg_color: Color32::from_rgba_unmultiplied(12, 17, 28, 220),
                box_border_enabled: true,
                box_border_color: Color32::from_white_alpha(45),
                box_border_width: 1.2,
                box_radius: 12.0,
                box_padding: 14.0,
                is_gradient: false,
                is_animated: false,
                is_favorite: false,
                sort_order: 31,
                options: Default::default(),
            },
        ]
    }

    /// Aplica a transformação de caso (MAIÚSCULAS, minúsculas, título, original) ao texto
    pub fn transform_text(&self, text: &str) -> String {
        match self.text_case.as_str() {
            "uppercase" => text.to_uppercase(),
            "lowercase" => text.to_lowercase(),
            "titlecase" => {
                let mut out = String::with_capacity(text.len());
                for line in text.lines() {
                    let mut prev_whitespace = true;
                    for c in line.chars() {
                        if c.is_whitespace() {
                            prev_whitespace = true;
                            out.push(c);
                        } else if prev_whitespace {
                            out.extend(c.to_uppercase());
                            prev_whitespace = false;
                        } else {
                            out.extend(c.to_lowercase());
                        }
                    }
                    out.push('\n');
                }
                if out.ends_with('\n') && !text.ends_with('\n') {
                    out.pop();
                }
                out
            }
            _ => text.to_string(),
        }
    }

    /// Retorna o nome internacionalizado do tema de acordo com o idioma fornecido
    pub fn display_name(&self, language: crate::core::i18n::Language) -> String {
        // 1. Shaders / temas animados
        let shader_kind = self
            .id
            .strip_prefix("shader_")
            .or_else(|| {
                if self.bg_type == "shader" && !self.options.shader.kind.is_empty() {
                    Some(self.options.shader.kind.as_str())
                } else {
                    None
                }
            })
            .or_else(|| match self.name.as_str() {
                "Aurora Boreal" | "Northern Aurora" | "Aurora · Dinâmico" | "Aurora · Animated" | "Aurora · Animado" | "Aurora" => Some("aurora"),
                "Nebulosa Estelar" | "Stellar Nebula" | "Nebulosa · Dinâmico" | "Nebula · Animated" | "Nebulosa · Animado" | "Nebula" | "Nebulosa" => Some("nebula"),
                "Ondas Suaves" | "Smooth Waves" | "Ondas · Dinâmico" | "Waves · Animated" | "Olas · Animado" | "Olas Suaves" | "Waves" | "Ondas" => Some("waves"),
                "Seda Fluida" | "Flowing Silk" | "Seda · Dinâmico" | "Silk · Animated" | "Seda · Animado" | "Silk" | "Seda" => Some("silk"),
                "Partículas de Luz" | "Luminous Particles" | "Partículas · Dinâmico" | "Particles · Animated" | "Partículas · Animado" | "Particles" | "Partículas" => Some("particles"),
                "Anéis Concéntricos" | "Concentric Rings" | "Anillos Concéntricos" | "Anéis · Dinâmico" | "Rings · Animated" | "Anillos · Animado" | "Rings" | "Anéis" => Some("rings"),
                "Fluxo Cósmico" | "Cosmic Flow" | "Flujo Cósmico" | "Fluido · Dinâmico" | "Fluid · Animated" | "Fluido · Animado" | "Fluid" | "Fluido" => Some("fluid"),
                "Grade Luminosa" | "Luminous Grid" | "Cuadrícula Luminosa" | "Grade de Luz · Dinâmico" | "Light grid · Animated" | "Grid" => Some("grid"),
                "Luzes Bokeh" | "Bokeh Lights" | "Luces Bokeh" | "Bokeh · Dinâmico" | "Bokeh · Animated" | "Bokeh · Animado" | "Bokeh" => Some("bokeh"),
                "Gradiente Fluido" | "Liquid Gradient" | "Fluid Gradient" | "Prisma Cristalino" | "Crystal Prism" | "Prisma · Dinâmico" | "Prism · Animated" | "Prisma · Animado" | "Prism" | "Prisma" => Some("mesh"),
                "Raios Celestiais" | "Celestial Rays" | "Rayos Celestiales" | "Rays" => Some("rays"),
                "Céu Estrelado" | "Starfield" | "Cielo Estrellado" | "Constelação" | "Constellation" | "Constelación" => Some("stars"),
                "Pulso Cósmico" | "Cosmic Pulse" | "Pulso" | "Pulse" => Some("pulse"),
                _ => None,
            });

        if let Some(kind) = shader_kind {
            let name_key = match kind {
                "aurora" => Some("settings-aurora"),
                "nebula" => Some("settings-nebula"),
                "waves" => Some("settings-waves"),
                "silk" => Some("settings-silk"),
                "particles" => Some("settings-particles"),
                "rings" => Some("settings-rings"),
                "fluid" => Some("settings-fluid"),
                "grid" => Some("settings-light-grid"),
                "bokeh" => Some("settings-bokeh"),
                "mesh" | "prism" => Some("settings-mesh"),
                "rays" => Some("settings-rays"),
                "stars" | "constellation" => Some("settings-stars"),
                "pulse" => Some("settings-pulse"),
                _ => None,
            };
            if let Some(key) = name_key {
                let translated = language.text(key);
                return translated.to_string();
            }
        }

        // 2. Temas padrão estáticos por ID
        match self.id.as_str() {
            "obsidian_pro" => return language.text("theme-obsidian-pro").to_string(),
            "golden_champagne" => return language.text("theme-golden-champagne").to_string(),
            "midnight_sapphire" => return language.text("theme-midnight-sapphire").to_string(),
            "warm_ember" => return language.text("theme-warm-ember").to_string(),
            "deep_emerald" => return language.text("theme-deep-emerald").to_string(),
            "nordic_sage" => return language.text("theme-nordic-sage").to_string(),
            "scripture_serif" => return language.text("theme-scripture-serif").to_string(),
            "celestial_violet" => return language.text("theme-celestial-violet").to_string(),
            "bold_anthem" => return language.text("theme-bold-anthem").to_string(),
            "clean_alabaster" => return language.text("theme-clean-alabaster").to_string(),
            "broadcast_lower_third" => return language.text("theme-broadcast-lower-third").to_string(),
            "frosted_glass_card" => return language.text("theme-frosted-glass-card").to_string(),
            "default_theme" => return language.text("services-new-theme").to_string(),
            _ => {}
        }

        // 3. Compatibilidade com nomes legados armazenados em banco
        match self.name.as_str() {
            "Obsidian Minimalista" | "Obsidian Minimalist" | "Obsidiana Minimalista" => {
                return language.text("theme-obsidian-pro").to_string();
            }
            "Safira da Meia-Noite" | "Midnight Sapphire" | "Zafiro de Medianoche" => {
                return language.text("theme-midnight-sapphire").to_string();
            }
            "Brasas & Bronze" | "Warm Ember" | "Brasas y Bronce" => {
                return language.text("theme-warm-ember").to_string();
            }
            "Esmeralda Profunda" | "Deep Emerald" => {
                return language.text("theme-deep-emerald").to_string();
            }
            "Escrituras & Liturgia" | "Scripture & Liturgy" | "Escrituras y Liturgia" => {
                return language.text("theme-scripture-serif").to_string();
            }
            "Ametista & Crepúsculo" | "Celestial Amethyst" | "Amatista Celestial" => {
                return language.text("theme-celestial-violet").to_string();
            }
            "Louvor Vibrante" | "Bold Anthem" | "Alabanza Vibrante" => {
                return language.text("theme-bold-anthem").to_string();
            }
            "Alabastro" | "Clean Alabaster" => {
                return language.text("theme-clean-alabaster").to_string();
            }
            "Tarja Simples" | "Broadcast Lower Third" | "Cintillo de Transmisión" => {
                return language.text("theme-broadcast-lower-third").to_string();
            }
            "Vidro Fumê Card" | "Frosted Glass Card" | "Tarjeta de Vidrio Esmerilado" => {
                return language.text("theme-frosted-glass-card").to_string();
            }
            "Novo Tema" | "New theme" | "Nuevo tema" => {
                return language.text("services-new-theme").to_string();
            }
            _ => {}
        }

        // 4. Se o nome for uma chave Fluent válida
        if self.name.starts_with("theme-") || self.name.starts_with("services-") {
            return language.text(&self.name).to_string();
        }
        if let Some(text) = language.lookup(&self.name) {
            return text.to_string();
        }

        // 5. Fallback para nome personalizado existente
        self.name.clone()
    }
}

// ==========================================
// Funções Utilitárias para Conversão de Cores
// ==========================================
pub fn color_to_hex(c: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}{:02X}", c.r(), c.g(), c.b(), c.a())
}

pub fn hex_to_color(s: &str) -> Color32 {
    let s = s.trim().trim_start_matches('#');
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(255);
        Color32::from_rgb(r, g, b)
    } else if s.len() == 8 {
        let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(255);
        let a = u8::from_str_radix(&s[6..8], 16).unwrap_or(255);
        Color32::from_rgba_premultiplied(r, g, b, a)
    } else {
        Color32::WHITE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_case_transformations() {
        let mut theme = PresentationTheme::default();
        let input = "Quem é este que vem de Edom?";

        theme.text_case = "uppercase".to_owned();
        assert_eq!(theme.transform_text(input), "QUEM É ESTE QUE VEM DE EDOM?");

        theme.text_case = "lowercase".to_owned();
        assert_eq!(theme.transform_text(input), "quem é este que vem de edom?");

        theme.text_case = "titlecase".to_owned();
        assert_eq!(theme.transform_text(input), "Quem É Este Que Vem De Edom?");

        theme.text_case = "original".to_owned();
        assert_eq!(theme.transform_text(input), input);
    }

    #[test]
    fn test_hex_color_roundtrip() {
        let color = Color32::from_rgb(59, 130, 246);
        let hex = color_to_hex(color);
        let parsed = hex_to_color(&hex);
        assert_eq!(parsed.r(), color.r());
        assert_eq!(parsed.g(), color.g());
        assert_eq!(parsed.b(), color.b());
    }

    #[test]
    fn test_theme_display_name_internationalization() {
        use crate::core::i18n::Language;

        // Static theme by ID
        let mut theme = PresentationTheme::default();
        theme.id = "obsidian_pro".into();
        theme.name = "theme-obsidian-pro".into();
        assert_eq!(theme.display_name(Language::English), "Obsidian Minimalist");
        assert_eq!(theme.display_name(Language::Portuguese), "Obsidian Minimalista");
        assert_eq!(theme.display_name(Language::Spanish), "Obsidiana Minimalista");

        // Shader theme by ID
        let mut shader = PresentationTheme::default();
        shader.id = "shader_aurora".into();
        shader.name = "Aurora".into();
        shader.bg_type = "shader".into();
        assert_eq!(shader.display_name(Language::English), "Aurora");
        assert_eq!(shader.display_name(Language::Portuguese), "Aurora");
        assert_eq!(shader.display_name(Language::Spanish), "Aurora");

        // Legacy stored name
        let mut legacy = PresentationTheme::default();
        legacy.id = "custom_123".into();
        legacy.name = "Safira da Meia-Noite".into();
        assert_eq!(legacy.display_name(Language::English), "Ocean");
        assert_eq!(legacy.display_name(Language::Portuguese), "Oceano");
        assert_eq!(legacy.display_name(Language::Spanish), "Océano");

        // Custom user-defined name
        let mut custom = PresentationTheme::default();
        custom.id = "my_custom_theme".into();
        custom.name = "Culto de Domingo".into();
        assert_eq!(custom.display_name(Language::English), "Culto de Domingo");
        assert_eq!(custom.display_name(Language::Portuguese), "Culto de Domingo");
        assert_eq!(custom.display_name(Language::Spanish), "Culto de Domingo");
    }
}
