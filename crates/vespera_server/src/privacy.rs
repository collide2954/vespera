use bevy::prelude::*;
use bevy_replicon::prelude::*;
use vespera_protocol::ServerMotion;
use vespera_rules::pack::Pack;
use vespera_rules::skills::Skills;

pub struct PrivacyPlugin;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
#[component(immutable)]
pub struct PrivateTo(pub Entity);

impl VisibilityFilter for PrivateTo {
    type ClientComponent = AuthorizedClient;
    type Scope = (Pack, Skills, ServerMotion);

    fn is_visible(&self, client: Entity, _component: Option<&Self::ClientComponent>) -> bool {
        self.0 == client
    }
}

impl Plugin for PrivacyPlugin {
    fn build(&self, app: &mut App) {
        app.add_visibility_filter::<PrivateTo>();
    }
}
