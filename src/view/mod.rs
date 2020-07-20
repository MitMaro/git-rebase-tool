pub mod line_segment;
pub mod scroll_position;
pub mod view_data;
pub mod view_line;

use crate::constants::{
	MINIMUM_COMPACT_WINDOW_WIDTH,
	MINIMUM_WINDOW_HEIGHT,
	TITLE,
	TITLE_HELP_INDICATOR_LENGTH,
	TITLE_LENGTH,
	TITLE_SHORT,
	TITLE_SHORT_LENGTH,
};
use crate::display::display_color::DisplayColor;
use crate::display::Display;
use crate::view::view_data::ViewData;
use crate::view::view_line::ViewLine;
use crate::Config;

pub struct View<'v> {
	config: &'v Config,
	display: &'v Display<'v>,
}

impl<'v> View<'v> {
	pub(crate) const fn new(display: &'v Display<'_>, config: &'v Config) -> Self {
		Self { display, config }
	}

	pub(crate) fn draw_str(&self, s: &str) {
		self.display.draw_str(s);
	}

	pub(crate) fn set_color(&self, color: DisplayColor, selected: bool) {
		self.display.color(color, selected);
	}

	pub(crate) fn set_style(&self, dim: bool, underline: bool, reverse: bool) {
		self.display.set_style(dim, underline, reverse);
	}

	pub(crate) fn check_window_size(&self) -> bool {
		let (window_width, window_height) = self.get_view_size();
		!(window_width <= MINIMUM_COMPACT_WINDOW_WIDTH || window_height <= MINIMUM_WINDOW_HEIGHT)
	}

	pub(crate) fn draw_error(&self, message: &str) {
		self.draw_title(false);
		self.display.color(DisplayColor::Normal, false);
		self.display.set_style(false, false, false);
		self.display.draw_str(message);
		self.display.draw_str("\n");
		self.display.color(DisplayColor::IndicatorColor, false);
		self.display.draw_str("Press any key to continue");
	}

	pub(crate) fn clear(&self) {
		self.display.clear();
	}

	pub(crate) fn get_view_size(&self) -> (usize, usize) {
		self.display.get_window_size()
	}

	pub(crate) fn refresh(&self) {
		self.display.refresh();
	}

	pub(crate) fn draw_view_data(&self, view_data: &ViewData) {
		let (_, window_height) = self.display.get_window_size();

		let mut line_index = 0;

		let leading_lines = view_data.get_leading_lines();
		let lines = view_data.get_lines();
		let trailing_lines = view_data.get_trailing_lines();

		let view_height = window_height - leading_lines.len() - trailing_lines.len();

		let show_scroll_bar = view_data.should_show_scroll_bar();
		let scroll_indicator_index = view_data.get_scroll_index();

		if view_data.show_title() {
			self.display.ensure_at_line_start(line_index);
			line_index += 1;
			self.draw_title(view_data.show_help());
		}

		for line in leading_lines {
			self.display.ensure_at_line_start(line_index);
			line_index += 1;
			self.draw_view_line(line);
		}

		for (index, line) in lines.iter().enumerate() {
			self.display.ensure_at_line_start(line_index);
			self.draw_view_line(line);
			if show_scroll_bar {
				self.display.ensure_at_line_start(line_index);
				self.display.move_from_end_of_line(1);
				self.display.color(DisplayColor::Normal, false);
				self.display.set_style(scroll_indicator_index != index, false, true);
				self.display.draw_str(" ");
			}
			self.display.color(DisplayColor::Normal, false);
			self.display.set_style(false, false, false);
			line_index += 1;
		}

		if view_height > lines.len() {
			self.display.color(DisplayColor::Normal, false);
			self.display.set_style(false, false, false);
			let draw_height = view_height - lines.len() - if view_data.show_title() { 1 } else { 0 };
			self.display.ensure_at_line_start(line_index);
			for _x in 0..draw_height {
				line_index += 1;
				self.display
					.draw_str(format!("{}\n", self.config.theme.character_vertical_spacing).as_str());
			}
		}

		for line in trailing_lines {
			self.display.ensure_at_line_start(line_index);
			line_index += 1;
			self.draw_view_line(line)
		}
	}

	fn draw_view_line(&self, line: &ViewLine) {
		for segment in line.get_segments() {
			self.display.color(segment.get_color(), line.get_selected());
			self.display
				.set_style(segment.is_dimmed(), segment.is_underlined(), segment.is_reversed());
			self.display.draw_str(segment.get_content());
		}

		// reset style
		self.display.color(DisplayColor::Normal, false);
		self.display.set_style(false, false, false);
		self.display.fill_end_of_line();
	}

	pub(crate) fn draw_title(&self, show_help: bool) {
		self.display.color(DisplayColor::Normal, false);
		self.display.set_style(false, true, false);
		let (window_width, _) = self.display.get_window_size();

		let title_help_indicator_total_length = TITLE_HELP_INDICATOR_LENGTH + self.config.key_bindings.help.len();

		if window_width >= TITLE_LENGTH {
			self.display.draw_str(TITLE);
			// only draw help if there is room
			if window_width > TITLE_LENGTH + title_help_indicator_total_length {
				if (window_width - TITLE_LENGTH - title_help_indicator_total_length) > 0 {
					let padding = " ".repeat(window_width - TITLE_LENGTH - title_help_indicator_total_length);
					self.display.draw_str(padding.as_str());
				}
				if show_help {
					self.display
						.draw_str(format!("Help: {}", self.config.key_bindings.help).as_str());
				}
				else {
					let padding = " ".repeat(title_help_indicator_total_length);
					self.display.draw_str(padding.as_str());
				}
			}
			else if (window_width - TITLE_LENGTH) > 0 {
				let padding = " ".repeat(window_width - TITLE_LENGTH);
				self.display.draw_str(padding.as_str());
			}
		}
		else {
			self.display.draw_str(TITLE_SHORT);
			if (window_width - TITLE_SHORT_LENGTH) > 0 {
				let padding = " ".repeat(window_width - TITLE_SHORT_LENGTH);
				self.display.draw_str(padding.as_str());
			}
		}

		// reset style
		self.display.color(DisplayColor::Normal, false);
		self.display.set_style(false, false, false);
	}

	fn draw_prompt(&self, message: &str) {
		self.draw_title(false);
		self.display.set_style(false, false, false);
		self.display.draw_str(&format!("\n{} ", message));
	}

	pub(crate) fn draw_confirm(&self, message: &str) {
		self.draw_prompt(&format!(
			"{} ({}/{})? ",
			message, self.config.key_bindings.confirm_yes, self.config.key_bindings.confirm_no
		));
	}
}
