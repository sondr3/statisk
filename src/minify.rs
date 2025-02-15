use anyhow::{anyhow, Result};
use lightningcss::{
    printer::PrinterOptions,
    stylesheet::{MinifyOptions, ParserOptions, StyleSheet},
    targets::{Browsers, Targets},
};
use swc_common::{BytePos, FileName, SourceFile};
use swc_html_codegen::{
    writer::basic::{BasicHtmlWriter, BasicHtmlWriterConfig},
    CodeGenerator, CodegenConfig, Emit,
};
use swc_html_minifier::{
    minify_document,
    option::{CollapseWhitespaces, MinifyJsOption, RemoveRedundantAttributes},
};
use swc_html_parser::{parse_file_as_document, parser::ParserConfig};

pub fn html(content: String) -> Result<Vec<u8>> {
    swc_common::GLOBALS.set(&swc_common::Globals::new(), || {
        let source_file = SourceFile::new(
            FileName::Anon.into(),
            false,
            FileName::Anon.into(),
            content,
            BytePos(1),
        );
        let mut errors = vec![];
        let mut document =
            parse_file_as_document(&source_file, ParserConfig::default(), &mut errors)
                .map_err(|err| anyhow!("Could not parse HTML: {:?}", err))?;

        if !errors.is_empty() {
            eprintln!("{errors:#?}");
        }
        minify_document(
            &mut document,
            &swc_html_minifier::option::MinifyOptions {
                force_set_html5_doctype: true,
                collapse_whitespaces: CollapseWhitespaces::Smart,
                remove_empty_metadata_elements: false,
                remove_comments: true,
                preserve_comments: None,
                minify_conditional_comments: false,
                remove_empty_attributes: true,
                remove_redundant_attributes: RemoveRedundantAttributes::Smart,
                collapse_boolean_attributes: true,
                merge_metadata_elements: true,
                normalize_attributes: true,
                minify_js: MinifyJsOption::Bool(true),
                ..Default::default()
            },
        );
        let mut minified_source = String::new();
        let mut code_generator = CodeGenerator::new(
            BasicHtmlWriter::new(&mut minified_source, None, BasicHtmlWriterConfig::default()),
            CodegenConfig {
                minify: true,
                ..CodegenConfig::default()
            },
        );
        code_generator.emit(&document)?;
        Ok(minified_source.into())
    })
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
