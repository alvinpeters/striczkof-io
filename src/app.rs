use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/striczkof-io.css"/>

        <Meta name="viewport" content="width=device-width, initial-scale=1.0, shrink-to-fit=no"/>
        <Title text="Alvin's Rubbish Dump"/>
        <Meta name="description" content="My personal portfolio of hot unfinished garbage."/>
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/striczkof-io.css"/>
        <Stylesheet href="/bootstrap/css/bootstrap.min.css"/>
        <Stylesheet href="https://fonts.googleapis.com/css?family=Lora:400,700,400italic,700italic&amp;display=swap"/>
        <Stylesheet href="https://fonts.googleapis.com/css?family=Cabin:700&amp;display=swap"/>
        <Stylesheet href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/4.7.0/css/font-awesome.min.css"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view() // what's this?
        }>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                </Routes>
            </main>

        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <Body attributes={vec!(("id", "page-top".into_attribute()), ("dataBsSpy", "scroll".into_attribute()), ("dataBsTarget", "#mainNav".into_attribute()), ("dataBsOffset", "77".into_attribute()))}/>
        <nav class="navbar navbar-expand-md fixed-top navbar-light" id="mainNav">
            <div class="container">
                <a class="navbar-brand" href="#">"Alvin's Rubbish Heap"</a>
                <button dataBsToggle="collapse" class="navbar-toggler navbar-toggler-right" dataBsTarget="#navbarResponsive" type="button" aria-controls="navbarResponsive" aria-expanded="false" aria-label="Toggle navigation" value="Menu"><i class="fa fa-bars"/></button>
                <div class="collapse navbar-collapse" id="navbarResponsive">
                    <ul class="navbar-nav ms-auto">
                        <li class="nav-item nav-link"><a class="nav-link active" href="#about">"About"</a></li>
                        <li class="nav-item nav-link"><a class="nav-link" href="#projects">"Projects"</a></li>
                        <li class="nav-item nav-link"><a class="nav-link" href="#contact">"Contact"</a></li>
                    </ul>
                </div>
            </div>
        </nav>
        <header class="masthead" style="background-image:url('/img/intro-bg.jpg?h=145cae4a8124ee5dac950230c1552d3c');">
            <div class="intro-body">
                <div class="container">
                    <div class="row">
                        <div class="col-lg-8 mx-auto">
                            <h1 class="brand-heading">"Hi! I'm Alvin!"</h1>
                            <p class="intro-text">"I create random projects for whatever and whenever I feel like it. Usually, I don't even get to finish them!"</p>
                            <a class="btn btn-link btn-circle" role="button" href="#about"><i class="fa fa-angle-double-down animated"/></a>
                        </div>
                    </div>
                </div>
            </div>
        </header>
        <footer>

            <div class="container text-center">
                <p>"Copyright Alvin Peters 2024"</p>
                <sub>
                    "This is based on the Grayscale template. This website is currently work in progress.\
                     The source code is hosted at "
                    <a href="https://github.com/striczkof/striczkof-io">my GitHub repository</a>
                    " under the "
                    <a href="https://www.gnu.org/licenses/agpl-3.0.en.html">"GNU Affero General Public License 3.0"</a>"."
                </sub>
            </div>
        </footer>
        <Script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.2/dist/js/bootstrap.bundle.min.js"/>
        <Script src="/js/bs-init.js"/>
        <Script src="/js/grayscale.js"/>
    }
}
