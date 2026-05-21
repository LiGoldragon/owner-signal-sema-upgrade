//! Owner-only signal contract for `sema-upgrade` policy.
//!
//! The ordinary contract attempts upgrades. This contract is the owner
//! surface for registering compiled migrations and deciding whether an
//! upgrade range may run.

use nota_codec::{NotaEnum, NotaRecord};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use signal_frame::signal_channel;
use signal_sema::SemaObservation;
use signal_sema_upgrade::{Attempt, ComponentName, MigrationIdentifier, Version};

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum MigrationState {
    Enabled,
    Disabled,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Registration {
    pub component: ComponentName,
    pub source: Version,
    pub target: Version,
    pub migration: MigrationIdentifier,
    pub state: MigrationState,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct PolicyRange {
    pub component: ComponentName,
    pub source: Version,
    pub target: Version,
}

impl PolicyRange {
    pub fn new(component: ComponentName, source: Version, target: Version) -> Self {
        Self {
            component,
            source,
            target,
        }
    }
}

impl From<Attempt> for PolicyRange {
    fn from(attempt: Attempt) -> Self {
        Self {
            component: attempt.component,
            source: attempt.source,
            target: attempt.target,
        }
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum BlockReason {
    Unsafe,
    Superseded,
    NotReviewed,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub component: ComponentName,
    pub source: Version,
    pub target: Version,
    pub reason: BlockReason,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, PartialEq, Eq)]
pub enum Query {
    All,
    Component(ComponentName),
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct PolicyEntry {
    pub component: ComponentName,
    pub source: Version,
    pub target: Version,
    pub state: MigrationState,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct PolicyReported {
    pub entries: Vec<PolicyEntry>,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum PolicyRejectionReason {
    UnknownMigration,
    AlreadyRegistered,
    NotAllowed,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct PolicyRejected {
    pub component: ComponentName,
    pub source: Version,
    pub target: Version,
    pub reason: PolicyRejectionReason,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum UnimplementedReason {
    NotBuiltYet,
    IntegrationNotLanded,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RequestUnimplemented {
    pub reason: UnimplementedReason,
}

signal_channel! {
    channel OwnerSemaUpgrade {
        operation Register(Registration),
        operation Allow(PolicyRange),
        operation Block(Block),
        operation Query(Query),
    }
    reply Reply {
        Registered(Registration),
        Allowed(PolicyRange),
        Blocked(Block),
        PolicyReported(PolicyReported),
        PolicyRejected(PolicyRejected),
        RequestUnimplemented(RequestUnimplemented),
    }
    observable {
        filter default;
        operation_event OperationReceived;
        effect_event EffectEmitted;
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct OperationReceived {
    pub operation: OperationKind,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct EffectEmitted {
    pub observation: SemaObservation,
}
