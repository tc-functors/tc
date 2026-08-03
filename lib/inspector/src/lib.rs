use core::time::Duration;
use std::time::Instant;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind};
use ratatui::layout::{Position, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Scrollbar, ScrollbarOrientation};
use ratatui::{Frame, Terminal, crossterm};
use tui_tree_widget::{Tree, TreeItem, TreeState};
use composer::Topology;

mod color;
mod detail;
mod tree;

use detail::Detail;

#[must_use]
struct App<'a> {
    namespace: String,
    topology: Topology,
    state: TreeState<&'a str>,
    items: Vec<TreeItem<'a, &'a str>>,
}

impl<'a> App<'a> {
    fn new(topology: &'a Topology) -> Self {

        Self {
            namespace: topology.namespace.clone(),
            topology: topology.clone(),
            state: TreeState::default(),
            items: vec![
                tree::make_events(topology),
                tree::make_routes(topology),
                tree::make_functions(topology),
                tree::make_mutations(topology),
                tree::make_pages(topology),
                tree::make_channels(topology),
                tree::make_queues(topology),
                tree::make_roles(topology),
            ]

        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(40),
                Constraint::Percentage(60)
            ])
            .split(area);


        let sidebar = Tree::new(&self.items)
            .expect("all item identifiers are unique")
            .block(
                Block::bordered()
                    .title(self.namespace.clone())
            )
            .experimental_scrollbar(Some(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .track_symbol(None)
                    .end_symbol(None),
            ))
            .highlight_style(
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("");

        let selected = self.state.selected();
        let entity = selected.into_iter().nth(0).unwrap_or(&"").to_string();
        let name = selected.into_iter().nth(1).unwrap_or(&"").to_string();
        let component = selected.into_iter().nth(2).unwrap_or(&"").to_string();


        let detail = Detail {
            topology: self.topology.clone(),
            entity: entity.clone(),
            name: name,
            component: component
        };

        frame.render_stateful_widget(sidebar, layout[0], &mut self.state);
        detail.render(frame, layout[1]);
    }
}

fn run_app<B>(
    terminal: &mut Terminal<B>,
    mut app: App
) -> Result<(), B::Error>
where
    B: Backend,
    B::Error: From<std::io::Error>,
{
    const DEBOUNCE: Duration = Duration::from_millis(20); // 50 FPS

    terminal.draw(|frame| app.draw(frame))?;

    let mut debounce: Option<Instant> = None;

    loop {
        let timeout = debounce.map_or(DEBOUNCE, |start| DEBOUNCE.saturating_sub(start.elapsed()));
        if crossterm::event::poll(timeout)? {
            let update = match crossterm::event::read()? {
                Event::Key(key) if !matches!(key.kind, KeyEventKind::Press) => false,
                Event::Key(key) => match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('\n' | ' ') => app.state.toggle_selected(),
                    KeyCode::Left => app.state.key_left(),
                    KeyCode::Right => app.state.key_right(),
                    KeyCode::Down => app.state.key_down(),
                    KeyCode::Up => app.state.key_up(),
                    KeyCode::Esc => app.state.select(Vec::new()),
                    KeyCode::Home => app.state.select_first(),
                    KeyCode::End => app.state.select_last(),
                    KeyCode::PageDown => app.state.scroll_down(3),
                    KeyCode::PageUp => app.state.scroll_up(3),
                    _ => false,
                },
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollDown => app.state.scroll_down(1),
                    MouseEventKind::ScrollUp => app.state.scroll_up(1),
                    MouseEventKind::Down(_button) => {
                        app.state.click_at(Position::new(mouse.column, mouse.row))
                    }
                    _ => false,
                },
                Event::Resize(_, _) => true,
                _ => false,
            };
            if update {
                debounce.get_or_insert_with(Instant::now);
            }
        }
        if debounce.is_some_and(|debounce| debounce.elapsed() > DEBOUNCE) {
            terminal.draw(|frame| {
                app.draw(frame);

                // Performance info in top right corner
                {

                    // frame.render_widget(
                    //     Span::styled(text, Style::new().fg(Color::Black).bg(Color::Gray)),
                    //     area,
                    // );
                }
            })?;

            debounce = None;
        }
    }
}

pub fn run(topology: &Topology) -> std::io::Result<()> {
    // Terminal initialization
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    // App
    let app = App::new(topology);
    let res = run_app(&mut terminal, app);

    // restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}
