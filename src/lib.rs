use bevy::prelude::*;
use bevy::asset::{embedded_asset, io::AssetSourceId, AssetPath};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};

use scraper::{Html, Selector};
use scraper::Node::{Document, Element, Doctype, Text};
use ego_tree::{NodeRef, NodeId};
use scraper::Node;
use scraper::StrTendril;
use std::collections::HashMap;
use markup5ever::interface::QualName;
use markup5ever::interface::QuirksMode;
use markup5ever::interface::TreeSink;

use std::fs;
use std::path::PathBuf;

use lightningcss::stylesheet::{StyleSheet, ParserOptions};
use lightningcss::rules::CssRuleList;
use lightningcss::rules::CssRule;
use lightningcss::rules::style::StyleRule;
use lightningcss::traits::ToCss;
use lightningcss::properties::Property;
use lightningcss::values::color::CssColor;
use lightningcss::properties::font::{FontWeight, AbsoluteFontWeight};
use lightningcss::values::length::LengthPercentage;
use lightningcss::values::length::LengthValue;
use lightningcss::properties::size::Size;
use lightningcss::properties::font::FontSize;
use lightningcss::properties::Property::Margin;
use lightningcss::properties::Property::Padding;
use lightningcss::values::length::LengthPercentageOrAuto;

use lightningcss::properties::display::{Display, DisplayPair, DisplayOutside, DisplayInside};

#[derive(Component, Default)]
pub struct HydaScrolling {
    position: f32,
}

#[derive(Debug, Clone)]
pub enum HydaAST {
    HElement {
        tag_name: String,
        attributes: HashMap<String, String>,
        content: Vec<HydaAST>,
        style: BevyHydaStyle,
    },
    HMetaElement {
        tag_name: String,
        attributes: HashMap<String, String>,
        content: Vec<HydaAST>,
        style: BevyHydaStyle,
    },
    HDoctype {
        info: String,
    },
    HText {
        text: String,
    },
    HEmpty,
}

#[derive(Debug, Clone)]
pub struct BevyHydaStyle {
    color: Option<bevy::color::Color>,
    background_color: Option<bevy::color::Color>,
    font_weight: Option<f32>,
    font_size: Option<f32>,
    width: Option<Val>,
    height: Option<Val>,
    flex_direction: Option<bevy::ui::FlexDirection>,
    flex_wrap: Option<bevy::ui::FlexWrap>,
    margin: Option<UiRect>,
    padding: Option<UiRect>,
    display: Option<lightningcss::properties::display::Display>,
    justify_content: Option<bevy::ui::JustifyContent>,
    align_content: Option<bevy::ui::AlignContent>,
}

impl Default for BevyHydaStyle {
    fn default() -> Self {
        Self {
            color: None,
            background_color: Some(bevy::prelude::Color::Srgba(Srgba::rgba_u8(0, 0, 0, 0))),
            font_weight: None,
            font_size: None,
            width: Some(Val::Auto),
            height: Some(Val::Auto),
            flex_direction: Some(bevy::ui::FlexDirection::Row),
            flex_wrap: Some(bevy::ui::FlexWrap::NoWrap),
            margin: Some(UiRect::all(Val::Px(0.0))),
            padding: Some(UiRect::all(Val::Px(0.0))),
            display: Some(lightningcss::properties::display::Display::Pair(
                DisplayPair
                {
                    outside: DisplayOutside::Block,
                    inside: DisplayInside::Flow,
                    is_list_item: false,
                })
            ),
            justify_content: Some(bevy::ui::JustifyContent::Default),
            align_content: Some(bevy::ui::AlignContent::Default),
        }
    }
}

fn css_color_to_bevy_color(col: CssColor) -> bevy::color::Color {
    match col {
        CssColor::RGBA(c) => {
            bevy::prelude::Color::Srgba(Srgba::rgba_u8(c.red, c.green, c.blue, c.alpha))
        },
        _ => todo!()
    }
}

