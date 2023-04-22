use std::ops::{Add, Mul};

use tracing::metadata::LevelFilter;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

use crate::{BasicAlgebraicExpr, SimpleExpr};

mod derivative;
mod parse;
mod variables;

#[derive(Debug, Clone)]
pub enum TestExpr {
    LazySym(&'static str, bool),
    Simplified(SimpleExpr),
    Basic(BasicAlgebraicExpr),
}

impl TestExpr {
    pub fn simplify(self) -> SimpleExpr {
        match self {
            TestExpr::LazySym(s, _) => SimpleExpr::new_symbol(s.to_string()),
            TestExpr::Simplified(a) => a,
            TestExpr::Basic(b) => b.simplify().unwrap(),
        }
    }

    pub fn mk_simple(b: BasicAlgebraicExpr) -> Self {
        Self::Simplified(SimpleExpr::assume_simplified(b))
    }
}

impl PartialEq for TestExpr {
    fn eq(&self, other: &Self) -> bool {
        self.clone().simplify() == other.clone().simplify()
    }
}

fn mk_expr(is_simple: bool) -> impl Fn(BasicAlgebraicExpr) -> TestExpr {
    if is_simple {
        |a| TestExpr::Simplified(SimpleExpr::assume_simplified(a))
    } else {
        |a| TestExpr::Basic(a)
    }
}

macro_rules! impl_op {
    ($Tr:ident($fn_name:ident), $variant:ident) => {
        impl $Tr<i128> for TestExpr {
            type Output = TestExpr;
            fn $fn_name(self, rhs: i128) -> Self::Output {
                match self {
                    TestExpr::LazySym(st, is_simple) => {
                        mk_expr(is_simple)(BasicAlgebraicExpr::$variant(vec![s(st), n(rhs)]))
                    }
                    TestExpr::Simplified(s) => {
                        TestExpr::mk_simple(BasicAlgebraicExpr::$variant(vec![
                            s.into_inner(),
                            n(rhs),
                        ]))
                    }
                    TestExpr::Basic(b) => {
                        TestExpr::Basic(BasicAlgebraicExpr::$variant(vec![b, n(rhs)]))
                    }
                }
            }
        }

        impl $Tr<TestExpr> for i128 {
            type Output = TestExpr;
            fn $fn_name(self, rhs: TestExpr) -> Self::Output {
                match rhs {
                    TestExpr::LazySym(st, is_simple) => {
                        mk_expr(is_simple)(BasicAlgebraicExpr::$variant(vec![n(self), s(st)]))
                    }
                    TestExpr::Simplified(s) => {
                        TestExpr::mk_simple(BasicAlgebraicExpr::$variant(vec![
                            n(self),
                            s.into_inner(),
                        ]))
                    }
                    TestExpr::Basic(b) => {
                        TestExpr::Basic(BasicAlgebraicExpr::$variant(vec![n(self), b]))
                    }
                }
            }
        }

        impl $Tr for TestExpr {
            type Output = TestExpr;

            fn $fn_name(self, rhs: Self) -> Self::Output {
                match (self, rhs) {
                    (TestExpr::LazySym(st, is_simple), TestExpr::LazySym(st2, is_simple2)) => {
                        if is_simple != is_simple2 {
                            panic!(
                                "Cannot add two lazy symbols with different simplification status"
                            );
                        }
                        if is_simple || is_simple2 {
                            TestExpr::mk_simple(BasicAlgebraicExpr::$variant(vec![s(st), s(st2)]))
                        } else {
                            TestExpr::Basic(s(st).$fn_name(s(st2)))
                        }
                    }
                    (TestExpr::LazySym(st, is_simple), TestExpr::Simplified(simplified))
                    | (TestExpr::Simplified(simplified), TestExpr::LazySym(st, is_simple)) => {
                        if is_simple {
                            TestExpr::mk_simple(BasicAlgebraicExpr::$variant(vec![
                                s(st),
                                simplified.into_inner(),
                            ]))
                        } else {
                            panic!(
                                "lazy_sym {{ is_simple: true }} can only add with simplified exprs"
                            );
                        }
                    }
                    (TestExpr::LazySym(st, is_simple), TestExpr::Basic(b))
                    | (TestExpr::Basic(b), TestExpr::LazySym(st, is_simple)) => {
                        if is_simple {
                            panic!("lazy_sym {{ is_simple: true }} added with non-simple exprs");
                        } else {
                            TestExpr::Basic(s(st).$fn_name(b))
                        }
                    }
                    (TestExpr::Simplified(s), TestExpr::Simplified(s2)) => {
                        TestExpr::mk_simple(BasicAlgebraicExpr::$variant(vec![
                            s.into_inner(),
                            s2.into_inner(),
                        ]))
                    }
                    (TestExpr::Basic(b), TestExpr::Basic(b2)) => TestExpr::Basic(b.$fn_name(b2)),
                    _ => panic!("Cannot perform op different types of expressions"),
                }
            }
        }
    };
}

impl_op!(Add(add), Sum);
impl_op!(Mul(mul), Product);

#[allow(non_upper_case_globals)]
const x: TestExpr = TestExpr::LazySym("x", false);

#[allow(non_upper_case_globals)]
const y: TestExpr = TestExpr::LazySym("y", false);

#[allow(non_upper_case_globals)]
const sx: TestExpr = TestExpr::LazySym("x", true);

#[allow(non_upper_case_globals)]
const sy: TestExpr = TestExpr::LazySym("y", true);

#[allow(dead_code)]
pub fn init_logging() {
    let layer = tracing_tree::HierarchicalLayer::default()
        .with_indent_lines(true)
        .with_targets(true)
        .with_indent_amount(2);

    let subscriber = Registry::default()
        .with(EnvFilter::default().add_directive(Directive::from(LevelFilter::TRACE)))
        .with(layer);

    let _ = tracing::subscriber::set_global_default(subscriber);
}

fn n(a: i128) -> BasicAlgebraicExpr {
    a.into()
}

fn s(a: &str) -> BasicAlgebraicExpr {
    BasicAlgebraicExpr::Symbol(a.into())
}

fn opaque() -> BasicAlgebraicExpr {
    s("opaque")
}

fn simplify(a: BasicAlgebraicExpr) -> SimpleExpr {
    a.simplify().unwrap()
}

#[test]
pub fn simplify_power() {
    // n^0 = 1
    assert_eq!(1, simplify(n(1) ^ n(0)));
    assert_eq!(1, simplify(opaque() ^ n(0)));
    // 1^n = 1
    assert_eq!(1, simplify(n(1) ^ opaque()));
    // 0^n = 0 if n > 0
    assert_eq!(0, simplify(n(0) ^ n(1)));
    assert_eq!(0, simplify(n(0) ^ n(2)));
    // 0^n = 0^n
    assert_eq!(
        SimpleExpr::assume_simplified(BasicAlgebraicExpr::Pow(Box::new((n(0), opaque())))),
        simplify(n(0) ^ opaque())
    );
}

macro_rules! assert_simplified_eq {
    ($left:expr, $right:expr) => {
        assert_eq!(($left).simplify(), ($right).simplify())
    };
}

#[test]
pub fn simplify_addition() {
    assert_simplified_eq!(3 * sx, x + 2 * x);
    assert_simplified_eq!(6 * sx, x + 2 * x + 3 * x);
}
