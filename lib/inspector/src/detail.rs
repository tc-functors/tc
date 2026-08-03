use std::collections::HashMap;
use serde_derive::{
    Deserialize,
    Serialize,
};
use composer::{Topology};
use composer::aws::mutation::Resolver;
use colored_json::to_colored_json_auto;
use ratatui::text::Span;
use ratatui::layout::Rect;
use ratatui::buffer::Buffer;
use ratatui::widgets::Widget;
use ratatui::Frame;
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

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Types {
    input: HashMap<String, String>,
    output: HashMap<String, String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Mutation {
    resolver: Resolver,
    types: Types,
}

pub struct Detail {
    pub topology: Topology,
    pub entity: String,
    pub name: String,
    pub component: String

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

                "pages" => {
                    let d = self.topology.pages.get(&self.name).unwrap();
                    let pretty = to_colored_json_auto(&d).unwrap();
                    let widget = JsonWidget { content: pretty };
                    frame.render_widget(widget, area);
                },

                "roles" => {
                    let d = self.topology.roles.get(&self.name).unwrap();
                    let pretty = to_colored_json_auto(&d).unwrap();
                    let widget = JsonWidget { content: pretty };
                    frame.render_widget(widget, area);
                },


                "queues" => {
                    let d = self.topology.queues.get(&self.name).unwrap();
                    let pretty = to_colored_json_auto(&d).unwrap();
                    let widget = JsonWidget { content: pretty };
                    frame.render_widget(widget, area);
                },

                "channels" => {
                    let d = self.topology.channels.get(&self.name).unwrap();
                    let pretty = to_colored_json_auto(&d).unwrap();
                    let widget = JsonWidget { content: pretty };
                    frame.render_widget(widget, area);
                },

                "mutations" => {
                    if let Some(m) = self.topology.mutations.get("default") {
                        let d = m.resolvers.get(&self.name);
                        let mutation = Mutation {
                            resolver: d.unwrap().clone(),
                            types: Types {
                                input: m.types_map.get(&d.unwrap().input).unwrap_or(&HashMap::new()).clone(),
                                output: m.types_map.get(&d.unwrap().output).unwrap_or(&HashMap::new()).clone()
                            }
                        };

                        let pretty = to_colored_json_auto(&mutation).unwrap();
                        let widget = JsonWidget { content: pretty };
                        frame.render_widget(widget, area);
                    }
                },

                "functions" => {
                    let f = self.topology.functions.get(&self.name).unwrap();
                    if !self.component.is_empty() {
                        let pretty = match self.component.as_ref() {
                            "build" => to_colored_json_auto(&f.build).unwrap(),
                            "runtime" => to_colored_json_auto(&f.runtime).unwrap(),
                            "environment" => to_colored_json_auto(&f.runtime.environment).unwrap(),
                            "role" => to_colored_json_auto(&f.runtime.role).unwrap(),
                            _ => to_colored_json_auto(&f).unwrap(),
                        };
                        let widget = JsonWidget { content: pretty };

                        frame.render_widget(widget, area);
                    }

                },

                _ => {
                    let d: HashMap<String, String> = HashMap::new();
                    let pretty = to_colored_json_auto(&d).unwrap();
                    let widget = JsonWidget { content: pretty };
                    frame.render_widget(widget, area);
                }
            }
        } else {
            match self.entity.as_ref() {
                "mutations" => {
                    let d: HashMap<String, String> = HashMap::new();
                    let pretty = to_colored_json_auto(&d).unwrap();
                    let widget = JsonWidget { content: pretty };
                    frame.render_widget(widget, area);
                },
                _ => ()

            }
        }
    }
}
