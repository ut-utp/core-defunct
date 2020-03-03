//! A tabs widget owns its tabs that allows users to switch between the tabs.
//!
//! TODO!

// Note: the key bindings thing is not nesting friendly; that is only the top
// most Tabs widget will actually receive and handle them. This is fine.

// For now, this requires that the tabs be _static_ (i.e. can't add or remove
// tabs after creating the item) but this restriction can be lifted later if
// there's a use case for it.

use super::widget_impl_support::*;

use tui::widgets::Tabs as TabsBar;

pub struct Tabs<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    /// Template `TabsBar` instance that gives us styling and dividers and such.
    tabs_bar: TabsBar<'a, String>,
    /// The titles of the tabs.
    titles: Vec<String>,
    /// The actual tabs.
    tabs: Vec<Box<dyn Widget<'a, 'int, C, I, O, B> + 'a>>,
    /// Current tab.
    current_tab: usize,
}

impl<'a, 'int, C, I, O, B> Tabs<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    // This style of constructor is there to ensure that there's at least one
    // tab.
    pub fn new<W: Widget<'a, 'int, C, I, O, B> + 'a, S: ToString>(first_tab: W, title: S) -> Self {
        Self {
            tabs_bar: TabsBar::default(),
            titles: vec![title.to_string()],
            tabs: vec![Box::new(first_tab)],
            current_tab: 0,
        }
    }

    pub fn add<W: Widget<'a, 'int, C, I, O, B> + 'a, S: ToString>(mut self, tab: W, title: S) -> Self {
        self.tabs.push(Box::new(tab));
        self.titles.push(title.to_string());

        self
    }

    pub fn with_tabs_bar(mut self, bar: TabsBar<'a, String>) -> Self {
        self.tabs_bar = bar;
        self
    }

    // TODO: possibly make this configurable
    fn area_split(&self, area: Rect) -> (Rect, Rect) {
        if let [bar, rest] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(4), Constraint::Min(1)].as_ref())
            .split(area)
            [..] {
            return (bar, rest)
        } else {
            unreachable!()
        }
    }

    fn propagate(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>) -> bool {
        self.tabs[self.current_tab].update(event, data)
    }
}

impl<'a, 'int, C, I, O, B> TuiWidget for Tabs<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        // Shouldn't actually be called, but just in case:
        // (note: if there are Widgets within our tabs this will be A Problem)
        let (bar, rest) = self.area_split(area);

        self.tabs_bar.draw(bar, buf);
        TuiWidget::draw(&mut *self.tabs[self.current_tab], rest, buf);
    }
}

impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for Tabs<'a, 'int, C, I, O, B>
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, sim: &C, area: Rect, buf: &mut Buffer) {
        let (bar, rest) = self.area_split(area);

        self.tabs_bar
            .clone()
            .titles(self.titles.as_ref())
            .select(self.current_tab)
            .draw(bar, buf);

        Widget::draw(&mut *self.tabs[self.current_tab], sim, rest, buf)
    }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>) -> bool {
        use WidgetEvent::*;

        match event {
            Key(e) => match e {
                KeyEvent { code: KeyCode::Char(n @ '1'..='9'), modifiers: KeyModifiers::CONTROL } |
                KeyEvent { code: KeyCode::Char(n @ '1'..='9'), modifiers: KeyModifiers::ALT } => {
                    // Switch to 0 indexing:
                    let idx = n as usize - '1' as usize;

                    if idx < self.tabs.len() {
                        self.current_tab = idx;
                        true
                    } else {
                        false
                    }
                },
                _ => self.propagate(event, data),
            }

            // TODO: handle mouse events!

            _ => self.propagate(event, data)
        }
    }
}
