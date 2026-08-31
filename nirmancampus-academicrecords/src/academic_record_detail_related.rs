//! Academic-record-detail related-sections hub — satellites register async HTML loaders via `cap_hook`.

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

pub struct AcademicRecordDetailRelatedTag;

#[derive(Clone)]
pub struct RelatedSection {
    pub order: u16,
    pub load: SectionLoader,
}

#[derive(Clone, Default)]
pub struct AcademicRecordDetailRelatedRegistry {
    sections: Vec<RelatedSection>,
}

impl AcademicRecordDetailRelatedRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(mut self, section: RelatedSection) -> Self {
        self.sections.push(section);
        self
    }
}

pub trait AcademicRecordDetailRelatedRegistrar: Sized {
    fn register_academic_record_detail_related(
        self,
        cap: AcademicRecordDetailRelatedRegistry,
    ) -> AcademicRecordDetailRelatedRegistry;
}

#[derive(Clone, Default)]
pub struct AcademicRecordDetailRelatedCap<Hooks> {
    pub hooks: Hooks,
    pub items: AcademicRecordDetailRelatedRegistry,
    _tag: PhantomData<fn() -> AcademicRecordDetailRelatedTag>,
}

impl<Hooks> AcademicRecordDetailRelatedCap<Hooks> {
    pub fn new() -> Self
    where
        Hooks: Default,
    {
        Self {
            hooks: Hooks::default(),
            items: AcademicRecordDetailRelatedRegistry::new(),
            _tag: PhantomData,
        }
    }

    pub fn add_hook<HTag, H>(
        self,
        hook: H,
    ) -> AcademicRecordDetailRelatedCap<HCons<Tagged<HTag, H>, Hooks>> {
        AcademicRecordDetailRelatedCap {
            hooks: HCons {
                head: Tagged::new(hook),
                tail: self.hooks,
            },
            items: self.items,
            _tag: PhantomData,
        }
    }
}

impl<Hooks> HasCapTag for AcademicRecordDetailRelatedCap<Hooks> {
    type Tag = AcademicRecordDetailRelatedTag;
}

impl<Hooks, Plugin, Hook> CapHookExt<Plugin, Hook> for AcademicRecordDetailRelatedCap<Hooks> {
    type Hooked = AcademicRecordDetailRelatedCap<HCons<Tagged<Plugin, Hook>, Hooks>>;

    fn prepend_cap_hook(self, hook: Hook) -> Self::Hooked {
        self.add_hook::<Plugin, Hook>(hook)
    }
}

pub trait FoldRelatedRegistrarHooks {
    fn fold(self, reg: AcademicRecordDetailRelatedRegistry) -> AcademicRecordDetailRelatedRegistry;
}

impl FoldRelatedRegistrarHooks for HNil {
    fn fold(self, reg: AcademicRecordDetailRelatedRegistry) -> AcademicRecordDetailRelatedRegistry {
        reg
    }
}

impl<Plugin, H, Tail> FoldRelatedRegistrarHooks for HCons<Tagged<Plugin, H>, Tail>
where
    Tail: FoldRelatedRegistrarHooks,
    H: AcademicRecordDetailRelatedRegistrar + Copy,
{
    fn fold(self, reg: AcademicRecordDetailRelatedRegistry) -> AcademicRecordDetailRelatedRegistry {
        let reg = self.tail.fold(reg);
        self.head.value.register_academic_record_detail_related(reg)
    }
}

impl<Hooks> Capability for AcademicRecordDetailRelatedCap<Hooks>
where
    Hooks: FoldRelatedRegistrarHooks,
{
    type Value = AcademicRecordDetailRelatedRegistry;
    type Output = Tagged<AcademicRecordDetailRelatedTag, AcademicRecordDetailRelatedRegistry>;
    type Hooks = Hooks;
    type Items = AcademicRecordDetailRelatedRegistry;

    fn mount(self) -> Self::Output {
        let registry = self.hooks.fold(self.items);
        let mut sections = registry.sections;
        sections.sort_by_key(|s| s.order);
        let loaders: Vec<(u16, SectionLoader)> =
            sections.into_iter().map(|s| (s.order, s.load)).collect();
        if SECTIONS.set(loaders).is_err() {
            tracing::error!("academic record detail related SECTIONS already initialized");
        }
        Tagged::new(AcademicRecordDetailRelatedRegistry::new())
    }
}

#[derive(Clone, Copy, Default)]
pub struct BaseHook;

impl AcademicRecordDetailRelatedRegistrar for BaseHook {
    fn register_academic_record_detail_related(
        self,
        cap: AcademicRecordDetailRelatedRegistry,
    ) -> AcademicRecordDetailRelatedRegistry {
        cap
    }
}

pub async fn related_sections_html(
    db: &DatabaseConnection,
    academic_record_id: i64,
    auth: &AuthContext,
) -> String {
    let Some(sections) = SECTIONS.get() else {
        return String::new();
    };
    let mut parts = Vec::with_capacity(sections.len());
    for (_, load) in sections {
        parts.push(load(db.clone(), academic_record_id, auth.clone()).await);
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
pub fn with_academic_record_detail_related<L, Proof>(
    app: App<L>,
) -> App<HCons<AcademicRecordDetailRelatedCap<HNil>, L>>
where
    L: HList + CapTagAbsent<AcademicRecordDetailRelatedTag, Proof>,
{
    app.add_capability(AcademicRecordDetailRelatedCap::new())
}
