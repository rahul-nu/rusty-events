---@diagnostic disable-next-line: lowercase-global
function setup(config)
	config.action("gerrit-hide", function()
		local change_id = context.change_id()

		if not change_id then
			flash({ text = "No revision selected", error = true })
			return
		end

		local output, err = jj("log", "--no-graph", "-r", 'tags("changes/**") & ' .. change_id, "-T", 'tags ++ "\n"')

		if err then
			flash({ text = err, error = true })
			return
		end

		local refs = {}
		if output then
			for ref in output:gmatch("[^\r\n]+") do
				table.insert(refs, ref)
			end
		end

		-- We require exactly one Gerrit patchset ref.
		if #refs == 0 then
			flash({
				text = "Selected revision is not a Gerrit patchset",
				error = true,
			})
			return
		end

		flash("Gerrit refs for " .. change_id .. " (" .. #refs .. "):\n" .. table.concat(refs, "\n"))

		if #refs > 1 then
			flash({
				text = "Multiple Gerrit refs point at this revision; refusing to hide",
				error = true,
			})
			return
		end

		local ref = refs[1]

		-- Convert:
		--   changes/58/1245558/7@origin
		-- into:
		--   hidden/58/1245558/7
		local suffix = ref:match("^changes/(.+)$")

		if not suffix then
			flash({
				text = "Unexpected Gerrit ref: " .. ref,
				error = true,
			})
			return
		end

		local tag = "hidden/" .. suffix

		-- Do not move an existing hide tag.
		local existing, existing_err = jj("log", "-r", 'tags("' .. tag .. '")', "-T", 'commit_id ++ "\n"')

		if existing_err then
			flash({ text = existing_err, error = true })
			return
		end

		if existing and existing:match("%S") then
			flash({
				text = "Already hidden: " .. tag,
			})
			return
		end

		-- Create the local hide tag.
		local _, tag_err = jj("tag", "set", tag, "-r", change_id)

		if tag_err then
			flash({ text = tag_err, error = true })
			return
		end

		revisions.refresh({
			keep_selections = true,
		})

		flash("Hidden " .. ref)
	end, {
		key = "H",
		scope = "revisions",
		desc = "Hide Gerrit patchset",
	})

	config.action("gerrit-unhide", function()
		local change_id = context.change_id()

		if not change_id then
			flash({ text = "No revision selected", error = true })
			return
		end

		local output, err = jj(
			"log",
			"--no-graph",
			"-r",
			'tags("hidden/*") & ' .. change_id,
			"-T",
			'tags.map(|t| "TAG=[" ++ t ++ "]\\n")'
		)

		if err then
			flash({ text = err, error = true })
			return
		end

		local tags = {}
		if output then
			for tag in output:gmatch("TAG=%[(hidden/[^]]+)%]") do
				flash({ text = tag })
				table.insert(tags, tag)
			end
		end

		if #tags == 0 then
			flash({
				text = "Selected revision is not hidden",
				error = true,
			})
			return
		end

		if #tags > 1 then
			flash({
				text = "Multiple hide tags found; refusing to unhide",
				error = true,
			})
			return
		end

		local tag = tags[1]

		local _, delete_err = jj("tag", "delete", tag)

		if delete_err then
			flash({ text = delete_err, error = true })
			return
		end
		revisions.refresh({
			keep_selections = true,
		})
		flash("Unhidden " .. tag)
	end, {
		key = "N",
		scope = "revisions",
		desc = "Unhide Gerrit patchset",
	})

	config.action("gerrit-upload", function()
		local _, _ = jj("duplicate")
		local _, _ = jj("gerrit", "upload")
	end, {
		key = "g",
		scope = "revisions",
		desc = "Gerrit upload",
	})
end