fn css_font_weight_to_f32(fw: FontWeight) -> f32 {
    match fw {
        FontWeight::Absolute(afw) => {
            match afw {
                AbsoluteFontWeight::Weight(cssn) => return cssn,
                AbsoluteFontWeight::Normal => return 400.0,
                AbsoluteFontWeight::Bold => return 700.0,
            }
        },
        FontWeight::Lighter => return 300.0,
        FontWeight::Bolder => return 800.0,
    }
}

fn css_length_value_to_bevy_val(lv: LengthValue) -> Val {
    match lv {
        LengthValue::Px(p) => Val::Px(p),
        // This is a val type that could've be good to propose to bevy (Val::Em)
        LengthValue::Em(em) => Val::Px(em * 16.0),
        LengthValue::Vw(vw) => Val::Vw(vw),
        LengthValue::Vh(vh) => Val::Vh(vh),
        _ => todo!(),
    }
}

fn css_length_percentage_to_bevy_val(lp: LengthPercentage) -> Val {
    match lp {
        LengthPercentage::Dimension(lv) => return css_length_value_to_bevy_val(lv),
        LengthPercentage::Percentage(pc) => return Val::Percent(pc.0 * 100.0),
        _ => todo!(),
    }
}

fn css_length_percentage_or_auto_to_bevy_val(lpoa: LengthPercentageOrAuto) -> Val {
    match lpoa {
        LengthPercentageOrAuto::Auto => Val::Auto,
        LengthPercentageOrAuto::LengthPercentage(lp) => css_length_percentage_to_bevy_val(lp),
    }
}

fn css_size_to_bevy_val(s: Size) -> Val {
    match s {
        Size::Auto => return Val::Auto,
        Size::LengthPercentage(lp) => return css_length_percentage_to_bevy_val(lp),
        _ => todo!(),
    }
}

fn css_font_size_to_f32(fos: FontSize) -> f32 {
    match fos {
        FontSize::Length(lp) => {
            let v = css_length_percentage_to_bevy_val(lp);

            if let Val::Px(p) = v {
                return p;
            }
        },
        _ => todo!(),
    }

    return 1.17 * 16.0;
}

fn css_flex_direction_to_bevy_flex_direction(fd: lightningcss::properties::flex::FlexDirection) -> bevy::ui::FlexDirection {
    match fd {
        lightningcss::properties::flex::FlexDirection::Row => bevy::ui::FlexDirection::Row,
        lightningcss::properties::flex::FlexDirection::Column => bevy::ui::FlexDirection::Column,
        lightningcss::properties::flex::FlexDirection::RowReverse => bevy::ui::FlexDirection::RowReverse,
        lightningcss::properties::flex::FlexDirection::ColumnReverse => bevy::ui::FlexDirection::ColumnReverse,
    }
}

fn css_flex_wrap_to_bevy_flex_wrap(fw: lightningcss::properties::flex::FlexWrap) -> bevy::ui::FlexWrap {
    match fw {
        lightningcss::properties::flex::FlexWrap::NoWrap => bevy::ui::FlexWrap::NoWrap,
        lightningcss::properties::flex::FlexWrap::Wrap => bevy::ui::FlexWrap::Wrap,
        lightningcss::properties::flex::FlexWrap::WrapReverse => bevy::ui::FlexWrap::WrapReverse,
    }
}

