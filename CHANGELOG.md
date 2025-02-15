## v0.2.2

> 2025-02-15

## Summary

HTML minification stopped working, this is now fixed.

### Commits

- [[`8e4bd70`](https://github.com/sondr3/statisk)] Fix SWC HTML minification

## v0.2.1

> 2025-02-14

## Summary

Bug fixes and version bumps. Now handles 404 and 500 pages explicitly, silences
WebSocket disconnect warnings and a major refactoring of the build pipeline.

### Commits

- [[`6072647`](https://github.com/sondr3/statisk/commit/6072647)] Handle 404/500 pages being included in sitemap
- [[`de48f40`](https://github.com/sondr3/statisk/commit/de48f40)] Fix some clippy lints
- [[`e9985f6`](https://github.com/sondr3/statisk/commit/e9985f6)] Handle 404 pages explicitly when building out_path
- [[`a16f762`](https://github.com/sondr3/statisk/commit/a16f762)] Silence WebSocket warnings for disconnected clients
- [[`34f2ad9`](https://github.com/sondr3/statisk/commit/34f2ad9)] Refactor whole context building and rendering pipeline
- [[`ecffc4b`](https://github.com/sondr3/statisk/commit/ecffc4b)] Bump jiff to 0.2
- [[`762c96d`](https://github.com/sondr3/statisk/commit/762c96d)] Use filename, not file stem in pages map
- [[`cfd6080`](https://github.com/sondr3/statisk/commit/cfd6080)] Move statisk binary to /usr/local/bin
- [[`446e3e3`](https://github.com/sondr3/statisk/commit/446e3e3)] Don't compile with lto=fat, IT IS SO SLOW
- [[`4dfa8b0`](https://github.com/sondr3/statisk/commit/4dfa8b0)] Correctly build page context too
- [[`7a13b75`](https://github.com/sondr3/statisk/commit/7a13b75)] Support XSL files in the template directory
- [[`c7d17e1`](https://github.com/sondr3/statisk/commit/c7d17e1)] Run action for example
- [[`ff54312`](https://github.com/sondr3/statisk/commit/ff54312)] Add GitHub Action

## 0.2.0

> 2025-02-09

## Summary

Initial release.

### Commits

- [[`de1b8c7`](https://github.com/sondr3/statisk/commit/de1b8c7)] Fix all clippy lints with a little pedantic stuff on
  top
- [[`c649507`](https://github.com/sondr3/statisk/commit/c649507)] Refactor templating and content to work for HTML and
  XML
- [[`c82b40c`](https://github.com/sondr3/statisk/commit/c82b40c)] Move template pages out from templates and into
  content
- [[`0c4fffa`](https://github.com/sondr3/statisk/commit/0c4fffa)] Add underscore to deserialize to silence warning
- [[`a0f30c7`](https://github.com/sondr3/statisk/commit/a0f30c7)] Move jotdown rendering to own file
- [[`88ab0bc`](https://github.com/sondr3/statisk/commit/88ab0bc)] Extract frontmatter into own file
- [[`e456214`](https://github.com/sondr3/statisk/commit/e456214)] Format stuff
- [[`5cbd4b4`](https://github.com/sondr3/statisk/commit/5cbd4b4)] Handle context and frontmatter in template pages
- [[`e502c9a`](https://github.com/sondr3/statisk/commit/e502c9a)] Refactor templating completely
- [[`451cb3a`](https://github.com/sondr3/statisk/commit/451cb3a)] Config cleanup
- [[`d246252`](https://github.com/sondr3/statisk/commit/d246252)] Clean out dir on launch to clean up files
- [[`a83f5b5`](https://github.com/sondr3/statisk/commit/a83f5b5)] Remove Metadata, pass Context around instead
- [[`fc298be`](https://github.com/sondr3/statisk/commit/fc298be)] Upgrade all dependencies
- [[`168eb1a`](https://github.com/sondr3/statisk/commit/168eb1a)] Rename Mode -> BuildMode, extract to own file
- [[`5d5fc9c`](https://github.com/sondr3/statisk/commit/5d5fc9c)] Bump minijinja, add get_asset function to env
- [[`6234412`](https://github.com/sondr3/statisk/commit/6234412)] Update snapshots for sitemap test
- [[`71392de`](https://github.com/sondr3/statisk/commit/71392de)] Fix some clippy lints
- [[`31758da`](https://github.com/sondr3/statisk/commit/31758da)] Remove Sass, use CSS and refactor whole CSS loading
  pipeline
- [[`7817132`](https://github.com/sondr3/statisk/commit/7817132)] Refactor watcher to report one file at a time
- [[`68b68b8`](https://github.com/sondr3/statisk/commit/68b68b8)] Fix XML urlset namespaces
- [[`848bbf0`](https://github.com/sondr3/statisk/commit/848bbf0)] Actually minify HTML using SWC
- [[`5a338b3`](https://github.com/sondr3/statisk/commit/5a338b3)] Fix serve also building site
- [[`dbae379`](https://github.com/sondr3/statisk/commit/dbae379)] Create date deserializers for content
- [[`c35d549`](https://github.com/sondr3/statisk/commit/c35d549)] Fix sitemap generation, again
- [[`bccd7cf`](https://github.com/sondr3/statisk/commit/bccd7cf)] Handle WebSocket with new axum and include
  livereload.js in app
- [[`1c6221f`](https://github.com/sondr3/statisk/commit/1c6221f)] Bump axum and tower-http, fix errors
- [[`037ab51`](https://github.com/sondr3/statisk/commit/037ab51)] Add CLI, handle paths better
- [[`b095091`](https://github.com/sondr3/statisk/commit/b095091)] Move things out of the site/ folder
- [[`9b6ab8d`](https://github.com/sondr3/statisk/commit/9b6ab8d)] Add a simple example site
- [[`bfd903a`](https://github.com/sondr3/statisk/commit/bfd903a)] Make last_modified optional in content
- [[`e668f51`](https://github.com/sondr3/statisk/commit/e668f51)] Add the most basic of basic configs
- [[`697cab5`](https://github.com/sondr3/statisk/commit/697cab5)] Fix some clippy lints
- [[`822e1db`](https://github.com/sondr3/statisk/commit/822e1db)] Move from minify-html to swc for HTML minification
- [[`d42fe1f`](https://github.com/sondr3/statisk/commit/d42fe1f)] Use jiff instead of time
- [[`f37f374`](https://github.com/sondr3/statisk/commit/f37f374)] Track sitemap snapshot test
- [[`49d7341`](https://github.com/sondr3/statisk/commit/49d7341)] Whoops, managed to merge both projects
- [[`e175e5c`](https://github.com/sondr3/statisk/commit/e175e5c)] Purge git-ignore, rename to statisk
- [[`88d3660`](https://github.com/sondr3/statisk/commit/88d3660)] Use xml-rs to build a sitemap
- [[`6a4465b`](https://github.com/sondr3/statisk/commit/6a4465b)] Copy over old static site generator
- [[`ff4150d`](https://github.com/sondr3/statisk/commit/ff4150d)] In the beginning there was darkness...
- [[`1de824b`](https://github.com/sondr3/statisk/commit/1de824b)] Merge pull request #1 from usure/patch-1
- [[`c278032`](https://github.com/sondr3/statisk/commit/c278032)] it now prints the help guide if ARGV is empty.
- [[`80cc591`](https://github.com/sondr3/statisk/commit/80cc591)] Start on CLI
- [[`667c826`](https://github.com/sondr3/statisk/commit/667c826)] Set version to 0.0.1
- [[`a96db99`](https://github.com/sondr3/statisk/commit/a96db99)] Initial commit


