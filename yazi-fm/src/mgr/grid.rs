use std::sync::{LazyLock, Mutex};

use ratatui_core::{buffer::Buffer, layout::Rect, widgets::Widget};
use tokio::task::JoinHandle;
use yazi_adapter::{ADAPTOR, ImageSpec};
use yazi_config::LAYOUT;
use yazi_core::Core;
use yazi_fs::FsUrl;
use yazi_shared::{Layer, url::UrlBuf};

static THUMBNAILS: LazyLock<Mutex<ThumbnailTask>> =
	LazyLock::new(|| Mutex::new(ThumbnailTask::default()));

#[derive(Clone, Eq, PartialEq)]
struct ThumbnailSnapshot {
	cwd: UrlBuf,
	specs: Vec<ImageSpec>,
}

#[derive(Default)]
struct ThumbnailTask {
	cwd: Option<UrlBuf>,
	applied: Option<Vec<ImageSpec>>,
	pending: Option<Vec<ImageSpec>>,
	handle: Option<JoinHandle<()>>,
}

impl ThumbnailTask {
	fn show(snapshot: ThumbnailSnapshot) {
		let mut task = THUMBNAILS.lock().unwrap();
		if task.cwd.as_ref() != Some(&snapshot.cwd) {
			task.handle.take().map(|h| h.abort());
			task.cwd = Some(snapshot.cwd.clone());
			task.applied = None;
			task.pending = None;
			ADAPTOR.image_hide_thumbnails().ok();
		}

		if task.pending.as_ref() == Some(&snapshot.specs)
			|| (task.applied.as_ref() == Some(&snapshot.specs) && task.handle.is_some())
		{
			return;
		}

		task.pending = Some(snapshot.specs);
		if task.handle.is_none() {
			task.handle = Some(tokio::spawn(Self::run(snapshot.cwd)));
		}
	}

	async fn run(cwd: UrlBuf) {
		loop {
			let (specs, refresh) = {
				let mut task = THUMBNAILS.lock().unwrap();
				if task.cwd.as_ref() != Some(&cwd) {
					return;
				}

				match task.pending.take() {
					Some(specs) => {
						let refresh = task.applied.as_ref() == Some(&specs);
						(specs, refresh)
					}
					None => {
						task.handle = None;
						return;
					}
				}
			};

			ADAPTOR.image_show_many(specs.clone()).await.ok();
			if refresh {
				ADAPTOR.image_refresh_many(specs.clone()).await.ok();
			}

			let mut task = THUMBNAILS.lock().unwrap();
			if task.cwd.as_ref() != Some(&cwd) {
				return;
			}
			task.applied = Some(specs);
		}
	}

	fn stop() {
		let mut task = THUMBNAILS.lock().unwrap();
		task.cwd = None;
		task.applied = None;
		task.pending = None;
		task.handle.take().map(|h| h.abort());
		ADAPTOR.image_hide_thumbnails().ok();
	}
}

pub(crate) struct Grid<'a> {
	core: &'a Core,
}

impl<'a> Grid<'a> {
	#[inline]
	pub(crate) fn new(core: &'a Core) -> Self {
		Self { core }
	}

	fn snapshot(&self) -> ThumbnailSnapshot {
		let layout = LAYOUT.get();
		let tab = self.core.active();
		let folder = &tab.current;

		let columns = layout.folder_columns();
		let rows = layout.folder_rows();
		if columns == 0 || rows == 0 {
			return ThumbnailSnapshot { cwd: folder.url.clone(), specs: Vec::new() };
		}

		let area = layout.current;
		let cell_w = layout.folder_cell_width();
		let cell_h = layout.folder_cell_height();
		let start = folder.offset - folder.offset % columns;

		let mut specs = Vec::with_capacity(columns * rows);
		for i in 0..columns * rows {
			let Some(file) = folder.entries.get(start + i) else {
				break;
			};
			if file.is_dir() {
				continue;
			}
			if !self.core.mgr.mimetype.get(&file.url).is_some_and(|m| m.starts_with("image/")) {
				continue;
			}

			let column = i % columns;
			let row = i / columns;
			let x = area.x + column as u16 * cell_w;
			let y = area.y + row as u16 * cell_h;
			if x >= area.right() || y >= area.bottom() {
				continue;
			}

			let width =
				if column + 1 == columns { area.right() - x } else { cell_w.min(area.right() - x) };
			let height = cell_h.min(area.bottom() - y).saturating_sub(1);
			if width == 0 || height == 0 {
				continue;
			}

			let inset = u16::from(width > 2);
			let max = Rect { x: x + inset, y, width: width.saturating_sub(inset * 2), height };
			if max.width == 0 || max.height == 0 {
				continue;
			}

			specs.push(ImageSpec {
				id: i as u32 + 1,
				path: file.url.clone().unified_path().into_owned(),
				max,
			});
		}
		ThumbnailSnapshot { cwd: folder.url.clone(), specs }
	}
}

impl Widget for Grid<'_> {
	fn render(self, _: Rect, _: &mut Buffer) {
		if !self.core.active().pref.grid || self.core.layer() != Layer::Mgr {
			ThumbnailTask::stop();
			return;
		}

		ThumbnailTask::show(self.snapshot());
	}
}