fn css_justify_content_to_bevy_justify_content(jc: lightningcss::properties::align::JustifyContent) -> bevy::ui::JustifyContent {
    match jc {
        lightningcss::properties::align::JustifyContent::Normal => { return bevy::ui::JustifyContent::Default; },
        lightningcss::properties::align::JustifyContent::ContentDistribution(cond) => {
            match cond {
                lightningcss::properties::align::ContentDistribution::SpaceBetween => { return bevy::ui::JustifyContent::SpaceBetween; },
                lightningcss::properties::align::ContentDistribution::SpaceAround => { return bevy::ui::JustifyContent::SpaceAround; },
                lightningcss::properties::align::ContentDistribution::SpaceEvenly => { return bevy::ui::JustifyContent::SpaceEvenly; },
                lightningcss::properties::align::ContentDistribution::Stretch => { return bevy::ui::JustifyContent::Stretch; },
            }
        },
        lightningcss::properties::align::JustifyContent::ContentPosition { value, .. } => {
            match value {
                lightningcss::properties::align::ContentPosition::Center => { return bevy::ui::JustifyContent::Center; },
                lightningcss::properties::align::ContentPosition::Start => { return bevy::ui::JustifyContent::Start; },
                lightningcss::properties::align::ContentPosition::End => { return bevy::ui::JustifyContent::End; },
                lightningcss::properties::align::ContentPosition::FlexStart => { return bevy::ui::JustifyContent::FlexStart; },
                lightningcss::properties::align::ContentPosition::FlexEnd => { return bevy::ui::JustifyContent::FlexEnd; },
            }
        },
        _ => todo!()
    }
}

fn css_align_content_to_bevy_align_content(ac: lightningcss::properties::align::AlignContent) -> bevy::ui::AlignContent {
    match ac {
        lightningcss::properties::align::AlignContent::Normal => { return bevy::ui::AlignContent::Default; },
        lightningcss::properties::align::AlignContent::ContentDistribution(cond) => {
            match cond {
                lightningcss::properties::align::ContentDistribution::SpaceBetween => { return bevy::ui::AlignContent::SpaceBetween; },
                lightningcss::properties::align::ContentDistribution::SpaceAround => { return bevy::ui::AlignContent::SpaceAround; },
                lightningcss::properties::align::ContentDistribution::SpaceEvenly => { return bevy::ui::AlignContent::SpaceEvenly; },
                lightningcss::properties::align::ContentDistribution::Stretch => { return bevy::ui::AlignContent::Stretch; },
            }
        },
        lightningcss::properties::align::AlignContent::ContentPosition { value, .. } => {
            match value {
                lightningcss::properties::align::ContentPosition::Center => { return bevy::ui::AlignContent::Center; },
                lightningcss::properties::align::ContentPosition::Start => { return bevy::ui::AlignContent::Start; },
                lightningcss::properties::align::ContentPosition::End => { return bevy::ui::AlignContent::End; },
                lightningcss::properties::align::ContentPosition::FlexStart => { return bevy::ui::AlignContent::FlexStart; },
                lightningcss::properties::align::ContentPosition::FlexEnd => { return bevy::ui::AlignContent::FlexEnd; },
            }
        },
        _ => todo!()
    }
}

