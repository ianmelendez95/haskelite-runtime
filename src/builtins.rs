use std::fmt;
use std::fmt::Formatter;
use std::rc::Rc;
use std::cell::RefCell;
use std::cell::RefMut;

#[derive(Clone)]
pub enum Node {
    Int(i64),
    ThunkRef(Rc<RefCell<Thunk>>)
}

impl Node {
    pub fn eval(self) -> Node {
        self.reduce();

        match self {
            Node::Int(_) => self,
            Node::ThunkRef(t_ref) => {
                match &*t_ref.borrow() {
                    Thunk::UThunk(_) => panic!(),
                    Thunk::EThunk(value) => {
                        value.clone()
                    }
                }
            }
        }
    }

    fn reduce(&self) {
        if let Node::ThunkRef(t_ref) = self {
            RefMut::map(t_ref.as_ref().borrow_mut(), |t_mut| {
                if let Thunk::UThunk(eval) = t_mut {
                    *t_mut = Thunk::EThunk(eval.eval());
                    t_mut
                } else {
                    t_mut  // noop
                }
            });
        }
    }
}

pub enum Thunk {
    UThunk(Box<dyn ThunkEval>),
    EThunk(Node)
}

pub trait ThunkEval {
    fn eval(&self) -> Node;
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Node::Int(i) => {
                write!(f, "{}", i)
            }
            Node::ThunkRef(t) => {
                unreachable!("Asked to display thunk: {:?}", (*t).borrow());
            }, 
        }
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Node::Int(i) => {
                write!(f, "Int({})", i)
            }, 
            Node::ThunkRef(t) => {
                write!(f, "{:?}", (*t).borrow())
            }
        }
    }
}

impl fmt::Debug for Thunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Thunk::UThunk(_) => write!(f, "#[UNEVALED]"),
            Thunk::EThunk(val) => write!(f, "#[{:?}]", val),
        }
    }
}


/* ***************** *
 * Builtin Functions *
 * ***************** */


pub fn int(int_val: i64) -> Node {
    return Node::Int(int_val);
}

pub fn thunk(boxed_t: Box<dyn ThunkEval>) -> Node {
    return Node::ThunkRef(Rc::new(RefCell::new(Thunk::UThunk(boxed_t))));
}

macro_rules! bin_arith {
    ($nl:ident, $nr:ident, $op:tt) => {
        let vl: Node = $nl.eval();
        let vr: Node = $nr.eval();

        if let Node::Int(vl) = vl {
            if let Node::Int(vr) = vr {
                return Node::Int(vl $op vr);
            } else {
                panic!("Expecting integer for right operand: {:?}", vr)
            }
        } else {
            panic!("Expecting integer for left operand: {:?}", vl)
        }
    };
}

macro_rules! bin_thunk {
    ($thunk_name:ident, $eval_fn:ident, $fn_name:ident) => {
        struct $thunk_name {
            nl: Node,
            nr: Node
        }

        impl ThunkEval for $thunk_name {
            fn eval(&self) -> Node {
                $eval_fn(self.nl.clone(), self.nr.clone())
            }
        }

        pub fn $fn_name(nl: Node, nr: Node) -> Node {
            thunk(Box::new($thunk_name { nl, nr }))
        }
    }
}

bin_thunk!(AddThunk, eval_add, add);
bin_thunk!(SubThunk, eval_sub, sub);
bin_thunk!(DivThunk, eval_div, div);
bin_thunk!(MulThunk, eval_mul, mul);


fn eval_add(nl: Node, nr: Node) -> Node {
    bin_arith!(nl, nr, +);
}

fn eval_sub(nl: Node, nr: Node) -> Node {
    bin_arith!(nl, nr, -);
}

fn eval_div(nl: Node, nr: Node) -> Node {
    bin_arith!(nl, nr, /);
}

fn eval_mul(nl: Node, nr: Node) -> Node {
    bin_arith!(nl, nr, *);
}