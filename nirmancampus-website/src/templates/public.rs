//! Public Nirman Campus pages (home, programs, contact, privacy, student zone).

use frunk::Generic;
use maud::{Markup, PreEscaped, html};

use crate::fee_session::StudentFeeView;
use lariv_rs::{
    components::{ShellBase, ShellChrome, shell_base},
    template::RenderTemplate,
};

const WHATSAPP_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="h-4 w-4 shrink-0" aria-hidden="true"><path d="M17.472 14.382c-.297-.149-1.758-.867-2.03-.967-.273-.099-.471-.148-.67.15-.197.297-.767.966-.94 1.164-.173.199-.347.223-.644.075-.297-.15-1.255-.463-2.39-1.475-.883-.788-1.48-1.761-1.653-2.059-.173-.297-.018-.458.13-.606.134-.133.298-.347.446-.52.149-.174.198-.298.298-.497.099-.198.05-.371-.025-.52-.075-.149-.669-1.612-.916-2.207-.242-.579-.487-.5-.669-.51-.173-.008-.371-.01-.57-.01-.198 0-.52.074-.792.372-.272.297-1.04 1.016-1.04 2.479 0 1.462 1.065 2.875 1.213 3.074.149.198 2.096 3.2 5.077 4.487.709.306 1.262.489 1.694.625.712.227 1.36.195 1.871.118.571-.085 1.758-.719 2.006-1.413.248-.694.248-1.289.173-1.413-.074-.124-.272-.198-.57-.347m-5.421 7.403h-.004a9.87 9.87 0 01-5.031-1.378l-.361-.214-3.741.982.998-3.648-.235-.374a9.86 9.86 0 01-1.51-5.26c.001-5.45 4.436-9.884 9.888-9.884 2.64 0 5.122 1.03 6.988 2.898a9.825 9.825 0 012.893 6.994c-.003 5.45-4.435 9.884-9.885 9.884m8.413-18.297A11.815 11.815 0 0012.05 0C5.495 0 .16 5.335.157 11.892c0 2.096.547 4.142 1.588 5.945L.057 24l6.305-1.654a11.882 11.882 0 005.683 1.448h.005c6.554 0 11.89-5.335 11.893-11.893a11.821 11.821 0 00-3.48-8.413Z"/></svg>"##;

fn is_external_url(url: &str) -> bool {
    let s = url.trim().to_ascii_lowercase();
    s.starts_with("http://") || s.starts_with("https://")
}

fn js_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

#[derive(Clone)]
pub struct PublicShell {
    pub is_authenticated: bool,
    pub year: i32,
}

#[derive(Clone)]
pub struct ImportantLinkItem {
    pub title: String,
    pub url: String,
}

#[derive(Clone)]
pub struct HomeAnnouncement {
    pub title: String,
    pub description_html: String,
    pub date: String,
    pub url: String,
}

#[derive(Clone)]
pub struct PublicProgram {
    pub name: String,
    pub code: String,
    pub description: String,
    pub university: String,
}

#[derive(Clone)]
pub struct StudentZonePublicItem {
    pub title: String,
    pub url: String,
}

#[derive(Clone)]
pub struct StudentZonePublicSection {
    pub title: String,
    pub items: Vec<StudentZonePublicItem>,
}

