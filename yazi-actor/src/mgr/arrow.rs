use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::ArrowForm;
use yazi_shared::data::Data;
use yazi_widgets::Step;

use crate::{Actor, Ctx};

pub struct Arrow;

impl Actor for Arrow {
	type Form = ArrowForm;

	const NAME: &str = "arrow";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let tab = cx.tab_mut();
		let step = if tab.pref.grid {
			match form.step {
				Step::Prev => Step::Offset(-(yazi_config::LAYOUT.get().folder_columns() as isize)),
				Step::Next => Step::Offset(yazi_config::LAYOUT.get().folder_columns() as isize),
				step => step,
			}
		} else {
			form.step
		};

		let old = tab.current.cursor;
		if !tab.current.arrow(step) {
			succ!();
		}

		// Retrace
		tab.current.retrace();

		// Visual selection
		if let Some(visual) = tab.mode.visual_mut() {
			visual.arrow(step, old, tab.current.cursor);
		}

		act!(mgr:hover, cx)?;
		act!(mgr:peek, cx)?;
		act!(mgr:watch, cx)?;

		cx.tasks.scheduler.behavior.reset();
		succ!(render!());
	}
}
