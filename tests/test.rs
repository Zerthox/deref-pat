use deref_pat::deref_pat;

#[derive(Debug)]
enum Tree {
    Node { left: Box<Tree>, right: Box<Tree> },
    Leaf,
}

fn tree() -> Tree {
    Tree::Node {
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
    }
}

#[test]
fn if_let_expr() {
    let x = deref_pat! {
        if let Tree::Node {
            left,
            #[deref]
                right:
                Tree::Node {
                    #[deref]
                        left: leaf @ Tree::Leaf,
                    right,
                },
        } = tree()
        {
            dbg!(left, leaf, right);
            true
        } else {
            false
        }
    };
}
