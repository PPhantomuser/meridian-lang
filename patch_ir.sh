sed -i '' -e 's/0/None/g' ir/src/lib.rs
sed -i '' -e 's/Expr::Error(_) => {/Expr::Error(_) => { 0 }/g' ir/src/lib.rs
