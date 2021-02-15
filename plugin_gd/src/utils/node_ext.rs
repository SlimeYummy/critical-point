use anyhow::{anyhow, Result};
use gdnative::prelude::*;

pub trait NodeExt {
    fn scene_tree(&self) -> Result<Ref<SceneTree, Shared>>;

    #[inline]
    unsafe fn scene_tree_tref(&self) -> Result<TRef<'_, SceneTree, Shared>> {
        let scene_ref = self.scene_tree()?;
        let scene_tref = scene_ref.assume_safe();
        return Ok(scene_tref);
    }

    fn viewport(&self) -> Result<Ref<Viewport, Shared>>;

    #[inline]
    unsafe fn viewport_tref(&self) -> Result<TRef<'_, Viewport, Shared>> {
        let viewport_ref = self.viewport()?;
        let viewport_tref = viewport_ref.assume_safe();
        return Ok(viewport_tref);
    }

    fn node<P>(&self, path: P) -> Result<Ref<Node, Shared>>
    where
        P: Into<NodePath>;

    #[inline]
    unsafe fn node_tref<P>(&self, path: P) -> Result<TRef<'_, Node, Shared>>
    where
        P: Into<NodePath>,
    {
        let node_ref = self.node(path)?;
        let node_tref = node_ref.assume_safe();
        return Ok(node_tref);
    }

    #[inline]
    unsafe fn typed_node<T, P>(&self, path: P) -> Result<Ref<T, Shared>>
    where
        T: GodotObject + SubClass<Node>,
        P: Into<NodePath>,
    {
        let typed_tref = self.typed_node_tref::<T, P>(path)?;
        let typed_ref = typed_tref.claim();
        return Ok(typed_ref);
    }

    #[inline]
    unsafe fn typed_node_tref<T, P>(&self, path: P) -> Result<TRef<'_, T, Shared>>
    where
        T: GodotObject + SubClass<Node>,
        P: Into<NodePath>,
    {
        let node_tref = self.node_tref(path)?;
        let typed_tref = node_tref.cast::<T>().ok_or(anyhow!("Node::cast()"))?;
        return Ok(typed_tref);
    }

    #[inline]
    unsafe fn instance<C, T, P>(&self, path: P) -> Result<Instance<C, Shared>>
    where
        C: NativeClass<Base = T>,
        T: GodotObject + SubClass<Node>,
        P: Into<NodePath>,
    {
        let instance_tref = self.instance_ref(path)?;
        let instance_ref = instance_tref.claim();
        return Ok(instance_ref);
    }

    #[inline]
    unsafe fn instance_ref<C, T, P>(&self, path: P) -> Result<RefInstance<'_, C, Shared>>
    where
        C: NativeClass<Base = T>,
        T: GodotObject + SubClass<Node>,
        P: Into<NodePath>,
    {
        let typed_tref = self.typed_node_tref::<T, P>(path)?;
        let instance_tref = typed_tref
            .cast_instance()
            .ok_or(anyhow!("TRef::cast_instance()"))?;
        return Ok(instance_tref);
    }

    #[inline]
    unsafe fn root_node<P>(&self, path: P) -> Result<Ref<Node, Shared>>
    where
        P: Into<NodePath>,
    {
        let root_tref = self.viewport_tref()?;
        let node_ref = root_tref.node(path)?;
        return Ok(node_ref);
    }

    #[inline]
    unsafe fn root_node_tref<P>(&self, path: P) -> Result<TRef<'_, Node, Shared>>
    where
        P: Into<NodePath>,
    {
        let root_tref = self.viewport_tref()?;
        let node_ref = root_tref.node(path)?;
        let node_tref = node_ref.assume_safe();
        return Ok(node_tref);
    }

    #[inline]
    unsafe fn root_typed_node<T, P>(&self, path: P) -> Result<Ref<T, Shared>>
    where
        T: GodotObject + SubClass<Node>,
        P: Into<NodePath>,
    {
        let root_tref = self.viewport_tref()?;
        let typed_ref = root_tref.typed_node(path)?;
        return Ok(typed_ref);
    }

    #[inline]
    unsafe fn root_typed_node_tref<T, P>(&self, path: P) -> Result<TRef<'_, T, Shared>>
    where
        T: GodotObject + SubClass<Node>,
        P: Into<NodePath>,
    {
        let root_tref = self.viewport_tref()?;
        let node_ref = root_tref.node(path)?;
        let node_tref = node_ref.assume_safe();
        let typed_tref = node_tref.cast::<T>().ok_or(anyhow!("Node::cast()"))?;
        return Ok(typed_tref);
    }
}

impl NodeExt for Node {
    #[inline]
    fn scene_tree(&self) -> Result<Ref<SceneTree, Shared>> {
        return self.get_tree().ok_or(anyhow!("Node::get_tree()"));
    }

    #[inline]
    fn viewport(&self) -> Result<Ref<Viewport, Shared>> {
        return self.get_viewport().ok_or(anyhow!("Node::get_viewport()"));
    }

    #[inline]
    fn node<P>(&self, path: P) -> Result<Ref<Node, Shared>>
    where
        P: Into<NodePath>,
    {
        return self
            .get_node(path.into())
            .ok_or(anyhow!("Node::get_node()"));
    }
}

impl NodeExt for Spatial {
    #[inline]
    fn scene_tree(&self) -> Result<Ref<SceneTree, Shared>> {
        return self.get_tree().ok_or(anyhow!("Spatial::get_tree()"));
    }

    #[inline]
    fn viewport(&self) -> Result<Ref<Viewport, Shared>> {
        return self
            .get_viewport()
            .ok_or(anyhow!("Spatial::get_viewport()"));
    }

    #[inline]
    fn node<P>(&self, path: P) -> Result<Ref<Node, Shared>>
    where
        P: Into<NodePath>,
    {
        return self
            .get_node(path.into())
            .ok_or(anyhow!("Spatial::get_node()"));
    }
}
