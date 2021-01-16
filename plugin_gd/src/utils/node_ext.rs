use anyhow::{anyhow, Result};
use core::utils::try_option;
use gdnative::prelude::*;

pub trait NodeExt {
    unsafe fn scene_tree(&self) -> Result<TRef<'_, SceneTree, Shared>>;

    unsafe fn viewport(&self) -> Result<TRef<'_, Viewport, Shared>>;

    unsafe fn node<P>(&self, path: P) -> Result<Ref<Node, Shared>>
    where
        P: Into<NodePath>;

    unsafe fn node_tref<P>(&self, path: P) -> Result<TRef<'_, Node, Shared>>
    where
        P: Into<NodePath>;

    unsafe fn typed_node<T, P>(&self, path: P) -> Result<Ref<T, Shared>>
    where
        T: GodotObject + SubClass<Node>,
        P: Into<NodePath>;

    unsafe fn typed_node_tref<T, P>(&self, path: P) -> Result<TRef<'_, T, Shared>>
    where
        T: GodotObject + SubClass<Node>,
        P: Into<NodePath>;

    unsafe fn instance_ref<C, T, P>(&self, path: P) -> Result<RefInstance<'_, C, Shared>>
    where
        C: NativeClass<Base = T>,
        T: GodotObject + SubClass<Node>,
        P: Into<NodePath>;

    unsafe fn root_node<P>(&self, path: P) -> Result<Ref<Node, Shared>>
    where
        P: Into<NodePath>;

    unsafe fn root_node_tref<P>(&self, path: P) -> Result<TRef<'_, Node, Shared>>
    where
        P: Into<NodePath>;

    unsafe fn typed_root_node<T, P>(&self, path: P) -> Result<Ref<T, Shared>>
    where
        T: GodotObject + SubClass<Node>,
        P: Into<NodePath>;

    unsafe fn typed_root_node_tref<T, P>(&self, path: P) -> Result<TRef<'_, T, Shared>>
    where
        T: GodotObject + SubClass<Node>,
        P: Into<NodePath>;

    unsafe fn root_instance_ref<C, T, P>(&self, path: P) -> Result<RefInstance<'_, C, Shared>>
    where
        C: NativeClass<Base = T>,
        T: GodotObject + SubClass<Node>,
        P: Into<NodePath>;
}

macro_rules! node_ext {
    ($go:ty) => {
        impl NodeExt for $go {
            unsafe fn scene_tree(&self) -> Result<TRef<'_, SceneTree, Shared>> {
                return self
                    .get_tree()
                    .map(|scene| scene.assume_safe())
                    .ok_or(anyhow!("NodeExt::scene_tree()"));
            }

            unsafe fn viewport(&self) -> Result<TRef<'_, Viewport, Shared>> {
                return self
                    .get_viewport()
                    .map(|view| view.assume_safe())
                    .ok_or(anyhow!("NodeExt::viewport()"));
            }

            unsafe fn node<P>(&self, path: P) -> Result<Ref<Node, Shared>>
            where
                P: Into<NodePath>,
            {
                return self
                    .get_node(path.into())
                    .ok_or(anyhow!("NodeExt::node_ref()"));
            }

            unsafe fn node_tref<P>(&self, path: P) -> Result<TRef<'_, Node, Shared>>
            where
                P: Into<NodePath>,
            {
                return try_option(|| {
                    let node = self.get_node(path.into())?.assume_safe();
                    return Some(node);
                })
                .ok_or(anyhow!("NodeExt::node_tref()"));
            }

            unsafe fn typed_node<T, P>(&self, path: P) -> Result<Ref<T, Shared>>
            where
                T: GodotObject + SubClass<Node>,
                P: Into<NodePath>,
            {
                return try_option(|| {
                    let node = self
                        .get_node(path.into())?
                        .assume_safe()
                        .cast::<T>()?
                        .claim();
                    return Some(node);
                })
                .ok_or(anyhow!("NodeExt::typed_node()"));
            }

            unsafe fn typed_node_tref<T, P>(&self, path: P) -> Result<TRef<'_, T, Shared>>
            where
                T: GodotObject + SubClass<Node>,
                P: Into<NodePath>,
            {
                return try_option(|| {
                    let node = self.get_node(path.into())?.assume_safe().cast()?;
                    return Some(node);
                })
                .ok_or(anyhow!("NodeExt::typed_node_tref()"));
            }

            unsafe fn instance_ref<C, T, P>(&self, path: P) -> Result<RefInstance<'_, C, Shared>>
            where
                C: NativeClass<Base = T>,
                T: GodotObject + SubClass<Node>,
                P: Into<NodePath>,
            {
                return try_option(|| {
                    return self
                        .get_node(path.into())?
                        .assume_safe()
                        .cast::<T>()?
                        .cast_instance();
                })
                .ok_or(anyhow!("NodeExt::instance_ref()"));
            }

            unsafe fn root_node<P>(&self, path: P) -> Result<Ref<Node, Shared>>
            where
                P: Into<NodePath>,
            {
                return try_option(|| {
                    return self
                        .get_tree()?
                        .assume_safe()
                        .root()?
                        .assume_safe()
                        .get_node(path.into());
                })
                .ok_or(anyhow!("NodeExt::root_node()"));
            }

            unsafe fn root_node_tref<P>(&self, path: P) -> Result<TRef<'_, Node, Shared>>
            where
                P: Into<NodePath>,
            {
                return try_option(|| {
                    let node = self
                        .get_tree()?
                        .assume_safe()
                        .root()?
                        .assume_safe()
                        .get_node(path.into())?
                        .assume_safe();
                    return Some(node);
                })
                .ok_or(anyhow!("NodeExt::root_node_tref()"));
            }

            unsafe fn typed_root_node<T, P>(&self, path: P) -> Result<Ref<T, Shared>>
            where
                T: GodotObject + SubClass<Node>,
                P: Into<NodePath>,
            {
                return try_option(|| {
                    let node = self
                        .get_tree()?
                        .assume_safe()
                        .root()?
                        .assume_safe()
                        .get_node(path.into())?
                        .assume_safe()
                        .cast::<T>()?
                        .claim();
                    return Some(node);
                })
                .ok_or(anyhow!("NodeExt::typed_root_node()"));
            }

            unsafe fn typed_root_node_tref<T, P>(&self, path: P) -> Result<TRef<'_, T, Shared>>
            where
                T: GodotObject + SubClass<Node>,
                P: Into<NodePath>,
            {
                return try_option(|| {
                    return self
                        .get_tree()?
                        .assume_safe()
                        .root()?
                        .assume_safe()
                        .get_node(path.into())?
                        .assume_safe()
                        .cast::<T>();
                })
                .ok_or(anyhow!("NodeExt::typed_root_node()"));
            }

            unsafe fn root_instance_ref<C, T, P>(
                &self,
                path: P,
            ) -> Result<RefInstance<'_, C, Shared>>
            where
                C: NativeClass<Base = T>,
                T: GodotObject + SubClass<Node>,
                P: Into<NodePath>,
            {
                return try_option(|| {
                    return self
                        .get_tree()?
                        .assume_safe()
                        .root()?
                        .assume_safe()
                        .get_node(path.into())?
                        .assume_safe()
                        .cast::<T>()?
                        .cast_instance();
                })
                .ok_or(anyhow!("NodeExt::root_instance_ref()"));
            }
        }
    };
}

node_ext!(Node);
node_ext!(Spatial);
