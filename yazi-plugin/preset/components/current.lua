Current = {
	_id = "current",
	_dragging = false,
	_dropping = false,
}

function Current:new(area, tab)
	local me = setmetatable({
		_area = area,
		_tab = tab,
		_folder = tab.current,
	}, { __index = self })
	me:layout()
	return me
end

function Current:layout()
	if self._tab.pref.grid then
		self._grid = true
		self._cell_w = math.max(1, math.min(self._area.w, rt.mgr.grid_width))
		self._columns = self._area.w == 0 and 0 or math.max(1, math.floor(self._area.w / self._cell_w))
		self._cell_w = self._columns == 0 and 0 or math.max(1, math.floor(self._area.w / self._columns))

		self._cell_h = math.max(1, math.min(self._area.h, rt.mgr.grid_height))
		self._rows = self._area.h == 0 and 0 or math.max(1, math.floor(self._area.h / self._cell_h))
	else
		self._grid = false
		self._rows = self._area.h
		self._columns = 1
		self._cell_w = self._area.w
		self._cell_h = 1
	end
	self._limit = self._rows * self._columns
end

function Current:empty()
	local s
	if self._folder.files.filter then
		s = "No filter results"
	else
		local done, err = self._folder.stage()
		s = not done and "Loading..." or not err and "No items" or string.format("Error: %s", err)
	end

	return {
		ui.Text(s):area(self._area):align(ui.Align.CENTER):wrap(ui.Wrap.YES),
	}
end

function Current:dropping()
	if Current._dropping then
		return Tip:new(self._area, "Drop to move here…"):redraw()
	else
		return {}
	end
end

function Current:reflow() return { self } end

function Current:window()
	local start = self._folder.offset
	if self._grid and self._columns > 1 then
		start = start - start % self._columns
	end

	local files, start = {}, start + 1
	for i = 0, self._limit - 1 do
		local f = self._folder.files[start + i]
		if not f then
			break
		end
		files[#files + 1] = f
	end
	return files
end

function Current:redraw()
	local files = self:window()
	if #files == 0 then
		return self:empty()
	end
	if self._grid then
		return self:redraw_grid(files)
	end

	local left, right = {}, {}
	for _, f in ipairs(files) do
		local entity = Entity:new(f)
		left[#left + 1], right[#right + 1] = entity:redraw(), Linemode:new(f):redraw()

		local max = math.max(0, self._area.w - right[#right]:width())
		left[#left]:truncate { max = max, ellipsis = entity:ellipsis(max) }
	end

	return {
		ui.List(left):area(self._area),
		ui.Text(right):area(self._area):align(ui.Align.RIGHT),
		table.unpack(self:dropping()),
	}
end

function Current:grid_span(f, line)
	local style = Entity:new(f):style()
	local text = ""
	if line == math.max(1, math.floor(self._cell_h / 2)) then
		local icon = th.icon:match(f, { hovered = f.is_hovered })
		text = icon and icon.text or ""
	elseif line == self._cell_h then
		local prefix = f:prefix() or ""
		text = (prefix ~= "" and prefix .. "/" or "") .. ui.printable(f.name or "")

		local marked = Marker:style(f)
		if marked then
			text = th.mgr.marker_symbol .. " " .. text
		end
	end

	text = ui.truncate(text, { max = self._cell_w })
	local width = ui.width(text)
	local left = math.floor((self._cell_w - width) / 2)
	local right = self._cell_w - width - left
	return ui.Span(string.rep(" ", left) .. text .. string.rep(" ", right)):style(style)
end

function Current:redraw_grid(files)
	local lines = {}
	for r = 1, self._rows do
		for line = 1, self._cell_h do
			local spans = {}
			for c = 1, self._columns do
				local f = files[(r - 1) * self._columns + c]
				spans[#spans + 1] = f and self:grid_span(f, line) or string.rep(" ", self._cell_w)
			end
			lines[#lines + 1] = ui.Line(spans)
		end
	end

	return {
		ui.List(lines):area(self._area),
		table.unpack(self:dropping()),
	}
end

-- Mouse events
function Current:click(event, up)
	if up or event.is_middle then
		return
	end

	local idx
	if self._grid then
		if self._cell_w == 0 or self._cell_h == 0 then
			return
		end

		local x = math.floor((event.x - self._area.x) / self._cell_w)
		local y = math.floor((event.y - self._area.y) / self._cell_h)
		idx = y * self._columns + x + 1
	else
		idx = event.y - self._area.y + 1
	end

	local file = self:window()[idx]
	if file then
		Entity:new(file):click(event, up)
	end
end

function Current:scroll(event, step)
	if self._grid then
		step = step * self._columns
	end
	ya.emit("arrow", { step })
end

function Current:touch(event, step) end

function Current:drag(event)
	if event.type == "offer" then
		Current._dragging = require("dnd").offer_uri_list()
	elseif event.type == "end" or event.type == "error" then
		Current._dragging = false
	end
end

function Current:drop(event)
	if Current._dragging then
		return
	elseif event.type == "enter" then
		rt.tty:queue("AgreeDrop", { type = "move", mimes = { "text/uri-list" } })
	elseif event.type == "ready" then
		rt.tty:queue("StartDrop", { idx = 1 })
	elseif event.type == "arrive" then
		rt.tty:queue("FinishDrop", { type = "move" })
		require("dnd").cut_uri_list(event.data)
	end
	rt.tty:flush()

	local d = event.type == "enter"
	if Current._dropping ~= d then
		Current._dropping = d
		ui.render()
	end
end
