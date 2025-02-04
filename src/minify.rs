use anyhow::Result;
use lightningcss::{
    printer::PrinterOptions,
    stylesheet::{MinifyOptions, ParserOptions, StyleSheet},
    targets::{Browsers, Targets},
};
use swc_common::{BytePos, FileName, SourceFile};
use swc_html_codegen::writer::basic::BasicHtmlWriter;
use swc_html_codegen::{CodeGenerator, CodegenConfig, Emit};
use swc_html_minifier::minify_document;
use swc_html_parser::parse_file_as_document;

pub fn html(content: String) -> Result<Vec<u8>> {
    let source_file = SourceFile::new(
        FileName::Anon.into(),
        false,
        FileName::Anon.into(),
        content,
        BytePos(1),
    );
    let mut errors = vec![];
    let mut document =
        parse_file_as_document(&source_file, Default::default(), &mut errors).unwrap();
    minify_document(&mut document, &Default::default());
    let mut minified_source = String::new();
    let mut code_generator = CodeGenerator::new(
        BasicHtmlWriter::new(&mut minified_source, None, Default::default()),
        CodegenConfig {
            minify: true,
            ..CodegenConfig::default()
        },
    );
    code_generator.emit(&document).unwrap();
    Ok(minified_source.into())
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
        let minified = String::from_utf8(super::html(html.to_string()).unwrap()).unwrap();
        insta::assert_snapshot!(minified);
    }
}
