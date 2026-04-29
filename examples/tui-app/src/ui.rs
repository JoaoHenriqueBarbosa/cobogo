use crate::App;
use clay::context::Context;
use clay::elements::*;
use clay::layout::*;
use clay::render::RenderCommand;
use clay::types::*;

fn measure_text(text: &str, config: &TextElementConfig, _user_data: usize) -> Dimensions {
    let width = text.len() as f32 * if config.font_size > 1 { 1.0 } else { 1.0 };
    let height = 1.0;
    Dimensions::new(width, height)
}

struct Theme {
    bg: Color,
    header_bg: Color,
    sidebar_bg: Color,
    content_bg: Color,
    footer_bg: Color,
    text: Color,
    text_dim: Color,
    accent: Color,
    selected: Color,
    border: Color,
    tab_active: Color,
    tab_inactive: Color,
}

impl Theme {
    fn dark() -> Self {
        Self {
            bg: Color::new(22.0, 22.0, 30.0, 255.0),
            header_bg: Color::new(40.0, 42.0, 54.0, 255.0),
            sidebar_bg: Color::new(30.0, 32.0, 44.0, 255.0),
            content_bg: Color::new(35.0, 37.0, 48.0, 255.0),
            footer_bg: Color::new(40.0, 42.0, 54.0, 255.0),
            text: Color::new(248.0, 248.0, 242.0, 255.0),
            text_dim: Color::new(98.0, 114.0, 164.0, 255.0),
            accent: Color::new(139.0, 233.0, 253.0, 255.0),
            selected: Color::new(68.0, 71.0, 90.0, 255.0),
            border: Color::new(68.0, 71.0, 90.0, 255.0),
            tab_active: Color::new(189.0, 147.0, 249.0, 255.0),
            tab_inactive: Color::new(68.0, 71.0, 90.0, 255.0),
        }
    }

    fn light() -> Self {
        Self {
            bg: Color::new(250.0, 250.0, 250.0, 255.0),
            header_bg: Color::new(60.0, 60.0, 120.0, 255.0),
            sidebar_bg: Color::new(235.0, 235.0, 245.0, 255.0),
            content_bg: Color::new(255.0, 255.0, 255.0, 255.0),
            footer_bg: Color::new(60.0, 60.0, 120.0, 255.0),
            text: Color::new(30.0, 30.0, 30.0, 255.0),
            text_dim: Color::new(120.0, 120.0, 140.0, 255.0),
            accent: Color::new(30.0, 100.0, 200.0, 255.0),
            selected: Color::new(210.0, 220.0, 240.0, 255.0),
            border: Color::new(200.0, 200.0, 210.0, 255.0),
            tab_active: Color::new(100.0, 60.0, 200.0, 255.0),
            tab_inactive: Color::new(180.0, 180.0, 190.0, 255.0),
        }
    }
}

pub fn build_layout(app: &App, width: f32, height: f32) -> Vec<RenderCommand> {
    let mut ctx = Context::new(Dimensions::new(width, height));
    ctx.set_measure_text_function(measure_text, 0);

    let theme = if app.theme_dark { Theme::dark() } else { Theme::light() };

    ctx.begin_layout();

    let root_id = ctx.id("Root");
    ctx.with_element(root_id, ElementDeclaration {
        layout: LayoutConfig {
            sizing: Sizing {
                width: SizingAxis::grow(0.0, f32::MAX),
                height: SizingAxis::grow(0.0, f32::MAX),
            },
            layout_direction: LayoutDirection::TopToBottom,
            ..Default::default()
        },
        background_color: theme.bg,
        ..Default::default()
    }, |ctx| {
        build_header(ctx, app, &theme);
        build_body(ctx, app, &theme);
        build_footer(ctx, app, &theme);
    });

    ctx.end_layout()
}

