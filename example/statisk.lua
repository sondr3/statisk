local statisk = require("statisk")

local M = {}

M.config = {
    url = "https://example.org",
    title = "Example Site",
    description = "An example site for demonstration purposes",
    author = {
        name = "Ola Nordmann",
        description = "A fictional character used for examples",
        contact = "blah@example.org"
    },
    extra = {
        keywords = "test"
    }
}

M.paths = {
    out_dir = "out",
    css = "assets/css"
}

return statisk.setup(M)
