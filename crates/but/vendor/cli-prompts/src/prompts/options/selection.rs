use super::multioption_prompt::MultiOptionPrompt;
use crate::{
    engine::CommandBuffer,
    input::Key,
    prompts::{options::Options, AbortReason, EventOutcome, Prompt},
    style::SelectionStyle,
};

const DEFAULT_OPTIONS_COUNT: u16 = 5;

/// Prompt that allows to select one option from the given list.
/// Supports filtering and moving the selection with arrow keys.
///
/// ```rust,no_run
/// use cli_prompts::{
///      prompts::{Selection, AbortReason},
///      DisplayPrompt,
/// };
///
/// fn main() {
///     let social_media = [
///         "Facebook",
///         "Instagram",
///         "Twitter",
///         "Snapchat"
///     ];
///
///     let prompt = Selection::new("Where you want to post?", social_media.into_iter())
///                     .displayed_options_count(3);
///     let selection : Result<&str, AbortReason> = prompt.display();
///     match selection {
///         Ok(media) => println!("Posting to {}", media),
///         Err(abort_reason) => println!("Prompt is aborted because of {:?}", abort_reason),
///     }
/// }
/// ```
pub struct Selection<T> {
    label: String,
    options: Options<T>,
    current_selection: usize,
    max_options: u16,
    current_filter: String,
    is_submitted: bool,
    style: SelectionStyle,
}

impl<T> Selection<T> {
    /// Create new prompt with the given label and the iterator over a type that is convertable to
    /// `String`
    pub fn new<S, I>(label: S, options: I) -> Self
    where
        T: Into<String> + Clone,
        S: Into<String>,
        I: Iterator<Item = T>,
    {
        let options = Options::from_iter(options);
        Self::new_internal(label.into(), options)
    }
}

impl<T> Selection<T> {
    /// Create new prompt with the given label and a transformation function that will convert the
    /// iterator items to strings
    pub fn new_with_transformation<S, I, F>(label: S, options: I, transformation: F) -> Self
    where
        S: Into<String>,
        I: Iterator<Item = T>,
        F: Fn(&T) -> String,
    {
        let options = Options::from_iter_transformed(options, transformation);
        Self::new_internal(label.into(), options)
    }

    /// Set maximum number of options that can be displayed on the screen
    pub fn displayed_options_count(mut self, options_count: u16) -> Self {
        self.max_options = options_count;
        self
    }

    /// Set the prompt style
    pub fn style(mut self, style: SelectionStyle) -> Self {
        self.style = style;
        self
    }

    fn new_internal(label: String, options: Options<T>) -> Self {
        Selection {
            label,
            options,
            current_selection: 0_usize,
            max_options: DEFAULT_OPTIONS_COUNT,
            current_filter: String::new(),
            is_submitted: false,
            style: SelectionStyle::default(),
        }
    }

    fn last_filtered_index(&self) -> usize {
        self.options.filtered_options().len().saturating_sub(1)
    }

    fn move_selection_up(&mut self) {
        self.current_selection = self.current_selection.saturating_sub(1);
    }

    fn move_selection_down(&mut self) {
        self.current_selection = self.current_selection.saturating_add(1);
        self.current_selection = self.current_selection.min(self.last_filtered_index());
    }

    fn move_selection_to_start(&mut self) {
        self.current_selection = 0;
    }

    fn move_selection_to_end(&mut self) {
        self.current_selection = self.last_filtered_index();
    }
}

impl<T> MultiOptionPrompt<T> for Selection<T> {
    fn max_options_count(&self) -> u16 {
        self.max_options
    }

    fn options(&self) -> &Options<T> {
        &self.options
    }

    fn currently_selected_index(&self) -> usize {
        self.current_selection
    }

    fn draw_option(
        &self,
        _: usize,
        option_label: &str,
        is_selected: bool,
        cmd_buffer: &mut impl CommandBuffer,
    ) {
        if is_selected {
            self.style.selected_marker.print(cmd_buffer);
            self.style
                .selected_option_formatting
                .print(option_label, cmd_buffer);
        } else {
            self.style.not_selected_marker.print(cmd_buffer);
            self.style.option_formatting.print(option_label, cmd_buffer)
        }
    }

