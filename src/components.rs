#![allow(non_snake_case)]

use crate::emitter::DiagnosticData;
use crate::models::severity;
use crate::render_tree::prelude::*;
use crate::{models, Location};
use crate::{ReportingFiles, ReportingSpan};

crate fn Diagnostic(data: DiagnosticData<'args, impl ReportingFiles>, into: Document) -> Document {
    let header = models::Header::new(&data.diagnostic);

    into.add(tree! {
        <Section name={severity(&data.diagnostic)} as {
            <Header args={header}>
            <Body args={data}>
        }>
    })
}

crate fn Header<'args>(header: models::Header<'args>, into: Document) -> Document {
    into.add(tree! {
        <Section name="header" as {
            <Line as {
                <Section name="primary" as {
                    // error
                    {header.severity()}
                    // [E0001]
                    {IfSome(header.code(), |code| tree! { "[" {code} "]" })}
                }>
                ": "
                // Unexpected type in `+` application
                {header.message()}
            }>
        }>
    })
}

crate fn Body(data: DiagnosticData<'args, impl ReportingSpan>, mut into: Document) -> Document {
    for label in &data.diagnostic.labels {
        let source_line = models::SourceLine::new(data.files, label, data.config);
        let labelled_line = models::LabelledLine::new(source_line.clone(), label);

        into = into.add(tree! {
            // - <test>:2:9
            <SourceCodeLocation args={source_line}>

            // 2 | (+ test "")
            //   |         ^^
            <SourceCodeLine args={labelled_line}>
        });
    }

    into
}

crate fn SourceCodeLocation(
    source_line: models::SourceLine<impl ReportingSpan>,
    into: Document,
) -> Document {
    let Location { line, column } = source_line.location();
    let filename = source_line.filename().to_string();

    into.add(tree! {
        <Section name="source-code-location" as {
            <Line as {
                // - <test>:3:9
                "- " {filename} ":" {line}
                ":" {column}
            }>
        }>
    })
}

crate fn SourceCodeLine(
    model: models::LabelledLine<'args, impl ReportingSpan>,
    into: Document,
) -> Document {
    let source_line = model.source_line();

    into.add(tree! {
        <Line as {
            <Section name="gutter" as {
                {source_line.line_number()}
                " | "
            }>

            <Section name="before-marked" as {
                {source_line.before_marked()}
            }>

            <Section name={model.style()} as {
                {model.source_line().marked()}
            }>

            <Section name="after-marked" as {
                {source_line.after_marked()}
            }>
        }>

        <Line as {
            <Section name="underline" as {
                <Section name="gutter" as {
                    {repeat(" ", model.source_line().line_number_len())}
                    " | "
                }>

                {repeat(" ", model.source_line().before_marked().len())}

                <Section name={model.style()} as {
                    {repeat(model.mark(), model.source_line().marked().len())}
                    {IfSome(model.message(), |message| tree!({" "} {message}))}
                }>
            }>
        }>
    })
}