pub fn render_topbar(is_authenticated: bool) -> Markup {
    html! {
        nav class="navbar bg-base-100 border-b border-base-300 px-4" {
            div class="navbar-start" {
                a href="/" {
                    img src="/nirman/static/images/logo.png" alt="Nirman Campus" class="h-10";
                }
            }
            div class="navbar-center hidden lg:flex" {
                ul class="menu menu-horizontal px-1 gap-2" {
                    li { a href="/" { "Home" } }
                    li { a href="/programs-offered/" { "Programs" } }
                    li { a href="/students-zone/" { "Student Zone" } }
                    li { a href="/contact-us/" { "Contact Us" } }
                }
            }
            div class="navbar-end gap-2" {
                @if is_authenticated {
                    a href="/apps/" class="btn btn-primary btn-sm" { "Dashboard" }
                } @else {
                    a href="/users/login/" class="btn btn-primary btn-sm" { "Sign In" }
                }
                details class="dropdown dropdown-end lg:hidden" {
                    summary class="btn btn-ghost btn-sm list-none" aria-label="Open navigation menu" {
                        (PreEscaped(r##"<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" /></svg>"##))
                    }
                    ul class="menu dropdown-content mt-3 z-[100] p-2 shadow bg-base-100 rounded-box w-52" {
                        li { a href="/" { "Home" } }
                        li { a href="/programs-offered/" { "Programs" } }
                        li { a href="/students-zone/" { "Student Zone" } }
                        li { a href="/contact-us/" { "Contact Us" } }
                    }
                }
            }
        }
    }
}

pub fn render_footer(year: i32, important_links: &[ImportantLinkItem]) -> Markup {
    html! {
        footer class="bg-base-200 border-t border-base-300 py-10 px-4" {
            div class="max-w-6xl mx-auto grid grid-cols-1 md:grid-cols-3 gap-8" {
                div {
                    h3 class="font-bold text-lg mb-3" { "Nirman Campus" }
                    p class="text-sm opacity-70" { "Empowering Education, Inspiring Futures" }
                }
                div {
                    h3 class="font-bold text-lg mb-3" { "Quick Links" }
                    ul class="space-y-1 text-sm" {
                        li { a href="/" class="hover:underline" { "Home" } }
                        li { a href="/programs-offered/" class="hover:underline" { "Programs" } }
                        li { a href="/contact-us/" class="hover:underline" { "Contact Us" } }
                        li { a href="/privacy-policy/" class="hover:underline" { "Privacy Policy" } }
                        @for link in important_links {
                            li { a href=(link.url) class="hover:underline" { (link.title) } }
                        }
                    }
                }
                div {
                    h3 class="font-bold text-lg mb-3" { "Contact" }
                    ul class="space-y-2 text-sm" {
                        li class="opacity-70" { "Address: Jakhepal Ghasiwala Rd, Sunam" }
                        li class="opacity-70" {
                            "Email: "
                            a href="mailto:nirmancampus@gmail.com" class="hover:underline" { "nirmancampus@gmail.com" }
                        }
                        li class="opacity-70" {
                            "Mobiles:" br;
                            a href="tel:+919815098210" class="hover:underline" { "+91 98150 98210" } br;
                            a href="tel:+919478450740" class="hover:underline" { "+91 94784 50740" } br;
                            a href="tel:+917717498210" class="hover:underline" { "+91 77174 98210" }
                        }
                        li {
                            a href="https://wa.me/919815098210" class="btn btn-success btn-sm w-full sm:w-auto" target="_blank" rel="noopener noreferrer" aria-label="WhatsApp admission line — chat for admissions" {
                                "Chat for admissions on WhatsApp"
                            }
                        }
                    }
                }
            }
            div class="max-w-6xl mx-auto mt-10 pt-8 border-t border-base-300" {
                h3 class="font-bold text-lg mb-3 text-center md:text-left" { "Connect with us" }
                p class="text-sm opacity-70 mb-4 text-center md:text-left" {
                    "Updates and announcements on WhatsApp, Telegram, YouTube, Facebook, and Instagram."
                }
                div class="flex flex-wrap justify-center md:justify-start gap-2" {
                    a href="https://whatsapp.com/channel/0029Vaj8AM0F6sn7LHwyQE0D" class="btn btn-sm btn-outline gap-2" target="_blank" rel="noopener noreferrer" aria-label="Nirman Campus WhatsApp channel" {
                        (PreEscaped(WHATSAPP_SVG))
                        "WhatsApp"
                    }
                    a href="https://t.me/nirmancampus" class="btn btn-sm btn-outline gap-2" target="_blank" rel="noopener noreferrer" aria-label="Telegram: Nirman Campus Sunam updates" { "Telegram" }
                    a href="https://www.youtube.com/channel/UC3ZaEEWH9hsCDM7alcIEvDA" class="btn btn-sm btn-outline gap-2" target="_blank" rel="noopener noreferrer" aria-label="Nirman Campus YouTube channel" { "YouTube" }
                    a href="https://www.facebook.com/nirmancampus.sunam/" class="btn btn-sm btn-outline gap-2" target="_blank" rel="noopener noreferrer" aria-label="Nirman Campus Facebook page" { "Facebook" }
                    a href="https://www.instagram.com/nirmancampussunam/" class="btn btn-sm btn-outline gap-2" target="_blank" rel="noopener noreferrer" aria-label="Nirman Campus Instagram" { "Instagram" }
                }
            }
            div class="text-center text-sm opacity-50 mt-8" {
                (PreEscaped("&copy; ")) (year) " Nirman Campus. All rights reserved."
            }
        }
    }
}

fn public_document(
    chrome: &ShellChrome,
    title: &str,
    shell: &PublicShell,
    body: Markup,
    important_links: &[ImportantLinkItem],
) -> Markup {
    shell_base(ShellBase {
        title,
        registry_head: chrome.head.clone(),
        body: html! {
            (render_topbar(shell.is_authenticated))
            (body)
            (render_footer(shell.year, important_links))
        },
        ..Default::default()
    })
}

#[derive(Generic)]
pub struct HomePage {
    pub shell: PublicShell,
    pub announcements: Vec<HomeAnnouncement>,
    pub important_links: Vec<ImportantLinkItem>,
    pub hero_url: String,
    pub director_img_url: String,
}

impl HomePage {
    fn content(&self) -> Markup {
        html! {
            div class="relative bg-cover flex items-center justify-center bg-center bg-fixed min-h-[50vh] md:min-h-[100vh]" style={"background-image:url('" (self.hero_url) "')"} {
                div class="absolute inset-0 bg-black/50" {}
                div class="relative z-10 flex flex-col items-center justify-center w-full text-center px-4 py-16" {
                    h1 class="text-4xl md:text-6xl font-extrabold text-white mb-4 text-center" { "Nirman Campus" }
                    p class="text-lg md:text-2xl text-gray-200 mb-5" { "Empowering Education, Inspiring Futures" }
                    div class="flex flex-col sm:flex-row items-center justify-center gap-3" {
                        a href="https://wa.me/919815098210" class="btn btn-success btn-sm gap-1.5 shadow-md" target="_blank" rel="noopener noreferrer" aria-label="WhatsApp admission line — chat for admissions" {
                            (PreEscaped(WHATSAPP_SVG))
                            "Chat for admissions on WhatsApp"
                        }
                    }
                }
            }
            @if !self.announcements.is_empty() {
                section class="py-14 px-4 bg-base-100" {
                    div class="max-w-6xl mx-auto" {
                        p class="text-sm font-semibold uppercase tracking-wide text-primary text-center mb-2" { "Latest updates" }
                        h2 class="text-3xl font-bold text-center mb-10" { "Announcements" }
                        div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4" {
                            @for a in &self.announcements {
                                div class={
                                    @if a.url.is_empty() { "card bg-base-200 relative overflow-hidden border border-base-300/50 shadow-sm" }
                                    @else { "card bg-base-200 relative overflow-hidden border-2 border-primary/40 shadow-md" }
                                } {
                                    div class="card-body relative" {
                                        h3 class="card-title" { (a.title) }
                                        @if !a.description_html.trim().is_empty() {
                                            div class="prose max-w-none [&_a]:text-blue-600 [&_a]:underline [&_a:hover]:text-blue-800 dark:[&_a]:text-blue-400 dark:[&_a:hover]:text-blue-300" {
                                                (PreEscaped(&a.description_html))
                                            }
                                        }
                                        div class="text-sm opacity-60 mt-2" { (a.date) }
                                        @if !a.url.is_empty() {
                                            div class="card-actions justify-end mt-4" {
                                                @if is_external_url(&a.url) {
                                                    a href=(a.url) class="btn btn-primary btn-sm" target="_blank" rel="noopener noreferrer" { "Click Here!" }
                                                } @else {
                                                    a href=(a.url) class="btn btn-primary btn-sm" { "Click Here!" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            section class="py-12 px-4 bg-primary/10 border-y border-primary/20" {
                div class="max-w-6xl mx-auto" {
                    p class="text-sm font-semibold uppercase tracking-wide text-primary text-center mb-2" { "Credibility & recognition" }
                    h2 class="text-2xl md:text-3xl font-bold text-center mb-4" { "Their excellence becomes your qualification" }
                    p class="text-center max-w-3xl mx-auto text-base-content/90 leading-relaxed mb-8" {
                        strong { "Nirman Campus of Education, Research & Training (NCERT)" }
                        " works in affiliation with "
                        strong { "MRSPTU (Maharaja Ranjit Singh Punjab Technical University, Bathinda)" }
                        " and is an IGNOU Study Center. When you study here, "
                        strong { "you inherit the stature of two of India's most trusted universities" }
                        ": national recognition, statutory backing, and reputations built over decades."
                    }
                    div class="max-w-4xl mx-auto mb-10 grid grid-cols-1 md:grid-cols-2 gap-6" {
                        div class="rounded-2xl border-2 border-primary bg-base-100 shadow-xl overflow-hidden" {
                            div class="bg-primary text-primary-content px-4 py-2.5 text-center text-xs sm:text-sm font-bold uppercase tracking-widest" {
                                "National Institutional Ranking Framework (NIRF)"
                            }
                            div class="px-5 py-8 sm:px-10 sm:py-10 text-center" {
                                p class="text-3xl font-black text-primary leading-none mb-2" { "NIRF #1 Rank" }
                                p class="text-lg sm:text-xl font-semibold text-base-content/80 mb-6" { "Open University category — Government of India" }
                                p class="text-base sm:text-lg text-base-content/90 leading-relaxed max-w-2xl mx-auto text-left sm:text-center" {
                                    "IGNOU is a Central University run by the Government of India. It has NIRF #1 Rank, is recognised by UGC and is the Largest Open University in India."
                                }
                            }
                        }
                        div class="rounded-2xl border-2 border-primary bg-base-100 shadow-xl overflow-hidden" {
                            div class="bg-primary text-primary-content px-4 py-2.5 text-center text-xs sm:text-sm font-bold uppercase tracking-widest" {
                                "NAAC Accreditation"
                            }
                            div class="px-5 py-8 sm:px-10 sm:py-10 text-center" {
                                p class="text-3xl font-black text-primary leading-none mb-2" { "NAAC A++" }
                                p class="text-lg sm:text-xl font-semibold text-base-content/80 mb-6" { "Quality assessed for academic excellence" }
                                p class="text-base sm:text-lg text-base-content/90 leading-relaxed max-w-2xl mx-auto text-left sm:text-center" {
                                    "Our programs align with NAAC evaluation standards to support strong academic outcomes and continuous improvement."
                                }
                            }
                        }
                    }
                    div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8" {
                        div class="card bg-base-100 shadow-md border border-primary/10 text-left" {
                            div class="card-body" {
                                h3 class="card-title text-lg" { "What we deliver through IGNOU" }
                                p class="text-base-content/90 leading-relaxed" {
                                    "From this study centre, "
                                    strong { "we offer you the same programmes that flow from that NIRF #1-ranked open university" }
                                    "—the credentials summarised above. "
                                    strong { "Your degree carries weight" }
                                    ": qualifications earned through us are valid in India and abroad, with the approvals that matter—UGC, DEB, AIU, AICTE, and the Ministry of Education, Government of India."
                                }
                            }
                        }
                        div class="card bg-base-100 shadow-md border border-primary/10 text-left" {
                            div class="card-body" {
                                h3 class="card-title text-lg" { "What we unlock with MRSPTU" }
                                p class="text-base-content/90 leading-relaxed" {
                                    "Our affiliation with "
                                    strong { "Maharaja Ranjit Singh Punjab Technical University (MRSPTU), Bathinda" }
                                    "—established under Punjab Act No. 5 of 2015—"
                                    strong { "anchors your technical and professional pathway in a statutory technical university" }
                                    " built for innovation and industry readiness. "
                                    strong { "You benefit from the same mission" }
                                    ": rigorous scientific and professional training designed so graduates are prepared for real workplaces, not just examinations."
                                }
                            }
                        }
                    }
                    div class="flex flex-col sm:flex-row flex-wrap items-center justify-center gap-3" {
                        span class="badge badge-neutral badge-lg h-auto min-h-8 py-2 px-4 text-center" { "MRSPTU affiliated institution" }
                        span class="badge badge-primary badge-lg whitespace-normal h-auto min-h-8 py-2 px-4 text-center leading-snug" { "IGNOU Study Center - 2299" }
                    }
                    div class="flex flex-col sm:flex-row items-center justify-center gap-3 mt-10 text-center" {
                        p class="text-base-content/80 max-w-md sm:mr-2 sm:text-left" { "See programmes you can pursue through our IGNOU and MRSPTU pathways." }
                        div class="flex flex-col sm:flex-row gap-3" {
                            a href="/programs-offered/" class="btn btn-outline btn-lg shrink-0" { "View programs offered" }
                        }
                    }
                }
            }
            section class="py-16 px-4 bg-base-100" {
                div class="max-w-6xl mx-auto grid grid-cols-1 md:grid-cols-2 gap-8 items-start" {
                    div class="card bg-base-200 shadow-md" {
                        div class="card-body" {
                            h2 class="card-title text-2xl" { "Important Links" }
                            p class="text-sm opacity-70 -mt-1 mb-4" { "Important links for Nirman Campus operations" }
                            @if !self.important_links.is_empty() {
                                div class="grid grid-cols-1 sm:grid-cols-2 gap-3 sm:gap-4" {
                                    @for link in &self.important_links {
                                        a href=(link.url) class="btn btn-primary btn-sm justify-start items-start h-auto min-h-9 py-2.5 px-4 whitespace-normal text-left leading-snug break-words" {
                                            (link.title)
                                        }
                                    }
                                }
                            } @else {
                                p class="opacity-60" { "No important links configured yet." }
                            }
                        }
                    }
                    div class="card bg-base-200 shadow-md" {
                        div class="card-body" {
                            h2 class="card-title text-2xl" { "Director's Corner" }
                            p class="text-sm opacity-70 -mt-1 mb-4" { "A note from the Founder" }
                            figure class="mb-6 -mx-2 sm:mx-0" {
                                img src=(self.director_img_url) alt="Kansal Foundation" class="w-full max-h-64 sm:max-h-80 object-cover rounded-xl border border-base-300/60 shadow-sm" width="1568" height="750" loading="lazy" decoding="async";
                            }
                            div class="flex flex-col md:flex-row items-center md:items-start gap-6" {
                                div class="w-full md:flex-1 flex flex-col gap-3 text-justify" {
                                    p {
                                        "Welcome to Nirman Campus! Your journey here contributes directly to our vision of "
                                        strong { "Vikasit Bharat" }
                                        "—a thriving, self-reliant nation built on the foundations of quality education and innovation."
                                    }
                                    p {
                                        "We believe in "
                                        strong { "Youth Empowerment" }
                                        ". You are the catalysts for change, and your talents and initiatives will shape communities, bridge gaps, and inspire others to join this transformative mission."
                                    }
                                    p {
                                        "True "
                                        strong { "Nationalism" }
                                        " is active service: uplifting those around you, upholding the values of integrity and unity, and dedicating your skills to the progress of our country."
                                    }
                                    p class="mt-2 font-semibold not-italic text-left" {
                                        "Dr. Amit Kansal" br;
                                        "Founder, Nirman Campus & Kansal Foundation"
                                    }
                                }
                            }
                            div class="card-actions justify-end mt-4" {
                                a href="https://amitkansal.in" target="_blank" rel="noopener noreferrer" class="btn btn-primary" { "Read More" }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl RenderTemplate for HomePage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        public_document(
            chrome,
            "Nirman Campus",
            &self.shell,
            self.content(),
            &self.important_links,
        )
    }
}

#[derive(Generic)]
pub struct ProgramsPage {
    pub shell: PublicShell,
    pub programs: Vec<PublicProgram>,
}

impl RenderTemplate for ProgramsPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        let body = html! {
            section class="py-16 px-4" x-data="{ search: '' }" {
                div class="max-w-6xl mx-auto" {
                    h1 class="text-4xl font-bold text-center mb-10" { "Programs Offered" }
                    p class="text-center text-sm opacity-70 mb-4" {
                        "Programs offered by MRSPTU are in Regular Mode, and programs offered by IGNOU are in ODL Mode"
                    }
                    div class="mb-8 max-w-md mx-auto" {
                        (PreEscaped(r##"<input type="text" x-model="search" placeholder="Search programs..." class="input input-bordered w-full">"##))
                    }
                    @if !self.programs.is_empty() {
                        div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6" {
                            @for p in &self.programs {
                                (PreEscaped(format!(
                                    r##"<div class="card bg-base-200 shadow-md" x-show="!search || '{needle}'.toLowerCase().includes(search.toLowerCase())">"##,
                                    needle = js_escape(&format!("{} {} {}", p.name, p.code, p.university)),
                                )))
                                div class="card-body" {
                                    h2 class="card-title" { (p.name) }
                                    div class="flex flex-wrap gap-2 items-center" {
                                        @if !p.code.is_empty() {
                                            div class="badge badge-outline" { (p.code) }
                                        }
                                        @if !p.university.is_empty() {
                                            div class="badge badge-secondary" { (p.university) }
                                        }
                                    }
                                    @if !p.description.is_empty() {
                                        p class="mt-2" { (p.description) }
                                    }
                                }
                                (PreEscaped("</div>"))
                            }
                        }
                    } @else {
                        p class="text-center opacity-60" { "No programs available at this time." }
                    }
                }
            }
        };
        public_document(
            chrome,
            "Programs Offered — Nirman Campus",
            &self.shell,
            body,
            &[],
        )
    }
}

#[derive(Generic)]
pub struct ContactPage {
    pub shell: PublicShell,
    pub essential_committees_url: String,
}

impl RenderTemplate for ContactPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        let body = html! {
            section class="py-12 px-4" {
                div class="max-w-6xl mx-auto" {
                    h1 class="text-4xl font-bold text-center mb-10" { "Contact Us" }
                    div class="mb-10" {
                        h2 class="text-2xl font-bold text-center mb-6" { "Our Team" }
                        div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4" {
                            div class="card bg-base-200 shadow-md" {
                                div class="card-body items-center text-center" {
                                    h3 class="card-title" { "Sh. Bhagwan Dass Kansal" }
                                    p class="opacity-60" { "Patron" }
                                }
                            }
                            div class="card bg-base-200 shadow-md" {
                                div class="card-body items-center text-center" {
                                    h3 class="card-title" { "Smt. Darshna Devi Kansal" }
                                    p class="opacity-60" { "Chairperson" }
                                }
                            }
                            div class="card bg-base-200 shadow-md" {
                                div class="card-body items-center text-center" {
                                    h3 class="card-title" { "Dr. Amit Kansal" }
                                    p class="opacity-60" { "Founder & CEO" }
                                }
                            }
                            div class="card bg-base-200 shadow-md" {
                                div class="card-body items-center text-center" {
                                    h3 class="card-title" { "Sh. Deepak Bansal" }
                                    p class="opacity-60" { "Director (Administration)" }
                                }
                            }
                        }
                    }
                    div class="grid grid-cols-1 md:grid-cols-2 gap-8" {
                        div class="card bg-base-200 shadow-md" {
                            div class="card-body" {
                                h2 class="card-title text-xl" { "Email & Phone Numbers" }
                                p class="font-medium" { "You can contact us anytime and we will respond to your query." }
                                ul class="list-disc pl-5 space-y-1 mt-2" {
                                    li { strong { "Email:" } " " a href="mailto:nirmancampus@gmail.com" class="link" { "nirmancampus@gmail.com" } }
                                    li { strong { "Student Support:" } " " a href="tel:+919815098210" class="link" { "+91-98150-98210" } }
                                    li { strong { "Admissions Helpline:" } " " a href="tel:+919478450740" class="link" { "+91-94784-50740" } }
                                }
                                p class="mt-3 font-medium" { "Head Office Address:" }
                                ul class="list-inside pl-5 space-y-1" {
                                    li { "Smt. Darshna Kansal, Chairperson" }
                                    li { "Nirman, Peer Bana Banoi Road" }
                                    li { "Sunam – 148028" }
                                }
                            }
                        }
                        div class="card bg-base-200 shadow-md" {
                            div class="card-body" {
                                h2 class="card-title text-xl" { "Postal Address & Account Details" }
                                p class="font-medium text-center" { "(Main Campus)" }
                                div class="space-y-1" {
                                    p { "Nirman Campus of Education, Research & Training" }
                                    p { "Jakhepal-Ghasiawala Road, Sunam-148028, Distt. Sangrur, Punjab" }
                                }
                                p class="font-medium mt-4 text-center" { "Bank Account Details" }
                                div class="space-y-1" {
                                    p { strong { "Beneficiary:" } " Nirman - A Social Organization" }
                                    p { strong { "Bank:" } " Federal Bank, Sunam" }
                                    p { strong { "Acc No:" } " 19750100042364" }
                                    p { strong { "IFSC:" } " FDRL0001975" }
                                }
                            }
                        }
                    }
                    div class="card bg-base-200 shadow-md mt-8" {
                        div class="card-body" {
                            h2 class="card-title text-xl" { "Nirman Executive Team" }
                            p { "These people are dedicated to execute the NIRMAN activities and to serve the society." }
                            ul class="space-y-1 mt-2" {
                                li class="bg-base-100 px-3 py-2 rounded" { strong { "Bhagwan Dass" } ", Patron" }
                                li class="bg-base-100 px-3 py-2 rounded" { strong { "Darshna Kansal" } ", Chairperson" }
                                li class="bg-base-100 px-3 py-2 rounded" { strong { "Dr. Amit Kansal" } ", Founder & CEO, Ph.D., M.A., MBA, LL.B." }
                                li class="bg-base-100 px-3 py-2 rounded" { strong { "Vikas Kansal" } ", Director (Education), MCA, UGC-NET Qualified, M.Sc.(Geography), MA Tourism" }
                                li class="bg-base-100 px-3 py-2 rounded" { strong { "Deepak Bansal" } ", Director (Administration)" }
                                li class="bg-base-100 px-3 py-2 rounded" { strong { "Raman Kansal" } ", Secretary Finance, M.Sc. (Chemistry), B.Ed." }
                                li class="bg-base-100 px-3 py-2 rounded" { strong { "Sonia" } ", Executive Director, M.Sc.(IT)" }
                                li class="bg-base-100 px-3 py-2 rounded" { strong { "Sakshi" } ", Director (Public Awareness & Health), M.A., M.Phil" }
                            }
                        }
                    }
                    div class="card bg-base-200 shadow-md mt-8" {
                        div class="card-body" {
                            h2 class="card-title text-xl" { "Our Supporting Members" }
                            p { "These people support NIRMAN activities to help the organization achieve its goals." }
                            ul class="space-y-1 mt-2" {
                                li class="bg-base-100 px-3 py-2 rounded" { strong { "Kamal Singla" } ", MD, Krishna Group of Institutes" }
                                li class="bg-base-100 px-3 py-2 rounded" { strong { "Dr. Rajeev Jindal" } ", MBBS, M.S." }
                                li class="bg-base-100 px-3 py-2 rounded" { strong { "Dr. Priya Singla" } ", Ph.D., M.Sc (Food and Nutrition), UGC-NET" }
                                li class="bg-base-100 px-3 py-2 rounded" { strong { "Dr. Rajinder" } ", Ph.D. (Mathematics), M.Sc., UGC-NET" }
                                li class="bg-base-100 px-3 py-2 rounded" { strong { "Dr. Rohtash Chauhan" } ", Ph.D. History, M.A., UGC-NET" }
                                li class="bg-base-100 px-3 py-2 rounded" { strong { "Dr. Ravi Kumar" } ", Ph.D. (Geography), M.Sc., UGC-NET" }
                                li class="bg-base-100 px-3 py-2 rounded" { strong { "Rahul Kumar" } ", M.Sc. (Psychology), UGC-NET" }
                                li class="bg-base-100 px-3 py-2 rounded" { strong { "Dr. Vikas" } ", Ph.D (Computer Science), MCA, UGC-NET, MTTM" }
                                li class="bg-base-100 px-3 py-2 rounded" { strong { "Raj Tagore" } ", M.Sc. (Robotics)" }
                            }
                        }
                    }
                    @if !self.essential_committees_url.is_empty() {
                        div class="flex justify-center mt-8" {
                            a href=(self.essential_committees_url) target="_blank" rel="noopener noreferrer" class="btn btn-outline btn-sm font-normal opacity-90" {
                                "View essential committees list (PDF)"
                            }
                        }
                    }
                    div class="mt-12" {
                        iframe src="https://www.google.com/maps/embed?pb=!1m18!1m12!1m3!1d3446.!2d75.8!3d30.12!2m3!1f0!2f0!3f0!3m2!1i1024!2i768!4f13.1!3m3!1m2!1s0x0%3A0x0!2sNirman+Campus!5e0!3m2!1sen!2sin!4v1" class="w-full h-80 rounded-lg border-0" allowfullscreen="" loading="lazy" referrerpolicy="no-referrer-when-downgrade" {}
                    }
                }
            }
        };
        public_document(chrome, "Contact Us — Nirman Campus", &self.shell, body, &[])
    }
}

#[derive(Generic)]
pub struct PrivacyPage {
    pub shell: PublicShell,
}

impl RenderTemplate for PrivacyPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        let body = html! {
            section class="py-12 px-4" {
                div class="max-w-4xl mx-auto" {
                    h1 class="text-4xl font-bold text-center mb-4" { "Privacy Policy" }
                    p class="text-center opacity-60 mb-8" { "Protecting Your Privacy & Personal Information" }
                    div class="prose max-w-none space-y-8" {
                        p { "This Privacy Policy describes Our policies and procedures on the collection, use and disclosure of Your information when You use the Service and tells You about Your privacy rights and how the law protects You." }
                        p { "We use Your Personal data to provide and improve the Service. By using the Service, You agree to the collection and use of information in accordance with this Privacy Policy." }
                        div class="card bg-base-200 shadow-md" {
                            div class="card-body" {
                                h2 class="card-title text-2xl" { "Interpretation and Definitions" }
                                h3 class="font-semibold text-lg mt-4" { "Interpretation" }
                                p { "The words of which the initial letter is capitalized have meanings defined under the following conditions. The following definitions shall have the same meaning regardless of whether they appear in singular or in plural." }
                                h3 class="font-semibold text-lg mt-4" { "Definitions" }
                                ul class="space-y-2 mt-2" {
                                    li { strong { "Company" } r#" (referred to as either "the Company", "We", "Us" or "Our") refers to Nirman Campus of Education, Research and Training, Jakhepal Ghasiwala Rd, Sunam Udham Singhwala, Punjab, India."# }
                                    li { strong { "Website" } " refers to Nirman Campus, accessible from nirmancampus.co.in" }
                                    li { strong { "Service" } " refers to the Website and educational services provided by Nirman Campus." }
                                    li { strong { "Personal Data" } " is any information that relates to an identified or identifiable individual." }
                                    li { strong { "Usage Data" } " refers to data collected automatically, either generated by the use of the Service or from the Service infrastructure itself." }
                                    li { strong { "You" } " means the individual accessing or using the Service, or the company, or other legal entity on behalf of which such individual is accessing or using the Service." }
                                }
                            }
                        }
                        div class="card bg-base-200 shadow-md" {
                            div class="card-body" {
                                h2 class="card-title text-2xl" { "Collecting and Using Your Personal Data" }
                                h3 class="font-semibold text-lg mt-4" { "Personal Data" }
                                p { "While using Our Service, We may ask You to provide Us with certain personally identifiable information that can be used to contact or identify You. This may include:" }
                                ul class="list-disc pl-5 space-y-1 mt-2" {
                                    li { "Email address" }
                                    li { "First name and last name" }
                                    li { "Phone number" }
                                    li { "Educational qualifications and academic records" }
                                    li { "Student enrollment information" }
                                    li { "Usage Data" }
                                }
                                h3 class="font-semibold text-lg mt-4" { "Usage Data" }
                                p { "Usage Data is collected automatically when using the Service. This may include information such as Your Device's Internet Protocol address (IP address), browser type, browser version, the pages of our Service that You visit, the time and date of Your visit, and other diagnostic data." }
                            }
                        }
                        div class="card bg-base-200 shadow-md" {
                            div class="card-body" {
                                h2 class="card-title text-2xl" { "Use of Your Personal Data" }
                                p { "The Company may use Personal Data for the following purposes:" }
                                ul class="list-disc pl-5 space-y-2 mt-2" {
                                    li { strong { "To provide and maintain our Service" } " — Including monitoring usage, managing student registrations, and providing educational services." }
                                    li { strong { "To manage Your Account" } " — Managing Your registration as a student and providing access to educational services." }
                                    li { strong { "To contact You" } " — For educational updates, announcements, examination schedules, and important notifications." }
                                    li { strong { "Educational Services" } " — To provide information about courses, admissions, and educational opportunities." }
                                }
                            }
                        }
                        div class="card bg-base-200 shadow-md" {
                            div class="card-body" {
                                h2 class="card-title text-2xl" { "Security of Your Personal Data" }
                                p { "The security of Your Personal Data is important to Us. We implement appropriate technical and organizational security measures to protect Your personal information against unauthorized access, alteration, disclosure, or destruction. However, no method of transmission over the Internet or electronic storage is 100% secure, and we cannot guarantee absolute security." }
                            }
                        }
                        div class="card bg-base-200 shadow-md" {
                            div class="card-body" {
                                h2 class="card-title text-2xl" { "Retention of Your Personal Data" }
                                p { "We will retain Your Personal Data only for as long as necessary for the purposes set out in this Privacy Policy. For educational records, we may retain data for the duration required by educational authorities and applicable laws. Usage Data is generally retained for a shorter period unless used to strengthen security or improve functionality." }
                            }
                        }
                        div class="card bg-base-200 shadow-md" {
                            div class="card-body" {
                                h2 class="card-title text-2xl" { "Children's Privacy" }
                                p { "Our Service does not address anyone under the age of 13. We do not knowingly collect personally identifiable information from anyone under the age of 13. If You are a parent or guardian and believe Your child has provided Us with Personal Data, please contact Us immediately." }
                            }
                        }
                        div class="card bg-base-200 shadow-md" {
                            div class="card-body" {
                                h2 class="card-title text-2xl" { "Changes to this Privacy Policy" }
                                p { r#"We may update Our Privacy Policy from time to time. We will notify You of any changes by posting the new Privacy Policy on this page and updating the "Last updated" date. You are advised to review this Privacy Policy periodically for any changes."# }
                            }
                        }
                        div class="card bg-base-200 shadow-md" {
                            div class="card-body" {
                                h2 class="card-title text-2xl" { "Contact Us" }
                                p { "If you have any questions about this Privacy Policy, You can contact us:" }
                                ul class="list-disc pl-5 space-y-1 mt-2" {
                                    li { "By email: " a href="mailto:nirmancampus@gmail.com" class="link" { "nirmancampus@gmail.com" } }
                                    li { "By phone: " a href="tel:+919478450740" class="link" { "+91-94784-50740" } }
                                    li { "By visiting: " a href="/contact-us/" class="link" { "Contact Us page" } }
                                }
                            }
                        }
                    }
                }
            }
        };
        public_document(
            chrome,
            "Privacy Policy — Nirman Campus",
            &self.shell,
            body,
            &[],
        )
    }
}

#[derive(Generic)]
pub struct StudentZonePage {
    pub shell: PublicShell,
    pub sections: Vec<StudentZonePublicSection>,
    pub login_error: String,
    pub userid: String,
    pub records: Vec<StudentFeeView>,
    pub logged_in: bool,
}

fn fee_field(label: &str, value: &str) -> Markup {
    html! {
        div {
            p class="text-xs uppercase tracking-wide opacity-60" { (label) }
            p class="font-medium break-words" { (if value.trim().is_empty() { "—" } else { value }) }
        }
    }
}

impl RenderTemplate for StudentZonePage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        let body = html! {
            section class="py-16 px-4" {
                div class="max-w-6xl mx-auto" {
                    h1 class="text-4xl font-bold text-center mb-10" { "Student Zone" }
                    div class="card bg-base-200 shadow-md mb-10" {
                        div class="card-body" {
                            h2 class="card-title" { "View my records" }
                            @if self.logged_in {
                                p class="text-sm opacity-70" { "Your fee records are listed below. Date of birth year and mobile number are shown masked." }
                                form method="post" action="/students-zone/logout/" class="mt-2" {
                                    button type="submit" class="btn btn-outline btn-sm" { "Log out" }
                                }
                                @if self.records.is_empty() {
                                    p class="opacity-60 mt-4" { "No records found." }
                                } @else {
                                    div class="grid grid-cols-1 gap-4 mt-4" {
                                        @for (i, rec) in self.records.iter().enumerate() {
                                            div class="card bg-base-100 border border-base-300" {
                                                div class="card-body py-4" {
                                                    div class="flex flex-col gap-3 text-sm" {
                                                        (fee_field("Receipt ID", &rec.receipt_id))
                                                        div class="grid grid-cols-1 sm:grid-cols-3 gap-3" {
                                                            (fee_field("Date of Deposit", &rec.date_of_deposit))
                                                            (fee_field("Session", &rec.session))
                                                            (fee_field("Submit Type", &rec.submit_type))
                                                        }
                                                        div class="grid grid-cols-1 sm:grid-cols-2 gap-3" {
                                                            (fee_field("Name", &rec.name))
                                                            (fee_field("Father Name", &rec.father_name))
                                                        }
                                                        div class="grid grid-cols-1 sm:grid-cols-3 gap-3" {
                                                            (fee_field("DOB", &rec.dob))
                                                            (fee_field("Category", &rec.category))
                                                            (fee_field("Mobile", &rec.mobile))
                                                        }
                                                        div class="grid grid-cols-1 sm:grid-cols-2 gap-3" {
                                                            (fee_field("Program Code", &rec.program_code))
                                                            (fee_field("Enrollment", &rec.enrollment))
                                                        }
                                                        (fee_field("Courses", &rec.courses))
                                                    }
                                                }
                                            }
                                            @if i + 1 < self.records.len() {
                                                hr class="border-base-300";
                                            }
                                        }
                                    }
                                }
                            } @else {
                                p class="text-sm opacity-70 mb-4" {
                                    "Enter your mobile number as userid. Password is your Enrollment No or Receipt ID."
                                }
                                @if !self.login_error.is_empty() {
                                    p class="text-error text-sm mb-3" { (self.login_error) }
                                }
                                form method="post" action="/students-zone/login/" class="grid grid-cols-1 sm:grid-cols-2 gap-3" {
                                    label class="form-control" {
                                        span class="label-text" { "Mobile number" }
                                        input class="input input-bordered w-full" type="text" name="userid" value=(self.userid) autocomplete="username" required;
                                    }
                                    label class="form-control" {
                                        span class="label-text" { "Enrollment No or Receipt ID" }
                                        input class="input input-bordered w-full" type="password" name="password" autocomplete="current-password" required;
                                    }
                                    div class="sm:col-span-2" {
                                        button type="submit" class="btn btn-primary" { "View records" }
                                    }
                                }
                            }
                        }
                    }
                    @if !self.sections.is_empty() {
                        div class="space-y-10" {
                            @for section in &self.sections {
                                div {
                                    h2 class="text-2xl font-semibold mb-4" { (section.title) }
                                    @if !section.items.is_empty() {
                                        div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4" {
                                            @for item in &section.items {
                                                div class="card bg-base-200 shadow-md" {
                                                    div class="card-body" {
                                                        h3 class="card-title text-lg" { (item.title) }
                                                        div class="card-actions justify-end mt-2" {
                                                            a href=(item.url) class="btn btn-primary btn-sm" { "Open" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } @else {
                                        p class="opacity-60" { "No items in this section." }
                                    }
                                }
                            }
                        }
                    } @else if !self.logged_in {
                        p class="text-center opacity-60" { "No content available at this time." }
                    }
                }
            }
        };
        public_document(
            chrome,
            "Student Zone — Nirman Campus",
            &self.shell,
            body,
            &[],
        )
    }
}
