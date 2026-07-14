use std::path::PathBuf;

use anyhow::Result;
use ratatui_core::layout::Rect;
use strum::{Display, IntoStaticStr};

use crate::drivers::{Chafa, Iip, Kgp, KgpOld, Sixel, Ueberzug};

#[derive(Clone, Copy, Debug, Display, Eq, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum Driver {
	Kgp,
	KgpOld,
	Iip,
	Sixel,

	// Supported by Überzug++
	X11,
	Wayland,
	Chafa,
}

impl Driver {
	pub async fn image_show<P>(self, id: u32, path: P, max: Rect) -> Result<Rect>
	where
		P: Into<PathBuf>,
	{
		if max.is_empty() {
			return Ok(Rect::default());
		}

		let path = path.into();
		match self {
			Self::Kgp => Kgp::image_show(id, path, max).await,
			Self::KgpOld => KgpOld::image_show(id, path, max).await,
			Self::Iip => Iip::image_show(id, path, max).await,
			Self::Sixel => Sixel::image_show(id, path, max).await,
			Self::X11 | Self::Wayland => Ueberzug::image_show(id, path, max).await,
			Self::Chafa => Chafa::image_show(id, path, max).await,
		}
	}

	pub fn image_erase(self, id: u32, area: Rect) -> Result<()> {
		match self {
			Self::Kgp => Kgp::image_erase(id, area),
			Self::KgpOld => KgpOld::image_erase(id, area),
			Self::Iip => Iip::image_erase(id, area),
			Self::Sixel => Sixel::image_erase(id, area),
			Self::X11 | Self::Wayland => Ueberzug::image_erase(id, area),
			Self::Chafa => Chafa::image_erase(id, area),
		}
	}

	pub async fn image_refresh<P>(self, id: u32, path: P, max: Rect, area: Rect) -> Result<()>
	where
		P: Into<PathBuf>,
	{
		match self {
			Self::Kgp => Kgp::image_refresh(id, area),
			Self::X11 | Self::Wayland => Ok(()),
			_ => self.image_show(id, path, max).await.map(|_| ()),
		}
	}

	pub fn needs_full_erase_for_update(self) -> bool {
		self == Self::KgpOld
	}

	pub(crate) fn start(self) {
		Ueberzug::start(self);
	}

	pub(crate) fn needs_ueberzug(self) -> bool {
		!matches!(self, Self::Kgp | Self::KgpOld | Self::Iip | Self::Sixel)
	}
}
