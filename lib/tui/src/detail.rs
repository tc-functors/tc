use std::collections::HashMap;
use composer::Topology;
use colored_json::to_colored_json_auto;
use ratatui::text::Span;
use ratatui::layout::Rect;
use ratatui::buffer::Buffer;
use ratatui::widgets::Widget;
use crate::color::ansi_to_spans;

pub struct DetailWidget {
    pub topology: Topology,
    pub entity: String,
    pub name: String
}

fn render_json(pretty: &str, area: Rect, buf: &mut Buffer) {
    let lines_iter = pretty.lines();

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

fn render_events(topology: &Topology, name: &str, area: Rect, buf: &mut Buffer) {
    let d = topology.events.get(name).unwrap();
    let pretty = to_colored_json_auto(&d).unwrap();
    render_json(&pretty, area, buf)
}

fn render_routes(topology: &Topology, name: &str, area: Rect, buf: &mut Buffer) {
    let d = topology.routes.get(name).unwrap();
    let pretty = to_colored_json_auto(&d).unwrap();
    render_json(&pretty, area, buf)
}

fn render_functions(topology: &Topology, name: &str, area: Rect, buf: &mut Buffer) {
    let d = topology.functions.get(name).unwrap();
    let pretty = to_colored_json_auto(&d).unwrap();
    render_json(&pretty, area, buf)
}

fn render_default(area: Rect, buf: &mut Buffer) {
    let d: HashMap<String, String> = HashMap::new();
    let pretty = to_colored_json_auto(&d).unwrap();
    render_json(&pretty, area, buf)
}

impl Widget for DetailWidget {

    fn render(self, area: Rect, buf: &mut Buffer) {

       if !self.name.is_empty() {
            match self.entity.as_ref() {
                "routes" => render_routes(&self.topology, &self.name, area, buf),
                "events" => render_events(&self.topology, &self.name, area, buf),
                "functions" => render_functions(&self.topology, &self.name, area, buf),
                _ => render_default(area, buf),
            }
        } else {
           render_default(area, buf)
        };

    }
}