fn build_header(ctx: &mut Context, app: &App, theme: &Theme) {
    let id = ctx.id("Header");
    ctx.with_element(id, ElementDeclaration {
        layout: LayoutConfig {
            sizing: Sizing {
                width: SizingAxis::grow(0.0, f32::MAX),
                height: SizingAxis::fixed(3.0),
            },
            padding: Padding { left: 2, right: 2, top: 1, bottom: 1 },
            child_alignment: ChildAlignment { y: AlignmentY::Center, ..Default::default() },
            layout_direction: LayoutDirection::LeftToRight,
            child_gap: 2,
            ..Default::default()
        },
        background_color: theme.header_bg,
        ..Default::default()
    }, |ctx| {
        let title_cfg = TextElementConfig {
            text_color: theme.accent,
            font_size: 1,
            ..Default::default()
        };
        ctx.text("◆ Clay TUI Demo", &title_cfg);

        // Spacer
        let spacer_id = ctx.id("HeaderSpacer");
        ctx.with_element(spacer_id, ElementDeclaration {
            layout: LayoutConfig {
                sizing: Sizing { width: SizingAxis::grow(0.0, f32::MAX), ..Default::default() },
                ..Default::default()
            },
            ..Default::default()
        }, |_| {});

        // Tabs
        let tab_names = ["Dashboard", "Settings", "About"];
        for (i, name) in tab_names.iter().enumerate() {
            let tab_id = ctx.idi("Tab", i as u32);
            let is_active = app.active_tab == i;
            let tab_color = if is_active { theme.tab_active } else { theme.tab_inactive };
            ctx.with_element(tab_id, ElementDeclaration {
                layout: LayoutConfig {
                    padding: Padding { left: 1, right: 1, top: 0, bottom: 0 },
                    ..Default::default()
                },
                background_color: if is_active { theme.selected } else { Color::default() },
                border: BorderConfig {
                    color: tab_color,
                    width: BorderWidth { bottom: 1, ..Default::default() },
                },
                ..Default::default()
            }, |ctx| {
                ctx.text(name, &TextElementConfig {
                    text_color: if is_active { theme.text } else { theme.text_dim },
                    font_size: 1,
                    ..Default::default()
                });
            });
        }
    });
}

fn build_body(ctx: &mut Context, app: &App, theme: &Theme) {
    let id = ctx.id("Body");
    ctx.with_element(id, ElementDeclaration {
        layout: LayoutConfig {
            sizing: Sizing {
                width: SizingAxis::grow(0.0, f32::MAX),
                height: SizingAxis::grow(0.0, f32::MAX),
            },
            layout_direction: LayoutDirection::LeftToRight,
            ..Default::default()
        },
        ..Default::default()
    }, |ctx| {
        if app.sidebar_visible {
            build_sidebar(ctx, app, theme);
        }
        build_content(ctx, app, theme);
    });
}

fn build_sidebar(ctx: &mut Context, app: &App, theme: &Theme) {
    let id = ctx.id("Sidebar");
    ctx.with_element(id, ElementDeclaration {
        layout: LayoutConfig {
            sizing: Sizing {
                width: SizingAxis::fixed(24.0),
                height: SizingAxis::grow(0.0, f32::MAX),
            },
            layout_direction: LayoutDirection::TopToBottom,
            padding: Padding { left: 1, right: 1, top: 1, bottom: 1 },
            child_gap: 0,
            ..Default::default()
        },
        background_color: theme.sidebar_bg,
        border: BorderConfig {
            color: theme.border,
            width: BorderWidth { right: 1, ..Default::default() },
        },
        ..Default::default()
    }, |ctx| {
        let label_cfg = TextElementConfig {
            text_color: theme.text_dim,
            font_size: 1,
            ..Default::default()
        };
        ctx.text("Navigation", &label_cfg);

        let items = ["Overview", "Analytics", "Reports", "Users", "Config", "Logs"];
        for (i, item) in items.iter().enumerate() {
            let item_id = ctx.idi("SidebarItem", i as u32);
            let is_selected = app.selected_item == i;
            ctx.with_element(item_id, ElementDeclaration {
                layout: LayoutConfig {
                    sizing: Sizing {
                        width: SizingAxis::grow(0.0, f32::MAX),
                        height: SizingAxis::fixed(1.0),
                    },
                    padding: Padding { left: 1, right: 1, top: 0, bottom: 0 },
                    child_alignment: ChildAlignment { y: AlignmentY::Center, ..Default::default() },
                    ..Default::default()
                },
                background_color: if is_selected { theme.selected } else { Color::default() },
                ..Default::default()
            }, |ctx| {
                let prefix = if is_selected { "▸ " } else { "  " };
                let label = format!("{}{}", prefix, item);
                ctx.text(&label, &TextElementConfig {
                    text_color: if is_selected { theme.accent } else { theme.text },
                    font_size: 1,
                    ..Default::default()
                });
            });
        }
    });
}

