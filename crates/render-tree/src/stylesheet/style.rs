use crate::stylesheet::Color;
use std;
use std::fmt;
use std::io;
use termcolor::WriteColor;
use termcolor::{self, ColorSpec};

pub trait WriteStyle: WriteColor {
    fn set_style<'a>(&mut self, style: impl Into<Style>) -> io::Result<()> {
        self.set_color(&style.into().to_color_spec())
    }
}

impl<T: WriteColor> WriteStyle for T {}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ColorAttribute {
    Reset,
    Inherit,
    Color(Color),
}

impl fmt::Display for ColorAttribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ColorAttribute::Reset => write!(f, "reset"),
            ColorAttribute::Inherit => write!(f, "inherit"),
            ColorAttribute::Color(color) => write!(f, "{}", color),
        }
    }
}

impl AttributeValue for ColorAttribute {
    type ApplyValue = Option<Color>;
    type SetValue = ColorAttribute;

    fn parse(s: &str) -> ColorAttribute {
        match s {
            "reset" => ColorAttribute::Reset,
            other => ColorAttribute::Color(other.into()),
        }
    }

    fn update(self, attribute: ColorAttribute) -> ColorAttribute {
        match attribute {
            ColorAttribute::Inherit => self,
            other => other,
        }
    }

    fn apply(&self, f: impl FnOnce(Option<Color>)) {
        match self {
            ColorAttribute::Color(color) => f(Some(*color)),
            ColorAttribute::Reset => f(None),
            _ => {}
        }
    }

    fn is_default(&self) -> bool {
        match self {
            ColorAttribute::Inherit => true,
            _ => false,
        }
    }

    fn set(self, color: ColorAttribute) -> ColorAttribute {
        color
    }

    fn debug_value(&self) -> Option<String> {
        Some(format!("{}", self))
    }
}

impl<'a> From<&'a str> for ColorAttribute {
    fn from(color: &'a str) -> ColorAttribute {
        ColorAttribute::Color(color.into())
    }
}

impl From<Color> for ColorAttribute {
    fn from(color: Color) -> ColorAttribute {
        ColorAttribute::Color(color)
    }
}

impl<'a> From<Option<&'a termcolor::Color>> for ColorAttribute {
    fn from(color: Option<&'a termcolor::Color>) -> ColorAttribute {
        match color {
            None => ColorAttribute::Inherit,
            Some(color) => ColorAttribute::Color(color.into()),
        }
    }
}

impl std::default::Default for ColorAttribute {
    fn default() -> ColorAttribute {
        ColorAttribute::Inherit
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum WeightAttribute {
    // bright
    Normal,

    // bright + bold
    Bold,

    // neither
    Dim,

    Inherit,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SetWeight {
    Normal,
    Bold,
    Dim,
}

impl fmt::Display for WeightAttribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WeightAttribute::Normal => write!(f, "normal"),
            WeightAttribute::Bold => write!(f, "bold"),
            WeightAttribute::Dim => write!(f, "dim"),
            WeightAttribute::Inherit => write!(f, "inherit"),
        }
    }
}

impl std::default::Default for WeightAttribute {
    fn default() -> WeightAttribute {
        WeightAttribute::Inherit
    }
}

impl AttributeValue for WeightAttribute {
    type ApplyValue = SetWeight;
    type SetValue = WeightAttribute;

    fn parse(s: &str) -> WeightAttribute {
        match s {
            "normal" => WeightAttribute::Normal,
            "bold" => WeightAttribute::Bold,
            "dim" => WeightAttribute::Dim,
            other => panic!("Unexpected value for `weight`: {}", other),
        }
    }

    fn update(self, attribute: WeightAttribute) -> WeightAttribute {
        match attribute {
            WeightAttribute::Normal => WeightAttribute::Normal,
            WeightAttribute::Bold => WeightAttribute::Bold,
            WeightAttribute::Dim => WeightAttribute::Dim,
            WeightAttribute::Inherit => self,
        }
    }

