use crate::simplify::SimpleExpr;
use crate::BasicAlgebraicExpr;

pub fn to_latex(x: &SimpleExpr) -> String {
    let mut f = String::new();
    latex_print(x.as_inner(), &mut f);
    f
}

pub fn latex_print(x: &BasicAlgebraicExpr, f: &mut String) {
    match x {
        BasicAlgebraicExpr::Numeric(x) if let Some(i) = x.as_integer() =>  {
            f.push_str(&i.to_string());
        }
        BasicAlgebraicExpr::Numeric(x) => {
            let rational = &**x;
            let num = rational.numer();
            let denom = rational.denom();
            f.push_str(&format!("\\frac {{ {num} }} {{ {denom} }}"));
        }
        BasicAlgebraicExpr::Symbol(x) => {
            f.push_str(&x);
        }
        BasicAlgebraicExpr::Product(x) => {
            if x.len() == 0 {
                unreachable!()
            } else {
                for (i, x) in x.iter().enumerate() {
                    if i != 0 {
                        f.push_str(" \\cdot ");
                    }
                    latex_print(x, f);
                }
            }
        }
        BasicAlgebraicExpr::Sum(x) => {
            for (i, x) in x.iter().enumerate() {
                if i != 0 {
                    f.push_str(" + ");
                }
                latex_print(x, f);
            }
        }
        BasicAlgebraicExpr::Pow(x) => {
            f.push_str("(");
            latex_print(&x.0, f);
            f.push_str(")^{");
            latex_print(&x.1, f);
            f.push_str("}");
        }
        BasicAlgebraicExpr::Factorial(x) => {
            latex_print(x, f);
            f.push_str("!");
        }
        BasicAlgebraicExpr::Function(x, y) => {
            f.push_str(&x);
            f.push_str("(");
            for (i, x) in y.iter().enumerate() {
                if i != 0 {
                    f.push_str(", ");
                }
                latex_print(x, f);
            }
            f.push_str(")");
        }
    }
}