    fn draw_header(&self, commands: &mut impl CommandBuffer, is_submitted: bool) {
        if is_submitted {
            let selected_option_index = self.options.filtered_options()[self.current_selection];
            let selected_option = &self.options.transformed_options()[selected_option_index];
            self.style
                .submitted_formatting
                .print(selected_option, commands);
        } else {
            self.style
                .filter_formatting
                .print(&self.current_filter, commands);
        }
    }
}

impl<T> Prompt<T> for Selection<T> {
    fn draw(&self, commands: &mut impl CommandBuffer) {
        self.draw_multioption(
            &self.label,
            self.is_submitted,
            &self.style.label_style,
            commands,
        )
    }

    fn on_key_pressed(&mut self, key: Key) -> EventOutcome<T> {
        match key {
            Key::Char(c) => {
                self.current_filter.push(c);
                self.options.filter(&self.current_filter);
                self.current_selection = 0;
                EventOutcome::Continue
            }
            Key::Backspace if !self.current_filter.is_empty() => {
                self.current_filter.pop();
                self.options.filter(&self.current_filter);
                self.current_selection = 0;
                EventOutcome::Continue
            }
            Key::Up | Key::Ctrl('p') | Key::Ctrl('P') => {
                self.move_selection_up();
                EventOutcome::Continue
            }
            Key::Down | Key::Ctrl('n') | Key::Ctrl('N') => {
                self.move_selection_down();
                EventOutcome::Continue
            }
            Key::Home | Key::Ctrl('a') | Key::Ctrl('A') => {
                self.move_selection_to_start();
                EventOutcome::Continue
            }
            Key::End | Key::Ctrl('e') | Key::Ctrl('E') => {
                self.move_selection_to_end();
                EventOutcome::Continue
            }
            Key::Enter if !self.options.filtered_options().is_empty() => {
                self.is_submitted = true;
                let selected_option_index = self.options.filtered_options()[self.current_selection];
                let result = self.options.all_options_mut().remove(selected_option_index);
                EventOutcome::Done(result)
            }
            Key::Esc => EventOutcome::Abort(AbortReason::Interrupt),
            _ => EventOutcome::Continue,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctrl_p_moves_selection_up() {
        let mut prompt = Selection::new("Pick one", ["a", "b", "c"].into_iter());
        prompt.current_selection = 2;

        assert!(matches!(
            prompt.on_key_pressed(Key::Ctrl('p')),
            EventOutcome::Continue
        ));
        assert_eq!(prompt.current_selection, 1);
    }

    #[test]
    fn ctrl_n_moves_selection_down() {
        let mut prompt = Selection::new("Pick one", ["a", "b", "c"].into_iter());
        prompt.current_selection = 0;

        assert!(matches!(
            prompt.on_key_pressed(Key::Ctrl('n')),
            EventOutcome::Continue
        ));
        assert_eq!(prompt.current_selection, 1);
    }

    #[test]
    fn ctrl_a_and_ctrl_e_jump_to_selection_bounds() {
        let mut prompt = Selection::new("Pick one", ["a", "b", "c"].into_iter());
        prompt.current_selection = 1;

        assert!(matches!(
            prompt.on_key_pressed(Key::Ctrl('e')),
            EventOutcome::Continue
        ));
        assert_eq!(prompt.current_selection, 2);

        assert!(matches!(
            prompt.on_key_pressed(Key::Ctrl('a')),
            EventOutcome::Continue
        ));
        assert_eq!(prompt.current_selection, 0);
    }

    #[test]
    fn ctrl_navigation_is_safe_when_filter_has_no_results() {
        let mut prompt = Selection::new("Pick one", ["a", "b", "c"].into_iter());
        prompt.on_key_pressed(Key::Char('z'));

        assert!(prompt.options.filtered_options().is_empty());
        assert!(matches!(
            prompt.on_key_pressed(Key::Ctrl('n')),
            EventOutcome::Continue
        ));
        assert_eq!(prompt.current_selection, 0);
    }
}
