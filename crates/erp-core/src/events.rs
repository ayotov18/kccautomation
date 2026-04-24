use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// A domain event emitted by any ERP subsystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    pub data: serde_json::Value,
    pub source: String,
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
}

impl Event {
    pub fn new(name: impl Into<String>, data: serde_json::Value, source: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data,
            source: source.into(),
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
        }
    }
}

type Handler = Arc<dyn Fn(Event) + Send + Sync>;

/// Async-safe event bus supporting pattern-based subscription.
pub struct EventBus {
    handlers: RwLock<HashMap<String, Vec<Handler>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
        }
    }

    /// Subscribe a handler to events matching `pattern` (exact match on event name).
    pub async fn subscribe<F>(&self, pattern: impl Into<String>, handler: F)
    where
        F: Fn(Event) + Send + Sync + 'static,
    {
        let mut map = self.handlers.write().await;
        map.entry(pattern.into())
            .or_default()
            .push(Arc::new(handler));
    }

    /// Publish an event, invoking all subscribed handlers synchronously.
    pub async fn publish(&self, event: Event) {
        let map = self.handlers.read().await;
        if let Some(handlers) = map.get(&event.name) {
            for handler in handlers {
                handler(event.clone());
            }
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_publish_subscribe() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        bus.subscribe("test.event", move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        })
        .await;

        let event = Event::new("test.event", serde_json::json!({}), "test");
        bus.publish(event).await;

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_no_handler_for_event() {
        let bus = EventBus::new();
        let event = Event::new("unhandled.event", serde_json::json!({}), "test");
        // Should not panic
        bus.publish(event).await;
    }
}