    fn apply(&self, f: impl FnOnce(SetWeight)) {
        match self {
            WeightAttribute::Normal => f(SetWeight::Normal),
            WeightAttribute::Bold => f(SetWeight::Bold),
            WeightAttribute::Dim => f(SetWeight::Dim),
            _ => {}
        }
    }

    fn is_default(&self) -> bool {
        match self {
            WeightAttribute::Inherit => true,
            _ => false,
        }
    }

    fn set(self, weight: Self) -> WeightAttribute {
        weight
    }

    fn debug_value(&self) -> Option<String> {
        match self {
            WeightAttribute::Inherit => None,
            other => Some(format!("{}", other)),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BooleanAttribute {
    On,
    Off,
    Inherit,
}

impl fmt::Display for BooleanAttribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BooleanAttribute::On => write!(f, "true"),
            BooleanAttribute::Off => write!(f, "false"),
            BooleanAttribute::Inherit => write!(f, "inherit"),
        }
    }
}

impl std::default::Default for BooleanAttribute {
    fn default() -> BooleanAttribute {
        BooleanAttribute::Inherit
    }
}

impl AttributeValue for BooleanAttribute {
    type ApplyValue = bool;
    type SetValue = BooleanAttribute;

    fn parse(s: &str) -> BooleanAttribute {
        match s {
            "true" => BooleanAttribute::On,
            "false" => BooleanAttribute::Off,
            other => panic!("Unexpected boolean attribute {}", other),
        }
    }

    fn update(self, attribute: BooleanAttribute) -> BooleanAttribute {
        match attribute {
            BooleanAttribute::On => BooleanAttribute::On,
            BooleanAttribute::Off => BooleanAttribute::Off,
            BooleanAttribute::Inherit => self,
        }
    }

    fn apply(&self, f: impl FnOnce(bool)) {
        match self {
            BooleanAttribute::On => f(true),
            BooleanAttribute::Off => f(false),
            _ => {}
        }
    }

    fn is_default(&self) -> bool {
        match self {
            BooleanAttribute::Inherit => true,
            _ => false,
        }
    }

    fn set(self, boolean: BooleanAttribute) -> BooleanAttribute {
        boolean
    }

    fn debug_value(&self) -> Option<String> {
        None
    }
}

impl<'a> Into<Style> for &'a str {
    fn into(self) -> Style {
        Style::from_stylesheet(self)
    }
}

impl<'a> Into<Style> for &'a Style {
    fn into(self) -> Style {
        self.clone()
    }
}

pub trait AttributeValue: Default + fmt::Display {
    type ApplyValue;
    type SetValue;

    fn parse(s: &str) -> Self;
    fn update(self, other: Self) -> Self;
    fn apply(&self, f: impl FnOnce(Self::ApplyValue));
    fn is_default(&self) -> bool;
    fn set(self, value: Self::SetValue) -> Self;
    fn debug_value(&self) -> Option<String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute<Value: AttributeValue> {
    name: AttributeName,
    value: Value,
}

impl<Value: AttributeValue> fmt::Display for Attribute<Value> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}={}", self.name, self.value)
    }
}

impl<Value: AttributeValue> Attribute<Value> {
    pub fn inherit(name: impl Into<AttributeName>) -> Attribute<Value> {
        Attribute(name.into(), Value::default())
    }

    pub fn tuple(&self) -> (AttributeName, Option<String>) {
        (self.name, self.value.debug_value())
    }
}

impl<Value: AttributeValue> Attribute<Value> {
    pub fn update(self, attribute: Attribute<Value>) -> Attribute<Value> {
        Attribute(self.name.clone(), self.value.update(attribute.value))
    }

    pub fn apply(&self, f: impl FnOnce(Value::ApplyValue)) {
        self.value.apply(f)
    }

    pub fn is_default(&self) -> bool {
        self.value.is_default()
    }

    pub fn has_value(&self) -> bool {
        !self.is_default()
    }

