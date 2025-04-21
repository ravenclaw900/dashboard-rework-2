use hyper::header;
use maud::{DOCTYPE, Markup, Render, html};

use crate::http::{
    request::{BackendData, ServerRequest},
    response::ServerResponse,
};

macro_rules! fetch_data {
    ($req:expr, $variant:ident) => {
        $req.send_backend_req_oneshot(DataRequestType::$variant)
            .await
            .map(|resp| match resp {
                DataResponseType::$variant(resp) => resp,
                _ => unreachable!(),
            })
    };
}

pub(crate) use fetch_data;

pub fn template(req: &ServerRequest, content: Markup) -> Result<ServerResponse, ServerResponse> {
    let page = if req.is_fixi() {
        content
    } else {
        let BackendData {
            backend_list,
            current_backend,
        } = req.extract_backends()?;

        html! {
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

                        label {
                            "Backend: "
                            select
                                onchange="document.cookie = `backend=${this.selectedOptions[0].value}; Path=/; MaxAge=999999999`; window.location.reload()"
                            {
                                @for backend in backend_list {
                                    @let is_current_backend = backend.0 == current_backend.0;
                                    option value=(backend.0) selected[is_current_backend] {
                                        (backend.1) " (" (backend.0) ")"
                                    }
                                }
                            }
                        }

                        theme-switcher {
                            meta name="color-scheme" {}
                            button {
                                (Icon::new("fa6-solid-sun"))
                                (Icon::new("fa6-solid-moon"))
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

                    footer {
                        "DietPi Dashboard v" (config::VERSION) " by ravenclaw900"
                        a href="https://github.com/ravenclaw900/DietPi-Dashboard" target="_blank" {
                            (Icon::new("cib-github").size(32))
                        }
                    }

                    script src="/static/main.js" {}
                }
            }
        }
    };

    Ok(ServerResponse::new()
        .header(header::CONTENT_TYPE, "text/html;charset=UTF-8")
        .body(page.into_string()))
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
