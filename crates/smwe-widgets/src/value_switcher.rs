use std::ops::RangeInclusive;

use egui::{emath::Numeric, Button, DragValue, Response, Ui, Widget, WidgetText};
use egui_phosphor::regular as icons;

type NumFormatter<'a> = Box<dyn 'a + Fn(f64, RangeInclusive<usize>) -> String>;
type NumParser<'a> = Box<dyn 'a + Fn(&str) -> Option<f64>>;

#[derive(Copy, Clone, Debug)]
pub enum ValueSwitcherButtons {
    MinusPlus,
    LeftRight,
}

pub struct ValueSwitcher<'a, V, L>
where
    V: Numeric,
    L: Into<WidgetText>,
{
    value:            &'a mut V,
    label:            L,
    buttons:          ValueSwitcherButtons,
    range:            RangeInclusive<V>,
    custom_formatter: Option<NumFormatter<'a>>,
    custom_parser:    Option<NumParser<'a>>,
}

impl<'a, V, L> ValueSwitcher<'a, V, L>
where
    V: Numeric,
    L: Into<WidgetText>,
{
    pub fn new(value: &'a mut V, label: L, buttons: ValueSwitcherButtons) -> Self {
        Self { value, label, buttons, range: V::MIN..=V::MAX, custom_formatter: None, custom_parser: None }
    }

    pub fn range(mut self, range: RangeInclusive<V>) -> Self {
        self.range = range;
        self
    }

    pub fn custom_formatter(mut self, formatter: impl 'a + Fn(f64, RangeInclusive<usize>) -> String) -> Self {
        self.custom_formatter = Some(Box::new(formatter));
        self
    }

    pub fn custom_parser(mut self, parser: impl 'a + Fn(&str) -> Option<f64>) -> Self {
        self.custom_parser = Some(Box::new(parser));
        self
    }

    pub fn binary(self, min_width: usize, twos_complement: bool) -> Self {
        assert!(min_width > 0, "Slider::binary: `min_width` must be greater than 0");
        if twos_complement {
            self.custom_formatter(move |n, _| format!("{:0>min_width$b}", n as i64))
        } else {
            self.custom_formatter(move |n, _| {
                let sign = if n < 0.0 { "-" } else { "" };
                format!("{sign}{:0>min_width$b}", n.abs() as i64)
            })
        }
        .custom_parser(|s| i64::from_str_radix(s, 2).map(|n| n as f64).ok())
    }

    pub fn octal(self, min_width: usize, twos_complement: bool) -> Self {
        assert!(min_width > 0, "Slider::octal: `min_width` must be greater than 0");
        if twos_complement {
            self.custom_formatter(move |n, _| format!("{:0>min_width$o}", n as i64))
        } else {
            self.custom_formatter(move |n, _| {
                let sign = if n < 0.0 { "-" } else { "" };
                format!("{sign}{:0>min_width$o}", n.abs() as i64)
            })
        }
        .custom_parser(|s| i64::from_str_radix(s, 8).map(|n| n as f64).ok())
    }

    pub fn hexadecimal(self, min_width: usize, twos_complement: bool, upper: bool) -> Self {
        assert!(min_width > 0, "Slider::hexadecimal: `min_width` must be greater than 0");
        match (twos_complement, upper) {
            (true, true) => self.custom_formatter(move |n, _| format!("{:0>min_width$X}", n as i64)),
            (true, false) => self.custom_formatter(move |n, _| format!("{:0>min_width$x}", n as i64)),
            (false, true) => self.custom_formatter(move |n, _| {
                let sign = if n < 0.0 { "-" } else { "" };
                format!("{sign}{:0>min_width$X}", n.abs() as i64)
            }),
            (false, false) => self.custom_formatter(move |n, _| {
                let sign = if n < 0.0 { "-" } else { "" };
                format!("{sign}{:0>min_width$x}", n.abs() as i64)
            }),
        }
        .custom_parser(|s| i64::from_str_radix(s, 16).map(|n| n as f64).ok())
    }
}

impl<V, L> Widget for ValueSwitcher<'_, V, L>
where
    V: Numeric,
    L: Into<WidgetText>,
{
    fn ui(self, ui: &mut Ui) -> Response {
        let (label_l, label_r) = match self.buttons {
            ValueSwitcherButtons::LeftRight => (icons::ARROW_LEFT, icons::ARROW_RIGHT),
            ValueSwitcherButtons::MinusPlus => (icons::MINUS, icons::PLUS),
        };

        let min = *self.range.start();
        let max = *self.range.end();

        let inner_response = ui.horizontal(|ui| {
            let button_l = ui.add_enabled(*self.value > min, Button::new(label_l));
            let mut drag_value = ui.add({
                let mut dv = DragValue::new(self.value).clamp_range(self.range);
                if let Some(custom_formatter) = self.custom_formatter {
                    dv = dv.custom_formatter(custom_formatter);
                }
                if let Some(custom_parser) = self.custom_parser {
                    dv = dv.custom_parser(custom_parser);
                }
                dv
            });
            let button_r = ui.add_enabled(*self.value < max, Button::new(label_r));
            ui.label(self.label);

            if button_l.clicked() {
                *self.value = V::from_f64(self.value.to_f64() - 1.);
                drag_value.mark_changed();
            }
            if button_r.clicked() {
                *self.value = V::from_f64(self.value.to_f64() + 1.);
                drag_value.mark_changed();
            }
            drag_value
        });

        inner_response.inner
    }
}
