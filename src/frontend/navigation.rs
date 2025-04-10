use anyhow::{anyhow, Result};
use bevy::{ecs::system::SystemParam, prelude::*};

pub mod prelude {
    pub use super::{is_focused, Navigation};
}

pub fn plugin(app: &mut App) {
    app.add_observer(gc_popups);
}

/// A system parameter for navigation. This is used to focus on entities and
/// navigate between them.
///
/// ## Example
///
/// ```rust
/// pub fn navigate(mut navigation: Navigation, state: Res<State>) {
/// 	...
/// }
/// ```
#[derive(SystemParam)]
pub struct Navigation<'w, 's> {
    breadcrumbs: Query<'w, 's, (Entity, &'static Breadcrumb)>,
    root: Query<'w, 's, Entity, With<NavigationRoot>>,
    focused: Query<'w, 's, Entity, With<Focus>>,
    commands: Commands<'w, 's>,
}

impl Navigation<'_, '_> {
    /// Returns whether the given entity is currently focused. Only one entity
    /// can be focused at a given time.
    pub fn is_focused(&self, entity: Entity) -> bool {
        self.focused.get(entity).is_ok()
    }

    /// Spawns the given bundle as a popup and focuses it. Popups are despawned
    /// when they lose focus.
    pub fn spawn_popup(&mut self, bundle: impl Bundle) -> Result<()> {
        let popup = self.commands.spawn((bundle, Popup)).id();
        self.focus(popup)?;
        Ok(())
    }

    /// Assigns the given entity as the navigation root. Only one entity can be
    /// the navigation root at a given time.
    pub fn focus_as_root(&mut self, entity: Entity) {
        self.reset_stack();

        for root in self.root.iter_mut() {
            self.commands.entity(root).remove::<NavigationRoot>();
        }

        self.commands.entity(entity).insert((NavigationRoot, Focus));
    }

    /// Focuses on the given entity. Any other entities which are currently
    /// focused or have breadcrumbs will be removed from the navigation stack.
    pub fn focus(&mut self, entity: Entity) -> Result<()> {
        if self.is_focused(entity) {
            return Ok(());
        }

        let root = self.root()?;
        self.reset_stack();

        let mut entity_commands = self.commands.entity(entity);

        entity_commands.insert(Focus);
        if entity != root {
            entity_commands.insert(Breadcrumb(root));
        }

        Ok(())
    }

    /// Navigate to the given entity. This will remove the current focus and add a
    /// breadcrumb to the current entity.
    ///
    /// ## Errors
    ///
    /// - Fails if there is no currently focused entity.
    pub fn go_to(&mut self, entity: Entity) -> Result<()> {
        let current = self.focused()?;

        (self.commands.entity(current))
            .remove::<Focus>()
            .insert(Breadcrumb(entity));
        self.commands.entity(entity).insert(Focus);

        Ok(())
    }

    /// Navigate back to the previous entity. This will remove the current focus
    /// and remove the breadcrumb from the current entity. If there is no
    /// breadcrumb, nothing happens.
    ///
    /// ## Errors
    ///
    /// - Fails if there is no currently focused entity.
    pub fn go_back(&mut self) -> Result<()> {
        let current = self.focused()?;

        if let Ok((_, breadcrumb)) = self.breadcrumbs.get(current) {
            self.commands
                .entity(current)
                .remove::<(Breadcrumb, Focus)>();
            self.commands.entity(breadcrumb.0).insert(Focus);
        }

        Ok(())
    }

    /// Removes focus and breadcrumbs from everything in the world.
    fn reset_stack(&mut self) {
        for focused in self.focused.iter_mut() {
            self.commands.entity(focused).remove::<Focus>();
        }

        for (entity, _) in self.breadcrumbs.iter_mut() {
            self.commands.entity(entity).remove::<Breadcrumb>();
        }
    }

    /// Returns the currently focused entity, of which there must be exactly
    /// one.
    fn focused(&self) -> Result<Entity> {
        self.focused
            .get_single()
            .map_err(|_| anyhow!("Zero or multiple focused entities found. This shouldn't happen."))
    }

    /// Returns the current navigation root, of which there must be exactly
    /// one.
    fn root(&self) -> Result<Entity> {
        self.root
            .get_single()
            .map_err(|_| anyhow!("Zero or multiple navigation roots found. This shouldn't happen."))
    }
}

/// Run condition that returns whether there is any entities with the given
/// component currently focused. Use this with a unique marker component so
/// you don't get false positives.
///
/// ## Example
///
/// ```rust
/// use crate::navigation::prelude::*;
///
/// #[derive(Component)]
/// pub struct SpaceMenu;
///
/// pub fn plugin(app: &mut App) {
/// 	app.add_systems(Update, read_keys.run_if(is_focused::<SpaceMenu>));
/// }
///
/// fn read_keys(ev_keypress: EventReader<KeyEvent>) { ... }
/// ```
pub fn is_focused<C: Component>(query: Query<Entity, (With<C>, With<Focus>)>) -> bool {
    !query.is_empty()
}

/// The root of the navigation stack. Manual focusing will always be based off
/// this entity.
#[derive(Component)]
pub struct NavigationRoot;

/// The currently focused entity. This component will always be on exactly one
/// entity in the world.
#[derive(Component)]
pub struct Focus;

/// The previous entity in the navigation stack. Every entity in the navigation
/// stack that is not [`NavigationRoot`] will have this component.
#[derive(Component, Deref, DerefMut)]
pub struct Breadcrumb(pub Entity);

/// A popup entity. Popups are despawned once they lose focus.
#[derive(Component)]
pub struct Popup;

fn gc_popups(
    unfocused: Trigger<OnRemove, Focus>,
    popups: Query<Entity, With<Popup>>,
    mut commands: Commands,
) {
    if let Ok(popup) = popups.get(unfocused.entity()) {
        commands.entity(popup).despawn_recursive();
    }
}
