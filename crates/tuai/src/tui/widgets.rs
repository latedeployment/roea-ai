//! Custom widgets for the TUI

use ratatui::prelude::*;

/// A sparkline widget for displaying mini charts
pub struct MiniChart<'a> {
    data: &'a [u64],
    max: Option<u64>,
    style: Style,
}

impl<'a> MiniChart<'a> {
    pub fn new(data: &'a [u64]) -> Self {
        Self {
            data,
            max: None,
            style: Style::default(),
        }
    }

    #[allow(dead_code)]
    pub fn max(mut self, max: u64) -> Self {
        self.max = Some(max);
        self
    }

    #[allow(dead_code)]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Widget for MiniChart<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.data.is_empty() || area.width == 0 || area.height == 0 {
            return;
        }

        let max = self.max.unwrap_or_else(|| *self.data.iter().max().unwrap_or(&1));
        let bars = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

        for (i, &value) in self.data.iter().enumerate().take(area.width as usize) {
            let bar_idx = if max > 0 {
                ((value as f64 / max as f64) * 7.0).round() as usize
            } else {
                0
            };
            let bar_char = bars[bar_idx.min(7)];

            buf.set_string(
                area.x + i as u16,
                area.y,
                bar_char,
                self.style,
            );
        }
    }
}

/// Status indicator widget
pub struct StatusIndicator {
    status: Status,
    label: String,
}

#[derive(Clone, Copy)]
pub enum Status {
    Active,
    Warning,
    Error,
    Inactive,
}

impl StatusIndicator {
    pub fn new(status: Status, label: impl Into<String>) -> Self {
        Self {
            status,
            label: label.into(),
        }
    }
}

impl Widget for StatusIndicator {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (symbol, color) = match self.status {
            Status::Active => ("●", Color::Green),
            Status::Warning => ("●", Color::Yellow),
            Status::Error => ("●", Color::Red),
            Status::Inactive => ("○", Color::DarkGray),
        };

        let text = format!("{} {}", symbol, self.label);
        buf.set_string(area.x, area.y, text, Style::default().fg(color));
    }
}

/// Progress bar widget with label
pub struct LabeledProgress<'a> {
    label: &'a str,
    value: f64,
    max: f64,
    style: Style,
}

impl<'a> LabeledProgress<'a> {
    #[allow(dead_code)]
    pub fn new(label: &'a str, value: f64, max: f64) -> Self {
        Self {
            label,
            value,
            max,
            style: Style::default(),
        }
    }

    #[allow(dead_code)]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Widget for LabeledProgress<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 || area.height == 0 {
            return;
        }

        // Render label
        let label_width = self.label.len().min(area.width as usize / 3);
        buf.set_string(area.x, area.y, &self.label[..label_width], Style::default().fg(Color::DarkGray));

        // Render progress bar
        let bar_start = area.x + label_width as u16 + 1;
        let bar_width = area.width.saturating_sub(label_width as u16 + 1);

        if bar_width == 0 {
            return;
        }

        let filled = ((self.value / self.max) * bar_width as f64).round() as u16;
        let filled = filled.min(bar_width);

        for i in 0..bar_width {
            let char = if i < filled { "█" } else { "░" };
            buf.set_string(bar_start + i, area.y, char, self.style);
        }
    }
}
