use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode};
use owner_signal_sema_upgrade::{
    Block, BlockReason, Frame, FrameBody, MigrationState, Operation, PolicyEntry, PolicyRange,
    PolicyReported, Query, Registration, Reply,
};
use signal_frame::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, NonEmpty, Reply as FrameReply, RequestPayload,
    SessionEpoch, SubReply,
};
use signal_sema_upgrade::{ComponentName, MigrationIdentifier, Version};

const CANONICAL: &str = include_str!("../examples/canonical.nota");

fn exchange() -> ExchangeIdentifier {
    ExchangeIdentifier::new(
        SessionEpoch::new(1),
        ExchangeLane::Connector,
        LaneSequence::first(),
    )
}

fn component() -> ComponentName {
    ComponentName::new("persona-spirit")
}

fn source() -> Version {
    Version::new(0, 1, 0)
}

fn target() -> Version {
    Version::new(0, 1, 1)
}

fn migration_identifier() -> MigrationIdentifier {
    MigrationIdentifier::new("persona-spirit-0-1-0-to-0-1-1")
}

fn registration() -> Registration {
    Registration {
        component: component(),
        source: source(),
        target: target(),
        migration: migration_identifier(),
        state: MigrationState::Enabled,
    }
}

fn range() -> PolicyRange {
    PolicyRange::new(component(), source(), target())
}

fn round_trip_request(operation: Operation) -> Operation {
    let frame = Frame::new(FrameBody::Request {
        exchange: exchange(),
        request: operation.clone().into_request(),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = Frame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        FrameBody::Request { request, .. } => request.payloads().head().clone(),
        other => panic!("expected request frame, got {other:?}"),
    }
}

fn round_trip_reply(reply: Reply) -> Reply {
    let frame = Frame::new(FrameBody::Reply {
        exchange: exchange(),
        reply: FrameReply::committed(NonEmpty::single(SubReply::Ok(reply.clone()))),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = Frame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        FrameBody::Reply { reply, .. } => match reply {
            FrameReply::Accepted { per_operation, .. } => match per_operation.into_head() {
                SubReply::Ok(payload) => payload,
                other => panic!("expected accepted reply payload, got {other:?}"),
            },
            other => panic!("expected accepted frame reply, got {other:?}"),
        },
        other => panic!("expected reply frame, got {other:?}"),
    }
}

fn round_trip_nota<T>(value: T, expected: &str)
where
    T: NotaEncode + NotaDecode + PartialEq + std::fmt::Debug,
{
    let mut encoder = Encoder::new();
    value.encode(&mut encoder).expect("encode nota");
    let encoded = encoder.into_string();
    assert_eq!(encoded, expected);

    let mut decoder = Decoder::new(&encoded);
    let recovered = T::decode(&mut decoder).expect("decode nota");
    assert_eq!(recovered, value);
    assert!(
        CANONICAL.contains(expected),
        "examples/canonical.nota missing line: {expected}"
    );
}

#[test]
fn owner_requests_round_trip_through_signal_frames() {
    let operations = [
        Operation::Register(registration()),
        Operation::Allow(range()),
        Operation::Block(Block {
            component: component(),
            source: source(),
            target: Version::new(0, 1, 2),
            reason: BlockReason::Unsafe,
        }),
        Operation::Query(Query::All),
    ];

    for operation in operations {
        assert_eq!(round_trip_request(operation.clone()), operation);
    }
}

#[test]
fn owner_replies_round_trip_through_signal_frames() {
    let replies = [
        Reply::Registered(registration()),
        Reply::Allowed(range()),
        Reply::Blocked(Block {
            component: component(),
            source: source(),
            target: Version::new(0, 1, 2),
            reason: BlockReason::Unsafe,
        }),
        Reply::PolicyReported(PolicyReported {
            entries: vec![PolicyEntry {
                component: component(),
                source: source(),
                target: target(),
                state: MigrationState::Enabled,
            }],
        }),
    ];

    for reply in replies {
        assert_eq!(round_trip_reply(reply.clone()), reply);
    }
}

#[test]
fn owner_canonical_nota_examples_round_trip() {
    round_trip_nota(
        Operation::Register(registration()),
        "(Register (persona-spirit (0 1 0) (0 1 1) persona-spirit-0-1-0-to-0-1-1 Enabled))",
    );
    round_trip_nota(
        Operation::Allow(range()),
        "(Allow (persona-spirit (0 1 0) (0 1 1)))",
    );
    round_trip_nota(
        Operation::Block(Block {
            component: component(),
            source: source(),
            target: Version::new(0, 1, 2),
            reason: BlockReason::Unsafe,
        }),
        "(Block (persona-spirit (0 1 0) (0 1 2) Unsafe))",
    );
    round_trip_nota(Operation::Query(Query::All), "(Query All)");
    round_trip_nota(
        Reply::Registered(registration()),
        "(Registered (persona-spirit (0 1 0) (0 1 1) persona-spirit-0-1-0-to-0-1-1 Enabled))",
    );
    round_trip_nota(
        Reply::Allowed(range()),
        "(Allowed (persona-spirit (0 1 0) (0 1 1)))",
    );
    round_trip_nota(
        Reply::Blocked(Block {
            component: component(),
            source: source(),
            target: Version::new(0, 1, 2),
            reason: BlockReason::Unsafe,
        }),
        "(Blocked (persona-spirit (0 1 0) (0 1 2) Unsafe))",
    );
    round_trip_nota(
        Reply::PolicyReported(PolicyReported {
            entries: vec![PolicyEntry {
                component: component(),
                source: source(),
                target: target(),
                state: MigrationState::Enabled,
            }],
        }),
        "(PolicyReported ([(persona-spirit (0 1 0) (0 1 1) Enabled)]))",
    );
}
