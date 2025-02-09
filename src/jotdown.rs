use anyhow::Result;
use jotdown::{Attributes, Container, Event, Render};

pub fn render_jotdown(input: &str) -> Result<String> {
    let events = jotdown::Parser::new(input).map(jotdown_event_mapper);
    let mut html = String::new();
    jotdown::html::Renderer::default().push(events, &mut html)?;
    Ok(html)
}

fn jotdown_event_mapper(event: Event) -> Event {
    match event {
        Event::Start(container, attrs) => jotdown_container_mapper(container, attrs).into(),
        _ => event,
    }
}

struct ContainerWrapper<'a>(Container<'a>, Attributes<'a>);

impl<'a> From<ContainerWrapper<'a>> for Event<'a> {
    fn from(val: ContainerWrapper<'a>) -> Self {
        Event::Start(val.0, val.1)
    }
}

fn jotdown_container_mapper<'a>(
    container: Container<'a>,
    attrs: Attributes<'a>,
) -> ContainerWrapper<'a> {
    match container {
        Container::Heading {
            id,
            level,
            has_section,
        } => ContainerWrapper(
            Container::Heading {
                level,
                id: id.to_lowercase().into(),
                has_section,
            },
            attrs,
        ),
        Container::Section { id } => ContainerWrapper(
            Container::Section {
                id: id.to_lowercase().into(),
            },
            attrs,
        ),
        _ => ContainerWrapper(container, attrs),
    }
}
