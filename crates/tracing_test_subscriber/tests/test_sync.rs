use tracing::{Level, event, span};
use tracing_test_subscriber::TestSubscriber;

#[test]
fn test_sync() {
    let subscriber = TestSubscriber::default();

    tracing::subscriber::with_default(subscriber.clone(), || {
        event!(Level::DEBUG, "root");

        let outer = span!(Level::TRACE, "outer");
        let _guard = outer.enter();

        event!(Level::DEBUG, "first");

        {
            let inner = span!(Level::TRACE, "inner");
            let _guard = inner.enter();

            event!(Level::DEBUG, "second");
        }

        event!(Level::DEBUG, "third");
    });

    let history = subscriber.history();

    assert_eq!(history.root_events.len(), 1);
    assert_eq!(history.root_events[0].fields["message"], "root");
    assert_eq!(*history.root_events[0].metadata.level(), Level::DEBUG);
    assert_eq!(history.spans.len(), 1);

    let outer = &history.spans[0];
    assert_eq!(outer.attributes.metadata.name(), "outer");
    assert_eq!(*outer.attributes.metadata.level(), Level::TRACE);
    assert_eq!(outer.events.len(), 2);
    assert_eq!(outer.events[0].fields["message"], "first");
    assert_eq!(*outer.events[0].metadata.level(), Level::DEBUG);
    assert_eq!(outer.events[1].fields["message"], "third");
    assert_eq!(*outer.events[1].metadata.level(), Level::DEBUG);
    assert_eq!(outer.spans.len(), 1);

    let inner = &outer.spans[0];
    assert_eq!(inner.attributes.metadata.name(), "inner");
    assert_eq!(*inner.attributes.metadata.level(), Level::TRACE);
    assert_eq!(inner.events.len(), 1);
    assert_eq!(inner.events[0].fields["message"], "second");
    assert_eq!(*inner.events[0].metadata.level(), Level::DEBUG);
    assert!(inner.spans.is_empty());
}