    pub fn mutate(&mut self, value: Value) {
        self.value = value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeName {
    Fg,
    Bg,
    Weight,
    Underline,
}

impl<'a> From<&'a str> for AttributeName {
    fn from(from: &'a str) -> AttributeName {
        match from {
            "fg" => AttributeName::Fg,
            "bg" => AttributeName::Bg,
            "weight" => AttributeName::Weight,
            "underline" => AttributeName::Underline,
            other => panic!("Invalid style attribute name {}", other),
        }
    }
}

impl<'a> From<String> for AttributeName {
    fn from(from: String) -> AttributeName {
        AttributeName::from(&from[..])
    }
}

impl fmt::Display for AttributeName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            AttributeName::Fg => "fg",
            AttributeName::Bg => "bg",
            AttributeName::Weight => "weight",
            AttributeName::Underline => "underline",
        };

        write!(f, "{}", name)
    }
}

#[allow(non_snake_case)]
fn Attribute<Value: AttributeValue>(name: AttributeName, value: Value) -> Attribute<Value> {
    Attribute {
        name: name.into(),
        value,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    weight: Attribute<WeightAttribute>,
    underline: Attribute<BooleanAttribute>,
    fg: Attribute<ColorAttribute>,
    bg: Attribute<ColorAttribute>,
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut has_prev = false;

        let mut space = |f: &mut fmt::Formatter| -> fmt::Result {
            if has_prev {
                write!(f, " ")?;
            } else {
                has_prev = true;
            }

            Ok(())
        };

        write!(f, "Style {{")?;

        if self.fg.has_value() {
            space(f)?;
            write!(f, "{}", self.fg)?;
        }

        if self.bg.has_value() {
            space(f)?;
            write!(f, "{}", self.bg)?;
        }

        if self.weight.has_value() {
            space(f)?;
            write!(f, "{}", self.weight)?;
        }

        if self.underline.has_value() {
            space(f)?;
            write!(f, "{}", self.underline)?;
        }

        write!(f, "}}")?;

        Ok(())
    }
}

#[allow(non_snake_case)]
pub fn Style(input: &str) -> Style {
    Style::from_stylesheet(input)
}

impl Style {
    pub fn empty() -> Style {
        Style {
            fg: Attribute(AttributeName::Fg, ColorAttribute::default()),
            bg: Attribute(AttributeName::Bg, ColorAttribute::default()),
            weight: Attribute(AttributeName::Weight, WeightAttribute::default()),
            underline: Attribute(AttributeName::Underline, BooleanAttribute::default()),
        }
    }

    pub fn new() -> Style {
        Style::empty()
    }

    pub fn from_stylesheet(input: &str) -> Style {
        let mut fg = Attribute::inherit(AttributeName::Fg);
        let mut bg = Attribute::inherit(AttributeName::Bg);
        let mut weight = Attribute::inherit(AttributeName::Weight);
        let mut underline = Attribute::inherit(AttributeName::Underline);

        for (key, value) in StyleString::new(input) {
            match key {
                AttributeName::Fg => fg = Attribute(key, ColorAttribute::parse(value)),
                AttributeName::Bg => bg = Attribute(key, ColorAttribute::parse(value)),
                AttributeName::Weight => weight = Attribute(key, WeightAttribute::parse(value)),
                AttributeName::Underline => {
                    underline = Attribute(key, BooleanAttribute::parse(value))
                }
            }
        }

        Style {
            weight,
            underline,
            bg,
            fg,
        }
    }

    pub fn from_color_spec(spec: ColorSpec) -> Style {
        let mut weight = WeightAttribute::Inherit;

        if spec.bold() && spec.intense() {
            weight = weight.update(WeightAttribute::Bold);
        } else if spec.intense() {
            weight = weight.update(WeightAttribute::Normal);
        } else if spec.bold() {
            panic!("ColorSpec bold + not intense is not supported as it is not portable");
        } else {
            weight = weight.update(WeightAttribute::Dim);
        }

        let mut underline = BooleanAttribute::Inherit;

        if spec.underline() {
            underline = underline.set(BooleanAttribute::On);
        }

        let foreground = spec.fg().into();
        let background = spec.bg().into();

        Style {
            weight: Attribute(AttributeName::Weight, weight),
            underline: Attribute(AttributeName::Underline, underline),
            fg: Attribute(AttributeName::Fg, foreground),
            bg: Attribute(AttributeName::Bg, background),
        }
    }

