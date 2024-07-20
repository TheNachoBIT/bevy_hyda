use bevy::asset::{embedded_asset, io::AssetSourceId, AssetPath};
use bevy::prelude::*;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use quick_xml::de::from_str;
use serde::Deserialize;

use std::fs;
use std::collections::HashMap;

use bevy::ecs::system::EntityCommands;

use lightningcss::stylesheet::{
  StyleSheet, ParserOptions, MinifyOptions, PrinterOptions
};

use bevy::color::palettes::css::BLUE;
use bevy::color::palettes::css::BLACK;

#[derive(Clone, PartialEq)]
pub enum HydaFontStyle {
    Normal,
    Italic,
}
    
#[derive(Clone, PartialEq)]
pub enum HydaDisplay {
    None,
    Block,
    InlineBlock,
    ListItem,
}

#[derive(Default, Clone)]
pub struct HydaStyle {
    font_size: Option<f32>,
    font_weight: Option<i32>,
    font_style: Option<HydaFontStyle>,

    margin: Option<UiRect>,
    padding: Option<UiRect>,
    flex_direction: Option<FlexDirection>,
    display: Option<HydaDisplay>,

    width: Option<Val>,
    height: Option<Val>,

    color: Option<Color>,
}

fn get_style_recursive(styles: &HashMap<String, HydaStyle>, st: String) -> Option<&HydaStyle> {

    let mut st_vec: Vec<&str> = st.split(" ").collect();

    let get_style = styles.get(&st);

    if let Some(style) = get_style {
        return get_style;
    }
    else {
        st_vec.remove(0);

        if st_vec.len() == 0 {
            return None;
        }

        let new_st: String = st_vec.join(" ");

        return get_style_recursive(styles, new_st);
    }
}

fn get_f32_from_style(styles: &HashMap<String, HydaStyle>, st: String, item: &str) -> f32 {

    let get_style = get_style_recursive(styles, st.clone()).unwrap();

    let final_var: Option<f32> = match item {
        "font-size" => get_style.font_size,
        _ => panic!("Unknown property {:?}", item)
    };

    if let Some(v) = final_var {
        return v;
    }
    else {
        let mut st_vec: Vec<&str> = st.split(" ").collect();

        st_vec.pop();

        if st_vec.len() == 0 {
            panic!("Stop!");
        }

        return get_f32_from_style(styles, st_vec.join(" "), item);
    }
}

fn get_i32_from_style(styles: &HashMap<String, HydaStyle>, st: String, item: &str) -> i32 {

    let get_style = get_style_recursive(styles, st.clone()).unwrap();

    let final_var: Option<i32> = match item {
        "font-weight" => get_style.font_weight,
        _ => panic!("Unknown property {:?}", item)
    };

    if let Some(v) = final_var {
        return v;
    }
    else {
        let mut st_vec: Vec<&str> = st.split(" ").collect();

        st_vec.pop();

        if st_vec.len() == 0 {
            panic!("Stop!");
        }

        return get_i32_from_style(styles, st_vec.join(" "), item);
    }
}

fn get_uirect_from_style(
    styles: &HashMap<String, HydaStyle>, 
    st: String, 
    item: &str, 
    or_get_default: bool, 
    is_inlined: bool, 
    is_first_child: bool) -> UiRect {

    let get_style = get_style_recursive(styles, st.clone()).unwrap();

    let final_var: Option<UiRect> = match item {
        "margin" => get_style.margin,
        "padding" => get_style.padding,
        _ => panic!("Unknown property {:?}", item)
    };

    if let Some(mut v) = final_var {

        if is_inlined {
            if item == "margin" {
                // TEMPORARY SOLUTION FOR INLINED HTML COMPONENTS.
                // YES, IT SUCKS.
                // PLEASE DON'T LET THIS BE PERMANENT.
                let mut final_bottom_val: f32 = 0.0;
                let mut final_top_val: f32 = 0.0;

                if let Val::Px(p) = v.bottom {
                    final_bottom_val = p;
                }

                if let Val::Px(p) = v.top {
                    final_top_val = p;
                }

                final_bottom_val -= 16.0;

                if final_bottom_val < 0.0 {
                    final_bottom_val = 0.0;
                }

                if is_first_child {
                    final_bottom_val = 0.0;
                    final_top_val = 0.0;
                }

                v.bottom = Val::Px(final_bottom_val);
                v.top = Val::Px(final_top_val);
            }
        }

        return v;
    }
    else {

        //if or_get_default && !is_inlined {
        //  return get_uirect_from_style(styles, "html".to_string(), item, false, is_inlined);
        //}

        let mut st_vec: Vec<&str> = st.split(" ").collect();

        st_vec.pop();

        if st_vec.len() == 0 {
            panic!("Stop!");
        }

        return get_uirect_from_style(styles, st_vec.join(" "), item, or_get_default, is_inlined, is_first_child);
    }
}