impl BevyHydaStyle {
    pub fn from_lcss(declarations: &Vec<Property>, important_declarations: &Vec<Property>) -> Self {

        let mut final_color: Option<bevy::color::Color> = None;
        let mut final_background_color: Option<bevy::color::Color> = None;
        let mut final_font_weight: Option<f32> = None;
        let mut final_font_size: Option<f32> = None;
        let mut final_width: Option<Val> = None;
        let mut final_height: Option<Val> = None;
        let mut final_flex_direction: Option<bevy::ui::FlexDirection> = None;
        let mut final_flex_wrap: Option<bevy::ui::FlexWrap> = None;
        let mut final_margin: Option<UiRect> = Some(UiRect::all(Val::Px(0.0)));
        let mut final_padding: Option<UiRect> = Some(UiRect::all(Val::Px(0.0)));
        let mut final_display: Option<lightningcss::properties::display::Display> = None;
        let mut final_justify_content: Option<bevy::ui::JustifyContent> = None;
        let mut final_align_content: Option<bevy::ui::AlignContent> = None;

        for i in declarations {
            match i {
                Property::Color(col) => final_color = Some(css_color_to_bevy_color(col.clone())),
                Property::BackgroundColor(col) => final_background_color = Some(css_color_to_bevy_color(col.clone())),
                Property::FontWeight(fw) => final_font_weight = Some(css_font_weight_to_f32(fw.clone())),
                Property::FontSize(fos) => final_font_size = Some(css_font_size_to_f32(fos.clone())),
                Property::Width(w) => final_width = Some(css_size_to_bevy_val(w.clone())),
                Property::Height(h) => final_height = Some(css_size_to_bevy_val(h.clone())),
                Property::FlexDirection(fd, _) => final_flex_direction = Some(css_flex_direction_to_bevy_flex_direction(fd.clone())),
                Property::FlexWrap(fw, _) => final_flex_wrap = Some(css_flex_wrap_to_bevy_flex_wrap(fw.clone())),
                Property::Margin(m) => { 
                    final_margin = Some(UiRect {
                        left: css_length_percentage_or_auto_to_bevy_val(m.left.clone()),
                        right: css_length_percentage_or_auto_to_bevy_val(m.right.clone()),
                        top: css_length_percentage_or_auto_to_bevy_val(m.top.clone()),
                        bottom: css_length_percentage_or_auto_to_bevy_val(m.bottom.clone()),
                    });
                },
                Property::Padding(p) => { 
                    final_padding = Some(UiRect {
                        left: css_length_percentage_or_auto_to_bevy_val(p.left.clone()),
                        right: css_length_percentage_or_auto_to_bevy_val(p.right.clone()),
                        top: css_length_percentage_or_auto_to_bevy_val(p.top.clone()),
                        bottom: css_length_percentage_or_auto_to_bevy_val(p.bottom.clone()),
                    });
                },

                // For some reason, this doesn't change the Some(UiRect) values individually.
                // So we have to do the Some(UiRect) all over again.
                Property::MarginTop(v) => {
                    final_margin = Some(UiRect { 
                        top: css_length_percentage_or_auto_to_bevy_val(v.clone()),
                        bottom: final_margin.unwrap().bottom,
                        left: final_margin.unwrap().left,
                        right: final_margin.unwrap().right
                    });
                },
                Property::MarginBottom(v) => {
                    final_margin = Some(UiRect { 
                        top: final_margin.unwrap().top,
                        bottom: css_length_percentage_or_auto_to_bevy_val(v.clone()),
                        left: final_margin.unwrap().left,
                        right: final_margin.unwrap().right
                    });
                },
                Property::MarginLeft(v) => {
                    final_margin = Some(UiRect { 
                        top: final_margin.unwrap().top,
                        bottom: final_margin.unwrap().bottom,
                        left: css_length_percentage_or_auto_to_bevy_val(v.clone()),
                        right: final_margin.unwrap().right
                    });
                },
                Property::MarginRight(v) => {
                    final_margin = Some(UiRect { 
                        top: final_margin.unwrap().top,
                        bottom: final_margin.unwrap().bottom,
                        left: final_margin.unwrap().left,
                        right: css_length_percentage_or_auto_to_bevy_val(v.clone())
                    });
                },

                // For some reason, this doesn't change the Some(UiRect) values individually.
                // So we have to do the Some(UiRect) all over again.
                Property::PaddingTop(v) => {
                    final_padding = Some(UiRect { 
                        top: css_length_percentage_or_auto_to_bevy_val(v.clone()),
                        bottom: final_padding.unwrap().bottom,
                        left: final_padding.unwrap().left,
                        right: final_padding.unwrap().right
                    });
                },
                Property::PaddingBottom(v) => {
                    final_padding = Some(UiRect { 
                        top: final_padding.unwrap().top,
                        bottom: css_length_percentage_or_auto_to_bevy_val(v.clone()),
                        left: final_padding.unwrap().left,
                        right: final_padding.unwrap().right
                    });
                },
                Property::PaddingLeft(v) => {
                    final_padding = Some(UiRect { 
                        top: final_padding.unwrap().top,
                        bottom: final_padding.unwrap().bottom,
                        left: css_length_percentage_or_auto_to_bevy_val(v.clone()),
                        right: final_padding.unwrap().right
                    });
                },
                Property::PaddingRight(v) => {
                    final_padding = Some(UiRect { 
                        top: final_padding.unwrap().top,
                        bottom: final_padding.unwrap().bottom,
                        left: final_padding.unwrap().left,
                        right: css_length_percentage_or_auto_to_bevy_val(v.clone())
                    });
                },

                Property::Display(d) => final_display = Some(d.clone()),
                Property::JustifyContent(jc, _) => final_justify_content = Some(css_justify_content_to_bevy_justify_content(jc.clone())),
                Property::AlignContent(ac, _) => final_align_content = Some(css_align_content_to_bevy_align_content(ac.clone())),
                _ => todo!()
            }
        }

        Self {
            color: final_color,
            background_color: final_background_color,
            font_weight: final_font_weight,
            font_size: final_font_size,
            width: final_width,
            height: final_height,
            flex_direction: final_flex_direction,
            flex_wrap: final_flex_wrap,
            margin: final_margin,
            padding: final_padding,
            display: final_display,
            justify_content: final_justify_content,
            align_content: final_align_content,
        }
    }
}

