use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::GridForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Grid;

impl Actor for Grid {
	type Form = GridForm;

	const NAME: &str = "grid";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let tab = cx.tab_mut();
		let state = form.state.bool(tab.pref.grid);
		if state == tab.pref.grid {
			succ!();
		}

		tab.pref.grid = state;
		tab.current.arrow(0);
		act!(mgr:update_paged, cx)?;
		act!(mgr:peek, cx, true)?;

		succ!(render!());
	}
}
