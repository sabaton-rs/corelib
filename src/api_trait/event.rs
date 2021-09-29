// trait for an Event published by an API
//

pub mod interface {
    //marker trait for Events
    pub trait EventType {}
    pub trait PropertyType {}
}

use crate::error::CoreError;

use self::interface::{EventType, PropertyType};

pub enum EventCacheUpdatePolicy {
    LastN(usize),
    NewestN(usize),
}

pub enum SubscriptionState {
    Unsubscribed,
    SubscriptionPending,
    Subscribed,
}

pub trait Event {
    type Item;

    fn subscribe(&mut self, policy: EventCacheUpdatePolicy) -> Result<(), CoreError>;
    fn get_subscription_state(&self) -> SubscriptionState;
    fn on_subscription_changed(&mut self, cb: Box<dyn FnMut(SubscriptionState)>);
    fn update(&mut self, filter: dyn Fn(&Self::Item)) -> bool;
    fn get_sample(&self) -> &[Self::Item];
    fn on_receive(&mut self, callback: dyn FnMut(&Self::Item));
    fn cancel_on_receive(&mut self) -> Result<(), CoreError>;
}

pub trait Property {
    type Item;
    fn subscribe(&mut self, policy: EventCacheUpdatePolicy) -> Result<(), CoreError>;
    fn get_subscription_state(&self) -> SubscriptionState;
    fn on_subscription_changed(&mut self, cb: Box<dyn FnMut(SubscriptionState)>);
    fn update(&mut self, filter: dyn Fn(&Self::Item)) -> bool;
    fn get_sample(&self) -> &[Self::Item];
    fn on_receive(&mut self, callback: dyn FnMut(&Self::Item));
    fn cancel_on_receive(&mut self) -> Result<(), CoreError>;
}

struct EventStruct {
    something: u32,
}
struct ExampleApi {
    event1: Box<dyn Event<Item = u32>>,
    event2: Box<dyn Event<Item = EventStruct>>,
    prop1: Box<dyn Property<Item = EventStruct>>,
}

use async_trait::async_trait;
#[async_trait]
pub trait AnApi {
    async fn hello(&self) -> Result<(), CoreError>;
}

impl ExampleApi {
    pub async fn hello(&self) -> Result<(), CoreError> {
        Err(CoreError::Unknown)
    }
}