fn build_content(ctx: &mut Context, app: &App, theme: &Theme) {
    let id = ctx.id("Content");
    ctx.with_element(id, ElementDeclaration {
        layout: LayoutConfig {
            sizing: Sizing {
                width: SizingAxis::grow(0.0, f32::MAX),
                height: SizingAxis::grow(0.0, f32::MAX),
            },
            layout_direction: LayoutDirection::TopToBottom,
            padding: Padding { left: 2, right: 2, top: 1, bottom: 1 },
            child_gap: 1,
            ..Default::default()
        },
        background_color: theme.content_bg,
        ..Default::default()
    }, |ctx| {
        match app.active_tab {
            0 => build_dashboard_tab(ctx, app, theme),
            1 => build_settings_tab(ctx, app, theme),
            2 => build_about_tab(ctx, theme),
            _ => {}
        }
    });
}

fn build_dashboard_tab(ctx: &mut Context, app: &App, theme: &Theme) {
    ctx.text("Dashboard", &TextElementConfig {
        text_color: theme.accent,
        font_size: 1,
        ..Default::default()
    });

    // Stats row
    let stats_id = ctx.id("StatsRow");
    ctx.with_element(stats_id, ElementDeclaration {
        layout: LayoutConfig {
            sizing: Sizing {
                width: SizingAxis::grow(0.0, f32::MAX),
                height: SizingAxis::fixed(5.0),
            },
            layout_direction: LayoutDirection::LeftToRight,
            child_gap: 2,
            ..Default::default()
        },
        ..Default::default()
    }, |ctx| {
        let cards = [
            ("Counter", format!("{}", app.counter)),
            ("Selected", format!("Item {}", app.selected_item)),
            ("Tab", format!("{}", app.active_tab + 1)),
            ("Sidebar", if app.sidebar_visible { "On".into() } else { "Off".into() }),
        ];
        for (i, (label, value)) in cards.iter().enumerate() {
            let card_id = ctx.idi("StatCard", i as u32);
            ctx.with_element(card_id, ElementDeclaration {
                layout: LayoutConfig {
                    sizing: Sizing {
                        width: SizingAxis::grow(0.0, f32::MAX),
                        height: SizingAxis::grow(0.0, f32::MAX),
                    },
                    layout_direction: LayoutDirection::TopToBottom,
                    padding: Padding { left: 2, right: 2, top: 1, bottom: 1 },
                    child_gap: 1,
                    ..Default::default()
                },
                background_color: theme.sidebar_bg,
                border: BorderConfig {
                    color: theme.border,
                    width: BorderWidth::outside(1),
                },
                corner_radius: CornerRadius::all(1.0),
                ..Default::default()
            }, |ctx| {
                ctx.text(label, &TextElementConfig {
                    text_color: theme.text_dim,
                    font_size: 1,
                    ..Default::default()
                });
                ctx.text(value, &TextElementConfig {
                    text_color: theme.accent,
                    font_size: 1,
                    ..Default::default()
                });
            });
        }
    });

    // Activity log
    ctx.text("Recent Activity", &TextElementConfig {
        text_color: theme.text_dim, font_size: 1, ..Default::default()
    });

    let log_entries = [
        "System initialized successfully",
        "User session started",
        "Configuration loaded from disk",
        "Background tasks running",
        "All services healthy",
    ];
    for (i, entry) in log_entries.iter().enumerate() {
        let entry_id = ctx.idi("LogEntry", i as u32);
        ctx.with_element(entry_id, ElementDeclaration {
            layout: LayoutConfig {
                sizing: Sizing {
                    width: SizingAxis::grow(0.0, f32::MAX),
                    height: SizingAxis::fixed(1.0),
                },
                padding: Padding { left: 1, ..Default::default() },
                ..Default::default()
            },
            ..Default::default()
        }, |ctx| {
            let line = format!("  {} │ {}", i + 1, entry);
            ctx.text(&line, &TextElementConfig {
                text_color: theme.text,
                font_size: 1,
                ..Default::default()
            });
        });
    }
}

