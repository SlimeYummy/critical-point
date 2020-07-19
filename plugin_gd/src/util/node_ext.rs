use core::util::try_option;
use failure::{format_err, Error};
use gdnative::prelude::*;

pub trait NodeExt {
    unsafe fn scene_tree(&self) -> Result<TRef<'_, SceneTree, Shared>, Error>;

    unsafe fn viewport(&self) -> Result<TRef<'_, Viewport, Shared>, Error>;

    unsafe fn node<P>(&self, path: P) -> Result<TRef<'_, Node, Shared>, Error>
    where
        P: Into<NodePath>;

    unsafe fn typed_node<T, P>(&self, path: P) -> Result<TRef<'_, T, Shared>, Error>
    where
        T: GodotObject + SubClass<Node>,
        P: Into<NodePath>;

    unsafe fn root_node<P>(&self, path: P) -> Result<TRef<'_, Node, Shared>, Error>
    where
        P: Into<NodePath>;

    unsafe fn typed_root_node<T, P>(&self, path: P) -> Result<TRef<'_, T, Shared>, Error>
    where
        T: GodotObject + SubClass<Node>,
        P: Into<NodePath>;
}

impl NodeExt for Node {
    unsafe fn scene_tree(&self) -> Result<TRef<'_, SceneTree, Shared>, Error> {
        return self.get_tree()
            .map(|scene| scene.assume_safe())
            .ok_or(format_err!("NodeExt::scene_tree()"));
    }

    unsafe fn viewport(&self) -> Result<TRef<'_, Viewport, Shared>, Error> {
        return try_option(|| {
            let scene = self.get_tree()?.assume_safe();
            let view = scene.root()?.assume_safe();
            return Some(view);
        }).ok_or(format_err!("NodeExt::viewport()"));
    }

    unsafe fn node<P>(&self, path: P) -> Result<TRef<'_, Node, Shared>, Error>
    where
        P: Into<NodePath>
    {
        return try_option(|| Some(self.get_node(path.into())?.assume_safe()))
            .ok_or(format_err!("NodeExt::node()"));
    }

    unsafe fn typed_node<T, P>(&self, path: P) -> Result<TRef<'_, T, Shared>, Error>
    where
        T: GodotObject + SubClass<Node>,
        P: Into<NodePath>
    {
        return try_option(|| self.get_node(path.into())?.assume_safe().cast())
            .ok_or(format_err!("NodeExt::typed_node()"));
    }

    unsafe fn root_node<P>(&self, path: P) -> Result<TRef<'_, Node, Shared>, Error>
    where
        P: Into<NodePath>
    {
        return try_option(|| {
            return Some(
                self.get_tree()?.assume_safe()
                    .root()?.assume_safe()
                    .get_node(path.into())?.assume_safe()
            );
        }).ok_or(format_err!("NodeExt::root_node()"));
    }

    unsafe fn typed_root_node<T, P>(&self, path: P) -> Result<TRef<'_, T, Shared>, Error>
    where
        T: GodotObject + SubClass<Node>,
        P: Into<NodePath>
    {
        return try_option(|| {
            return self.get_tree()?.assume_safe()
                .root()?.assume_safe()
                .get_node(path.into())?.assume_safe()
                .cast::<T>();
        }).ok_or(format_err!("NodeExt::typed_root_node()"));
    }
}

pub unsafe fn node_from_root<P>(
    tree: Option<Ref<SceneTree>>,
    path: P,
) -> Result<Ref<Node>, Error>
where
    P: Into<NodePath>,
{
    return try_option(|| {
        return tree?
            .assume_safe()
            .root()?
            .assume_safe()
            .get_node(path.into());
    })
        .ok_or(format_err!("node_from_root() => failed"));
}