fn get_flex_direction_from_style(styles: &HashMap<String, HydaStyle>, st: String, item: &str) -> FlexDirection {

    let get_style = get_style_recursive(styles, st.clone()).unwrap();

    let final_var: Option<FlexDirection> = match item {
        "flex-direction" => get_style.flex_direction,
        _ => panic!("Unknown property {:?}", item)
    };

    if let Some(v) = final_var {
        return v;
    }
    else {
        let mut st_vec: Vec<&str> = st.split(" ").collect();

        st_vec.pop();

        if st_vec.len() == 0 {
            panic!("Stop!");
        }

        return get_flex_direction_from_style(styles, st_vec.join(" "), item);
    }
}

fn get_val_from_style(styles: &HashMap<String, HydaStyle>, st: String, item: &str) -> Val {

    let get_style = get_style_recursive(styles, st.clone()).unwrap();

    let final_var: Option<Val> = match item {
        "width" => get_style.width,
        "height" => get_style.height,
        _ => panic!("Unknown property {:?}", item)
    };

    if let Some(v) = final_var {
        return v;
    }
    else {
        let mut st_vec: Vec<&str> = st.split(" ").collect();

        st_vec.pop();

        if st_vec.len() == 0 {
            panic!("Stop!");
        }

        return get_val_from_style(styles, st_vec.join(" "), item);
    }
}

fn get_color_from_style(styles: &HashMap<String, HydaStyle>, st: String, item: &str) -> Color {

    let get_style = get_style_recursive(styles, st.clone()).unwrap();

    let final_var: Option<Color> = match item {
        "color" => get_style.color,
        _ => panic!("Unknown property {:?}", item)
    };

    if let Some(v) = final_var {
        return v;
    }
    else {
        let mut st_vec: Vec<&str> = st.split(" ").collect();

        st_vec.pop();

        if st_vec.len() == 0 {
            panic!("Stop!");
        }

        return get_color_from_style(styles, st_vec.join(" "), item);
    }
}

fn get_display_from_style(styles: &HashMap<String, HydaStyle>, st: String) -> HydaDisplay {

    let get_style = get_style_recursive(styles, st.clone()).unwrap();

    if let Some(v) = &get_style.display {
        return v.clone();
    }
    else {
        let mut st_vec: Vec<&str> = st.split(" ").collect();

        st_vec.pop();

        if st_vec.len() == 0 {
            panic!("Stop!");
        }

        return get_display_from_style(styles, st_vec.join(" "));
    }
}

fn get_font_style_from_style(styles: &HashMap<String, HydaStyle>, st: String) -> HydaFontStyle {

    let get_style = get_style_recursive(styles, st.clone()).unwrap();

    if let Some(v) = &get_style.font_style {
        return v.clone();
    }
    else {
        let mut st_vec: Vec<&str> = st.split(" ").collect();

        st_vec.pop();

        if st_vec.len() == 0 {
            panic!("Stop!");
        }

        return get_font_style_from_style(styles, st_vec.join(" "));
    }
}

#[allow(dead_code)]
#[derive(Component)]
pub struct HydaClass(String);