#[derive(Default, Debug)]
pub struct HydaStyleSheet {
    selector: String,
    node_ids: Vec<NodeId>,
    bevy_style: BevyHydaStyle,
}

macro_rules! add_if_not_none {
    ($a:ident, $b:expr, $value:ident) => {
        if $b.$value != None {
            $a.$value = $b.$value;
        }
    }
}

macro_rules! add_clone_if_not_none {
    ($a:ident, $b:expr, $value:ident) => {
        if $b.$value != None {
            $a.$value = $b.$value.clone();
        }
    }
}

fn compose_final_style(styles: &Vec<HydaStyleSheet>, parent_style: &BevyHydaStyle, id: NodeId) -> BevyHydaStyle {

    let mut get_style: BevyHydaStyle = BevyHydaStyle::default();

    add_if_not_none!(get_style, parent_style, color);
    add_if_not_none!(get_style, parent_style, font_weight);
    add_if_not_none!(get_style, parent_style, font_size);

    for s in styles {
        for nid in &s.node_ids {
            if *nid == id {
                //dbg!(&s.bevy_style, &id);

                add_if_not_none!(get_style, s.bevy_style, color);
                add_if_not_none!(get_style, s.bevy_style, background_color);
                add_if_not_none!(get_style, s.bevy_style, font_weight);
                add_if_not_none!(get_style, s.bevy_style, font_size);
                add_if_not_none!(get_style, s.bevy_style, width);
                add_if_not_none!(get_style, s.bevy_style, height);
                add_if_not_none!(get_style, s.bevy_style, flex_direction);
                add_if_not_none!(get_style, s.bevy_style, flex_wrap);
                add_if_not_none!(get_style, s.bevy_style, margin);
                add_if_not_none!(get_style, s.bevy_style, padding);

                add_clone_if_not_none!(get_style, s.bevy_style, display);

                add_if_not_none!(get_style, s.bevy_style, justify_content);
                add_if_not_none!(get_style, s.bevy_style, align_content);
            }
        }
    }

    return get_style;
}

