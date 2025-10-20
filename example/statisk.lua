local statisk = require("statisk")

local M = {}

M.meta = {
	url = "https://example.org",
	title = "Example Site",
	description = "An example site for demonstration purposes",
	author = {
		name = "Ola Nordmann",
		description = "A fictional character used for examples",
		contact = "blah@example.org",
	},
	extra = {
		keywords = "test",
	},
}

M.config = {
	out_dir = "dist",
	template_root = "templates",
	public_files = "public",
}

M.paths = {
	out_dir = "out",
	css = "css",
}

M.outputs = {
	-- statisk.asset("css/styles.css"):output("styles.css"):watch("css/*.css"),
	statisk.public_file("public/**/*"):out("./"),
	statisk.template("templates/sitemap.xml"):out("sitemap.xml"),
	statisk.template("templates/404.html"):out("404/index.html"),
	statisk.template("templates/page.html"):filter(function(page)
		return page.in_category("page")
	end),
}

return statisk.setup(M)
