sed -i '' -e '/Expr::UnsafeBlock { statements, span: _ } => {/i\
            Expr::Error(_) => {\
                Type::Error\
            },
' semantic/src/lib.rs