fn parse_scraper_node(node: NodeRef<'_, Node>, styles: &Vec<HydaStyleSheet>, parent_style: &BevyHydaStyle) -> HydaAST {

    match node.value() {
        Document => {
            let mut child_vec: Vec<HydaAST> = Vec::new();

            let get_style = compose_final_style(styles, parent_style, node.id());

            for c in node.children() {
                child_vec.push(parse_scraper_node(c, styles, &get_style));
            }

            return HydaAST::HElement { tag_name: "html".to_string(), attributes: HashMap::new(), content: child_vec, style: get_style };
        },
        Element(e) => {

            let tag_name = e.name.local.to_string();

            let mut child_vec: Vec<HydaAST> = Vec::new();

            let mut text_section_content: Vec<HydaAST> = Vec::new();

            let get_style = compose_final_style(styles, parent_style, node.id());

            for c in node.children() {
                child_vec.push(parse_scraper_node(c, styles, &get_style));
            }

            let mut attrs_hashmap: HashMap<String, String> = HashMap::new();

            for a in &e.attrs {
                attrs_hashmap.insert(a.0.local.to_string(), a.1.to_string());
            }

            let is_meta_element = match tag_name.as_str() {
                "meta" | "link" | "title" | "head"  => true,
                _ => false
            };

            if is_meta_element {
                return HydaAST::HMetaElement { tag_name: tag_name.clone(), attributes: attrs_hashmap.clone(), content: child_vec, style: get_style };
            }

            return HydaAST::HElement { tag_name: tag_name.clone(), attributes: attrs_hashmap.clone(), content: child_vec, style: get_style };
        },
        Doctype(d) => {
            return HydaAST::HDoctype { info: "html".to_string() };
        },
        Text(t) => {

            let get_text = t.to_string();

            if get_text.trim().is_empty() {
                return HydaAST::HEmpty;
            }

            return HydaAST::HText { text: get_text.to_string() };
        },
        _ => { return HydaAST::HEmpty; }
    }
}

fn is_tag_inlined_text(tag: &str) -> bool {
    return tag == "a" || tag == "b" || tag == "i" || tag == "strong" || tag == "abbr" || tag == "span";
}

fn get_default_firasans(weight: f32) -> String {

    let mut final_path: String = "embedded://bevy_hyda/fonts/FiraSans-Regular.ttf".to_string();

    if weight >= 100.0 || weight < 100.0 { final_path = "embedded://bevy_hyda/fonts/FiraSans-Thin.ttf".to_string(); }

    if weight >= 200.0 { final_path = "embedded://bevy_hyda/fonts/FiraSans-ExtraLight.ttf".to_string(); }
    if weight >= 300.0 { final_path = "embedded://bevy_hyda/fonts/FiraSans-Light.ttf".to_string(); }
    if weight >= 400.0 { final_path = "embedded://bevy_hyda/fonts/FiraSans-Regular.ttf".to_string(); }
    if weight >= 500.0 { final_path = "embedded://bevy_hyda/fonts/FiraSans-Medium.ttf".to_string(); }
    if weight >= 600.0 { final_path = "embedded://bevy_hyda/fonts/FiraSans-SemiBold.ttf".to_string(); }
    if weight >= 700.0 { final_path = "embedded://bevy_hyda/fonts/FiraSans-Bold.ttf".to_string(); }
    if weight >= 800.0 { final_path = "embedded://bevy_hyda/fonts/FiraSans-ExtraBold.ttf".to_string(); }
    if weight >= 900.0 { final_path = "embedded://bevy_hyda/fonts/FiraSans-Black.ttf".to_string(); }

    //if *fs == HydaFontStyle::Italic {
    //    if weight >= 400 && weight < 500 {
    //        return "embedded://bevy_hyda/fonts/FiraSans-Italic.ttf".to_string();
    //    }

    //    return final_path.replace(".ttf", "Italic.ttf");
    //}

    return final_path;
}

impl HydaAST {
    pub fn spawn_ui(&self, commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
        return self.spawn_ui_impl(commands, asset_server, &BevyHydaStyle::default(), &mut Vec::new()).0;
    }

