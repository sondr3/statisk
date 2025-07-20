local statisk = require("statisk")

local M = {}

M.config = statisk.config({
    url = "https://example.org",
})

return M