#[allow(dead_code)]
#[derive(Component)]
pub struct HydaId(String);

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HydaComponent {

    // Order is based on Mozilla's HTML elements reference:
    // https://developer.mozilla.org/en-US/docs/Web/HTML/Element

    // Main Root
    Html {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },

    // Document Metadata
    Head {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },
    Title {
        #[serde(rename = "$text")]
        content: Option<String>
    },
    Style {
        #[serde(rename = "$text")]
        content: Option<String>
    },

    // Sectioning Root
    Body {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },

    // Content Sectioning
    // - Header
    Header {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },

    // - Heading
    #[serde(rename = "hgroup")]
    HeadingGroup {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },

    #[serde(rename = "h1")]
    Heading1 {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },
    #[serde(rename = "h2")]
    Heading2 {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },
    #[serde(rename = "h3")]
    Heading3 {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },
    #[serde(rename = "h4")]
    Heading4 {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },
    #[serde(rename = "h5")]
    Heading5 {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },
    #[serde(rename = "h6")]
    Heading6 {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },

    // - Main
    Main {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },

    // - Footer
    Footer {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },

    // - Nav
    #[serde(rename = "nav")]
    Navigator {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },

    // - Address
    Address {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },

    // Text Content
    Div {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },
    #[serde(rename = "p")]
    Paragraph {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },
    #[serde(rename = "ol")]
    OrderedList {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },
    #[serde(rename = "ul")]
    UnorderedList {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },
    #[serde(rename = "dl")]
    DescriptiveList {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },
    #[serde(rename = "dt")]
    DescriptionTerm {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },
    #[serde(rename = "dd")]
    DescriptionDetail {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },
    #[serde(rename = "li")]
    ListItem {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },
    Blockquote {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },

    // Text Handling
    #[serde(rename = "$text")]
    Text(String),

    #[serde(rename = "b")]
    BoldText {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },
    #[serde(rename = "strong")]
    StrongText {
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },

    #[serde(rename = "a")]
    Anchor {
        href: Option<String>,
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },

    // Forms
    Form {
        action: Option<String>,
        method: Option<String>,
        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    },
    Input {
        #[serde(rename = "@type")]
        input_type: Option<String>,

        #[serde(rename = "@value")]
        input_value: Option<String>,

        name: Option<String>,
        id: Option<String>,
        class: Option<String>,

        #[serde(rename = "$value")]
        content: Option<Vec<HydaComponent>>
    }
}

// DEFAULT MARGINS
const BODY_DEFAULT_MARGIN: f32 = 8.0;

const H1_DEFAULT_MARGIN: f32 = 21.440;
const H2_DEFAULT_MARGIN: f32 = 19.920;
const H3_DEFAULT_MARGIN: f32 = 18.720;
const H4_DEFAULT_MARGIN: f32 = 21.280;
const H5_DEFAULT_MARGIN: f32 = 21.178;
const H6_DEFAULT_MARGIN: f32 = 24.978;

const P_DEFAULT_MARGIN: f32 = 16.0;

const WEBKIT_LINK_COLOR: Color = bevy::prelude::Color::Srgba(BLUE);