    pub fn spawn_ui_impl(&self, commands: &mut Commands, asset_server: &Res<AssetServer>, parent_style: &BevyHydaStyle, text_section_vector: &mut Vec<TextSection>) -> (Entity, bool) {
        match self {
            HydaAST::HElement { tag_name, content, style, .. } => {

                let mut child_vec: Vec<Entity> = Vec::new();

                for c in content {
                    let final_c = c.spawn_ui_impl(commands, asset_server, &style, text_section_vector);

                    if !final_c.1 {
                        child_vec.push(final_c.0);
                    }
                }

                if !is_tag_inlined_text(&tag_name) {
                    if text_section_vector.len() != 0 {
                        child_vec.push(commands.spawn(TextBundle::from_sections(text_section_vector.clone())).id());
                        text_section_vector.clear();
                    }
                }

                let mut final_display = bevy::ui::Display::Block;

                match style.display.clone().unwrap() {
                    lightningcss::properties::display::Display::Pair(p) => {
                        match p.outside {
                            DisplayOutside::Block => final_display = bevy::ui::Display::Block,
                            _ => {},
                        }

                        match p.inside {
                            DisplayInside::Flex { .. } => final_display = bevy::ui::Display::Flex,
                            _ => {},
                        }
                    },
                    lightningcss::properties::display::Display::Keyword(_) => todo!()
                }

                let mut is_empty: bool = false;
                let mut result = if !is_tag_inlined_text(&tag_name) {
                    commands.spawn(
                        NodeBundle {
                            style: Style {
                                display: final_display,
                                width: style.width.unwrap(),
                                height: style.height.unwrap(),
                                flex_direction: style.flex_direction.unwrap(),
                                flex_wrap: style.flex_wrap.unwrap(),
                                margin: style.margin.unwrap(),
                                padding: style.padding.unwrap(),
                                justify_content: style.justify_content.unwrap(),
                                align_content: style.align_content.unwrap(),
                                ..default()
                            },
                            background_color: bevy::prelude::BackgroundColor(style.background_color.unwrap()),
                            ..default()
                    })
                }
                else {
                    is_empty = true;
                    commands.spawn_empty()
                };

                if tag_name == "body" {
                    result.insert(HydaScrolling::default());
                }

                for c in child_vec {
                    result.add_child(c);
                }

                return (result.id(), is_empty);
            },
            HydaAST::HText { text } => {
                let new_text = TextSection::new(
                    text.to_string().replace("\n", " "),
                    TextStyle {
                        font: asset_server.load(
                            get_default_firasans(parent_style.font_weight.unwrap())
                        ),
                        font_size: parent_style.font_size.unwrap(),
                        color: parent_style.color.unwrap(),
                    }
                );

                text_section_vector.push(new_text);

                return (commands.spawn_empty().id(), true);
            },
            _ => { return (commands.spawn_empty().id(), true); }
        }
    }
}

pub fn html_string(get_str: String) -> HydaAST {
    return html_ast_impl(get_str, "".to_string());
}

pub fn html_file(get_url: String) -> HydaAST {
    return html_ast_impl(fs::read_to_string(&get_url).unwrap(), get_url);
}

fn add_stylesheet(stylesheet_cont: String, styles: &mut Vec<HydaStyleSheet>, document: &Html) {

    let mut stylesheet = StyleSheet::parse(
        &stylesheet_cont, 
        ParserOptions::default()).unwrap();

    //dbg!(&stylesheet);

    let mut final_name: String = String::new();
    let mut final_nodeid_vec: Vec<NodeId> = Vec::new();

    let mut bhs = BevyHydaStyle::default();

    if let CssRuleList(ref c) = stylesheet.rules {
        for style in c {

            final_name = "".to_string();
            final_nodeid_vec.clear();
            bhs = BevyHydaStyle::default();

            if let CssRule::Style(StyleRule { selectors, declarations, .. }) = style {

                let mut selector_count: i32 = 0;
                for css_selector in &selectors.0 {
                    final_name += &format!("{:?}", css_selector.iter());

                    if selector_count < (selectors.0.len() - 1).try_into().unwrap() {
                        final_name += ", ";
                        selector_count += 1;
                    }
                }

                bhs = BevyHydaStyle::from_lcss(&declarations.declarations, &declarations.important_declarations);
            }

            let selector_test = Selector::parse(&final_name).unwrap();
            for e2 in document.select(&selector_test) {
                final_nodeid_vec.push(e2.id());
            }

            styles.push(HydaStyleSheet {
                selector: final_name.clone(),
                node_ids: final_nodeid_vec.clone(),
                bevy_style: bhs.clone(),
            });
        }
    }
}