    pub fn debug_attributes(&self) -> Vec<(AttributeName, Option<String>)> {
        let mut attrs: Vec<(AttributeName, Option<String>)> = vec![];

        if self.weight.has_value() {
            attrs.push(self.weight.tuple());
        }

        if self.fg.has_value() {
            attrs.push(self.fg.tuple());
        }

        if self.bg.has_value() {
            attrs.push(self.bg.tuple());
        }

        attrs
    }

    pub fn union(self, other: Style) -> Style {
        Style {
            weight: self.weight.update(other.weight),
            underline: self.underline.update(other.underline),
            fg: self.fg.update(other.fg),
            bg: self.bg.update(other.bg),
        }
    }

    pub fn to_color_spec(&self) -> ColorSpec {
        let mut spec = ColorSpec::new();

        self.weight.apply(|w| match w {
            SetWeight::Normal => {
                spec.set_intense(true);
            }
            SetWeight::Bold => {
                spec.set_bold(true).set_intense(true);
            }
            SetWeight::Dim => {
                spec.set_bold(false).set_intense(false);
            }
        });

        self.underline.apply(|b| {
            spec.set_underline(b);
        });

        self.fg.apply(|fg| {
            spec.set_fg(fg.map(|fg| fg.into()));
        });

        self.bg.apply(|bg| {
            spec.set_bg(bg.map(|bg| bg.into()));
        });

        spec
    }

    pub fn has_value(&self) -> bool {
        !self.is_default()
    }

    pub fn is_default(&self) -> bool {
        self.weight.is_default()
            && self.underline.is_default()
            && self.fg.is_default()
            && self.bg.is_default()
    }

    pub fn fg(&self, color: impl Into<Color>) -> Style {
        let color_attribute = ColorAttribute::Color(color.into());
        self.update(|style| style.fg.mutate(color_attribute))
    }

    pub fn bg(&self, color: impl Into<Color>) -> Style {
        let color_attribute = ColorAttribute::Color(color.into());
        self.update(|style| style.bg.mutate(color_attribute))
    }

    pub fn weight(&self, weight: WeightAttribute) -> Style {
        self.update(|style| style.weight.mutate(weight))
    }

    pub fn bold(&self) -> Style {
        self.update(|style| style.weight.mutate(WeightAttribute::Bold))
    }

    pub fn dim(&self) -> Style {
        self.update(|style| style.weight.mutate(WeightAttribute::Dim))
    }

    pub fn normal(&self) -> Style {
        self.update(|style| style.weight.mutate(WeightAttribute::Normal))
    }

    pub fn underline(&self) -> Style {
        self.update(|style| style.underline.mutate(BooleanAttribute::On))
    }

    pub fn nounderline(&self) -> Style {
        self.update(|style| style.underline.mutate(BooleanAttribute::Off))
    }

    fn update(&self, f: impl FnOnce(&mut Style)) -> Style {
        let mut style = self.clone();
        f(&mut style);
        style
    }
}

struct StyleString<'a> {
    rest: &'a str,
}

impl<'a> StyleString<'a> {
    fn new(input: &str) -> StyleString {
        StyleString { rest: input }
    }
}

impl<'a> Iterator for StyleString<'a> {
    type Item = (AttributeName, &'a str);

    fn next(&mut self) -> Option<(AttributeName, &'a str)> {
        if self.rest.len() == 0 {
            return None;
        }

        let name = if let Some(next) = self.rest.find(':') {
            let next_part = &self.rest[..next];
            self.rest = &self.rest[(next + 1)..];
            next_part.trim()
        } else {
            panic!("Unexpected style string, missing `:`")
        };

        if let Some(next) = self.rest.find(';') {
            let next_part = self.rest[..next].trim();
            self.rest = &self.rest[(next + 1)..];
            Some((AttributeName::from(name), next_part))
        } else {
            let next_part = self.rest.trim();
            self.rest = "";
            Some((AttributeName::from(name), next_part))
        }
    }
}
