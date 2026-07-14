use anyhow::Result;
use mlua::{LuaString, Value};
use ratatui_core::layout::Position;
use tracing::error;
use yazi_actor::lives::Lives;
use yazi_config::LAYOUT;
use yazi_macro::{render, succ};
use yazi_parser::app::ReflowForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Reflow;

impl Actor for Reflow {
	type Form = ReflowForm;

	const NAME: &str = "reflow";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let Some(size) = cx.term.as_ref().and_then(|t| t.size().ok()) else { succ!() };
		let mut layout = LAYOUT.get();

		let result = Lives::scope(cx.core, |_| {
			let comps = (form.reflow)((Position::ORIGIN, size).into())?;

			for v in comps.sequence_values::<Value>() {
				let Value::Table(t) = v? else {
					error!("`reflow()` must return a table of components");
					continue;
				};

				let id: LuaString = t.get("_id")?;
				match &*id.as_bytes() {
					b"current" => {
						let area = *t.raw_get::<yazi_binding::elements::Rect>("_area")?;
						layout.set_current(
							area,
							t.raw_get("_rows").unwrap_or(area.height),
							t.raw_get("_columns").unwrap_or(1),
							t.raw_get("_cell_w").unwrap_or(area.width),
							t.raw_get("_cell_h").unwrap_or(1),
						);
					}
					b"preview" => layout.preview = *t.raw_get::<yazi_binding::elements::Rect>("_area")?,
					b"progress" => layout.progress = *t.raw_get::<yazi_binding::elements::Rect>("_area")?,
					_ => {}
				}
			}
			Ok(())
		});

		if layout != LAYOUT.get() {
			LAYOUT.set(layout);
			render!();
		}

		if let Err(ref e) = result {
			error!("Failed to `reflow()` the `Root` component:\n{e}");
		}
		succ!();
	}
}
