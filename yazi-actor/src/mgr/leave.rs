use anyhow::Result;
use yazi_core::mgr::CdSource;
use yazi_macro::{act, succ};
use yazi_parser::VoidForm;
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx};

pub struct Leave;

impl Actor for Leave {
	type Form = VoidForm;

	const NAME: &str = "leave";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		if cx.source().is_key() && cx.tab().pref.grid {
			let layout = yazi_config::LAYOUT.get();
			let columns = layout.folder_columns();
			let start = cx.tab().current.offset - cx.tab().current.offset % columns;
			let index = cx.tab().current.cursor.saturating_sub(start);
			if columns > 1 && index % columns != 0 {
				return act!(mgr:arrow, cx, -1);
			}
		}

		let url = cx
			.hovered()
			.and_then(|h| h.url.parent())
			.filter(|u| u != cx.cwd())
			.or_else(|| cx.cwd().parent());

		let Some(mut url) = url else { succ!() };
		if url.is_search() {
			url = url.as_regular()?;
		}

		act!(mgr:cd, cx, (url, CdSource::Leave))
	}
}
