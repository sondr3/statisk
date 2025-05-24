use anyhow::Result;
use lightningcss::{
    printer::PrinterOptions,
    stylesheet::{MinifyOptions, ParserOptions, StyleSheet},
    targets::{Browsers, Targets},
};
use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_mangler::MangleOptions;
use oxc_minifier::{CompressOptions, Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;
use simple_minify_html::minify;

pub fn html(content: &str) -> Result<Vec<u8>> {
    Ok(minify(content.as_bytes(), None))
}

pub fn js(content: &str, kind: Option<SourceType>) -> String {
    let allocator = Allocator::default();
    let source_type = kind.unwrap_or(SourceType::cjs());
    let ret = Parser::new(&allocator, content, source_type).parse();
    let mut program = ret.program;
    let options = MinifierOptions {
        mangle: Some(MangleOptions::default()),
        compress: Some(CompressOptions::default()),
    };
    let ret = Minifier::new(options).build(&allocator, &mut program);
    CodeGenerator::new()
        .with_options(CodegenOptions {
            minify: true,
            ..CodegenOptions::default()
        })
        .with_scoping(ret.scoping)
        .build(&program)
        .code
}

pub fn css(content: &str) -> Result<String> {
    let mut stylesheet = StyleSheet::parse(content, ParserOptions::default())
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    let targets = Targets {
        browsers: Browsers::from_browserslist(["> .5% and last 5 versions"])?,
        ..Default::default()
    };

    let minify_opts = MinifyOptions {
        targets,
        ..Default::default()
    };

    stylesheet.minify(minify_opts)?;

    let printer_opts = PrinterOptions {
        minify: true,
        targets,
        ..Default::default()
    };

    let res = stylesheet.to_css(printer_opts)?;
    Ok(res.code)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_minify_html() {
        let html = r#"
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>Test</title>
  </head>
  <body>
    <h1>Hello</h1>
    <p class="test">World</p>
  </body>
</html>        
        "#
        .trim();
        let minified = String::from_utf8(super::html(&html.to_string()).unwrap()).unwrap();
        insta::assert_snapshot!(minified);
    }
}
