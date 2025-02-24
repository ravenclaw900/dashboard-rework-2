use maud::{DOCTYPE, Markup, Render, html};

use crate::http::{
    request::BackendData,
    response::{ServerResponse, html},
};

pub fn template(
    BackendData {
        backend_list,
        current_backend,
    }: &BackendData,
    content: Markup,
) -> ServerResponse {
    let page = html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1";

                title { "DietPi Dashboard" }

                link rel="stylesheet" href="/static/main.css";
            }
            body {
                h1 { "DietPi Dashboard" }

                header {
                    button onclick="body.classList.toggle('nav-closed')" {
                        (Icon::new("fa6-solid-bars").size(48))
                    }

                    "DietPi Dashboard"

                    select name="backend" fx-action="/set-backend" fx-swap="none"
                    {
                        @for backend in backend_list {
                            @let is_current_backend = backend.0 == current_backend.0;
                            option value=(backend.0) selected[is_current_backend] {
                                (backend.1) " (" (backend.0) ")"
                            }
                        }
                    }
                }

                nav {
                    a href="/system" {
                        (Icon::new("fa6-solid-database"))
                        "System"
                    }
                }

                main {
                    (content)
                }

                footer {}

                script src="/static/fixi-0.6.4.js" {}
                script src="/static/ext-fixi.js" {}
            }
        }
    };

    html(page.into_string())
}

pub struct Icon {
    name: &'static str,
    size: u8,
}

impl Icon {
    pub fn new(name: &'static str) -> Self {
        Self { name, size: 24 }
    }

    pub fn size(mut self, size: u8) -> Self {
        self.size = size;
        self
    }
}

impl Render for Icon {
    fn render(&self) -> Markup {
        html! {
            svg width=(self.size) height=(self.size) {
                use href={"/static/icons.svg#" (self.name)} {}
            }
        }
    }
}
