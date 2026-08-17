local function split_path(path)
	local result = {}
	for part in path:gmatch("[^/]+") do
		result[#result + 1] = part
	end
	return result
end

local function get_change_url()
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
			text = "Multiple Gerrit refs point at this revision; I broke state? You broke state.",
			error = true,
		})
		return
	end
	return refs[1]
end

local function curl_vote(change_id, patch_num, label, value)
	local url = string.format("https://gerrithub.io/a/changes/%s/revisions/%s/review", change_id, patch_num)

	local payload = string.format('{"labels":{"%s":%d}}', label, value)

	local cmd = string.format(
		[[curl -sS --fail-with-body -u "%s:%s" -X POST -H "Content-Type: application/json" --data '%s' "%s"]],
		os.getenv("GERRIT_USER"),
		os.getenv("GERRIT_HTTP_PASSWORD"),
		payload,
		url
	)

	local handle = assert(io.popen(cmd, "r"))
	local output = handle:read("*a")
	local ok, _, code = handle:close()

	if not ok then
		error(string.format("curl failed with exit code %s: %s", code, output))
	end

	return output
end

local function gerrit_vote(change, patch, label, vote)
	local ok, response = pcall(curl_vote, change, patch, label, 0)

	if not ok then
		flash({ text = response })
		return
	end

	local ok2, response2 = pcall(curl_vote, change, patch, label, vote)

	if not ok2 then
		flash({ text = response2 })
	end
end

---@diagnostic disable-next-line: lowercase-global
function setup(config)
	config.action("gerrit.hide", function()
		local ref = get_change_url()
		if ref == nil then
			return
		end

		local parts = split_path(ref)

		local tag = "hidden/" .. table.concat(parts, "/", 2)

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
		local _, tag_err = jj("tag", "set", tag, "-r", ref)
		revisions.refresh({ keep_selections = true })

		if tag_err then
			flash({ text = tag_err, error = true })
		else
			flash("Hidden " .. ref)
		end
	end, {
		seq = { "g", "h" },
		scope = "revisions",
		desc = "Hide Gerrit patchset",
	})

	config.action("gerrit.unhide", function()
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
		seq = { "g", "n" },
		scope = "revisions",
		desc = "Unhide Gerrit patchset",
	})

	config.action("gerrit-upload", function()
		local _, _ = jj("duplicate")
		local _, _ = jj("gerrit", "upload")
	end, {
		seq = { "g", "u" },
		scope = "revisions",
		desc = "Gerrit upload",
	})

	config.action("gerrit.build", function()
		local ref = get_change_url()
		if ref == nil then
			return
		end
		local parts = split_path(ref)
		gerrit_vote(parts[3], parts[4], "Agent", 2)
	end, {
		seq = { "g", "b" },
		scope = "revisions",
		desc = "Trigger Gerrit vote to start agent-build",
	})

	config.action("gerrit.submit", function()
		local ref = get_change_url()
		if ref == nil then
			return
		end
		local parts = split_path(ref)
		gerrit_vote(parts[3], parts[4], "Agent", 4)
	end, {
		seq = { "g", "s" },
		scope = "revisions",
		desc = "Trigger Gerrit vote to start agent-build",
	})
end
