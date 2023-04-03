use super::{
    builtin_subscriber::BuiltinSubscriber, listener_subscriber::ListenerSubscriber,
    user_defined_subscriber::UserDefinedSubscriber,
};

#[derive(PartialEq, Debug)]
pub enum SubscriberKind {
    BuiltIn(BuiltinSubscriber),
    UserDefined(UserDefinedSubscriber),
    Listener(ListenerSubscriber),
}
