//! Course-detail related-sections hub — satellites register async HTML loaders via `cap_hook`.

use std::marker::PhantomData;
use std::sync::{Arc, OnceLock};

use frunk::{HCons, HNil, hlist::HList};
use futures_util::future::BoxFuture;
use lariv_rs::{
    app::App,
    capability::{CapHookExt, Capability, HasCapTag},
    plugins::users::state::AuthContext,
    tag::Tagged,
    traits::add::{AddCapability, CapTagAbsent},
};
use sea_orm::DatabaseConnection;

type SectionLoader =
    Arc<dyn Fn(DatabaseConnection, i64, AuthContext) -> BoxFuture<'static, String> + Send + Sync>;

static SECTIONS: OnceLock<Vec<(u16, SectionLoader)>> = OnceLock::new();

pub struct CourseDetailRelatedTag;

#[derive(Clone)]
pub struct RelatedSection {
    pub order: u16,
    pub load: SectionLoader,
}

#[derive(Clone, Default)]
pub struct CourseDetailRelatedRegistry {
    sections: Vec<RelatedSection>,
}

impl CourseDetailRelatedRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(mut self, section: RelatedSection) -> Self {
        self.sections.push(section);
        self
    }
}

pub trait CourseDetailRelatedRegistrar: Sized {
    fn register_course_detail_related(
        self,
        cap: CourseDetailRelatedRegistry,
    ) -> CourseDetailRelatedRegistry;
}

#[derive(Clone, Default)]
pub struct CourseDetailRelatedCap<Hooks> {
    pub hooks: Hooks,
    pub items: CourseDetailRelatedRegistry,
    _tag: PhantomData<fn() -> CourseDetailRelatedTag>,
}

impl<Hooks> CourseDetailRelatedCap<Hooks> {
    pub fn new() -> Self
    where
        Hooks: Default,
    {
        Self {
            hooks: Hooks::default(),
            items: CourseDetailRelatedRegistry::new(),
            _tag: PhantomData,
        }
    }

    pub fn add_hook<HTag, H>(
        self,
        hook: H,
    ) -> CourseDetailRelatedCap<HCons<Tagged<HTag, H>, Hooks>> {
        CourseDetailRelatedCap {
            hooks: HCons {
                head: Tagged::new(hook),
                tail: self.hooks,
            },
            items: self.items,
            _tag: PhantomData,
        }
    }
}

impl<Hooks> HasCapTag for CourseDetailRelatedCap<Hooks> {
    type Tag = CourseDetailRelatedTag;
}

impl<Hooks, Plugin, Hook> CapHookExt<Plugin, Hook> for CourseDetailRelatedCap<Hooks> {
    type Hooked = CourseDetailRelatedCap<HCons<Tagged<Plugin, Hook>, Hooks>>;

    fn prepend_cap_hook(self, hook: Hook) -> Self::Hooked {
        self.add_hook::<Plugin, Hook>(hook)
    }
}

pub trait FoldRelatedRegistrarHooks {
    fn fold(self, reg: CourseDetailRelatedRegistry) -> CourseDetailRelatedRegistry;
}

impl FoldRelatedRegistrarHooks for HNil {
    fn fold(self, reg: CourseDetailRelatedRegistry) -> CourseDetailRelatedRegistry {
        reg
    }
}

impl<Plugin, H, Tail> FoldRelatedRegistrarHooks for HCons<Tagged<Plugin, H>, Tail>
where
    Tail: FoldRelatedRegistrarHooks,
    H: CourseDetailRelatedRegistrar + Copy,
{
    fn fold(self, reg: CourseDetailRelatedRegistry) -> CourseDetailRelatedRegistry {
        let reg = self.tail.fold(reg);
        self.head.value.register_course_detail_related(reg)
    }
}

impl<Hooks> Capability for CourseDetailRelatedCap<Hooks>
where
    Hooks: FoldRelatedRegistrarHooks,
{
    type Value = CourseDetailRelatedRegistry;
    type Output = Tagged<CourseDetailRelatedTag, CourseDetailRelatedRegistry>;
    type Hooks = Hooks;
    type Items = CourseDetailRelatedRegistry;

    fn mount(self) -> Self::Output {
        let registry = self.hooks.fold(self.items);
        let mut sections = registry.sections;
        sections.sort_by_key(|s| s.order);
        let loaders: Vec<(u16, SectionLoader)> =
            sections.into_iter().map(|s| (s.order, s.load)).collect();
        if SECTIONS.set(loaders).is_err() {
            tracing::error!("course detail related SECTIONS already initialized");
        }
        Tagged::new(CourseDetailRelatedRegistry::new())
    }
}

#[derive(Clone, Copy, Default)]
pub struct BaseHook;

impl CourseDetailRelatedRegistrar for BaseHook {
    fn register_course_detail_related(
        self,
        cap: CourseDetailRelatedRegistry,
    ) -> CourseDetailRelatedRegistry {
        cap
    }
}

pub async fn related_sections_html(
    db: &DatabaseConnection,
    course_id: i64,
    auth: &AuthContext,
) -> String {
    let Some(sections) = SECTIONS.get() else {
        return String::new();
    };
    let mut parts = Vec::with_capacity(sections.len());
    for (_, load) in sections {
        parts.push(load(db.clone(), course_id, auth.clone()).await);
    }
    parts.concat()
}

pub fn section<F, Fut>(order: u16, f: F) -> RelatedSection
where
    F: Fn(DatabaseConnection, i64, AuthContext) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = String> + Send + 'static,
{
    RelatedSection {
        order,
        load: Arc::new(move |db, id, auth| Box::pin(f(db, id, auth))),
    }
}

#[allow(dead_code)]
pub fn with_course_detail_related<L, Proof>(
    app: App<L>,
) -> App<HCons<CourseDetailRelatedCap<HNil>, L>>
where
    L: HList + CapTagAbsent<CourseDetailRelatedTag, Proof>,
{
    app.add_capability(CourseDetailRelatedCap::new())
}
