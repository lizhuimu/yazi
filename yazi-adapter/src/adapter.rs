use std::{
	fmt::{self, Debug},
	ops::Deref,
	path::PathBuf,
	sync::Mutex,
};

use anyhow::Result;
use ratatui_core::layout::Rect;
use yazi_emulator::EMULATOR;
use yazi_widgets::clear::ClearInventory;

use crate::{
	ADAPTOR,
	drivers::{Driver, Drivers},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImageSpec {
	pub id: u32,
	pub path: PathBuf,
	pub max: Rect,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ShownImage {
	id: u32,
	path: PathBuf,
	max: Rect,
	area: Rect,
}

pub struct Adapter {
	driver: Driver,
	shown: Mutex<Vec<ShownImage>>,
	pub collision: yazi_shim::cell::SyncCell<bool>,
}

impl Deref for Adapter {
	type Target = Driver;

	fn deref(&self) -> &Self::Target {
		&self.driver
	}
}

impl Debug for Adapter {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.driver.fmt(f)
	}
}

impl Adapter {
	pub(super) fn new() -> Self {
		Self {
			driver: Drivers::matches(&EMULATOR),
			shown: Mutex::default(),
			collision: yazi_shim::cell::SyncCell::new(false),
		}
	}

	pub async fn image_show<P>(&self, path: P, max: Rect) -> Result<Rect>
	where
		P: Into<PathBuf>,
	{
		self.image_hide()?;

		let path = path.into();
		let area = self.driver.image_show(0, path.clone(), max).await?;
		self.store_shown(ShownImage { id: 0, path, max, area });
		Ok(area)
	}

	pub async fn image_show_many(&self, desired: Vec<ImageSpec>) -> Result<()> {
		if self.driver.needs_full_erase_for_update() {
			self.image_hide()?;
			for spec in desired {
				let area = self.driver.image_show(spec.id, spec.path.clone(), spec.max).await?;
				self.store_shown(ShownImage { id: spec.id, path: spec.path, max: spec.max, area });
			}
			return Ok(());
		}

		let shown = self.shown.lock().unwrap().clone();

		let mut erase = Vec::new();
		for old in &shown {
			let keep =
				desired.iter().any(|new| new.id == old.id && new.path == old.path && new.max == old.max);
			if !keep {
				erase.push((old.id, old.area));
			}
		}

		let mut show = Vec::new();
		for new in &desired {
			let exists =
				shown.iter().any(|old| old.id == new.id && old.path == new.path && old.max == new.max);
			if !exists {
				show.push(new.clone());
			}
		}

		for (id, area) in erase {
			self.driver.image_erase(id, area)?;
			self.shown.lock().unwrap().retain(|image| image.id != id);
		}

		for spec in show {
			let area = self.driver.image_show(spec.id, spec.path.clone(), spec.max).await?;
			self.store_shown(ShownImage { id: spec.id, path: spec.path, max: spec.max, area });
		}

		Ok(())
	}

	pub async fn image_refresh_many(&self, desired: Vec<ImageSpec>) -> Result<()> {
		let shown = self.shown.lock().unwrap().clone();
		for spec in desired {
			let Some(image) = shown
				.iter()
				.find(|image| image.id == spec.id && image.path == spec.path && image.max == spec.max)
			else {
				continue;
			};

			self.driver.image_refresh(spec.id, spec.path, spec.max, image.area).await?;
		}
		Ok(())
	}

	pub fn image_hide(&self) -> Result<()> {
		for image in std::mem::take(&mut *self.shown.lock().unwrap()) {
			self.driver.image_erase(image.id, image.area)?;
		}
		Ok(())
	}

	pub fn image_hide_thumbnails(&self) -> Result<()> {
		let mut shown = self.shown.lock().unwrap();
		let mut keep = Vec::with_capacity(shown.len());
		for image in std::mem::take(&mut *shown) {
			if image.id == 0 {
				keep.push(image);
			} else {
				self.driver.image_erase(image.id, image.area)?;
			}
		}
		*shown = keep;
		Ok(())
	}

	fn clear(&self, area: Rect) -> Option<Rect> {
		let mut shown = self.shown.lock().unwrap();

		if self.driver.needs_full_erase_for_update() {
			if !shown.iter().any(|image| area.intersection(image.area).area() > 0) {
				return None;
			}

			let mut union = None;
			for image in std::mem::take(&mut *shown) {
				self.driver.image_erase(image.id, image.area).ok();
				union = Some(match union {
					Some(rect) => union_rect(rect, image.area),
					None => image.area,
				});
			}
			self.collision.set(true);
			return union;
		}

		let mut next = Vec::with_capacity(shown.len());
		let mut union = None;

		for image in std::mem::take(&mut *shown) {
			if area.intersection(image.area).area() == 0 {
				next.push(image);
				continue;
			}

			self.driver.image_erase(image.id, image.area).ok();
			self.collision.set(true);
			union = Some(match union {
				Some(rect) => union_rect(rect, image.area),
				None => image.area,
			});
		}

		*shown = next;
		union
	}

	fn store_shown(&self, image: ShownImage) {
		let mut shown = self.shown.lock().unwrap();
		shown.retain(|old| old.id != image.id);
		shown.push(image);
	}
}

fn union_rect(a: Rect, b: Rect) -> Rect {
	let left = a.left().min(b.left());
	let top = a.top().min(b.top());
	let right = a.right().max(b.right());
	let bottom = a.bottom().max(b.bottom());

	Rect { x: left, y: top, width: right - left, height: bottom - top }
}

inventory::submit! {
	ClearInventory {
		clear: |area| ADAPTOR.clear(area),
	}
}
