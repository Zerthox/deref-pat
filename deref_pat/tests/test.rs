use deref_pat::deref_pat;
use pretty_assertions::assert_eq;

#[test]
fn tree() {
    #[derive(Debug)]
    enum Tree {
        Node { left: Box<Tree>, right: Box<Tree> },
        Leaf,
    }

    let tree = Tree::Node {
        left: Tree::Node {
            left: Tree::Leaf.into(),
            right: Tree::Node {
                left: Tree::Leaf.into(),
                right: Tree::Leaf.into(),
            }
            .into(),
        }
        .into(),
        right: Tree::Leaf.into(),
    };

    let matches = deref_pat! {
        if let Tree::Node {
            #[deref]
                left:
                Tree::Node {
                    #[deref]
                        left: leaf @ Tree::Leaf,
                    right: sibling,
                },
                right: other_tree,
        } = tree
        {
            let _ = (other_tree, leaf, sibling);
            true
        } else {
            false
        }
    };
    assert_eq!(matches, true);
}

#[test]
fn vec() {
    struct Parent {
        vec: Vec<u32>,
    }

    let parent = Parent { vec: vec![0, 1, 2] };

    deref_pat! {
        if let Parent { #[deref] vec: [first, second, third] } = &parent {
            assert_eq!(*first, 0);
            assert_eq!(*second, 1);
            assert_eq!(*third, 2);
        } else {
            panic!("pattern did not match");
        }
    }
}

#[test]
fn string() {
    struct Parent {
        string: String,
    }

    let parent = Parent {
        string: "foo".into(),
    };

    deref_pat! {
        if let Parent { #[deref] string: bound @ "foo" } = &parent {
            assert_eq!(bound, "foo");
        } else {
            panic!("pattern did not match");
        }
    }
}

#[test]
fn mutate() {
    struct Parent {
        inner: Box<usize>,
    }

    let mut parent = Parent { inner: 123.into() };

    deref_pat! {
        if let Parent { #[deref] inner: inner @ 123 } = &mut parent {
            *inner = 456;
        } else {
            panic!("pattern did not match");
        }
    }

    assert_eq!(*parent.inner, 456);
}