fn initialize_default_styles() -> HashMap<String, HydaStyle> {

    let mut final_hashmap: HashMap<String, HydaStyle> = HashMap::new();

    // HTML STYLE IS NOT ALLOWED TO HAVE NONE VALUES.
    let html_style = HydaStyle {

        font_size: Some(18.400),
        font_weight: Some(400),
        font_style: Some(HydaFontStyle::Normal),

        flex_direction: Some(FlexDirection::Column),
        width: Some(Val::Percent(100.0)),
        height: Some(Val::Percent(100.0)),
        margin: Some(UiRect {
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
        }),
        padding: Some(UiRect {
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
        }),
        display: Some(HydaDisplay::Block),
        color: Some(bevy::prelude::Color::Srgba(BLACK)),
    };

    let body_style = HydaStyle {
        margin: Some(UiRect {
            top: Val::Px(BODY_DEFAULT_MARGIN),
            bottom: Val::Px(BODY_DEFAULT_MARGIN),
            left: Val::Px(BODY_DEFAULT_MARGIN),
            right: Val::Px(BODY_DEFAULT_MARGIN),
        }),
        flex_direction: Some(FlexDirection::Column),
        width: Some(Val::Auto),
        height: Some(Val::Auto),
        ..default()
    };

    let div_style = HydaStyle {
        ..default()
    };

    final_hashmap.insert("html".to_string(), html_style);
    final_hashmap.insert("body".to_string(), body_style);
    final_hashmap.insert("div".to_string(), div_style);

    let header_style = HydaStyle {
        ..default()
    };

    let main_style = HydaStyle {
        ..default()
    };

    final_hashmap.insert("header".to_string(), header_style);
    final_hashmap.insert("main".to_string(), main_style);

    let hgroup_style = HydaStyle {
        ..default()
    };

    let h1_style = HydaStyle {
        font_size: Some(37.600),
        font_weight: Some(700),

        margin: Some(UiRect {
            top: Val::Px(H1_DEFAULT_MARGIN),
            bottom: Val::Px(H1_DEFAULT_MARGIN),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
        }),

        flex_direction: Some(FlexDirection::Row),
        ..default()
    };

    let h2_style = HydaStyle {
        font_size: Some(27.200),
        font_weight: Some(700),

        margin: Some(UiRect {
            top: Val::Px(H2_DEFAULT_MARGIN),
            bottom: Val::Px(H2_DEFAULT_MARGIN),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
        }),
        flex_direction: Some(FlexDirection::Row),
        ..default()
    };

    let h3_style = HydaStyle {
        font_size: Some(21.600),
        font_weight: Some(700),

        margin: Some(UiRect {
            top: Val::Px(H3_DEFAULT_MARGIN),
            bottom: Val::Px(H3_DEFAULT_MARGIN),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
        }),
        flex_direction: Some(FlexDirection::Row),
        ..default()
    };

    let h4_style = HydaStyle {
        font_size: Some(18.400),
        font_weight: Some(700),

        margin: Some(UiRect {
            top: Val::Px(H4_DEFAULT_MARGIN),
            bottom: Val::Px(H4_DEFAULT_MARGIN),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
        }),
        flex_direction: Some(FlexDirection::Row),
        ..default()
    };

    let h5_style = HydaStyle {
        font_size: Some(16.),
        font_weight: Some(700),

        margin: Some(UiRect {
            top: Val::Px(H5_DEFAULT_MARGIN),
            bottom: Val::Px(H5_DEFAULT_MARGIN),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
        }),
        flex_direction: Some(FlexDirection::Row),
        ..default()
    };

    let h6_style = HydaStyle {
        font_size: Some(12.800),
        font_weight: Some(700),

        margin: Some(UiRect {
            top: Val::Px(H6_DEFAULT_MARGIN),
            bottom: Val::Px(H6_DEFAULT_MARGIN),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
        }),
        flex_direction: Some(FlexDirection::Row),
        ..default()
    };

    final_hashmap.insert("hgroup".to_string(), hgroup_style);
    final_hashmap.insert("h1".to_string(), h1_style);
    final_hashmap.insert("h2".to_string(), h2_style);
    final_hashmap.insert("h3".to_string(), h3_style);
    final_hashmap.insert("h4".to_string(), h4_style);
    final_hashmap.insert("h5".to_string(), h5_style);
    final_hashmap.insert("h6".to_string(), h6_style);

    let nav_style = HydaStyle {
        ..default()
    };

    final_hashmap.insert("nav".to_string(), nav_style);

    let footer_style = HydaStyle {
        ..default()
    };

    final_hashmap.insert("footer".to_string(), footer_style);

    let blockquote_style = HydaStyle {
        margin: Some(UiRect {
            top: Val::Px(16.0),
            bottom: Val::Px(16.0),
            left: Val::Px(40.0),
            right: Val::Px(40.0),
        }),
        ..default()
    };

    final_hashmap.insert("blockquote".to_string(), blockquote_style);

    let address_style = HydaStyle {
        margin: Some(UiRect {
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
        }),
        font_style: Some(HydaFontStyle::Italic),
        ..default()
    };

    final_hashmap.insert("address".to_string(), address_style);

    let p_style = HydaStyle {
        font_size: Some(18.400),
        font_weight: Some(400),

        margin: Some(UiRect {
            top: Val::Px(P_DEFAULT_MARGIN),
            bottom: Val::Px(P_DEFAULT_MARGIN),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
        }),
        ..default()
    };

    let b_style = HydaStyle {
        font_weight: Some(700),
        ..default()
    };

    let strong_style = b_style.clone();

    let a_style = HydaStyle {
        color: Some(WEBKIT_LINK_COLOR),
        ..default()
    };

    let ol_style = HydaStyle {
        margin: Some(UiRect {
            top: Val::Px(P_DEFAULT_MARGIN),
            bottom: Val::Px(P_DEFAULT_MARGIN),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
        }),
        padding: Some(UiRect {
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
            left: Val::Px(40.0),
            right: Val::Px(0.0),
        }),
        ..default()
    };

    let ul_style = ol_style.clone();

    let li_style = HydaStyle {
        display: Some(HydaDisplay::ListItem),
        padding: Some(UiRect {
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
        }),
        margin: Some(UiRect {
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
        }),
        ..default()
    };

    let dl_style = p_style.clone();

    let dt_style = HydaStyle {
        padding: Some(UiRect {
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
        }),
        margin: Some(UiRect {
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
        }),
        ..default()
    };

    let dd_style = HydaStyle {
        margin: Some(UiRect {
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
            left: Val::Px(40.0),
            right: Val::Px(0.0),
        }),
        ..default()
    };

    final_hashmap.insert("p".to_string(), p_style);
    final_hashmap.insert("b".to_string(), b_style);
    final_hashmap.insert("strong".to_string(), strong_style);
    final_hashmap.insert("a".to_string(), a_style);
    final_hashmap.insert("ol".to_string(), ol_style);
    final_hashmap.insert("ul".to_string(), ul_style);
    final_hashmap.insert("dl".to_string(), dl_style);
    final_hashmap.insert("dt".to_string(), dt_style);
    final_hashmap.insert("dd".to_string(), dd_style);
    final_hashmap.insert("li".to_string(), li_style);

    let form_style = HydaStyle {
        margin: Some(UiRect {
            top: Val::Px(0.0),
            bottom: Val::Px(16.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
        }),
        ..default()
    };

    final_hashmap.insert("form".to_string(), form_style);

    let input_style = HydaStyle {
        ..default()
    };

    final_hashmap.insert("input".to_string(), input_style);

    final_hashmap
}

fn get_default_firasans(weight: i32, fs: &HydaFontStyle) -> String {

    let mut final_path: String = "embedded://bevy_hyda/fonts/FiraSans-Regular.ttf".to_string();

    if weight >= 100 || weight < 100 { final_path = "embedded://bevy_hyda/fonts/FiraSans-Thin.ttf".to_string(); }

    if weight >= 200 { final_path = "embedded://bevy_hyda/fonts/FiraSans-ExtraLight.ttf".to_string(); }
    if weight >= 300 { final_path = "embedded://bevy_hyda/fonts/FiraSans-Light.ttf".to_string(); }
    if weight >= 400 { final_path = "embedded://bevy_hyda/fonts/FiraSans-Regular.ttf".to_string(); }
    if weight >= 500 { final_path = "embedded://bevy_hyda/fonts/FiraSans-Medium.ttf".to_string(); }
    if weight >= 600 { final_path = "embedded://bevy_hyda/fonts/FiraSans-SemiBold.ttf".to_string(); }
    if weight >= 700 { final_path = "embedded://bevy_hyda/fonts/FiraSans-Bold.ttf".to_string(); }
    if weight >= 800 { final_path = "embedded://bevy_hyda/fonts/FiraSans-ExtraBold.ttf".to_string(); }
    if weight >= 900 { final_path = "embedded://bevy_hyda/fonts/FiraSans-Black.ttf".to_string(); }

    if *fs == HydaFontStyle::Italic {
        if weight >= 400 && weight < 500 {
            return "embedded://bevy_hyda/fonts/FiraSans-Italic.ttf".to_string();
        }

        return final_path.replace(".ttf", "Italic.ttf");
    }

    return final_path;
}

fn is_component_inline_by_default(comp: &HydaComponent) -> bool {
    return match comp {
        HydaComponent::Heading1 { .. } |
        HydaComponent::Heading2 { .. } |
        HydaComponent::Heading3 { .. } |
        HydaComponent::Heading4 { .. } |
        HydaComponent::Heading5 { .. } |
        HydaComponent::Heading6 { .. } |
        HydaComponent::Paragraph { .. } |
        HydaComponent::OrderedList { .. } |
        HydaComponent::UnorderedList { .. } |
        HydaComponent::DescriptiveList { .. } |
        HydaComponent::DescriptionTerm { .. } |
        HydaComponent::DescriptionDetail { .. } |
        HydaComponent::ListItem { .. } => {
            true
        },
        _ => false
    }
}

fn is_component_inlined(comp: &HydaComponent, display: &HydaDisplay) -> bool {

    if *display == HydaDisplay::InlineBlock {
        return false;
    }

    return is_component_inline_by_default(comp);
}

fn spawn_text(comp: &HydaComponent, commands: &mut Commands, final_tag: String, get_styles: &HashMap<String, HydaStyle>, text_section_vector: &Vec<TextSection>, child_id: i32) -> Entity {

    let get_display = get_display_from_style(get_styles, final_tag.clone());

    if is_component_inline_by_default(comp) {
        return commands.spawn(TextBundle::from_sections(text_section_vector.clone())).id();
    }
    else {

        let tag_plus_p: String = final_tag.to_string() + " p";

        return commands.spawn(
            NodeBundle {
                style: Style {
                    flex_direction: get_flex_direction_from_style(&get_styles, tag_plus_p.clone(), "flex-direction"),
                    margin: get_uirect_from_style(&get_styles, tag_plus_p.clone(), "margin", true, is_component_inlined(comp, &get_display), child_id == 0),
                    padding: get_uirect_from_style(&get_styles, tag_plus_p.clone(), "padding", true, is_component_inlined(comp, &get_display), child_id == 0),
    
                    width: get_val_from_style(&get_styles, tag_plus_p.clone(), "width"),
                    height: get_val_from_style(&get_styles, tag_plus_p.clone(), "height"),
                    ..default()
                },
                ..default()
            }
        ).with_children(|parent| {
            parent.spawn(
                TextBundle::from_sections(text_section_vector.clone())
            );
        }).id();
    }
}

fn tag_to_string(tag: &HydaComponent) -> String {
    return match tag {
        HydaComponent::Html { .. } => "html".to_string(),
        HydaComponent::Body { .. } => "body".to_string(),
        HydaComponent::Div { .. } => "div".to_string(),

        HydaComponent::Header { .. } => "header".to_string(),
        HydaComponent::Main { .. } => "main".to_string(),

        HydaComponent::HeadingGroup { .. } => "hgroup".to_string(),
        HydaComponent::Heading1 { .. } => "h1".to_string(),
        HydaComponent::Heading2 { .. } => "h2".to_string(),
        HydaComponent::Heading3 { .. } => "h3".to_string(),
        HydaComponent::Heading4 { .. } => "h4".to_string(),
        HydaComponent::Heading5 { .. } => "h5".to_string(),
        HydaComponent::Heading6 { .. } => "h6".to_string(),
        HydaComponent::Footer { .. } => "footer".to_string(),

        HydaComponent::Navigator { .. } => "nav".to_string(),
        HydaComponent::Blockquote { .. } => "blockquote".to_string(),
        HydaComponent::Address { .. } => "address".to_string(),

        HydaComponent::Paragraph { .. } => "p".to_string(),
        HydaComponent::Anchor { .. } => "a".to_string(),
        HydaComponent::BoldText { .. } => "b".to_string(),
        HydaComponent::StrongText { .. } => "strong".to_string(),
        HydaComponent::OrderedList { .. } => "ol".to_string(),
        HydaComponent::UnorderedList { .. } => "ul".to_string(),
        HydaComponent::DescriptiveList { .. } => "dl".to_string(),
        HydaComponent::DescriptionTerm { .. } => "dt".to_string(),
        HydaComponent::DescriptionDetail { .. } => "dd".to_string(),
        HydaComponent::ListItem { .. } => "li".to_string(),

        HydaComponent::Form { .. } => "form".to_string(),
        HydaComponent::Input { .. } => "input".to_string(),
        _ => todo!(),
    }
}

fn create_text_section(get_styles: &HashMap<String, HydaStyle>, st: String, text_tag: String, asset_server: &Res<AssetServer>) -> TextSection {
    TextSection::new(
        st.to_string().replace("\n", " ") + " ",
        TextStyle {
            font: asset_server.load(
                get_default_firasans(
                    get_i32_from_style(&get_styles, text_tag.clone(), "font-weight"),
                    &get_font_style_from_style(&get_styles, text_tag.clone())
                )
            ),
            font_size: get_f32_from_style(&get_styles, text_tag.clone(), "font-size"),
            color: get_color_from_style(&get_styles, text_tag.clone(), "color").into(),
        }
    )
}

impl HydaComponent {

    pub fn spawn_ui(&self, commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {

        let get_styles = initialize_default_styles();
        self.spawn_ui_impl(commands, asset_server, "".to_string(), 0, &get_styles)
    }

    pub fn create_children(&self,
        get_content: &Option<Vec<HydaComponent>>, 
        commands: &mut Commands, 
        asset_server: &Res<AssetServer>, 
        get_styles: &HashMap<String, HydaStyle>,
        final_tag: String,
        child_id: i32) -> Vec<Entity> {

        let mut child_c: i32 = 0;
    
        let mut child_vec: Vec<Entity> = Vec::new();
    
        let mut text_section_vector: Vec<TextSection> = Vec::new();
        let mut is_inside_text: bool = false;

        let get_display = get_display_from_style(&get_styles, final_tag.clone());

        let is_list_item: bool = match self {
            HydaComponent::ListItem { .. } => get_display == HydaDisplay::ListItem,
            _ => false
        };
    
        if let Some(cont) = get_content {

            if is_list_item {
                is_inside_text = true;
                text_section_vector.push(create_text_section(&get_styles, (child_id + 1).to_string() + ". ", final_tag.clone(), asset_server));
            }

            for c in cont {
                match c {
                    HydaComponent::Text(st) => {
                        is_inside_text = true;
    
                        let mut text_tag = final_tag.to_string();
                        text_tag += &(" ".to_owned() + &tag_to_string(&self));

                        text_section_vector.push(create_text_section(&get_styles, st.to_string(), text_tag.clone(), asset_server));
                    },
                    HydaComponent::Anchor { content, .. } | HydaComponent::BoldText { content, .. } | HydaComponent::StrongText { content, .. } => {
                        if let Some(cont2) = content {
                            for c2 in cont2 {
                                match c2 {
                                    HydaComponent::Text(st) => {
                                        is_inside_text = true;

                                        let mut text_tag = final_tag.to_string();
                                        text_tag += &(" ".to_owned() + &tag_to_string(&c));
                
                                        text_section_vector.push(create_text_section(&get_styles, st.to_string(), text_tag.clone(), asset_server));
                                    },
                                    _ => panic!("Not implemented {:?}", c2),
                                }
                            }
                        }
                    },
                    _ => {
                        if is_inside_text {
                            let final_text = spawn_text(&self, commands, final_tag.clone(), &get_styles, &text_section_vector, child_c);
                            child_vec.push(final_text);
                            text_section_vector.clear();
                            is_inside_text = false;
    
                            child_c += 1;
                        }
    
                        let child = c.spawn_ui_impl(commands, asset_server, final_tag.clone(), child_c, &get_styles);
                        child_vec.push(child);
    
                        child_c += 1;
                    }
                }
            }
        }
    
        if is_inside_text {
            let final_text = spawn_text(&self, commands, final_tag.clone(), &get_styles, &text_section_vector, child_c);
            child_vec.push(final_text);
            text_section_vector.clear();
            is_inside_text = false;
    
            child_c += 1;
        }
    
        return child_vec;
    }

    pub fn spawn_ui_impl(&self, commands: &mut Commands, asset_server: &Res<AssetServer>, parent_name: String, child_id: i32, get_styles: &HashMap<String, HydaStyle>) -> Entity {

        match self {
            HydaComponent::Html { content, class, id, .. } | 
            HydaComponent::Body { content, class, id, .. } | 
            HydaComponent::Div { content, class, id, .. } |

            HydaComponent::Header { content, class, id, .. } |
            HydaComponent::Main { content, class, id, .. } |

            HydaComponent::HeadingGroup { content, class, id, .. } |
            HydaComponent::Heading1 { content, class, id, .. } |
            HydaComponent::Heading2 { content, class, id, .. } |
            HydaComponent::Heading3 { content, class, id, .. } |
            HydaComponent::Heading4 { content, class, id, .. } |
            HydaComponent::Heading5 { content, class, id, .. } |
            HydaComponent::Heading6 { content, class, id, .. } |
            HydaComponent::Footer { content, class, id, .. } |

            HydaComponent::Navigator { content, class, id, .. } |
            HydaComponent::Blockquote { content, class, id, .. } |
            HydaComponent::Address { content, class, id, .. } |

            HydaComponent::Paragraph { content, class, id, .. } |
            HydaComponent::OrderedList { content, class, id, .. } |
            HydaComponent::UnorderedList { content, class, id, .. } |
            HydaComponent::DescriptiveList { content, class, id, .. } |
            HydaComponent::DescriptionTerm { content, class, id, .. } |
            HydaComponent::DescriptionDetail { content, class, id, .. } |
            HydaComponent::ListItem { content, class, id, .. } | 

            HydaComponent::Form { content, class, id, .. } => {

                let tag_as_string = tag_to_string(&self);

                let mut final_tag: String = if parent_name != "".to_string() {
                    parent_name.clone() + " " + &tag_as_string
                }
                else {
                    tag_as_string
                };

                let mut text_section_vector: Vec<TextSection> = Vec::new();
                let mut is_inside_text: bool = false;

                let get_display = get_display_from_style(&get_styles, final_tag.clone());

                let child_vec = self.create_children(&content, commands, asset_server, &get_styles, final_tag.clone(), child_id);

                let mut result: EntityCommands;

                if child_vec.len() != 0 && get_display != HydaDisplay::None {
                    result = commands.spawn(
                        NodeBundle {
                            style: Style {
    
                                flex_direction: get_flex_direction_from_style(&get_styles, final_tag.clone(), "flex-direction"),
                                margin: get_uirect_from_style(&get_styles, final_tag.clone(), "margin", true, is_component_inlined(&self, &get_display), child_id == 0),
                                padding: get_uirect_from_style(&get_styles, final_tag.clone(), "padding", true, is_component_inlined(&self, &get_display), child_id == 0),
    
                                width: get_val_from_style(&get_styles, final_tag.clone(), "width"),
                                height: get_val_from_style(&get_styles, final_tag.clone(), "height"),
    
                                ..default()
                            },
                            background_color: Color::WHITE.into(),
                            ..default()
                        }
                    );
                }
                else {
                    result = commands.spawn_empty();
                }

                if let Some(st) = id {
                    result.insert(HydaId(st.to_string()));
                }

                if let Some(st) = class {
                    result.insert(HydaClass(st.to_string()));
                }

                for c in child_vec {
                    result.add_child(c);
                }

                return result.id();
            },
            HydaComponent::Head { content, .. } => {

                let mut child_vec: Vec<Entity> = Vec::new();
                let mut child_c: i32 = 0;

                if let Some(cont) = content {
                    for c in cont {
                        match c {
                            HydaComponent::Style { .. } | HydaComponent::Title { .. } => {
                                child_vec.push(c.spawn_ui_impl(commands, asset_server, "".to_string(), child_c, &get_styles));
                            },
                            _ => {
                                panic!("Only document metadata is allowed in <head>!");
                            }
                        }

                        child_c += 1;
                    }
                }

                let mut result = commands.spawn_empty();

                for c in child_vec {
                    result.add_child(c);
                }

                return result.id();
            },
            HydaComponent::Title { .. } => {
                return commands.spawn_empty().id();
            },
            HydaComponent::Style { content, .. } => {

                if let Some(cont) = content {

                    // Parse a style sheet from a string.
                    let mut stylesheet = StyleSheet::parse(cont, ParserOptions::default()).unwrap();

                    dbg!(&stylesheet);
                }

                return commands.spawn_empty().id();
            },
            HydaComponent::Input { .. } => {
                println!("Warning: HydaComponent::Input is not yet implemented!");
                return commands.spawn_empty().id();
            }
            _ => todo!("{:?}", dbg!(&self)),
        }
    }
}

pub fn html_string(get_str: String) -> HydaComponent {
    let h: HydaComponent = from_str(&get_str).expect("Oops!");
    return h;
}

pub fn html_file(get_file: String) -> HydaComponent {
    let get_str = fs::read_to_string(&get_file);
    html_string(get_str.unwrap())
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
    }
}