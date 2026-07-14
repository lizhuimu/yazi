use ratatui_core::layout::Rect;

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct Layout {
	pub current: Rect,
	pub preview: Rect,
	pub progress: Rect,
	pub current_rows: u16,
	pub current_columns: u16,
	pub current_cell_width: u16,
	pub current_cell_height: u16,
}

impl Layout {
	pub const fn default() -> Self {
		Self {
			current: Rect::ZERO,
			preview: Rect::ZERO,
			progress: Rect::ZERO,
			current_rows: 0,
			current_columns: 1,
			current_cell_width: 0,
			current_cell_height: 1,
		}
	}

	pub fn set_current(
		&mut self,
		current: Rect,
		rows: u16,
		columns: u16,
		cell_width: u16,
		cell_height: u16,
	) {
		self.current = current;
		self.current_rows = rows;
		self.current_columns = columns.max(1);
		self.current_cell_width = cell_width;
		self.current_cell_height = cell_height.max(1);
	}

	pub const fn folder_rows(self) -> usize {
		if self.current_rows == 0 { self.current.height as _ } else { self.current_rows as _ }
	}

	pub const fn folder_columns(self) -> usize {
		self.current_columns as usize
	}

	pub const fn folder_cell_width(self) -> u16 {
		if self.current_cell_width == 0 { self.current.width } else { self.current_cell_width }
	}

	pub const fn folder_cell_height(self) -> u16 {
		self.current_cell_height
	}

	pub const fn folder_limit(self) -> usize {
		self.folder_rows() * self.folder_columns()
	}
}
