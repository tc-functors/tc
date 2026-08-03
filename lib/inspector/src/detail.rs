use std::collections::HashMap;
use composer::Topology;
use colored_json::to_colored_json_auto;
use ratatui::text::Span;
use ratatui::layout::{Rect, Layout, Constraint};
use ratatui::buffer::Buffer;
use ratatui::widgets::{Widget, Tabs};
use ratatui::style::{Color, Style};
use ratatui::Frame;
use ratatui::symbols;
use crate::color::ansi_to_spans;

struct JsonWidget {
    content: String,
}

impl Widget for JsonWidget {

    fn render(self, area: Rect, buf: &mut Buffer) {

        let lines_iter = self.content.lines();

        for (i, line) in lines_iter.take(area.height as usize).enumerate() {
            let y = area.y + i as u16;

            let x0 = area.x;
            let mut x = x0;

            let spans: Vec<Span<'static>> = ansi_to_spans(line);

            for span in spans {
                let text = span.content.clone();
                for (j, ch) in text.chars().enumerate() {
                    let px = x + j as u16;
                    if px >= area.x + area.width {
                        break;
                    }
                    buf.set_string(px, y, &ch.to_string(), span.style);
                }

                let span_len = text.chars().count() as u16;
                x = x0 + (x - x0) + span_len;
                if x >= area.x + area.width {
                    break;
                }
            }
        }
    }
}


pub struct FunctionWidget {
    pub build: String,
    pub runtime: String,
    pub selected_tab: usize,
}

impl FunctionWidget {

    fn render(&self, frame: &mut Frame, area: Rect) {

        let parts = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        let tabs_area = parts[0];
        let content_area = parts[1];

         let tabs = Tabs::new(vec!["build", "runtime"])
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Magenta).bg(Color::Black).add_modifier(ratatui::style::Modifier::BOLD))
            .select(self.selected_tab)
            .divider(symbols::DOT)
            .padding(" ", " ");

        frame.render_widget(tabs, tabs_area);

        let widget = match self.selected_tab {
            0 => JsonWidget { content: self.build.clone() },
            1 => JsonWidget { content: self.runtime.clone() },
            _ => JsonWidget { content: self.build.clone() },
        };


        frame.render_widget(widget, content_area);

    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            crossterm::event::KeyCode::Left => {
               self.selected_tab = self.selected_tab.saturating_sub(1);
            }
            crossterm::event::KeyCode::Right => {
                // clamp to max tab index (2 tabs total => max index 1)
                self.selected_tab = (self.selected_tab + 1).min(1);
            }
            _ => {}
        }
    }
}


pub struct Detail {
    pub topology: Topology,
    pub entity: String,
    pub name: String
}


impl Detail {

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.name.is_empty() {
            match self.entity.as_ref() {
                "routes" => {
                    let d = self.topology.routes.get(&self.name).unwrap();
                    let pretty = to_colored_json_auto(&d).unwrap();
                    let widget = JsonWidget { content: pretty };
                    frame.render_widget(widget, area);
                },
                "events" => {
                    let d = self.topology.events.get(&self.name).unwrap();
                    let pretty = to_colored_json_auto(&d).unwrap();
                    let widget = JsonWidget { content: pretty };
                    frame.render_widget(widget, area);
                },

                "functions" => {
                    let f = self.topology.functions.get(&self.name).unwrap();
                    let fw = FunctionWidget {
                        build: to_colored_json_auto(&f.build).unwrap(),
                        runtime: to_colored_json_auto(&f.build).unwrap(),
                        selected_tab: 0
                    };
                    fw.render(frame, area);
                },

                _ => {
                    let d: HashMap<String, String> = HashMap::new();
                    let pretty = to_colored_json_auto(&d).unwrap();
                    let widget = JsonWidget { content: pretty };
                    frame.render_widget(widget, area);
                }
            }
        }
    }
}