fn build_settings_tab(ctx: &mut Context, app: &App, theme: &Theme) {
    ctx.text("Settings", &TextElementConfig {
        text_color: theme.accent, font_size: 1, ..Default::default()
    });

    let settings = [
        ("Theme", if app.theme_dark { "Dark" } else { "Light" }),
        ("Sidebar", if app.sidebar_visible { "Visible" } else { "Hidden" }),
        ("Active Tab", match app.active_tab {
            0 => "Dashboard", 1 => "Settings", _ => "About",
        }),
    ];

    for (i, (key, val)) in settings.iter().enumerate() {
        let row_id = ctx.idi("SettingRow", i as u32);
        ctx.with_element(row_id, ElementDeclaration {
            layout: LayoutConfig {
                sizing: Sizing {
                    width: SizingAxis::grow(0.0, f32::MAX),
                    height: SizingAxis::fixed(1.0),
                },
                layout_direction: LayoutDirection::LeftToRight,
                padding: Padding { left: 1, right: 1, ..Default::default() },
                child_gap: 2,
                ..Default::default()
            },
            background_color: if i % 2 == 0 { theme.sidebar_bg } else { Color::default() },
            ..Default::default()
        }, |ctx| {
            ctx.text(key, &TextElementConfig {
                text_color: theme.text_dim, font_size: 1, ..Default::default()
            });
            ctx.text(val, &TextElementConfig {
                text_color: theme.text, font_size: 1, ..Default::default()
            });
        });
    }
}

fn build_about_tab(ctx: &mut Context, theme: &Theme) {
    ctx.text("About Clay TUI", &TextElementConfig {
        text_color: theme.accent, font_size: 1, ..Default::default()
    });

    let lines = [
        "",
        "Clay is a high-performance UI layout library.",
        "This is a complete port from C to idiomatic Rust.",
        "",
        "Features:",
        "  - Flexbox-like layout system",
        "  - Floating elements",
        "  - Text wrapping",
        "  - Scroll containers",
        "  - Border rendering",
        "",
        "Renderer: ratatui (terminal)",
    ];

    for (i, line) in lines.iter().enumerate() {
        let line_id = ctx.idi("AboutLine", i as u32);
        ctx.with_element(line_id, ElementDeclaration {
            layout: LayoutConfig {
                sizing: Sizing {
                    width: SizingAxis::grow(0.0, f32::MAX),
                    height: SizingAxis::fixed(1.0),
                },
                padding: Padding { left: 1, ..Default::default() },
                ..Default::default()
            },
            ..Default::default()
        }, |ctx| {
            ctx.text(line, &TextElementConfig {
                text_color: theme.text, font_size: 1, ..Default::default()
            });
        });
    }
}

fn build_footer(ctx: &mut Context, app: &App, theme: &Theme) {
    let id = ctx.id("Footer");
    ctx.with_element(id, ElementDeclaration {
        layout: LayoutConfig {
            sizing: Sizing {
                width: SizingAxis::grow(0.0, f32::MAX),
                height: SizingAxis::fixed(1.0),
            },
            padding: Padding { left: 2, right: 2, top: 0, bottom: 0 },
            layout_direction: LayoutDirection::LeftToRight,
            child_alignment: ChildAlignment { y: AlignmentY::Center, ..Default::default() },
            child_gap: 2,
            ..Default::default()
        },
        background_color: theme.footer_bg,
        ..Default::default()
    }, |ctx| {
        ctx.text(&app.status_message, &TextElementConfig {
            text_color: theme.text,
            font_size: 1,
            ..Default::default()
        });

        let spacer_id = ctx.id("FooterSpacer");
        ctx.with_element(spacer_id, ElementDeclaration {
            layout: LayoutConfig {
                sizing: Sizing { width: SizingAxis::grow(0.0, f32::MAX), ..Default::default() },
                ..Default::default()
            },
            ..Default::default()
        }, |_| {});

        let mode = if app.theme_dark { "DARK" } else { "LIGHT" };
        ctx.text(mode, &TextElementConfig {
            text_color: theme.text_dim,
            font_size: 1,
            ..Default::default()
        });
    });
}