pub fn html_ast_impl(html: String, url: String) -> HydaAST {
    
    let mut document = Html::parse_document(&html);
    document.set_quirks_mode(QuirksMode::NoQuirks);

    let selector = Selector::parse("link").unwrap();

    let mut final_dir = PathBuf::from(&url);
    final_dir.pop();
    final_dir.push(".");

    let mut styles: Vec<HydaStyleSheet> = Vec::new();

    add_stylesheet(include_str!("styles/default.css").to_string(), &mut styles, &document);

    for element in document.select(&selector) {

        let mut attrs_hashmap: HashMap<String, String> = HashMap::new();
        
        for a in &element.value().attrs {
            attrs_hashmap.insert(a.0.local.to_string(), a.1.to_string());
        }

        let final_href = if let Some(st) = attrs_hashmap.get("href") { st.to_string() } else { "".to_string() };
        
        if let Some(st) = attrs_hashmap.get("rel") {
            if st == "stylesheet" {

                final_dir.push(final_href);

                //dbg!(final_dir.clone().into_os_string().into_string().unwrap());
                let stylesheet_cont = fs::read_to_string(final_dir.clone()).unwrap();

                final_dir.pop();

                add_stylesheet(stylesheet_cont, &mut styles, &document);
            }
        }
    }

    let root = document.tree.root();
    return parse_scraper_node(root, &styles, &BevyHydaStyle::default());
}

pub struct BevyHydaPlugin;

impl Plugin for BevyHydaPlugin {
    fn build(&self, app: &mut App) {

        embedded_asset!(app, "fonts/FiraSans-ThinItalic.ttf");
        embedded_asset!(app, "fonts/FiraSans-Thin.ttf");
        embedded_asset!(app, "fonts/FiraSans-SemiBoldItalic.ttf");
        embedded_asset!(app, "fonts/FiraSans-SemiBold.ttf");
        embedded_asset!(app, "fonts/FiraSans-Regular.ttf");
        embedded_asset!(app, "fonts/FiraSans-MediumItalic.ttf");
        embedded_asset!(app, "fonts/FiraSans-Medium.ttf");
        embedded_asset!(app, "fonts/FiraSans-LightItalic.ttf");
        embedded_asset!(app, "fonts/FiraSans-Light.ttf");
        embedded_asset!(app, "fonts/FiraSans-Italic.ttf");
        embedded_asset!(app, "fonts/FiraSans-ExtraLightItalic.ttf");
        embedded_asset!(app, "fonts/FiraSans-ExtraLight.ttf");
        embedded_asset!(app, "fonts/FiraSans-ExtraBoldItalic.ttf");
        embedded_asset!(app, "fonts/FiraSans-ExtraBold.ttf");
        embedded_asset!(app, "fonts/FiraSans-BoldItalic.ttf");
        embedded_asset!(app, "fonts/FiraSans-Bold.ttf");
        embedded_asset!(app, "fonts/FiraSans-BlackItalic.ttf");
        embedded_asset!(app, "fonts/FiraSans-Black.ttf");

        app.add_systems(Update, mouse_scroll);
    }
}

fn mouse_scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query_list: Query<(&mut HydaScrolling, &mut Style, &Parent, &bevy::prelude::Node)>,
    query_node: Query<&bevy::prelude::Node>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        for (mut scrolling_list, mut style, parent, list_node) in &mut query_list {
            let items_height = list_node.size().y;
            let container_height = query_node.get(parent.get()).unwrap().size().y;

            let max_scroll = (items_height - container_height).max(0.);

            let dy = match mouse_wheel_event.unit {
                MouseScrollUnit::Line => mouse_wheel_event.y * 20.,
                MouseScrollUnit::Pixel => mouse_wheel_event.y,
            };

            scrolling_list.position += dy;
            scrolling_list.position = scrolling_list.position.clamp(-max_scroll, 0.);
            style.top = Val::Px(scrolling_list.position);
        }
    }
}