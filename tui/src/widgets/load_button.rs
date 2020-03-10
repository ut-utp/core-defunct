//! A button that loads the current program file.

use super::widget_impl_support::*;

use lc3_traits::control::load::{load_whole_memory_dump, Progress};

// No block!
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoadButton {
    area: Option<Rect>,
}

impl LoadButton {
    pub fn new() -> Self {
        Self {
            area: None,
        }
    }
}

impl TuiWidget for LoadButton {
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        unimplemented!("Don't call this! We need TuiData to draw!")
    }
}

impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for LoadButton
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
{
    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        self.area = Some(area);

        match &data.program_path {
            None => {
                let msg = TuiText::styled("No Program File!\n", Style::default().fg(Colour::Red));

                Paragraph::new([msg].iter())
                    .style(Style::default().fg(Colour::White))
                    .alignment(Alignment::Center)
                    .wrap(true)
                    .draw(area, buf)
            },

            Some(p) => {
                let file_name = p.file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("<unprintable>");

                let msg1 = TuiText::styled("Load Program\n", Style::default().fg(Colour::Cyan));
                let msg2 = TuiText::styled(format!("(from: `{}`)", file_name), Style::default().fg(Colour::Gray));

                Paragraph::new([msg1, msg2].iter())
                    .style(Style::default().fg(Colour::White))
                    .alignment(Alignment::Center)
                    .wrap(true)
                    .draw(area, buf)
            },
        }
    }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, terminal: &mut Terminal<B>) -> bool {
        use WidgetEvent::*;

        match event {
            Focus(FocusEvent::GotFocus) => data.program_path.is_some(),
            Focus(FocusEvent::LostFocus) => false,
            Mouse(MouseEvent::Up(_, _, _, _)) => true,

            // TODO: make this a function! stomach the trait bounds
            Mouse(MouseEvent::Down(_, _, _, _)) => {
                match data.program_path {
                    Some(ref p) => {
                        let path = format!("{}", p.display());
                        if p.exists() {
                            match lc3_shims::memory::FileBackedMemoryShim::from_existing_file(p) {
                                Ok(mem) => {
                                    // TODO: scoped thread to display progress!
                                    let p = Progress::new();

                                    match load_whole_memory_dump(data.sim, &mem.into(), Some(&p)) {
                                        Ok(()) => data.log(format!("[Load] Successful Load (`{}`)!\n", path), Colour::Green),
                                        Err(e) => data.log(format!("[Load] Error during load: {:?}\n", e), Colour::Red),

                                    }
                                }
                                Err(e) => {
                                    data.log(format!("[Load] Failed to load `{}` as a MemoryDump; got: {:?}\n", path, e), Colour::Red)
                                }
                            }
                        } else {
                            data.log(format!("[Load] `{}` does not exist!\n", path), Colour::Red)
                        }

                        true
                    },
                    None => false,
                }
            }
            _ => false,
        }
    }
}