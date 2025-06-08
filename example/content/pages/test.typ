+++
title = "Test"
description = "Test page"
+++

#let page(body) = {
    html.elem(
        "html",
        attrs: (lang: "en"),
        {
            html.elem("body", body)
        }
    )
}

#show: page.with()

= Introduction

In this report, we will explore the
various factors that influence _fluid
dynamics_ in glaciers and how they
contribute to the formation and
behaviour of these natural structures.
