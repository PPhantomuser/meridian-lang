sed -i '' -e '/Expr::UnsafeBlock { span, .. } => span.clone(),/a\
            Expr::Error(span) => span.clone(),
' lint/src/lib.rs

sed -i '' -e '/Expr::UnsafeBlock { statements, span: _ } => {/i\
            Expr::Error(_) => {\
                None\
            },
' ir/src/lib.rs
